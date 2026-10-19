//! Multi-bitstream fuzzer / inspector for GW5AST-138C `.fs` bitstreams.
//!
//! Loads the chipdb once, decodes multiple `.fs` files in parallel via Rayon, and extracts:
//! - All 12 PLL sites (L0..L3, R0..R3, B0..B3): active non-const bits per companion tile,
//!   divider row bits (row 20 on L/R, row 21 on B), ODIV0..6 values, CLKOUT0..6_EN, and
//!   decoded shortval["PLL"] attributes.
//! - All 6 HCLK control sites: row 21 active bits (CLKDIV DIV_MODE), HCLK shortval attrs,
//!   and active hclk_pips.
//! - DDRDLL sites (0,0) and (108,0): DLL0 shortval attrs and active non-const bits.
//! - DQS sites (ttyp 241 and col 0 companion): active non-const bits and decoded attrs.
//! - IODELAY / IOLOGIC / IOB / BANK: decoded attributes AND raw active bits on SSTL15 /
//!   SSTL15D / LVDS25 / LVPECL33E pins and banks.

use crate::attrs::{parse_attrvals, AttrCtx, AttrIds};
use crate::bitstream::{self, BitMatrix, TileBits};
use crate::chipdb::{loc2pin_name, Coord, Device};
use indexmap::IndexMap;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub const PLL_SITES: &[(&str, usize, usize, usize, i32)] = &[
    ("PLL_L0", 27, 1, 3, 20),
    ("PLL_L1", 45, 1, 3, 20),
    ("PLL_L2", 63, 1, 3, 20),
    ("PLL_L3", 81, 1, 3, 20),
    ("PLL_R0", 27, 177, 3, 20),
    ("PLL_R1", 45, 178, 3, 20),
    ("PLL_R2", 63, 178, 3, 20),
    ("PLL_R3", 81, 177, 3, 20),
    ("PLL_B0", 108, 28, 4, 21),
    ("PLL_B1", 108, 32, 4, 21),
    ("PLL_B2", 108, 146, 4, 21),
    ("PLL_B3", 108, 150, 4, 21),
];

pub const HCLK_SITES: &[(&str, usize, usize)] = &[
    ("HCLK0_L0", 27, 0),
    ("HCLK1_L1", 81, 0),
    ("HCLK2_R0", 27, 181),
    ("HCLK3_R1", 81, 181),
    ("HCLK4_B0", 108, 64),
    ("HCLK5_B1", 108, 117),
];

#[derive(Serialize)]
pub struct PllDump {
    pub site: String,
    pub head: (usize, usize),
    pub ttypes: Vec<i32>,
    pub cfg_row: i32,
    pub head_cfg_row_cols: Vec<i32>,
    pub tile1_cfg_row_cols: Vec<i32>,
    pub tile2_cfg_row_cols: Vec<i32>,
    pub decoded_odiv: BTreeMap<usize, i32>,
    pub decoded_clkout_en: Vec<usize>,
    pub shortval_attrs: BTreeMap<String, String>,
    pub all_nonconst_bits: Vec<(usize, i32, i32)>,
}

#[derive(Serialize)]
pub struct HclkDump {
    pub site: String,
    pub rc: (usize, usize),
    pub ttyp: i32,
    pub row21_cols: Vec<i32>,
    pub hclk_attrs: BTreeMap<String, String>,
    pub active_hclk_pips: BTreeMap<String, String>,
    pub all_nonconst_bits: Vec<(i32, i32)>,
}

#[derive(Serialize)]
pub struct TileDump {
    pub rc: (usize, usize),
    pub pin_prefix: String,
    pub ttyp: i32,
    pub nonconst_bits: Vec<(i32, i32)>,
    pub shortval_decoded: BTreeMap<String, BTreeMap<String, String>>,
    pub longval_decoded: BTreeMap<String, BTreeMap<String, String>>,
}

#[derive(Serialize)]
pub struct FsReport {
    pub path: String,
    pub active_tiles: usize,
    pub plls: Vec<PllDump>,
    pub hclks: Vec<HclkDump>,
    pub ddrdlls: Vec<TileDump>,
    pub dqs_tiles: Vec<TileDump>,
    pub banks: Vec<TileDump>,
    pub special_iobs: Vec<TileDump>,
    pub iologics: Vec<TileDump>,
}

fn tile_window<'a>(db: &Device, m: &'a BitMatrix, target_r: usize, target_c: usize) -> TileBits<'a> {
    let mut y = 0;
    for r in 0..target_r {
        y += db.tile(r, 0).height;
    }
    let mut x = 0;
    for c in 0..target_c {
        x += db.tile(target_r, c).width;
    }
    TileBits { m, y, x }
}

fn nonconst_bits(db: &Device, r: usize, c: usize, tb: &TileBits) -> Vec<Coord> {
    let td = db.tile(r, c);
    let const_set: HashSet<Coord> = db.consts.get(&td.ttyp).map(|v| v.iter().copied().collect()).unwrap_or_default();
    let mut out = Vec::new();
    for br in 0..td.height as i32 {
        for bc in 0..td.width as i32 {
            if tb.get(br, bc) && !const_set.contains(&(br, bc)) {
                out.push((br, bc));
            }
        }
    }
    out
}

fn table_li(table_name: &str) -> (&'static str, &'static str, &'static str) {
    if table_name.starts_with("CLS") {
        ("SLICE", "cls_attrids", "cls_num2val")
    } else if table_name.starts_with("IOB") || table_name.starts_with("BANK") {
        ("IOB", "iob_attrids", "iob_num2val")
    } else if table_name.starts_with("IOLOGIC") {
        ("IOLOGIC", "iologic_attrids", "iologic_num2val")
    } else if table_name == "PLL" {
        ("PLL", "pll_attrids", "pll_num2val")
    } else if table_name.starts_with("BSRAM") {
        ("BSRAM", "bsram_attrids", "bsram_num2val")
    } else if table_name == "5A_DSP" {
        ("5A_DSP", "dsp_5a_attrids", "dsp_5a_num2val")
    } else if table_name.starts_with("DSP") {
        ("DSP", "dsp_attrids", "dsp_num2val")
    } else if table_name == "OSC" {
        ("OSC", "osc_attrids", "osc_num2val")
    } else if table_name == "HCLK" {
        ("HCLK", "hclk_attrids", "hclk_num2val")
    } else if table_name.starts_with("DLL") {
        ("DLLDLY", "dlldly_attrids", "dlldly_num2val")
    } else {
        ("", "", "")
    }
}

fn decode_all_tables(
    db: &Device,
    ids: &AttrIds,
    ttyp: i32,
    tb: &TileBits,
    is_short: bool,
) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut res = BTreeMap::new();
    let map = if is_short { db.shortval.get(&ttyp) } else { db.longval.get(&ttyp) };
    let Some(tables) = map else { return res };
    for (tname, ftable) in tables {
        let (li, attr_tab, val_tab) = table_li(tname);
        if li.is_empty() {
            continue;
        }
        let rev = db.rev_logicinfo(li);
        if tname == "BANK" && !is_short {
            let mut by_bank: IndexMap<i32, crate::chipdb::FuseTable> = IndexMap::new();
            for (k, v) in ftable {
                if !k.is_empty() {
                    by_bank.entry(k[0]).or_default().insert(k[1..].to_vec(), v.clone());
                }
            }
            for (bnum, btab) in by_bank {
                let av = parse_attrvals(tb, &AttrCtx { ids, logicinfo: &rev, names: attr_tab }, &btab);
                if !av.is_empty() {
                    let mut m = BTreeMap::new();
                    for (a, v) in av {
                        let vs = ids.num2val(val_tab, v).map_or(v.to_string(), |s| s.to_string());
                        m.insert(a, vs);
                    }
                    res.insert(format!("BANK{bnum}"), m);
                }
            }
            continue;
        }
        let av = parse_attrvals(tb, &AttrCtx { ids, logicinfo: &rev, names: attr_tab }, ftable);
        if !av.is_empty() {
            let mut m = BTreeMap::new();
            for (a, v) in av {
                let vs = ids.num2val(val_tab, v).map_or(v.to_string(), |s| s.to_string());
                m.insert(a, vs);
            }
            res.insert(tname.clone(), m);
        }
    }
    res
}

fn inspect_one(db: &Device, ids: &AttrIds, path: &str) -> Option<FsReport> {
    let bm = bitstream::read_bitstream(path)?;
    let tiles = bitstream::tile_bitmap(db, &bm);
    let active_tiles = tiles.len();

    // 1. PLL sites
    let mut plls = Vec::new();
    let odiv_bases: [(usize, usize, i32); 7] = [
        (0, 0, 90), (1, 0, 100), (2, 0, 110),
        (3, 1, 0), (4, 1, 10), (5, 1, 20), (6, 1, 30),
    ];
    let rev_pll = db.rev_logicinfo("PLL");
    for &(site, r, c, n_tiles, cfg_row) in PLL_SITES {
        let mut all_bits = Vec::new();
        let mut ttypes = Vec::new();
        let mut shortval_attrs = BTreeMap::new();
        for ti in 0..n_tiles {
            let tc = c + ti;
            let td = db.tile(r, tc);
            ttypes.push(td.ttyp);
            let tb = tile_window(db, &bm, r, tc);
            for (br, bc) in nonconst_bits(db, r, tc, &tb) {
                all_bits.push((ti, br, bc));
            }
            if let Some(ft) = db.shortval.get(&td.ttyp).and_then(|t| t.get("PLL")) {
                let av = parse_attrvals(&tb, &AttrCtx { ids, logicinfo: &rev_pll, names: "pll_attrids" }, ft);
                for (a, v) in av {
                    let vs = ids.num2val("pll_num2val", v).map_or(v.to_string(), |s| s.to_string());
                    shortval_attrs.insert(format!("T{ti}:{a}"), vs);
                }
            }
        }
        // Only include if PLL is active (has bits in cfg_row or rows 2..9)
        let has_pll_bits = all_bits.iter().any(|&(_, br, _)| br == cfg_row || (2..=9).contains(&br));
        if !has_pll_bits {
            continue;
        }
        let head_cfg_row_cols: Vec<i32> = all_bits.iter()
            .filter(|&&(ti, br, _)| ti == 0 && br == cfg_row)
            .map(|&(_, _, bc)| bc)
            .collect();
        let tile1_cfg_row_cols: Vec<i32> = all_bits.iter()
            .filter(|&&(ti, br, _)| ti == 1 && br == cfg_row)
            .map(|&(_, _, bc)| bc)
            .collect();
        let tile2_cfg_row_cols: Vec<i32> = all_bits.iter()
            .filter(|&&(ti, br, _)| ti == 2 && br == cfg_row)
            .map(|&(_, _, bc)| bc)
            .collect();

        let bit_set: BTreeSet<(usize, i32, i32)> = all_bits.iter().copied().collect();
        let mut decoded_clkout_en = Vec::new();
        let mut decoded_odiv = BTreeMap::new();
        for (k, ti, base) in odiv_bases {
            let en = bit_set.contains(&(1, cfg_row, 60 + k as i32));
            if en {
                decoded_clkout_en.push(k);
            }
            let mut v = 0i32;
            for bit in 0..7 {
                if bit_set.contains(&(ti, cfg_row, base + bit)) {
                    v |= 1 << bit;
                }
            }
            if v != 0 || en {
                decoded_odiv.insert(k, 128 - v);
            }
        }
        plls.push(PllDump {
            site: site.to_string(),
            head: (r, c),
            ttypes,
            cfg_row,
            head_cfg_row_cols,
            tile1_cfg_row_cols,
            tile2_cfg_row_cols,
            decoded_odiv,
            decoded_clkout_en,
            shortval_attrs,
            all_nonconst_bits: all_bits,
        });
    }

    // 2. HCLK sites
    let mut hclks = Vec::new();
    for &(site, r, c) in HCLK_SITES {
        let td = db.tile(r, c);
        let tb = tile_window(db, &bm, r, c);
        let nc = nonconst_bits(db, r, c, &tb);
        if nc.is_empty() {
            continue;
        }
        let row21_cols: Vec<i32> = nc.iter().filter(|&&(br, _)| br == 21).map(|&(_, bc)| bc).collect();
        let sv = decode_all_tables(db, ids, td.ttyp, &tb, true);
        let hclk_attrs = sv.get("HCLK").cloned().unwrap_or_default();
        let mut active_hclk_pips = BTreeMap::new();
        if let Some(pt) = db.hclk_pips.get(&(r as i32, c as i32)) {
            for (dest, srcs) in pt {
                for (src, bits) in srcs {
                    if !bits.is_empty() && bits.iter().all(|&(br, bc)| tb.get(br, bc)) {
                        active_hclk_pips.insert(dest.clone(), src.clone());
                    }
                }
            }
        }
        hclks.push(HclkDump {
            site: site.to_string(),
            rc: (r, c),
            ttyp: td.ttyp,
            row21_cols,
            hclk_attrs,
            active_hclk_pips,
            all_nonconst_bits: nc,
        });
    }

    // 3. DDRDLL sites: (0, 0) and (108, 0)
    let mut ddrdlls = Vec::new();
    for (r, c) in [(0usize, 0usize), (108, 0)] {
        let td = db.tile(r, c);
        let tb = tile_window(db, &bm, r, c);
        let nc = nonconst_bits(db, r, c, &tb);
        if !nc.is_empty() {
            ddrdlls.push(TileDump {
                rc: (r, c),
                pin_prefix: loc2pin_name(db, r, c),
                ttyp: td.ttyp,
                nonconst_bits: nc,
                shortval_decoded: decode_all_tables(db, ids, td.ttyp, &tb, true),
                longval_decoded: decode_all_tables(db, ids, td.ttyp, &tb, false),
            });
        }
    }

    // 4. DQS tiles (ttyp 241 and their col 0 companions, plus any left-edge IO tiles with active nonconst bits)
    let mut dqs_tiles = Vec::new();
    let mut banks = Vec::new();
    let mut special_iobs = Vec::new();
    let mut iologics = Vec::new();

    let bank_coords: HashSet<(usize, usize)> = db.bank_tiles().values().copied().collect();

    for &((r, c), tb) in &tiles {
        let td = db.tile(r, c);
        let nc = nonconst_bits(db, r, c, &tb);
        if nc.is_empty() {
            continue;
        }
        let sv = decode_all_tables(db, ids, td.ttyp, &tb, true);
        let lv = decode_all_tables(db, ids, td.ttyp, &tb, false);

        if td.ttyp == 241 || (c == 0 && (r == 64 || r == 65 || r == 68 || r == 69 || r == 93 || r == 94 || r == 95 || r == 96)) {
            dqs_tiles.push(TileDump {
                rc: (r, c),
                pin_prefix: loc2pin_name(db, r, c),
                ttyp: td.ttyp,
                nonconst_bits: nc.clone(),
                shortval_decoded: sv.clone(),
                longval_decoded: lv.clone(),
            });
        }
        if bank_coords.contains(&(r, c)) {
            banks.push(TileDump {
                rc: (r, c),
                pin_prefix: loc2pin_name(db, r, c),
                ttyp: td.ttyp,
                nonconst_bits: nc.clone(),
                shortval_decoded: sv.clone(),
                longval_decoded: lv.clone(),
            });
        }
        if sv.keys().any(|k| k.starts_with("IOLOGIC")) || td.ttyp == 247 {
            iologics.push(TileDump {
                rc: (r, c),
                pin_prefix: loc2pin_name(db, r, c),
                ttyp: td.ttyp,
                nonconst_bits: nc.clone(),
                shortval_decoded: sv.clone(),
                longval_decoded: lv.clone(),
            });
        }
        // Record IOBs on Bank 2/3 (left edge r>=54) or differential IOBs on right edge (r=16..20, c=181)
        // Record every tile that decodes an IOB attribute table (all edges), plus the
        // DDR/USB regions even when the IOB table is silent.
        let is_ddr_or_usb_io = (c == 0 && r >= 54) || (c == db.cols() - 1 && (15..=25).contains(&r));
        if lv.keys().any(|k| k.starts_with("IOB")) || (is_ddr_or_usb_io && !nc.is_empty()) {
            special_iobs.push(TileDump {
                rc: (r, c),
                pin_prefix: loc2pin_name(db, r, c),
                ttyp: td.ttyp,
                nonconst_bits: nc,
                shortval_decoded: sv,
                longval_decoded: lv,
            });
        }
    }

    Some(FsReport {
        path: path.to_string(),
        active_tiles,
        plls,
        hclks,
        ddrdlls,
        dqs_tiles,
        banks,
        special_iobs,
        iologics,
    })
}

pub fn run_fuzz(db: &Device, ids: &AttrIds, paths: &[String], out_json: &str) -> Option<()> {
    let reports: Vec<FsReport> = paths
        .par_iter()
        .filter_map(|p| inspect_one(db, ids, p))
        .collect();
    let f = std::fs::File::create(out_json).map_err(|e| eprintln!("{out_json}: {e}")).ok()?;
    serde_json::to_writer_pretty(f, &reports).map_err(|e| eprintln!("{out_json}: {e}")).ok()?;
    eprintln!("[gowin_unpack --fuzz] wrote {} bitstream reports to {out_json}", reports.len());
    Some(())
}

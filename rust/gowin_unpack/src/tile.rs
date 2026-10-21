//! `gowin_unpack.parse_tile_`: one tile's bitmap -> bels (name -> flags), pips, clock pips.

use crate::attrs::{parse_attrvals, pll_attrs_refine, reg_type, AttrCtx, AttrIds, AttrVals};
use crate::bitstream::TileBits;
use crate::chipdb::{Bel, Coord, Device, FuseTable, PipTable};
use indexmap::{IndexMap, IndexSet};
use std::collections::HashMap;
use std::sync::Mutex;

pub type Flags = IndexSet<String>;
pub type Bels = IndexMap<String, Flags>;
pub type Pips = IndexMap<String, String>;

/// Shared, read-only decode context plus the cross-tile PLL/BSRAM numbering.
pub struct Ctx<'a> {
    pub db: &'a Device,
    pub ids: &'a AttrIds,
    pub device: String,
    pub rev_li: HashMap<&'static str, HashMap<i32, Coord>>,
    /// `{ttyp: {"BANK<n>": table}}` built from longval["BANK"].
    pub bank_tables: HashMap<i32, HashMap<String, FuseTable>>,
    pub pll_cells: Mutex<IndexMap<(usize, usize), usize>>,
    pub bsram_cells: Mutex<IndexMap<(usize, usize), usize>>,
    pub bm: &'a HashMap<(usize, usize), TileBits<'a>>,
}

pub struct TileOut {
    pub bels: Bels,
    pub pips: Pips,
    pub clock_pips: Pips,
}

const LI_TABLES: [&str; 14] = [
    "SLICE", "IOB", "IOLOGIC", "PLL", "BSRAM", "DSP", "5A_DSP", "OSC", "ADC",
    "HCLK", "DLLDLY", "GSR", "CFG", "DCS",
];

impl<'a> Ctx<'a> {
    pub fn new(db: &'a Device, ids: &'a AttrIds, device: &str, bm: &'a HashMap<(usize, usize), TileBits<'a>>) -> Self {
        let rev_li = LI_TABLES.iter().map(|n| (*n, db.rev_logicinfo(n))).collect();
        let mut bank_tables: HashMap<i32, HashMap<String, FuseTable>> = HashMap::new();
        for (ttyp, tables) in &db.longval {
            let Some(bank) = tables.get("BANK") else { continue };
            for (key, val) in bank {
                let t = bank_tables.entry(*ttyp).or_default().entry(format!("BANK{}", key[0])).or_default();
                t.insert(key[1..].to_vec(), val.clone());
            }
        }
        Ctx {
            db, ids, device: device.to_string(), rev_li, bank_tables,
            pll_cells: Mutex::new(IndexMap::new()), bsram_cells: Mutex::new(IndexMap::new()), bm,
        }
    }

    fn attrs(&self, tile: &TileBits, li: &str, names: &str, table: Option<&FuseTable>) -> AttrVals {
        let (Some(t), Some(l)) = (table, self.rev_li.get(li)) else { return AttrVals::new() };
        parse_attrvals(tile, &AttrCtx { ids: self.ids, logicinfo: l, names }, t)
    }

    fn shortval(&self, ttyp: i32, name: &str) -> Option<&FuseTable> {
        self.db.shortval.get(&ttyp)?.get(name)
    }

    fn cls(&self, ttyp: &str, k: &str) -> i64 {
        self.ids.code(ttyp, k).unwrap_or(i64::MIN)
    }
}

fn on_bits(tile: &TileBits, bits: &[Coord]) -> Vec<Coord> {
    let mut v: Vec<Coord> = bits.iter().copied().filter(|c| tile.get(c.0, c.1)).collect();
    v.sort_unstable();
    v.dedup();
    v
}

fn same(a: &[Coord], b: &[Coord]) -> bool {
    let mut a = a.to_vec();
    a.sort_unstable();
    a.dedup();
    a == b
}

/// Mode matching and simple flags shared by every bel kind.
fn modes_and_flags(tile: &TileBits, name: &str, bel: &Bel, bels: &mut Bels, alu: bool, default: bool) {
    let used = on_bits(tile, &bel.mode_bits());
    for (mode, bits) in &bel.modes {
        if same(bits, &used) && (default || !bits.is_empty()) {
            if alu {
                bels.insert(name.to_string(), Flags::from([mode.clone()]));
            } else {
                bels.entry(name.to_string()).or_default().insert(mode.clone());
            }
        }
    }
}

fn flags(tile: &TileBits, name: &str, bel: &Bel, bels: &mut Bels) {
    for (flag, bits) in &bel.flags {
        if bits.iter().all(|c| tile.get(c.0, c.1)) {
            if name == "RAM16" && !bels.contains_key(name) {
                continue;
            }
            // integer flags are kept apart from mode names ("0" is also a RAM16 mode)
            bels.entry(name.to_string()).or_default().insert(format!("#{flag}"));
        }
    }
}

pub(crate) fn pips(tile: &TileBits, table: &PipTable, default: bool, out: &mut Pips) {
    for (dest, srcs) in table {
        let all: Vec<Coord> = srcs.values().flatten().copied().collect();
        let used = on_bits(tile, &all);
        for (src, bits) in srcs {
            if same(bits, &used) && (default || !bits.is_empty()) {
                out.insert(dest.clone(), src.clone());
            }
        }
    }
}

/// `parse_tile_` with default=True.
pub fn parse_tile(cx: &Ctx, row: usize, col: usize, tile: &TileBits) -> TileOut {
    let td = cx.db.tile(row, col);
    let mut bels = Bels::new();
    let mut clock_pips = Pips::new();
    let mut skip: IndexSet<String> = IndexSet::new();
    for (name, bel) in &td.bels {
        if crate::tilebels::special(cx, row, col, td.ttyp, name, tile, &mut bels, &mut skip) {
            continue;
        }
        if name.starts_with("DFF") {
            dff(cx, td.ttyp, name, tile, &mut bels);
            continue;
        }
        let alu = name.starts_with("ALU");
        if alu {
            let idx: usize = name[3..4].parse().unwrap_or(0);
            let av = cx.attrs(tile, "SLICE", "cls_attrids", cx.shortval(td.ttyp, &format!("CLS{}", idx / 2)));
            if av.get("MODE") != Some(&cx.cls("cls_attrvals", "ALU")) {
                continue;
            }
            bels.insert(name.clone(), Flags::from(["C2L".to_string()]));
        }
        modes_and_flags(tile, name, bel, &mut bels, alu, true);
        flags(tile, name, bel, &mut bels);
        if let Some(num) = name.strip_prefix("BUFS") {
            let have = bels.get(name).cloned().unwrap_or_default();
            let half = if row != 0 { "B" } else { "T" };
            for qd in ["R", "L"].iter().filter(|q| !have.contains(**q)) {
                clock_pips.insert(format!("LWSPINE{half}{qd}{num}"), format!("LW{half}{num}"));
            }
            for qd in have.iter().filter(|q| *q != "R" && *q != "L") {
                let qd = qd.trim_start_matches('#');
                clock_pips.insert(format!("LWSPINE{half}{qd}{num}"), format!("LW{half}{num}"));
            }
        }
    }
    let mut p = Pips::new();
    pips(tile, &td.pips, true, &mut p);
    pips(tile, &td.clock_pips, true, &mut clock_pips);
    crate::tilebels::fix_elvds(&mut bels, &skip);
    bels.retain(|k, _| !skip.contains(k));
    TileOut { bels, pips: p, clock_pips }
}

fn dff(cx: &Ctx, ttyp: i32, name: &str, tile: &TileBits, bels: &mut Bels) {
    let idx: usize = name[3..4].parse().unwrap_or(0);
    let av = cx.attrs(tile, "SLICE", "cls_attrids", cx.shortval(ttyp, &format!("CLS{}", idx / 2)));
    if av.get("MODE") == Some(&cx.cls("cls_attrvals", "SSRAM")) {
        return;
    }
    let latch = av.get("REGMODE") == Some(&cx.cls("cls_attrvals", "LATCH"));
    let Some(t) = reg_type(cx.ids, idx, &av, latch) else { return };
    bels.insert(name.to_string(), Flags::from([t.to_string()]));
    if !latch && av.contains_key(&format!("REG{}_SD", idx % 2)) {
        bels.entry(name.to_string()).or_default().insert("SD".into());
    }
}

/// PLL attributes for RPLL/PLLVR bels.
pub fn pll_modes(cx: &Ctx, ttyp: i32, tile: &TileBits) -> Flags {
    let mut modes = Flags::from([format!("DEVICE=\"{}\"", cx.device)]);
    let av = cx.attrs(tile, "PLL", "pll_attrids", cx.shortval(ttyp, "PLL"));
    modes.extend(pll_attrs_refine(cx.ids, &av));
    modes
}

impl Ctx<'_> {
    pub fn attrs_pub(&self, tile: &TileBits, li: &str, names: &str, table: Option<&FuseTable>) -> AttrVals {
        self.attrs(tile, li, names, table)
    }
    pub fn shortval_pub(&self, ttyp: i32, name: &str) -> Option<&FuseTable> {
        self.shortval(ttyp, name)
    }
}

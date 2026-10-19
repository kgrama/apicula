//! The bel kinds of `parse_tile_` that read fuse tables: PLL, OSC, BSRAM, DSP, IOLOGIC, IOB,
//! BANK, ADC.  `special` returns true where python `continue`s past mode/flag matching.

use crate::bitstream::TileBits;
use crate::tile::{pll_modes, Bels, Ctx, Flags};
use indexmap::IndexSet;

#[allow(clippy::too_many_arguments)]
pub fn special(cx: &Ctx, row: usize, col: usize, ttyp: i32, name: &str, tile: &TileBits,
               bels: &mut Bels, skip: &mut IndexSet<String>) -> bool {
    if name.starts_with("RPLL") || name == "PLLVR" {
        let col_a = if name.starts_with("RPLL") && name.as_bytes().get(4) == Some(&b'B') { col.wrapping_sub(1) } else { col };
        let idx = next_idx(&cx.pll_cells, (row, col_a));
        bels.insert(format!("{name}{idx}"), pll_modes(cx, ttyp, tile));
        return true;
    }
    if name.starts_with("OSC") {
        osc(cx, ttyp, name, tile, bels);
        return true;
    }
    if name.starts_with("BSRAM") {
        bsram(cx, row, col, ttyp, name, tile, bels);
        return true;
    }
    if name.starts_with("ALU54D") {
        return true;
    }
    if name.starts_with("DSP") {
        dsp(cx, ttyp, name, tile, bels);
        return true;
    }
    if name.starts_with("IOLOGIC") {
        return iologic(cx, ttyp, name, tile, bels);
    }
    if name.starts_with("IOB") {
        return iob(cx, row, col, ttyp, name, tile, bels, skip);
    }
    if name.starts_with("BANK") {
        let table = cx.bank_tables.get(&ttyp).and_then(|t| t.get(name));
        let av = cx.attrs_pub(tile, "IOB", "iob_attrids", table);
        for (a, v) in av {
            let val = cx.ids.num2val("iob_num2val", v).map_or(v.to_string(), |s| s.to_string());
            bels.entry(name.to_string()).or_default().insert(format!("{a}={val}"));
        }
    }
    false
}

/// `osc_attrs_refine`.  Python reads an undefined module-level `osc_attrvals` for OSCREG;
/// attrids.osc_attrvals is used here.
fn osc(cx: &Ctx, ttyp: i32, name: &str, tile: &TileBits, bels: &mut Bels) {
    let av = cx.attrs_pub(tile, "OSC", "osc_attrids", cx.shortval_pub(ttyp, "OSC"));
    let mut modes = Flags::new();
    if let Some(v) = av.get("MCLKCIB") {
        modes.insert(format!("FREQ_DIV={v}"));
    } else if av.contains_key("MCLKCIB_EN") {
        modes.insert("FREQ_DIV=128".into());
    }
    if let Some(v) = av.get("OSCREG") {
        let n = cx.ids.name_of("osc_attrvals", *v).unwrap_or("");
        modes.insert(format!("REGULATOR_EN=\"{n}\""));
    }
    if !modes.is_empty() {
        bels.insert(name.to_string(), modes);
    }
}

fn next_idx(m: &std::sync::Mutex<indexmap::IndexMap<(usize, usize), usize>>, key: (usize, usize)) -> usize {
    let mut g = m.lock().unwrap_or_else(|p| p.into_inner());
    let n = g.len();
    *g.entry(key).or_insert(n)
}

fn bsram(cx: &Ctx, row: usize, mut col: usize, ttyp: i32, name: &str, tile: &TileBits, bels: &mut Bels) {
    let tables = ["BSRAM_SP", "BSRAM_SDP", "BSRAM_DP", "BSRAM_ROM"];
    let mut found_av = None;
    let mut found_kind = "";
    for k in tables {
        if let Some(table) = cx.shortval_pub(ttyp, k) {
            let av = cx.attrs_pub(tile, "BSRAM", "bsram_attrids", Some(table));
            if !av.is_empty() {
                found_av = Some(av);
                found_kind = k;
                break;
            }
        }
    }
    let Some(av) = found_av else { return };
    if name.ends_with("_AUX") {
        col = col.saturating_sub(1);
        if cx.db.tile(row, col).bels.contains_key("BSRAM_AUX") {
            col = col.saturating_sub(1);
        }
    }
    let idx = next_idx(&cx.bsram_cells, (row, col));
    let mut flags = Flags::new();
    flags.insert(format!("KIND={found_kind}"));
    for (a, v) in av {
        let val = cx.ids.num2val("bsram_num2val", v).map_or(v.to_string(), |s| s.to_string());
        flags.insert(format!("{a}={val}"));
    }
    bels.insert(format!("{name}{idx}"), flags);
}

fn dsp(cx: &Ctx, ttyp: i32, name: &str, tile: &TileBits, bels: &mut Bels) {
    let idx = &name[name.len() - 1..];
    let av = if let Some(t) = cx.shortval_pub(ttyp, &format!("DSP{idx}")) {
        cx.attrs_pub(tile, "DSP", "dsp_attrids", Some(t))
    } else if let Some(t) = cx.shortval_pub(ttyp, "5A_DSP") {
        cx.attrs_pub(tile, "5A_DSP", "dsp_5a_attrids", Some(t))
    } else {
        return;
    };
    // python adds the attribute names (dict iteration) as modes
    let modes: Flags = av.keys().cloned().collect();
    if !modes.is_empty() && !name.starts_with("DSP_AUX") {
        bels.insert(format!("{name}{idx}"), modes);
    }
}

const IOL_MODE: &[(&str, &str)] = &[
    ("MODDRX2", "OSER4_MEM"), ("ODDRX2", "OSER4"), ("MODDRX21", "OSER4"), ("MODDRX4", "OSER8_MEM"),
    ("ODDRX4", "OSER8"), ("MODDRX5", "OSER10"), ("ODDRX5", "OSER10"), ("VIDEORX", "OVIDEO"),
    ("ODDRX8", "OSER16"), ("MIDDRX2", "IDES4_MEM"), ("IDDRX2", "IDES4"), ("MIDDRX4", "IDES8_MEM"),
    ("IDDRX4", "IDES8"), ("MIDDRX5", "IDES10"), ("IDDRX5", "IDES10"), ("IDDRX8", "IDES16"),
];

/// IOLOGIC; returns true where python `continue`s.
/// Supports dual-mode (simultaneous OUTMODE and INMODE, e.g. DDR3 DQ OSER8_MEM + IDES8_MEM).
fn iologic(cx: &Ctx, ttyp: i32, name: &str, tile: &TileBits, bels: &mut Bels) -> bool {
    let idx = &name[name.len() - 1..];
    let av = cx.attrs_pub(tile, "IOLOGIC", "iologic_attrids", cx.shortval_pub(ttyp, &format!("IOLOGIC{idx}")));
    if av.is_empty() {
        return true;
    }
    let code = |k: &str| cx.ids.code("iologic_attrvals", k).unwrap_or(i64::MIN);
    let mut any_emitted = false;
    let both = av.contains_key("OUTMODE") && av.contains_key("INMODE");
    for (dir_attr, sfx, x1, x1b, lsr, ddr) in [
        ("OUTMODE", if both { "_O" } else { "" }, code("MODDRX1"), code("ODDRX1"), "LSROMUX_0", "ODDR"),
        ("INMODE", if both { "_I" } else { "" }, code("MIDDRX1"), code("IDDRX1"), "LSRIMUX_0", "IDDR"),
    ] {
        let Some(&key) = av.get(dir_attr) else { continue };
        let bel_key = format!("{name}{sfx}");
        let ent = bels.entry(bel_key.clone()).or_default();
        if key == x1 || key == x1b {
            let c = if av.contains_key(lsr) { "C" } else { "" };
            ent.insert(format!("MODE={ddr}{c}"));
            any_emitted = true;
            continue;
        }
        if key == code("DDRENABLE") {
            if ent.is_empty() {
                bels.shift_remove(&bel_key);
            }
            continue;
        }
        if let Some(m) = cx.ids.num2val("iologic_num2val", key).and_then(|v| IOL_MODE.iter().find(|(k, _)| *k == v)) {
            let mut mode = if ddr == "IDDR" && m.1 == "OVIDEO" { "IVIDEO" } else { m.1 };
            // On GW5AST-138C, OSER8_MEM encodes OUTMODE=ODDRX4 plus WRFCLKSEL=DQSW/DQSW270
            if cx.device == "GW5AST-138C" && mode == "OSER8" {
                if let Some(&w) = av.get("WRFCLKSEL") {
                    if let Some(wn) = cx.ids.num2val("iologic_num2val", w) {
                        if wn == "DQSW" || wn == "DQSW270" {
                            mode = "OSER8_MEM";
                            ent.insert(format!("TCLK_SOURCE=\"{wn}\""));
                        }
                    }
                }
            }
            ent.insert(format!("MODE={mode}"));
            any_emitted = true;
        }
        if let Some(v) = av.get("CLKODDRMUX_ECLK") {
            let n = cx.ids.num2val("iologic_num2val", *v).unwrap_or("");
            ent.insert(format!("CLKODDRMUX_ECLK={n}"));
        }
        if ent.is_empty() {
            bels.shift_remove(&bel_key);
        }
    }
    if av.contains_key("INDEL") || av.contains_key("OUTDEL") {
        let mut dly: u32 = 0;
        for b in 0..7 {
            if av.contains_key(&format!("DELAY_DEL{b}")) {
                dly |= 1 << b;
            }
        }
        let mut flags = Flags::new();
        flags.insert(format!("C_STATIC_DLY={dly}"));
        bels.insert(format!("IODELAY{idx}"), flags);
    }
    !any_emitted
}

#[allow(clippy::too_many_arguments)]
fn iob(cx: &Ctx, row: usize, col: usize, ttyp: i32, name: &str, tile: &TileBits,
       bels: &mut Bels, skip: &mut IndexSet<String>) -> bool {
    let idx = &name[name.len() - 1..];
    let bel = &cx.db.tile(row, col).bels[name];
    let (mut io_tile, mut io_ttyp) = (*tile, ttyp);
    if idx == "B" {
        if let Some((dr, dc)) = bel.fuse_cell_offset {
            let (r, c) = ((row as i32 + dr) as usize, (col as i32 + dc) as usize);
            io_ttyp = cx.db.tile(r, c).ttyp;
            match cx.bm.get(&(r, c)) {
                Some(t) => io_tile = *t,
                None => return false, // python raises KeyError on an empty tile
            }
        }
    }
    let lv = cx.db.longval.get(&io_ttyp);
    if idx == "B" && !lv.is_some_and(|t| t.contains_key("IOBB")) {
        return true;
    }
    let av = cx.attrs_pub(&io_tile, "IOB", "iob_attrids", lv.and_then(|t| t.get(&format!("IOB{idx}"))));
    if av.is_empty() {
        return false;
    }
    let mode = iob_mode(cx, row, idx, name, &av, skip);
    let ent = bels.entry(name.to_string()).or_default();
    ent.insert(mode);
    for (a, v) in &av {
        if matches!(a.as_str(), "IO_TYPE" | "DRIVE" | "PULLMODE" | "SLEWRATE" | "BANK_VCCIO" | "HYSTERESIS" | "OPENDRAIN" | "CLAMP" | "VREF" | "DIFFRESISTOR" | "SINGLERESISTOR" | "DDR_DYNTERM") {
            let val = cx.ids.num2val("iob_num2val", *v).map_or(v.to_string(), |s| s.to_string());
            ent.insert(format!("{a}={val}"));
        }
    }
    false
}

fn iob_mode(cx: &Ctx, row: usize, idx: &str, name: &str, av: &crate::attrs::AttrVals,
            skip: &mut IndexSet<String>) -> String {
    let b_name = format!("{}B", &name[..name.len() - 1]);
    let mut mode = "IBUF".to_string();
    if av.get("PERSISTENT") == cx.ids.code("iob_attrvals", "OFF").as_ref() {
        mode = "IOBUF".into();
    } else if av.contains_key("ODMUX") || av.contains_key("ODMUX_1") {
        mode = "OBUF".into();
    }
    if (cx.device == "GW1NZ-1" || cx.device == "GW1N-1") && row == 5 {
        mode = "IOBUF".into();
    }
    if av.contains_key("LVDS_OUT") {
        if mode == "IOBUF" {
            mode = "TBUF".into();
        }
        mode = if av.contains_key("MIPI") { "MIPI_OBUF".into() } else { format!("TLVDS_{mode}") };
        skip.insert(b_name);
    } else if av.contains_key("OD") {
        mode = "I3C_IOBUF".into();
    } else if idx == "B" && !av.contains_key("DRIVE") && av.contains_key("IO_TYPE") {
        mode = format!("ELVDS_{mode}");
        skip.insert(name.to_string());
    } else if av.contains_key("LPRX_A1") {
        mode = "MIPI_IBUF".into();
        skip.insert(b_name);
    } else if av.contains_key("IOBUF_MIPI_LP") {
        mode = format!("ELVDS_{mode}");
        skip.insert(b_name);
    }
    mode
}

/// ELVDS: the A bel takes the B bel's differential mode.
pub fn fix_elvds(bels: &mut Bels, skip: &IndexSet<String>) {
    const DIFF: [&str; 10] = ["ELVDS_IBUF", "ELVDS_OBUF", "ELVDS_IOBUF", "ELVDS_TBUF", "TLVDS_IBUF",
                              "TLVDS_OBUF", "TLVDS_IOBUF", "TLVDS_TBUF", "MIPI_IBUF", "MIPI_OBUF"];
    for name in skip {
        let a = format!("{}A", &name[..name.len() - 1]);
        let Some(fa) = bels.get(&a) else { continue };
        if fa.iter().any(|f| DIFF.contains(&f.as_str())) {
            continue;
        }
        let mode: Vec<String> = bels.get(name).map_or(Vec::new(), |f| {
            f.iter().filter(|m| DIFF[..4].contains(&m.as_str())).cloned().collect()
        });
        if mode.is_empty() {
            continue;
        }
        if let Some(fa) = bels.get_mut(&a) {
            fa.retain(|m| !["IBUF", "OBUF", "IOBUF", "TBUF"].contains(&m.as_str()));
            fa.extend(mode);
        }
    }
}

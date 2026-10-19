//! Attribute decoding: `parse_attrvals`, DFF/latch typing and the PLL attribute refinement
//! of `apycula.gowin_unpack`, with the `apycula.attrids` tables loaded from data/attrids.json.

use crate::bitstream::TileBits;
use crate::chipdb::{Coord, FuseTable};
use indexmap::{IndexMap, IndexSet};
use std::collections::{HashMap, HashSet};

/// `apycula.attrids` dict tables, in definition order.  `<x>_num2val` maps are keyed by the
/// stringified integer.
pub struct AttrIds {
    tables: IndexMap<String, IndexMap<String, serde_json::Value>>,
}

static ATTRIDS_JSON: &str = include_str!("../data/attrids.json");

impl AttrIds {
    pub fn load() -> Option<AttrIds> {
        serde_json::from_str(ATTRIDS_JSON)
            .map(|tables| AttrIds { tables })
            .map_err(|e| eprintln!("attrids.json: {e}"))
            .ok()
    }

    /// `table[name]` as an integer code.
    pub fn code(&self, table: &str, name: &str) -> Option<i64> {
        self.tables.get(table)?.get(name)?.as_i64()
    }

    /// First name in `table` whose code is `code` (python's linear scan).
    pub fn name_of(&self, table: &str, code: i64) -> Option<&str> {
        let t = self.tables.get(table)?;
        t.iter().find(|(_, v)| v.as_i64() == Some(code)).map(|(k, _)| k.as_str())
    }

    /// `<x>_num2val[code]`.
    pub fn num2val(&self, table: &str, code: i64) -> Option<&str> {
        self.tables.get(table)?.get(&code.to_string())?.as_str()
    }
}

/// Decoded attributes: `{attr name: raw value}`.
pub type AttrVals = IndexMap<String, i64>;

fn sorted(bits: &[Coord]) -> Vec<Coord> {
    let mut v = bits.to_vec();
    v.sort_unstable();
    v.dedup();
    v
}

fn subset(a: &[Coord], b: &HashSet<Coord>) -> bool {
    a.iter().all(|c| b.contains(c))
}

fn subset_vec(a: &[Coord], b: &[Coord]) -> bool {
    a.iter().all(|c| b.binary_search(c).is_ok())
}

/// Fuse table entries split into positive / negative keys, bits sorted for set tests.
struct Split {
    pos: Vec<(Vec<i32>, Vec<Coord>)>,
    neg: Vec<(Vec<i32>, Vec<Coord>)>,
}

fn split(table: &FuseTable) -> Split {
    let mut s = Split { pos: Vec::new(), neg: Vec::new() };
    for (av, bits) in table {
        let e = (av.clone(), sorted(bits));
        if av.iter().any(|k| *k < 0) { s.neg.push(e) } else { s.pos.push(e) }
    }
    s
}

/// Entries that are not a strict subset of another entry in the same list.
fn maximal(c: &[&(Vec<i32>, Vec<Coord>)]) -> Vec<usize> {
    (0..c.len())
        .filter(|&i| !c.iter().any(|o| o.1 != c[i].1 && subset_vec(&c[i].1, &o.1)))
        .collect()
}

/// Context for one `parse_attrvals` call.
pub struct AttrCtx<'a> {
    pub ids: &'a AttrIds,
    pub logicinfo: &'a HashMap<i32, Coord>,
    pub names: &'a str,
}

impl AttrCtx<'_> {
    fn put(&self, res: &mut AttrVals, idx: i32) {
        let Some((attr, val)) = self.logicinfo.get(&idx) else { return };
        let name = self.ids.name_of(self.names, *attr as i64).unwrap_or("");
        res.insert(name.to_string(), *val as i64);
    }
}

/// `gowin_unpack.parse_attrvals`.
pub fn parse_attrvals(tile: &TileBits, cx: &AttrCtx, table: &FuseTable) -> AttrVals {
    let s = split(table);
    let on = |v: &[(Vec<i32>, Vec<Coord>)]| -> HashSet<Coord> {
        v.iter().flat_map(|e| e.1.iter()).filter(|c| tile.get(c.0, c.1)).copied().collect()
    };
    let (set_bits, neg_bits) = (on(&s.pos), on(&s.neg));
    let mut res = AttrVals::new();

    let cnd: Vec<_> = s.pos.iter().filter(|e| subset(&e.1, &set_bits)).collect();
    let mut attrvals: IndexSet<i32> = IndexSet::new();
    for i in maximal(&cnd) {
        let mut clean: Vec<i32> = cnd[i].0.iter().copied().filter(|a| *a > 0).collect();
        clean.sort_unstable();
        clean.dedup();
        for idx in clean {
            attrvals.insert(idx);
            cx.put(&mut res, idx);
        }
    }

    // negative keys whose fuses are set (python's "attr in res" test compares an id with a
    // name and never fires, so only the subset rule decides)
    let cnd: Vec<_> = s.neg.iter().filter(|e| subset(&e.1, &neg_bits)).collect();
    let mut neg_attrvals: IndexSet<i32> = IndexSet::new();
    let mut ignore: HashSet<i32> = HashSet::new();
    for i in maximal(&cnd) {
        neg_attrvals.extend(cnd[i].0.iter().copied().filter(|a| *a > 0));
        ignore.extend(cnd[i].0.iter().filter(|a| **a < 0).map(|a| a.abs()));
    }
    let mut nv: Vec<i32> = neg_attrvals.into_iter().collect();
    nv.sort_unstable();
    for idx in nv {
        cx.put(&mut res, idx);
    }

    // negative keys whose fuses are not all set
    for (av, _) in s.neg.iter().filter(|e| !subset(&e.1, &neg_bits)) {
        let negs: Vec<i32> = av.iter().filter(|a| **a < 0).map(|a| a.abs()).collect();
        let pos_ok = av.iter().filter(|a| **a > 0).all(|a| attrvals.contains(a));
        if negs.iter().any(|i| ignore.contains(i)) || (!negs.is_empty() && !pos_ok) {
            continue;
        }
        for idx in negs {
            cx.put(&mut res, idx);
        }
    }
    res
}

fn val_name<'a>(ids: &'a AttrIds, av: &AttrVals, key: &str, default: &'a str) -> &'a str {
    match av.get(key) {
        Some(v) => ids.name_of("cls_attrvals", *v).unwrap_or(""),
        None => default,
    }
}

/// `get_dff_type` / `get_latch_type`: the primitive name for DFF `idx` of a slice.
pub fn reg_type(ids: &AttrIds, idx: usize, av: &AttrVals, latch: bool) -> Option<&'static str> {
    let lsr = val_name(ids, av, "LSRONMUX", "");
    let clk = val_name(ids, av, "CLKMUX_CLK", "SIG");
    let srm = val_name(ids, av, "SRMODE", "");
    let set = val_name(ids, av, &format!("REG{}_REGSET", idx % 2), "SET");
    let key = (set, lsr, clk, srm);
    let dff: &[(&str, &str, &str, &str, &'static str)] = &[
        ("RESET", "", "SIG", "", "DFF"), ("RESET", "", "INV", "", "DFFN"),
        ("RESET", "LSRMUX", "SIG", "ASYNC", "DFFC"), ("RESET", "LSRMUX", "INV", "ASYNC", "DFFNC"),
        ("RESET", "LSRMUX", "SIG", "", "DFFR"), ("RESET", "LSRMUX", "INV", "", "DFFNR"),
        ("SET", "LSRMUX", "SIG", "ASYNC", "DFFP"), ("SET", "LSRMUX", "INV", "ASYNC", "DFFNP"),
        ("SET", "LSRMUX", "SIG", "", "DFFS"), ("SET", "LSRMUX", "INV", "", "DFFNS"),
    ];
    let lat: &[(&str, &str, &str, &str, &'static str)] = &[
        ("RESET", "", "SIG", "", "DL"), ("RESET", "", "INV", "", "DLN"),
        ("RESET", "LSRMUX", "SIG", "ASYNC", "DLC"), ("RESET", "LSRMUX", "INV", "ASYNC", "DLNC"),
        ("SET", "LSRMUX", "SIG", "ASYNC", "DLP"), ("SET", "LSRMUX", "INV", "ASYNC", "DLNP"),
    ];
    let t = if latch { lat } else { dff };
    t.iter().find(|e| (e.0, e.1, e.2, e.3) == key).map(|e| e.4)
}

const PLL_ATTRS: &[(&str, &str)] = &[
    ("IDIV", "IDIV_SEL"), ("IDIVSEL", "DYN_IDIV_SEL"), ("FDIV", "FBDIV_SEL"),
    ("FDIVSEL", "DYN_FBDIV_SEL"), ("ODIV", "ODIV_SEL"), ("ODIVSEL", "DYN_ODIV_SEL"),
    ("PHASE", "PSDA_SEL"), ("DUTY", "DUTYDA_SEL"), ("DPSEL", "DYN_DA_EN"),
    ("OPDLY", "CLKOUT_DLY_STEP"), ("OSDLY", "CLKOUTP_DLY_STEP"), ("SDIV", "DYN_SDIV_SEL"),
    ("CLKOUTDIVSEL", "CLKOUTD_SRC"), ("CLKOUTDIV3SEL", "CLKOUTD3_SRC"), ("BYPCK", "CLKOUT_BYPASS"),
    ("BYPCKPS", "CLKOUTP_BYPASS"), ("BYPCKDIV", "CLKOUTD_BYPASS"),
];

/// `pll_attrs_refine`: `ATTR=value` strings.  An unknown value is logged and skipped.
pub fn pll_attrs_refine(ids: &AttrIds, av: &AttrVals) -> Vec<String> {
    let mut res = Vec::new();
    for (attr, val) in av {
        let Some((_, ext)) = PLL_ATTRS.iter().find(|(i, _)| i == attr) else {
            if ["INSEL", "FBSEL", "PWDEN", "RSTEN", "CLKOUTDIV3", "CLKOUTPS"].contains(&attr.as_str()) {
                if let Some(n) = ids.name_of("pll_attrvals", *val) {
                    res.push(format!("{attr}=\"{n}\""));
                }
            }
            continue;
        };
        let v = match *ext {
            "CLKOUTP_DLY_STEP" | "CLKOUT_DLY_STEP" => format!("{}", *val as f64 / 50.0),
            "PSDA_SEL" | "DUTYDA_SEL" => format!("\"{val:04b}\""),
            "IDIV_SEL" | "FBDIV_SEL" => format!("{}", val - 1),
            "DYN_SDIV_SEL" | "ODIV_SEL" => format!("{val}"),
            _ => match ids.name_of("pll_attrvals", *val) {
                Some("DYN") | Some("BYPASS") => "\"true\"".into(),
                Some("CLKOUTPS") => "\"CLKOUTP\"".into(),
                Some(n) => format!("\"{n}\""),
                None => {
                    eprintln!("PLL no {ext} = {val}");
                    continue;
                }
            },
        };
        res.push(format!("{ext}={v}"));
    }
    res
}

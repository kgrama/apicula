//! `fix_plls`: drop unconfigured rPLL/PLLVR, prune ports of disabled features, resolve the
//! input/feedback selectors and gather numbered ports into buses.

use crate::chipdb::Device;
use crate::codegen::{Module, Primitive, Wire};

fn drop_ports(p: &mut Primitive, base: &str, n: usize) {
    for i in 0..n {
        p.portmap.shift_remove(&format!("{base}{i}"));
    }
}

/// `disable_unused_pll_ports`.
fn disable_unused(p: &mut Primitive) {
    if !p.params.contains_key("DYN_DA_EN") {
        for b in ["PSDA", "DUTYDA", "FDLY"] {
            drop_ports(p, b, 4);
        }
    }
    for (param, base) in [("DYN_IDIV_SEL", "IDSEL"), ("DYN_FBDIV_SEL", "FBDSEL"), ("DYN_ODIV_SEL", "ODSEL")] {
        if !p.params.contains_key(param) {
            drop_ports(p, base, 6);
        }
    }
    for (param, port) in [("PWDEN", "RESET_P"), ("RSTEN", "RESET"), ("CLKOUTDIV3", "CLKOUTD3"),
                          ("CLKOUTDIV", "CLKOUTD"), ("CLKOUTPS", "CLKOUTP")] {
        if let Some(v) = p.params.shift_remove(param) {
            if v.trim_matches('"') == "DISABLE" {
                p.portmap.shift_remove(port);
            }
        }
    }
}

/// `find_pll_in_pin`: the IOB output wire of the package pin tagged [L]RPLL_T_IN.
fn pll_in_pin(db: &Device, device: &str, p: &mut Primitive) {
    let pkg = db.pinout.get(device).and_then(|v| v.values().next());
    let Some(loc) = pkg.and_then(|pins| {
        pins.values().find(|(_, cfg)| cfg.iter().any(|c| c == "RPLL_T_IN" || c == "LRPLL_T_IN")).map(|(l, _)| l.clone())
    }) else {
        eprintln!("No [RL]PLL_T_IN pin in the current package");
        return;
    };
    let side = &loc[2..3];
    let digits: String = loc[3..].chars().take_while(|c| c.is_ascii_digit()).collect();
    let bel_idx = &loc[3 + digits.len()..];
    let num: usize = digits.parse().unwrap_or(1);
    let (row, col) = match side {
        "T" => (0, num - 1),
        "B" => (db.rows() - 1, num - 1),
        "L" => (num - 1, 0),
        _ => (num - 1, db.cols() - 1),
    };
    if let Some(w) = db.tile(row, col).bels.get(&format!("IOB{bel_idx}")).and_then(|b| b.port("O")) {
        p.port("CLKIN", format!("R{}C{}_{w}", row + 1, col + 1));
    }
}

/// `modify_pll_inputs`.
fn modify_inputs(db: &Device, device: &str, p: &mut Primitive) {
    if let Some(insel) = p.params.shift_remove("INSEL") {
        match insel.trim_matches('"') {
            "CLKIN1" => {}
            "CLKIN0" => pll_in_pin(db, device, p),
            _ => { p.portmap.shift_remove("CLKIN"); }
        }
    }
    if let Some(fb) = p.params.shift_remove("FBSEL") {
        match fb.trim_matches('"') {
            "CLKFB3" => {
                p.params.insert("CLKFB_SEL".into(), "\"internal\"".into());
                p.portmap.shift_remove("CLKFB");
            }
            "CLKFB0" | "CLKFB2" => { p.params.insert("CLKFB_SEL".into(), "\"external\"".into()); }
            _ => {}
        }
    }
}

/// `fix_pll_ports`: PSDA0..3 etc. into one bus port.
fn gather(p: &mut Primitive) {
    for (base, n) in [("PSDA", 4), ("DUTYDA", 4), ("FDLY", 4), ("FBDSEL", 6), ("IDSEL", 6), ("ODSEL", 6)] {
        let mut bus = Vec::new();
        for i in 0..n {
            if let Some(Wire::One(w)) = p.portmap.shift_remove(&format!("{base}{i}")) {
                bus.push(w);
            }
        }
        if !bus.is_empty() {
            p.portmap.insert(base.into(), Wire::Bus(bus));
        }
    }
}

pub fn fix_plls(db: &Device, device: &str, m: &mut Module) {
    let names: Vec<String> = m.primitives.iter()
        .filter(|(_, p)| p.typ == "rPLL" || p.typ == "PLLVR").map(|(k, _)| k.clone()).collect();
    for n in names {
        let Some(mut p) = m.primitives.shift_remove(&n) else { continue };
        if !p.params.contains_key("INSEL") {
            continue;
        }
        disable_unused(&mut p);
        modify_inputs(db, device, &mut p);
        let w: Vec<String> = p.wires().cloned().collect();
        m.wires.extend(w);
        gather(&mut p);
        m.primitives.insert(n, p);
    }
}

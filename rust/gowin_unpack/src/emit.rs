//! `gowin_unpack.tile2verilog` for the logic primitives: LUT (+ wide-LUT muxes), DFF/latch,
//! ALU, RAM16, PLL, BSRAM, OSC.  IO primitives are in emit_io.rs.

use crate::chipdb::{wire2global, Device};
use crate::codegen::{Constraints, Module, Primitive, Wire};
use crate::tile::{Flags, TileOut};

pub struct Emit<'a> {
    pub db: &'a Device,
    pub device: &'a str,
    pub row: usize,
    pub col: usize,
}

const SIDES: [char; 2] = ['A', 'B'];
const BEL_TYPES: [&str; 19] = ["IOLOGIC", "IODELAY", "IOB", "LUT", "DFF", "BANK", "CFG", "ALU", "RAM16", "ODDR",
    "OSCZ", "OSCF", "OSCH", "OSCW", "OSCO", "OSC", "BUFS", "PLLVR", "BSRAM"];

/// The `belre` split: (type, index suffix).
fn bel_type(bel: &str) -> Option<(&str, &str)> {
    if bel.starts_with("DSP") {
        return Some(("DSP", &bel[3..]));
    }
    if bel.starts_with("RPLLA") || bel.starts_with("RPLLB") {
        return Some((&bel[..5], &bel[5..]));
    }
    // python's alternation order: IOB before IOLOGIC never matches "IOLOGIC" (IOL != IOB)
    BEL_TYPES.iter().find(|t| bel.starts_with(**t)).map(|t| (*t, &bel[t.len()..]))
}

impl Emit<'_> {
    fn w(&self, wire: &str) -> String {
        format!("R{}C{}_{wire}", self.row, self.col)
    }

    pub fn w_pub(&self, wire: &str) -> String {
        self.w(wire)
    }

    fn global(&self, wire: &str) -> String {
        wire2global(self.row as i64, self.col as i64, self.db.rows() as i64, self.db.cols() as i64, wire)
    }

    fn cell(&self, cst: &mut Constraints, name: &str, idx: usize) {
        cst.cells.insert(name.into(), (self.row, self.col, idx / 2, SIDES[idx % 2]));
    }
}

/// One tile into the module and constraints (python `tile2verilog`).
pub fn tile2verilog(e: &Emit, out: &mut TileOut, m: &mut Module, cst: &mut Constraints) {
    for (dest, src) in out.pips.iter().chain(out.clock_pips.iter()) {
        let (s, d) = (e.global(src), e.global(dest));
        m.wires.insert(s.clone());
        m.wires.insert(d.clone());
        m.assign(d, s);
    }
    let mut iol_detected: Vec<String> = Vec::new();
    // move_iologic: IOLOGIC/ODDR bels first
    let mut order: Vec<String> = ["IOLOGICA", "IOLOGICA_O", "IOLOGICA_I", "IOLOGICB", "IOLOGICB_O", "IOLOGICB_I", "ODDRA", "ODDRB"].iter()
        .filter(|b| out.bels.contains_key(**b)).map(|b| b.to_string()).collect();
    let rest: Vec<String> = out.bels.keys().filter(|b| !order.contains(b)).cloned().collect();
    order.extend(rest);
    for bel in order {
        let Some(flags) = out.bels.get(&bel).cloned() else { continue };
        let Some((typ, idx)) = bel_type(&bel) else { continue };
        match typ {
            "LUT" => lut(e, idx, &flags, m, cst),
            "DFF" => dff(e, idx, &flags, m, cst),
            "ALU" => alu(e, idx, &flags, m),
            "RAM16" => ram16(e, &flags, m),
            "RPLLA" | "RPLLB" | "PLLVR" => pll(e, &bel, typ, idx, &flags, m),
            "BSRAM" => bsram(e, idx, &flags, m),
            "OSC" | "OSCZ" | "OSCF" | "OSCH" | "OSCW" | "OSCO" => osc(e, &bel, typ, &flags, m),
            "IOLOGIC" => crate::emit_io::iologic(e, &bel, idx, &flags, m, &mut iol_detected),
            "IODELAY" => crate::emit_io::iodelay(e, idx, &flags, m),
            "IOB" => crate::emit_io::iob(e, &bel, idx, &flags, m, cst, &iol_detected),
            _ => {}
        }
    }
    m.assign("VCC".into(), "1'b1".into());
    m.assign("VSS".into(), "1'b0".into());
}

fn lut(e: &Emit, idx: &str, flags: &Flags, m: &mut Module, cst: &mut Constraints) {
    let bits: u32 = flags.iter().filter_map(|f| f.strip_prefix('#')?.parse::<u32>().ok()).map(|f| 1u32 << f).sum();
    let val = 0xffff_u32.wrapping_sub(bits) & 0xffff;
    if val == 0 {
        m.assign(e.w(&format!("F{idx}")), "VSS".into());
    } else {
        let name = e.w(&format!("LUT4_{idx}"));
        let mut p = Primitive::new("LUT4", &name);
        p.params.insert("INIT".into(), format!("16'h{val:04x}"));
        p.port("F", e.w(&format!("F{idx}")));
        for (port, l) in [("I0", "A"), ("I1", "B"), ("I2", "C"), ("I3", "D")] {
            p.port(port, e.w(&format!("{l}{idx}")));
        }
        m.add_prim(p);
        e.cell(cst, &name, idx.parse().unwrap_or(0));
    }
    make_muxes(e, m);
}

fn mux(e: &Emit, m: &mut Module, name: &str, ports: [String; 4]) {
    let mut p = Primitive::new("MUX2", &e.w(name));
    for (port, w) in ["I0", "I1", "O", "S0"].iter().zip(ports) {
        p.port(port, w);
    }
    m.add_prim(p);
}

fn make_muxes(e: &Emit, m: &mut Module) {
    if m.primitives.contains_key(&e.w("MUX2_LUT50")) {
        return;
    }
    let (r, c) = (e.row, e.col);
    let w = |s: String| format!("R{r}C{c}_{s}");
    if c < e.db.cols() {
        mux(e, m, "MUX2_LUT80", [format!("R{r}C{}_OF3", c + 1), w("OF3".into()), w("OF7".into()), w("SEL7".into())]);
    }
    mux(e, m, "MUX2_LUT70", [w("OF5".into()), w("OF1".into()), w("OF3".into()), w("SEL3".into())]);
    for i in 0..2 {
        mux(e, m, &format!("MUX2_LUT6{i}"), [w(format!("OF{}", i * 4 + 2)), w(format!("OF{}", i * 4)),
            w(format!("OF{}", i * 4 + 1)), w(format!("SEL{}", i * 4 + 1))]);
    }
    for i in 0..4 {
        mux(e, m, &format!("MUX2_LUT5{i}"), [w(format!("F{}", i * 2)), w(format!("F{}", i * 2 + 1)),
            w(format!("OF{}", i * 2)), w(format!("SEL{}", i * 2))]);
    }
}

const DFF_PORT: [(&str, &str); 16] = [
    ("DFF", ""), ("DFFN", ""), ("DFFS", "SET"), ("DFFR", "RESET"), ("DFFP", "PRESET"), ("DFFC", "CLEAR"),
    ("DFFNS", "SET"), ("DFFNR", "RESET"), ("DFFNP", "PRESET"), ("DFFNC", "CLEAR"),
    ("DL", ""), ("DLN", ""), ("DLC", "CLEAR"), ("DLNC", "CLEAR"), ("DLP", "PRESET"), ("DLNP", "PRESET"),
];

fn dff(e: &Emit, idx: &str, flags: &Flags, m: &mut Module, cst: &mut Constraints) {
    let sd = flags.contains("SD");
    let kinds: Vec<&String> = flags.iter().filter(|f| *f != "SD").collect();
    let [kind] = kinds.as_slice() else { return };
    let Some((_, port)) = DFF_PORT.iter().find(|(k, _)| k == kind) else { return };
    let i: usize = idx.parse().unwrap_or(0);
    let latch = kind.starts_with("DL");
    let name = if latch { e.w(&format!("DLE_{i}")) } else { e.w(&format!("DFFE_{i}")) };
    let mut p = Primitive::new(&format!("{kind}E"), &name);
    p.port("CLK", e.w(&format!("CLK{}", i / 2)));
    p.port("D", e.w(&if sd { format!("SEL{i}") } else { format!("F{i}") }));
    p.port("Q", e.w(&format!("Q{i}")));
    p.port("CE", e.w(&format!("CE{}", i / 2)));
    if !port.is_empty() {
        p.port(port, e.w(&format!("LSR{}", i / 2)));
    }
    m.add_prim(p);
    e.cell(cst, &name, i);
}

fn alu(e: &Emit, idx: &str, flags: &Flags, m: &mut Module) {
    let [kind] = flags.iter().collect::<Vec<_>>()[..] else { return };
    let i: usize = idx.parse().unwrap_or(0);
    let kind = if kind == "hadder" { "0" } else { kind.as_str() };
    if !(kind == "C2L" || (kind.len() == 1 && "012346789".contains(kind))) {
        return;
    }
    let mut p = Primitive::new("ALU", &e.w(&format!("ALU_{i}")));
    p.params.insert("ALU_MODE".into(), kind.into());
    if kind != "C2L" {
        p.port("SUM", e.w(&format!("F{i}")));
    }
    p.port("CIN", e.w(&format!("CIN{i}")));
    p.port("I2", e.w(&format!("C{i}")));
    let cout = if i != 5 { e.w(&format!("CIN{}", i + 1)) } else { format!("R{}C{}_CIN0", e.row, e.col + 1) };
    p.port("COUT", cout);
    let (a, b, d) = (e.w(&format!("A{i}")), e.w(&format!("B{i}")), e.w(&format!("D{i}")));
    if kind.len() == 1 && "2346789".contains(kind) {
        p.port("I0", a);
        p.port("I1", b);
        if kind == "2" || kind == "8" {
            p.port("I3", d);
        }
    } else if kind == "0" {
        p.port("I0", b);
        p.port("I1", d);
        p.port("I3", a);
    } else if kind == "C2L" {
        p.port("I0", b);
        p.port("I1", d);
        p.port("COUT", e.w(&format!("F{i}")));
        p.params.insert("ALU_MODE".into(), "9".into());
    } else {
        p.port("I0", a);
        p.port("I1", d);
    }
    m.add_prim(p);
}

fn ram16(e: &Emit, flags: &Flags, m: &mut Module) {
    let has = |x: u32| flags.contains(&format!("#{x}"));
    let init = |k: u32| (0..16).filter(|x| !has(k * 16 + x)).fold(0u32, |a, x| a | (1 << x));
    let mut p = Primitive::new("RAM16SDP4", &e.w("RAM16"));
    for k in 0..4 {
        p.params.insert(format!("INIT_{k}"), format!("16'b{:016b}", init(k)));
    }
    let bus = |f: &dyn Fn(&str) -> String| Wire::Bus("DCBA".chars().map(|c| f(&c.to_string())).collect());
    p.portmap.insert("DI".into(), bus(&|c| e.w(&format!("{c}5"))));
    p.port("CLK", e.w("CLK2"));
    p.port("WRE", e.w("LSR2"));
    p.portmap.insert("WAD".into(), bus(&|c| e.w(&format!("{c}4"))));
    p.portmap.insert("RAD".into(), bus(&|c| e.w(&format!("{c}0"))));
    p.portmap.insert("DO".into(), Wire::Bus((0..5).rev().map(|x| e.w(&format!("F{x}"))).collect()));
    m.add_prim(p);
}

fn params_into(p: &mut Primitive, flags: &Flags) {
    for f in flags {
        let (k, v) = f.split_once('=').unwrap_or((f, ""));
        p.params.insert(k.into(), v.into());
    }
}

fn bel_ports(e: &Emit, p: &mut Primitive, bel: &str) {
    let Some(b) = e.db.tile(e.row - 1, e.col - 1).bels.get(bel) else { return };
    for (port, w) in &b.portmap {
        if let Some(w) = w.as_str() {
            p.port(port, e.w(w));
        }
    }
}

fn pll(e: &Emit, bel: &str, typ: &str, idx: &str, flags: &Flags, m: &mut Module) {
    let name = format!("PLL_{idx}");
    let kind = if typ == "PLLVR" { "PLLVR" } else { "rPLL" };
    let p = m.primitives.entry(name.clone()).or_insert_with(|| Primitive::new(kind, &name));
    params_into(p, flags);
    let base = if typ == "PLLVR" { &bel[..bel.len() - 1] } else { &bel[..5] };
    bel_ports(e, p, base);
}

fn bsram(e: &Emit, idx: &str, flags: &Flags, m: &mut Module) {
    if idx.starts_with("_AUX") {
        return;
    }
    let name = format!("BSRAM_{idx}");
    let p = m.primitives.entry(name.clone()).or_insert_with(|| Primitive::new("BSRAM", &name));
    params_into(p, flags);
    bel_ports(e, p, "BSRAM");
}

fn osc(e: &Emit, bel: &str, typ: &str, flags: &Flags, m: &mut Module) {
    let mut p = Primitive::new(typ, &e.w(typ));
    params_into(&mut p, flags);
    bel_ports(e, &mut p, bel);
    m.add_prim(p);
}

//! `tile2verilog` for IOLOGIC (serdes/DDR) and IOB primitives.

use crate::chipdb::loc2pin_name;
use crate::codegen::{Constraints, Module, Primitive};
use crate::emit::Emit;
use crate::tile::Flags;
use indexmap::IndexMap;

/// `_iologic_ports[mode]`: bel port -> primitive port (16-wide modes: primitive port -> wire).
fn iol_ports(mode: &str) -> &'static [(&'static str, &'static str)] {
    match mode {
        "ODDR" => &[("D0", "D0"), ("D1", "D1"), ("Q0", "Q0"), ("Q1", "Q1"), ("CLK", "CLK")],
        "ODDRC" => &[("D0", "D0"), ("D1", "D1"), ("Q0", "Q0"), ("Q1", "Q1"), ("CLK", "CLK"), ("CLEAR", "CLEAR")],
        "OSER4" | "OSER4_MEM" => &[("D0", "D0"), ("D1", "D1"), ("D2", "D2"), ("D3", "D3"), ("Q0", "Q0"), ("Q1", "Q1"),
            ("RESET", "RESET"), ("TX0", "TX0"), ("TX1", "TX1"), ("PCLK", "PCLK"), ("FCLK", "FCLK")],
        "OSER8" | "OSER8_MEM" => &[("D0", "D0"), ("D1", "D1"), ("D2", "D2"), ("D3", "D3"), ("D4", "D4"), ("D5", "D5"),
            ("D6", "D6"), ("D7", "D7"), ("Q0", "Q0"), ("Q1", "Q1"), ("RESET", "RESET"), ("TX0", "TX90"),
            ("TX1", "TX1"), ("TX2", "TX2"), ("TX3", "TX3"), ("PCLK", "PCLK"), ("FCLK", "FCLK")],
        "OVIDEO" => &[("D0", "D0"), ("D1", "D1"), ("D2", "D2"), ("D3", "D3"), ("D4", "D4"), ("D5", "D5"),
            ("D6", "D6"), ("Q", "Q"), ("RESET", "RESET"), ("PCLK", "PCLK"), ("FCLK", "FCLK")],
        "OSER10" => &[("D0", "D0"), ("D1", "D1"), ("D2", "D2"), ("D3", "D3"), ("D4", "D4"), ("D5", "D5"),
            ("D6", "D6"), ("D7", "D7"), ("D8", "D8"), ("D9", "D9"), ("Q", "Q"), ("RESET", "RESET"),
            ("PCLK", "PCLK"), ("FCLK", "FCLK")],
        "OSER16" => &[("D0", "A0"), ("D1", "A1"), ("D2", "A2"), ("D3", "A3"), ("D4", "C1"), ("D5", "C0"),
            ("D6", "D1"), ("D7", "D0"), ("D8", "C3"), ("D9", "C2"), ("D10", "B4"), ("D11", "B5"),
            ("D12", "A0"), ("D13", "A1"), ("D14", "A2"), ("D15", "A3")],
        "IDDR" => &[("D", "D"), ("Q8", "Q0"), ("Q9", "Q1"), ("CLK", "CLK")],
        "IDDRC" => &[("D", "D"), ("Q8", "Q0"), ("Q9", "Q1"), ("CLK", "CLK"), ("CLEAR", "CLEAR")],
        "IDES4" | "IDES4_MEM" => &[("D", "D"), ("Q6", "Q0"), ("Q7", "Q1"), ("Q8", "Q2"), ("Q9", "Q3"), ("RESET", "RESET"),
            ("CALIB", "CALIB"), ("PCLK", "PCLK"), ("FCLK", "FCLK")],
        "IDES8" | "IDES8_MEM" => &[("D", "D"), ("Q2", "Q0"), ("Q3", "Q1"), ("Q4", "Q2"), ("Q5", "Q3"), ("Q6", "Q4"),
            ("Q7", "Q5"), ("Q8", "Q6"), ("Q9", "Q7"), ("RESET", "RESET"), ("CALIB", "CALIB"),
            ("PCLK", "PCLK"), ("FCLK", "FCLK")],
        "IVIDEO" => &[("D", "D"), ("Q3", "Q0"), ("Q4", "Q1"), ("Q5", "Q2"), ("Q6", "Q3"), ("Q7", "Q4"),
            ("Q8", "Q5"), ("Q9", "Q6"), ("RESET", "RESET"), ("CALIB", "CALIB"), ("PCLK", "PCLK"), ("FCLK", "FCLK")],
        "IDES10" => &[("D", "D"), ("Q0", "Q0"), ("Q1", "Q1"), ("Q2", "Q2"), ("Q3", "Q3"), ("Q4", "Q4"),
            ("Q5", "Q5"), ("Q6", "Q6"), ("Q7", "Q7"), ("Q8", "Q8"), ("Q9", "Q9"), ("RESET", "RESET"),
            ("CALIB", "CALIB"), ("PCLK", "PCLK"), ("FCLK", "FCLK")],
        "IDES16" => &[("Q0", "F2"), ("Q1", "F3"), ("Q2", "F4"), ("Q3", "F5"), ("Q4", "Q0"), ("Q5", "Q1"),
            ("Q6", "Q2"), ("Q7", "Q3"), ("Q8", "Q4"), ("Q9", "Q5"), ("Q10", "F0"), ("Q11", "F1"),
            ("Q12", "F2"), ("Q13", "F3"), ("Q14", "F4"), ("Q15", "F5")],
        _ => &[],
    }
}

/// `iologic_ports_by_type`: (primitive port, bel wire) pairs.
fn ports_by_type(mode: &str, pm: &IndexMap<String, String>) -> Vec<(String, String)> {
    let t = iol_ports(mode);
    if mode != "OSER16" && mode != "IDES16" {
        return pm.iter()
            .filter_map(|(p, w)| t.iter().find(|(k, _)| k == p).map(|(_, v)| (v.to_string(), w.clone())))
            .collect();
    }
    let mut v: Vec<(String, String)> = t.iter().map(|(p, w)| (p.to_string(), w.to_string())).collect();
    let mut extra = vec!["RESET", "PCLK", "FCLK"];
    extra.extend(if mode == "IDES16" { vec!["CALIB", "D"] } else { vec!["Q"] });
    for p in extra {
        v.push((p.into(), pm.get(p).cloned().unwrap_or_default()));
    }
    v
}

const OSER: [&str; 9] = ["ODDR", "ODDRC", "OSER4", "OSER4_MEM", "OVIDEO", "OSER8", "OSER8_MEM", "OSER10", "OSER16"];

pub fn iologic(e: &Emit, bel: &str, idx: &str, flags: &Flags, m: &mut Module, det: &mut Vec<String>) {
    let base_idx = idx.trim_end_matches("_O").trim_end_matches("_I");
    let base_bel = bel.trim_end_matches("_O").trim_end_matches("_I");
    if !det.iter().any(|d| d == base_idx) {
        det.push(base_idx.to_string());
    }
    let mut mode = "IVIDEO".to_string();
    let eclk = "HCLK0"; // python compares (==) instead of assigning the ECLK mux value
    let mut params = IndexMap::new();
    for f in flags {
        let (k, v) = f.split_once('=').unwrap_or((f, ""));
        match k {
            "MODE" => mode = v.to_string(),
            "CLKODDRMUX_ECLK" | "CLKIDDRMUX_ECLK" => {}
            _ => { params.insert(k.to_string(), v.to_string()); }
        }
    }
    let name = e.w_pub(&format!("{mode}_{base_idx}"));
    let pm: IndexMap<String, String> = e.db.tile(e.row - 1, e.col - 1).bels.get(base_bel)
        .map(|b| b.portmap.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect())
        .unwrap_or_default();
    let fclk = if e.device == "GW1N-1" { "CLK2" } else { eclk };
    let oser = OSER.contains(&mode.as_str());
    let (r, c, rows) = (e.row, e.col, e.db.rows());
    let next = |w: &str| if r == 1 || r == rows { format!("R{r}C{}_{w}", c + 1) } else { format!("R{}C{c}_{w}", r + 1) };
    let iol = m.primitives.entry(name.clone()).or_insert_with(|| Primitive::new(&mode, &name));
    iol.params.extend(params);
    let get = |k: &str| pm.get(k).cloned().unwrap_or_default();
    for (port, w) in ports_by_type(&mode, &pm) {
        let wire = if port == "FCLK" {
            format!("R{r}C{c}_{fclk}")
        } else if oser && ["Q", "Q0", "Q1"].contains(&port.as_str()) {
            format!("R{r}C{c}_{}_IOL", if port == "Q1" { get("TX0") } else { get("D0") })
        } else if oser && mode == "OSER16" && ["D12", "D13", "D14", "D15"].contains(&port.as_str()) {
            next(&w)
        } else if !oser && port == "D" {
            format!("R{r}C{c}_{}_IOL", get("D"))
        } else if !oser && mode == "IDES16" && ["Q0", "Q1", "Q2", "Q3"].contains(&port.as_str()) {
            next(&w)
        } else {
            format!("R{r}C{c}_{w}")
        };
        iol.port(&port, wire);
    }
}

pub fn iodelay(e: &Emit, idx: &str, flags: &Flags, m: &mut Module) {
    let name = e.w_pub(&format!("IODELAY_{idx}"));
    let mut p = Primitive::new("IODELAY", &name);
    for f in flags {
        let (k, v) = f.split_once('=').unwrap_or((f, ""));
        p.params.insert(k.to_string(), v.to_string());
    }
    m.add_prim(p);
}

/// `iobmap[kind]`: (wires, inputs, outputs, inouts).
fn iobmap(kind: &str) -> Option<(&'static [&'static str], &'static [&'static str], &'static [&'static str], &'static [&'static str])> {
    Some(match kind {
        "IBUF" => (&["O"], &["I"], &[], &[]),
        "OBUF" => (&["I"], &[], &["O"], &[]),
        "TBUF" => (&["I", "OE"], &[], &["O"], &[]),
        "IOBUF" => (&["I", "O", "OE"], &[], &[], &["IO"]),
        "TLVDS_OBUF" | "ELVDS_OBUF" => (&["I"], &[], &["O", "OB"], &[]),
        "TLVDS_TBUF" | "ELVDS_TBUF" => (&["I", "OE"], &[], &["O", "OB"], &[]),
        "TLVDS_IBUF" | "ELVDS_IBUF" => (&["O"], &["I", "IB"], &[], &[]),
        "ELVDS_IOBUF" => (&["I", "O", "OE"], &[], &[], &["IO", "IOB"]),
        "MIPI_IBUF" => (&["I", "IB", "OEN", "OENB", "HSREN", "OH", "OL", "OB"], &[], &[], &["IO", "IOB"]),
        "MIPI_OBUF" => (&["I", "IB", "IL", "MODESEL"], &[], &[], &["O", "OB"]),
        "I3C_IOBUF" => (&["I", "O", "MODESEL"], &[], &[], &["IO"]),
        _ => return None,
    })
}

fn iob_portmap(e: &Emit, bel: &str, kind: &str) -> IndexMap<String, String> {
    let t = e.db.tile(e.row - 1, e.col - 1);
    let p = |b: &str, k: &str| t.bels.get(b).and_then(|b| b.port(k)).unwrap_or("").to_string();
    let own: IndexMap<String, String> = t.bels.get(bel)
        .map(|b| b.portmap.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect())
        .unwrap_or_default();
    match kind {
        "MIPI_IBUF" => {
            let right = e.db.tile(e.row - 1, e.col).bels.get("IOBA").and_then(|b| b.port("O")).unwrap_or("").to_string();
            IndexMap::from([("I".into(), p("IOBA", "I")), ("IB".into(), p("IOBB", "I")), ("OEN".into(), p("IOBA", "OE")),
                ("OENB".into(), p("IOBB", "OE")), ("OH".into(), p("IOBA", "O")), ("OB".into(), p("IOBB", "O")),
                ("HSREN".into(), p("IOLOGICB", "SETN")), ("OL".into(), right)])
        }
        "MIPI_OBUF" => IndexMap::from([("I".into(), p("IOBA", "I")), ("IB".into(), p("IOBB", "I")),
            ("IL".into(), p("IOBA", "I")), ("MODESEL".into(), p("IOBA", "OE"))]),
        "I3C_IOBUF" => {
            let mut m = own;
            if let Some(oe) = m.shift_remove("OE") {
                m.insert("MODESEL".into(), oe);
            }
            m
        }
        _ => own,
    }
}

pub fn iob(e: &Emit, bel: &str, idx: &str, flags: &Flags, m: &mut Module, cst: &mut Constraints, det: &[String]) {
    let kinds: Vec<&String> = flags.iter().filter(|f| iobmap(f).is_some()).collect();
    let [kind] = kinds.as_slice() else { return };
    let Some((wires, ins, outs, inouts)) = iobmap(kind) else { return };
    let pm = iob_portmap(e, bel, kind);
    let name = e.w_pub(&format!("{kind}_{idx}"));
    let mut p = Primitive::new(kind, &name);
    let sfx = if det.iter().any(|d| d == idx) { "_IOL" } else { "" };
    for port in wires {
        let w = pm.get(*port).cloned().unwrap_or_default();
        let s = match kind.as_str() {
            "MIPI_IBUF" => if *port == "OH" { sfx } else { "" },
            "MIPI_OBUF" => if *port == "I" { sfx } else { "" },
            _ => sfx,
        };
        p.port(if *port == "OE" { "OEN" } else { port }, e.w_pub(&format!("{w}{s}")));
    }
    for port in ins.iter().chain(outs).chain(inouts) {
        p.port(port, e.w_pub(&format!("{port}{idx}")));
    }
    for port in wires {
        m.wires.insert(e.w_pub(pm.get(*port).map_or("", |s| s)));
    }
    for (set, list) in [(&mut m.inputs, ins), (&mut m.outputs, outs), (&mut m.inouts, inouts)] {
        set.extend(list.iter().map(|w| e.w_pub(&format!("{w}{idx}"))));
    }
    m.primitives.insert(name.clone(), p);
    let pos = loc2pin_name(e.db, e.row - 1, e.col - 1);
    let loc = if kind.starts_with("TLVDS") {
        let nxt = (idx.as_bytes()[0] + 1) as char;
        format!("{pos}{idx},{pos}{nxt}")
    } else {
        format!("{pos}{idx}")
    };
    cst.ports.insert(name.clone(), loc);
    for f in flags.iter().filter(|f| *f != *kind) {
        if let Some((k, v)) = f.split_once('=') {
            cst.attrs.entry(name.clone()).or_default().insert(k.into(), v.into());
        }
    }
}

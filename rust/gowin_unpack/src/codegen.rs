//! `apycula.codegen` Module / Primitive / Constraints and their Verilog / CST writers.

use indexmap::{IndexMap, IndexSet};
use std::collections::HashMap;
use std::io::Write;

#[derive(Clone, Debug)]
pub enum Wire {
    One(String),
    Bus(Vec<String>),
}

#[derive(Clone, Debug)]
pub struct Primitive {
    pub typ: String,
    pub inst: String,
    pub portmap: IndexMap<String, Wire>,
    pub params: IndexMap<String, String>,
}

impl Primitive {
    pub fn new(typ: &str, inst: &str) -> Self {
        Primitive { typ: typ.into(), inst: inst.into(), portmap: IndexMap::new(), params: IndexMap::new() }
    }

    pub fn port(&mut self, p: &str, w: String) {
        self.portmap.insert(p.into(), Wire::One(w));
    }

    pub fn wires(&self) -> impl Iterator<Item = &String> {
        self.portmap.values().flat_map(|w| match w {
            Wire::One(s) => std::slice::from_ref(s).iter(),
            Wire::Bus(v) => v.iter(),
        })
    }

    fn write(&self, f: &mut impl Write) -> std::io::Result<()> {
        write!(f, "{} {} (", self.typ, self.inst)?;
        for (i, (port, wire)) in self.portmap.iter().enumerate() {
            let w = match wire {
                Wire::One(s) => s.clone(),
                Wire::Bus(v) => format!("{{{}}}", v.join(", ")),
            };
            write!(f, "{}\n.{port}({w})", if i > 0 { "," } else { "" })?;
        }
        f.write_all(b"\n);\n")?;
        for (k, v) in &self.params {
            writeln!(f, "defparam {}.{k} = {v};", self.inst)?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct Module {
    pub inputs: IndexSet<String>,
    pub outputs: IndexSet<String>,
    pub inouts: IndexSet<String>,
    pub wires: IndexSet<String>,
    pub wire_aliases: HashMap<String, String>,
    /// dest -> src; python's dict(assigns): first position, last value
    pub assigns: IndexMap<String, String>,
    pub primitives: IndexMap<String, Primitive>,
}

impl Module {
    pub fn assign(&mut self, dest: String, src: String) {
        self.assigns.insert(dest, src);
    }

    pub fn add_prim(&mut self, p: Primitive) {
        let w: Vec<String> = p.wires().cloned().collect();
        self.wires.extend(w);
        self.primitives.insert(p.inst.clone(), p);
    }

    pub fn write(&self, f: &mut impl Write) -> std::io::Result<()> {
        let ports: Vec<String> = self.inputs.iter().chain(&self.outputs).chain(&self.inouts)
            .map(|p| p.split('[').next().unwrap_or(p).trim().to_string())
            .collect();
        writeln!(f, "module top({});", ports.join(", "))?;
        for (kind, set) in [("input", &self.inputs), ("output", &self.outputs), ("inout", &self.inouts)] {
            for p in set {
                writeln!(f, "{kind} {p};")?;
            }
        }
        for w in &self.wires {
            match self.wire_aliases.get(w) {
                Some(a) => writeln!(f, "`define {w} {a}")?,
                None => writeln!(f, "wire {w};")?,
            }
        }
        let px = |w: &str| if self.wire_aliases.contains_key(w) { "`" } else { "" };
        for (d, s) in &self.assigns {
            writeln!(f, "assign {}{d} = {}{s};", px(d), px(s))?;
        }
        for p in self.primitives.values() {
            p.write(f)?;
        }
        f.write_all(b"endmodule\n")
    }
}

#[derive(Default)]
pub struct Constraints {
    /// name -> (row, col, slice, "A"/"B"), 1-based row/col
    pub cells: IndexMap<String, (usize, usize, usize, char)>,
    pub ports: IndexMap<String, String>,
    pub attrs: IndexMap<String, IndexMap<String, String>>,
}

impl Constraints {
    pub fn write(&self, f: &mut impl Write) -> std::io::Result<()> {
        for (k, (r, c, s, l)) in &self.cells {
            writeln!(f, "INS_LOC \"{k}\" R{r}C{c}[{s}][{l}];")?;
        }
        for (k, v) in &self.ports {
            writeln!(f, "IO_LOC \"{k}\" {v};")?;
        }
        for (k, v) in &self.attrs {
            write!(f, "IO_PORT \"{k}\" ")?;
            for (a, val) in v {
                write!(f, "{a}={val} ")?;
            }
            f.write_all(b";\n")?;
        }
        Ok(())
    }
}

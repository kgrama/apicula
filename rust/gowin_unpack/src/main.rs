//! gowin_unpack: Rust port of `apycula.gowin_unpack` (Gowin `.fs` -> Verilog + CST).
//!
//! Usage: gowin_unpack -d <device> [-o out.v] [-s out.cst] [--noalu] [--chipdb path] <bitstream.fs>

mod attrs;
mod bitstream;
mod chipdb;
mod codegen;
mod emit;
mod emit_io;
mod fuzz;
mod lw;
mod pipdiff;
mod pll;
mod tile;
mod tilebels;

use bitstream::TileBits;
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::time::Instant;

struct Args {
    fs: Vec<String>,
    device: String,
    out: String,
    cst: Option<String>,
    noalu: bool,
    chipdb: Option<String>,
    fuzz: bool,
    lw: bool,
    pipdiff: Option<String>,
}

fn parse_args() -> Option<Args> {
    let mut a = Args { fs: Vec::new(), device: String::new(), out: "unpack.v".into(), cst: None, noalu: false, chipdb: None, fuzz: false, lw: false, pipdiff: None };
    let mut it = std::env::args().skip(1);
    while let Some(x) = it.next() {
        match x.as_str() {
            "-d" | "--device" => a.device = it.next()?,
            "-o" | "--output" => a.out = it.next()?,
            "-s" | "--cst" => a.cst = Some(it.next()?),
            "--chipdb" => a.chipdb = Some(it.next()?),
            "--noalu" => a.noalu = true,
            "--fuzz" => a.fuzz = true,
            "--lw" => a.lw = true,
            "--pipdiff" => a.pipdiff = Some(it.next()?),
            _ => a.fs.push(x),
        }
    }
    if a.fs.is_empty() || a.device.is_empty() {
        eprintln!("usage: gowin_unpack -d <device> [-o out.v] [-s out.cst] [--noalu] [--fuzz] [--lw] [--pipdiff base.fs] [--chipdb path] <bitstream.fs ...>");
        return None;
    }
    Some(a)
}

fn chipdb_path(a: &Args) -> String {
    a.chipdb.clone().unwrap_or_else(|| {
        let here = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        here.join("../../apycula").join(format!("{}.msgpack.xz", a.device)).to_string_lossy().into_owned()
    })
}

/// Himbaechel node aliases and the short-wire (SN/EW) aliases of `main`.
fn wire_aliases(db: &chipdb::Device) -> HashMap<String, String> {
    let mut al = HashMap::new();
    for (_, (_, wires)) in &db.nodes {
        let mut w: Vec<&(i32, i32, String)> = wires.iter().collect();
        w.sort_by_key(|x| x.2.len());
        let Some(root) = w.first().map(|x| format!("R{}C{}_{}", x.0 + 1, x.1 + 1, x.2)) else { continue };
        for x in &w[1..] {
            al.insert(format!("R{}C{}_{}", x.0 + 1, x.1 + 1, x.2), root.clone());
        }
    }
    let (rows, cols) = (db.rows() as i64, db.cols() as i64);
    let g = |r: i64, c: i64, w: &str| chipdb::wire2global(r, c, rows, cols, w);
    for row in 0..rows {
        for col in 0..cols {
            for i in [1, 2] {
                let sn = format!("R{}C{}_SN{i}0", row + 1, col + 1);
                let ew = format!("R{}C{}_EW{i}0", row + 1, col + 1);
                al.insert(g(row, col + 1, &format!("N1{i}1")), sn.clone());
                al.insert(g(row + 2, col + 1, &format!("S1{i}1")), sn);
                al.insert(g(row + 1, col, &format!("W1{i}1")), ew.clone());
                al.insert(g(row + 1, col + 2, &format!("E1{i}1")), ew);
            }
        }
    }
    al
}

/// Tile processing order: bank tiles first, then the rest in bitmap order.
fn order(db: &chipdb::Device, tiles: &[((usize, usize), TileBits)]) -> Vec<(usize, usize)> {
    let banks: Vec<(usize, usize)> = db.bank_tiles().values().copied().collect();
    let present: std::collections::HashSet<_> = tiles.iter().map(|t| t.0).collect();
    let mut v: Vec<_> = banks.iter().copied().filter(|b| present.contains(b)).collect();
    v.extend(tiles.iter().map(|t| t.0).filter(|p| !banks.contains(p)));
    v
}

/// Pre-assign PLL/BSRAM cell numbers in processing order so tiles can decode in parallel.
fn number_cells(cx: &tile::Ctx, ord: &[(usize, usize)]) {
    let mut pll = cx.pll_cells.lock().unwrap_or_else(|p| p.into_inner());
    let mut bs = cx.bsram_cells.lock().unwrap_or_else(|p| p.into_inner());
    for &(r, c) in ord {
        let td = cx.db.tile(r, c);
        for name in td.bels.keys() {
            if name.starts_with("RPLL") || name == "PLLVR" {
                let col = if name.as_bytes().get(4) == Some(&b'B') && name.starts_with("RPLL") { c.wrapping_sub(1) } else { c };
                let n = pll.len();
                pll.entry((r, col)).or_insert(n);
            } else if name.starts_with("BSRAM") && cx.db.shortval.get(&td.ttyp).is_some_and(|t| t.keys().any(|k| k.starts_with("BSRAM_"))) {
                let mut col = c;
                if name.ends_with("_AUX") {
                    col = col.saturating_sub(1);
                    if cx.db.tile(r, col).bels.contains_key("BSRAM_AUX") {
                        col = col.saturating_sub(1);
                    }
                }
                let n = bs.len();
                bs.entry((r, col)).or_insert(n);
            }
        }
    }
}

fn prune(bels: &mut tile::Bels, noalu: bool) {
    let alus: Vec<String> = bels.keys().filter(|b| b.starts_with("ALU")).cloned().collect();
    for a in alus {
        let key = if noalu { a.clone() } else { format!("LUT{}", &a[3..]) };
        bels.shift_remove(&key);
    }
    if bels.contains_key("RAM16") {
        for x in 0..6 {
            bels.shift_remove(&format!("LUT{x}"));
        }
        for x in 4..6 {
            bels.shift_remove(&format!("DFF{x}"));
        }
    }
}

fn run(a: &Args) -> Option<()> {
    let t0 = Instant::now();
    let db = chipdb::load_chipdb(&chipdb_path(a))?;
    let ids = attrs::AttrIds::load()?;
    let t1 = Instant::now();
    if let Some(base) = &a.pipdiff {
        let out_json = if a.out == "unpack.v" { "pipdiff.json" } else { &a.out };
        pipdiff::run_pipdiff(&db, base, &a.fs, out_json)?;
        eprintln!("[gowin_unpack --pipdiff] load {:.2}s scan {:.2}s", (t1 - t0).as_secs_f64(), t1.elapsed().as_secs_f64());
        return Some(());
    }
    if a.lw {
        let out_json = if a.out == "unpack.v" { "lw_report.json" } else { &a.out };
        lw::run_lw(&db, &a.fs, out_json)?;
        eprintln!("[gowin_unpack --lw] load {:.2}s scan {:.2}s", (t1 - t0).as_secs_f64(), t1.elapsed().as_secs_f64());
        return Some(());
    }
    if a.fuzz {
        let out_json = if a.out == "unpack.v" { "fuzz_report.json" } else { &a.out };
        fuzz::run_fuzz(&db, &ids, &a.fs, out_json)?;
        eprintln!("[gowin_unpack --fuzz] load {:.2}s fuzz {:.2}s", (t1 - t0).as_secs_f64(), t1.elapsed().as_secs_f64());
        return Some(());
    }
    let fs_path = a.fs.first()?;
    let bitmap = bitstream::read_bitstream(fs_path)?;
    let tiles = bitstream::tile_bitmap(&db, &bitmap);
    let t2 = Instant::now();
    let bm: HashMap<(usize, usize), TileBits> = tiles.iter().map(|(k, v)| (*k, *v)).collect();
    let cx = tile::Ctx::new(&db, &ids, &a.device, &bm);
    let ord = order(&db, &tiles);
    number_cells(&cx, &ord);
    let mut outs: Vec<tile::TileOut> = ord.par_iter().map(|&(r, c)| {
        let mut o = tile::parse_tile(&cx, r, c, &bm[&(r, c)]);
        prune(&mut o.bels, a.noalu);
        o
    }).collect();
    let t3 = Instant::now();
    let mut m = codegen::Module { wire_aliases: wire_aliases(&db), ..Default::default() };
    let mut cst = codegen::Constraints::default();
    for (&(r, c), o) in ord.iter().zip(outs.iter_mut()) {
        let e = emit::Emit { db: &db, device: &a.device, row: r + 1, col: c + 1 };
        emit::tile2verilog(&e, o, &mut m, &mut cst);
    }
    pll::fix_plls(&db, &a.device, &mut m);
    write_out(a, &m, &cst)?;
    eprintln!("[gowin_unpack] tiles {} load {:.2}s bitstream {:.2}s decode {:.2}s emit+write {:.2}s",
              ord.len(), (t1 - t0).as_secs_f64(), (t2 - t1).as_secs_f64(), (t3 - t2).as_secs_f64(), t3.elapsed().as_secs_f64());
    Some(())
}

fn write_out(a: &Args, m: &codegen::Module, cst: &codegen::Constraints) -> Option<()> {
    let f = std::fs::File::create(&a.out).map_err(|e| eprintln!("{}: {e}", a.out)).ok()?;
    let mut w = BufWriter::new(f);
    m.write(&mut w).and_then(|_| w.flush()).map_err(|e| eprintln!("{}: {e}", a.out)).ok()?;
    if let Some(p) = &a.cst {
        let f = std::fs::File::create(p).map_err(|e| eprintln!("{p}: {e}")).ok()?;
        let mut w = BufWriter::new(f);
        cst.write(&mut w).and_then(|_| w.flush()).map_err(|e| eprintln!("{p}: {e}")).ok()?;
    }
    Some(())
}

fn main() {
    let ok = parse_args().and_then(|a| run(&a)).is_some();
    std::process::exit(if ok { 0 } else { 1 });
}

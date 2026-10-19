//! `--lw`: long-wire (segment) and clock-pip census over vendor `.fs` bitstreams.
//!
//! Collects active segment mux pips (LT00/LT10/LT20/LT30/LT02/LT13 on the mux rows) and active
//! tap pips (`X <- LB<k>1`) per bitstream, then histograms, for every mux at column `s` and every
//! tap at column `c` in `s-1..=s+2` within 9 rows and the same i/i+4 half, `s % 4 -> k % 4`.

use crate::bitstream::{self};
use crate::chipdb::Device;
use crate::tile::{pips, Pips};
use rayon::prelude::*;
use serde::Serialize;

const MUX_ROWS: [usize; 13] = [0, 9, 18, 27, 36, 45, 54, 63, 72, 81, 90, 99, 108];

/// One active pip: (row, col, dest, src).
type Rec = (usize, usize, String, String);

#[derive(Serialize, Default)]
struct FileLw {
    path: String,
    muxes: Vec<Rec>,
    taps: Vec<Rec>,
    /// Active clock pips (spines, GT, PLL/BDCLK/MDCLK gates).
    clock: Vec<Rec>,
}

#[derive(Serialize)]
struct LwReport {
    files: Vec<FileLw>,
    /// `hist[s % 4][k % 4]` observation counts.
    hist: [[u64; 4]; 4],
}

/// Half of the segment pair a mux wire drives: 0 for i, 1 for i + 4.
fn mux_half(w: &str) -> Option<usize> {
    match w {
        "LT00" | "LT02" | "LT20" => Some(0),
        "LT10" | "LT13" | "LT30" => Some(1),
        _ => None,
    }
}

/// Tap index `k` of a `LB<k>1` wire.
fn tap_idx(w: &str) -> Option<usize> {
    let b = w.as_bytes();
    if b.len() == 4 && w.starts_with("LB") && b[3] == b'1' && b[2].is_ascii_digit() {
        return Some((b[2] - b'0') as usize);
    }
    None
}

fn scan_file(db: &Device, path: &str) -> FileLw {
    let mut out = FileLw { path: path.to_string(), ..Default::default() };
    let Some(bm) = bitstream::read_bitstream(path) else { return out };
    for ((r, c), tb) in bitstream::tile_bitmap(db, &bm) {
        let td = db.tile(r, c);
        let mut cp = Pips::new();
        pips(&tb, &td.clock_pips, false, &mut cp);
        out.clock.extend(cp.into_iter().map(|(d, s)| (r, c, d, s)));
        let mut p = Pips::new();
        pips(&tb, &td.pips, false, &mut p);
        for (dest, src) in p {
            if mux_half(&dest).is_some() && MUX_ROWS.contains(&r) {
                out.muxes.push((r, c, dest, src));
            } else if tap_idx(&src).is_some() {
                out.taps.push((r, c, dest, src));
            }
        }
    }
    out
}

fn add_hist(f: &FileLw, hist: &mut [[u64; 4]; 4]) {
    for (mr, s, wire, _) in &f.muxes {
        let Some(half) = mux_half(wire) else { continue };
        for (tr, c, _, src) in &f.taps {
            let Some(k) = tap_idx(src) else { continue };
            let dc = *c as i64 - *s as i64;
            if (-1..=2).contains(&dc) && (k >= 4) == (half == 1) && tr.abs_diff(*mr) <= 9 {
                hist[s % 4][k % 4] += 1;
            }
        }
    }
}

/// Scan `files` in parallel and write the long-wire report to `out_json`.
pub fn run_lw(db: &Device, files: &[String], out_json: &str) -> Option<()> {
    let scanned: Vec<FileLw> = files.par_iter().map(|p| scan_file(db, p)).collect();
    let mut hist = [[0u64; 4]; 4];
    for f in &scanned {
        add_hist(f, &mut hist);
        eprintln!("{}: muxes {} taps {} clock {}", f.path, f.muxes.len(), f.taps.len(), f.clock.len());
    }
    for (m, row) in hist.iter().enumerate() {
        eprintln!("s%4={m} -> k%4 counts {:?}", row);
    }
    let rep = LwReport { files: scanned, hist };
    let s = serde_json::to_string(&rep).map_err(|e| eprintln!("json: {e}")).ok()?;
    std::fs::write(out_json, s).map_err(|e| eprintln!("{out_json}: {e}")).ok()?;
    Some(())
}

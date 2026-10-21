//! `--pipdiff <base.fs>`: active pips added/removed per bitstream relative to a base bitstream.
//!
//! Used for single-change probe sweeps (e.g. one hard-macro port tied or routed per build):
//! the pips that differ from the base name the fabric wires the changed port uses.

use crate::bitstream;
use crate::chipdb::Device;
use crate::tile::{pips, Pips};
use rayon::prelude::*;
use serde::Serialize;
use std::collections::BTreeSet;

/// One active pip: (row, col, dest, src).
type Rec = (usize, usize, String, String);

#[derive(Serialize, Default)]
struct FileDiff {
    path: String,
    added: Vec<Rec>,
    removed: Vec<Rec>,
}

/// All active pips (fabric and clock tables) of one bitstream.
fn active_pips(db: &Device, path: &str) -> BTreeSet<Rec> {
    let mut out = BTreeSet::new();
    let Some(bm) = bitstream::read_bitstream(path) else { return out };
    for ((r, c), tb) in bitstream::tile_bitmap(db, &bm) {
        let td = db.tile(r, c);
        let mut p = Pips::new();
        pips(&tb, &td.pips, false, &mut p);
        pips(&tb, &td.clock_pips, false, &mut p);
        out.extend(p.into_iter().map(|(d, s)| (r, c, d, s)));
    }
    out
}

/// Diff every file against `base` in parallel and write the report to `out_json`.
pub fn run_pipdiff(db: &Device, base: &str, files: &[String], out_json: &str) -> Option<()> {
    let b = active_pips(db, base);
    if b.is_empty() {
        eprintln!("{base}: no active pips (unreadable?)");
        return None;
    }
    let diffs: Vec<FileDiff> = files
        .par_iter()
        .map(|p| {
            let cur = active_pips(db, p);
            FileDiff {
                path: p.clone(),
                added: cur.difference(&b).cloned().collect(),
                removed: b.difference(&cur).cloned().collect(),
            }
        })
        .collect();
    for d in &diffs {
        eprintln!("{}: +{} -{}", d.path, d.added.len(), d.removed.len());
    }
    let s = serde_json::to_string(&diffs).map_err(|e| eprintln!("json: {e}")).ok()?;
    std::fs::write(out_json, s).map_err(|e| eprintln!("{out_json}: {e}")).ok()?;
    Some(())
}

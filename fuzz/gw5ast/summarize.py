#!/usr/bin/env python3
"""summarize.py — collect all fuzzed cells' bits.json + the differential param fuses
into one fold-ready table.  Run: $PY summarize.py
Reads $FUZZ_WORK/<cell>/bits.json (full diffs) and prints per-cell bel-row localization.
The hand-recorded param-fuse table lives in FUSES_EXTRACTED.md (differential results).
"""
import os, json, glob
from collections import Counter

work = os.environ.get("FUZZ_WORK", "/tmp/gw5ast_fuzz")
cells = sorted(glob.glob(f"{work}/*/bits.json"))
print(f"=== fuzzed cells in {work} ===")
for path in cells:
    name = os.path.basename(os.path.dirname(path))
    try:
        d = json.load(open(path))
    except Exception:
        continue
    bits = d.get("bits", [])
    if not bits:
        print(f"  {name:16s}: 0 bits"); continue
    rows = Counter(b[0] for b in bits)
    top = rows.most_common(3)
    rmin, rmax = min(rows), max(rows)
    print(f"  {name:16s}: {d['n_bits']:5d} bits | rows {rmin}-{rmax} | densest rows {top}")

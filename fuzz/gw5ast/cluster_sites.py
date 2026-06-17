#!/usr/bin/env python3
"""cluster_sites.py — given a multi-instance fuzz diff (bits.json), cluster the set
bits into distinct site tiles. Each cell-unique tile with a strong bit count = one
placed instance's location. Filters common/config tiles (shared by all instances).
  $PY cluster_sites.py <bits.json>
"""
import sys, os, json, bisect
sys.path.insert(0, os.environ.get("APICULA_ROOT", "."))
from apycula.chipdb import load_chipdb
from collections import Counter

root = os.environ.get("APICULA_ROOT", ".")
dev = os.environ.get("GW5AST_DEV", "GW5AST-138C")
db = load_chipdb(f"{root}/apycula/{dev}.msgpack.xz")
ro = [0]
for r in range(db.rows): ro.append(ro[-1] + db[r, 0].height)
co = [0]
for c in range(db.cols): co.append(co[-1] + db[0, c].width)

def tile(R, C):
    return bisect.bisect_right(ro, R) - 1, bisect.bisect_right(co, C) - 1

bits = json.load(open(sys.argv[1]))["bits"]
tc = Counter()
for b in bits:
    r, bytecol = b[0], b[2]
    tc[tile(r, bytecol)] += 1
# sites = tiles with a meaningful bit count, grouped by ttyp
sites = [(t, n) for t, n in tc.items() if n >= 3]
sites.sort(key=lambda x: (-x[1]))
by_ttyp = Counter(db.grid[t[0]][t[1]] for t, n in sites)
print(f"== {len(bits)} bits over {len(tc)} tiles; {len(sites)} sites (>=3 bits) ==")
print("  site tiles by ttyp:", dict(by_ttyp.most_common()))
print("  distinct site tiles (row,col,ttyp,bits):")
for (tr, tcc), n in sites[:60]:
    print(f"    ({tr},{tcc}) tt{db.grid[tr][tcc]}: {n}")
json.dump([[t[0], t[1], db.grid[t[0]][t[1]], n] for t, n in sites],
          open(sys.argv[1].replace("bits.json", "sites.json"), "w"))

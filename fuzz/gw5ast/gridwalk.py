#!/usr/bin/env python3
"""gridwalk.py — map absolute bitstream (row,col) from diff_bits.py to chipdb
tile (tile_row, tile_col, local_row, local_col), accounting for the 5A-series
fliplr+transpose orientation in bslib.read_bitstream.

Usage: $PY gridwalk.py <cell>   (reads $FUZZ_WORK/<cell>/bits.json)
Prints the tile(s) the cell's fuses land on + local coords -> the fold target.
"""
import sys, os, json
sys.path.insert(0, os.environ.get("APICULA_ROOT", "."))
from apycula.chipdb import load_chipdb

root = os.environ.get("APICULA_ROOT", ".")
dev = os.environ.get("GW5AST_DEV", "GW5AST-138C")
db = load_chipdb(f"{root}/apycula/{dev}.msgpack.xz")

# cumulative tile bit-offsets (gridwalk)
row_off = [0]
for r in range(db.rows): row_off.append(row_off[-1] + db[r, 0].height)
col_off = [0]
for c in range(db.cols): col_off.append(col_off[-1] + db[0, c].width)
BR, BC = row_off[-1], col_off[-1]   # 1513 x 21872 for 138C

import bisect
def to_tile(R, C):
    if not (0 <= R < BR and 0 <= C < BC): return None
    tr = bisect.bisect_right(row_off, R) - 1
    tc = bisect.bisect_right(col_off, C) - 1
    return (tr, tc, R - row_off[tr], C - col_off[tc], db.grid[tr][tc])

def in_bounds_count(bits, swap):
    n = 0
    for a, b in bits:
        R, C = (b, a) if swap else (a, b)
        if 0 <= R < BR and 0 <= C < BC: n += 1
    return n

# diff_bits emits (a,b) in the post-read (fliplr+transpose) frame.  For 5A the
# bitmap is TRANSPOSED, so gridwalk (row,col) = (b,a).  We report both and let the
# tile-type (ttyp) disambiguate which orientation is physically right per cell.
def main():
    cell = sys.argv[1]
    raw = json.load(open(f"{os.environ['FUZZ_WORK']}/{cell}/bits.json"))["bits"]
    # bits.json fmt: [row, bitcol, bytecol, bitpos]. Use (row, bytecol).
    bits = [(b[0], b[2]) for b in raw] if raw and len(raw[0]) >= 3 else [(b[0], b[1]) for b in raw]
    from collections import Counter
    nd, nt = in_bounds_count(bits, False), in_bounds_count(bits, True)
    swap = nt > nd
    orient = "TRANSPOSED (b,a)" if swap else "DIRECT (a,b)"
    tiles = Counter()
    for a, b in bits:
        R, C = (b, a) if swap else (a, b)
        t = to_tile(R, C)
        if t: tiles[(t[0], t[1], t[4])] += 1
    print(f"== {cell}: {len(bits)} bits | orient={orient} (in-bounds direct={nd} transp={nt}) ==")
    for (tr, tc, ty), n in tiles.most_common(6):
        print(f"    tile({tr},{tc}) ttyp={ty}: {n} bits")

if __name__ == "__main__":
    main()

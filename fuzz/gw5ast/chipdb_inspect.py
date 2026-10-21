#!/usr/bin/env python3
"""inspect.py — report what infra/cells the GW5AST chipdb already has vs. missing.
Run with the apicula venv:  $PY fuzz/gw5ast/inspect.py [device]
Reusable coverage probe so we don't re-derive the loader each step.
"""
import sys, os
sys.path.insert(0, os.environ.get("APICULA_ROOT", "."))
from apycula.chipdb import load_chipdb

dev = sys.argv[1] if len(sys.argv) > 1 else os.environ.get("GW5AST_DEV", "GW5AST-138C")
root = os.environ.get("APICULA_ROOT", ".")
path = f"{root}/apycula/{dev}.msgpack.xz"
db = load_chipdb(path)
print(f"=== {dev} chipdb: {path} ===")
print("rows x cols:", getattr(db, "rows", "?"), "x", getattr(db, "cols", "?"))

# top-level structures present
for attr in ["hclk_pips", "extra_func", "grid", "timing", "pll", "iologic",
             "bels", "primitives", "packages"]:
    v = getattr(db, attr, "__MISSING__")
    if v == "__MISSING__":
        print(f"  {attr:14s}: -- absent --")
    else:
        try: print(f"  {attr:14s}: present (len={len(v)})")
        except TypeError: print(f"  {attr:14s}: present")

# extra_func kinds (dhcen/dqs/dll/osc/dcs live here)
ef = getattr(db, "extra_func", {}) or {}
kinds = {}
items = ef.items() if hasattr(ef, "items") else []
for loc, d in items:
    if isinstance(d, dict):
        for k in d:
            kinds[k] = kinds.get(k, 0) + 1
print("  extra_func kinds:", dict(sorted(kinds.items())) or "(none)")

# scan bels across grid for primitive types present
beltypes = {}
grid = getattr(db, "grid", None)
if grid is not None:
    try:
        for row in grid:
            for tile in row:
                for bel in getattr(tile, "bels", {}) or {}:
                    beltypes[bel] = beltypes.get(bel, 0) + 1
    except Exception as e:
        print("  (grid bel scan skipped:", e, ")")
# also try db as 2D dict
if not beltypes:
    try:
        for (r, c), tile in db.items() if hasattr(db, "items") else []:
            for bel in getattr(tile, "bels", {}) or {}:
                beltypes[bel] = beltypes.get(bel, 0) + 1
    except Exception:
        pass
print("  bel types present:", dict(sorted(beltypes.items())) if beltypes else "(none found via scan)")

# the target cells we want to fuzz
WANT = ["DQS", "DDRDLL", "IODELAY", "DCS", "DHCEN", "SDPB", "DLLDLY", "OSER8_MEM", "IDES8_MEM"]
print("\n=== target-cell coverage ===")
have = set(kinds) | set(beltypes)
for w in WANT:
    present = any(w.lower() in str(h).lower() for h in have)
    print(f"  {w:12s}: {'HAVE' if present else 'MISSING'}")

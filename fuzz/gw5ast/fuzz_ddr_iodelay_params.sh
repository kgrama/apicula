#!/usr/bin/env bash
# fuzz_ddr_iodelay_params.sh — complete the IODELAY model: sweep the two remaining params.
#
# C_STATIC_DLY is already SOLVED (fuzz_ddr_iodelay.sh: 7-bit BINARY field, local row 20 cols 0..6,
# LSB at the HIGHEST col).  The prim has exactly two more params, both currently UNFUZZED:
#   DYN_DLY_EN  FALSE/TRUE — dynamic delay enable (gates the DLYSTEP[7:0] stepping port)
#   ADAPT_EN    FALSE/TRUE — adaptive delay
# Both are "FALSE" in what the open LiteDRAM GW5DDRPHY emits (gw5ddrphy.py:211-214), so they're
# not on the open-DDR critical path — this is for a COMPLETE bel model.
#
# METHOD: one baseline (all defaults) + one variant per param flipped to TRUE.  Only that param
# changes -> XOR-diff isolates its fuse.  4 builds incl. a C_STATIC_DLY cross-check.
#
# OUT: $FUZZ_WORK/ddr_iodelay_params/<tag>.fs + results.csv, then auto-diffs each vs baseline.
# Usage: ./fuzz_ddr_iodelay_params.sh
set -uo pipefail
. "$(dirname "$0")/env.sh"
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$FUZZ_WORK/ddr_iodelay_params"; mkdir -p "$OUT"
CSV="$OUT/results.csv"
[ -f "$CSV" ] || echo "variant,status,secs" > "$CSV"
TIMEOUT="${TIMEOUT:-600}"
BASEV="$HERE/cells/iodelay_params.v"

# tag : sed expression applied to the baseline fixture (flip exactly one param)
declare -A VAR=(
  [base]='s/XXNOOPXX/XXNOOPXX/'                                   # unchanged baseline
  [dyn1]='s/\.DYN_DLY_EN("FALSE")/.DYN_DLY_EN("TRUE")/'           # dynamic delay ON
  [adapt1]='s/\.ADAPT_EN("FALSE")/.ADAPT_EN("TRUE")/'             # adaptive delay ON
  [dly40]='s/\.C_STATIC_DLY(0)/.C_STATIC_DLY(40)/'                # cross-check vs the solved field
)

echo "=== IODELAY param sweep -> $OUT ==="
for TAG in base dyn1 adapt1 dly40; do
    FSOUT="$OUT/${TAG}.fs"
    if [ -f "$FSOUT" ]; then echo "  $TAG: cached"; continue; fi
    V="$OUT/${TAG}.v"
    sed "${VAR[$TAG]}" "$BASEV" > "$V"
    T0=$SECONDS
    FS=$(timeout "$TIMEOUT" "$HERE/oracle.sh" "$V" top 2>"$OUT/${TAG}.err")
    SECS=$((SECONDS-T0))
    if [ -n "${FS:-}" ] && [ -f "$FS" ]; then
        cp -f "$FS" "$FSOUT"; echo "  $TAG: ok (${SECS}s)"; echo "$TAG,ok,$SECS" >> "$CSV"
    else
        echo "  $TAG: FAIL (${SECS}s) — see $OUT/${TAG}.err"; echo "$TAG,FAIL,$SECS" >> "$CSV"
    fi
done

# ── differentials vs the baseline: each should isolate that param's fuse ──
B="$OUT/base.fs"
if [ -f "$B" ]; then
    echo "=== diff vs base (bits = that param's fuse) ==="
    for TAG in dyn1 adapt1 dly40; do
        F="$OUT/${TAG}.fs"; [ -f "$F" ] || continue
        echo "  --- $TAG ---"
        "$PY" "$HERE/diff_bits.py" "$B" "$F" 2>/dev/null | "$PY" -c '
import sys,json
d=json.load(sys.stdin)
print("      n_bits:", d["n_bits"])
# print [row, bitcol, bytecol, bitpos] — the chipdb attr frame
for b in d["bits"][:10]: print("      ", b)
' 2>/dev/null
    done
    echo "NOTE: dly40 should be 2 bits (w32+w8) confirming the solved binary field."
fi

#!/usr/bin/env bash
# fuzz_ddr_iodelay.sh — 20-variant IODELAY C_STATIC_DLY sweep -> 20 .fs for fuse diffing.
#
# WHY: the GW5A IODELAY is the open-DDR blocker.  nextpnr-himbaechel models IODELAY with GW1N
# semantics (a SETN port) while the GW5A prim uses DLYSTEP[7:0] (pack_iologic.cc:633 moves SETN,
# never DLYSTEP) — and the open LiteDRAM GW5DDRPHY instantiates 53 IODELAYs.  Extracting the real
# delay-tap fuse field is step 1 to modelling it properly.
#
# METHOD: rebuild cells/iodelay.v with C_STATIC_DLY swept over 20 points (0..127).  Only that
# param changes, so a pairwise XOR of the .fs isolates the delay-tap bits.  Memory says the field
# is a 7-bit thermometer at cfg-tile local row 20, cols 0..6 — this sweep validates/completes it
# and shows the encoding (thermometer vs binary) across the full range.
#
# OUT: $FUZZ_WORK/ddr_iodelay/dly<N>.fs        (20 bitstreams)
#      $FUZZ_WORK/ddr_iodelay/results.csv      (variant, status, secs)
#      then:  $PY diff_bits.py dly0.fs dly<N>.fs   -> the tap bits for value N
#
# Usage: ./fuzz_ddr_iodelay.sh              # all 20
#        ONLY_DLY=40 ./fuzz_ddr_iodelay.sh  # single point (spot check)
# A per-variant timeout is NOT fatal: a wedge/FAIL is recorded and the sweep continues.
set -uo pipefail
. "$(dirname "$0")/env.sh"
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$FUZZ_WORK/ddr_iodelay"; mkdir -p "$OUT"
CSV="$OUT/results.csv"
[ -f "$CSV" ] || echo "variant,c_static_dly,status,secs" > "$CSV"

TIMEOUT="${TIMEOUT:-600}"

# 20 sweep points across the 0..127 tap range: dense at the low end (where the thermometer
# field turns on bit-by-bit) + powers-of-two boundaries + the extremes.
DLYS="${ONLY_DLY:-0 1 2 3 4 6 8 12 16 24 32 40 48 63 64 80 96 112 126 127}"

echo "=== IODELAY C_STATIC_DLY sweep -> $OUT ==="
for D in $DLYS; do
    TAG="dly${D}"
    FSOUT="$OUT/${TAG}.fs"
    if [ -f "$FSOUT" ]; then echo "  $TAG: cached"; continue; fi

    # fixture variant: same file, only C_STATIC_DLY differs
    V="$OUT/${TAG}.v"
    sed "s/\.C_STATIC_DLY([0-9]*)/.C_STATIC_DLY($D)/" "$HERE/cells/iodelay.v" > "$V"

    T0=$SECONDS
    FS=$(timeout "$TIMEOUT" "$HERE/oracle.sh" "$V" top 2>"$OUT/${TAG}.err")
    SECS=$((SECONDS-T0))

    if [ -n "${FS:-}" ] && [ -f "$FS" ]; then
        cp -f "$FS" "$FSOUT"
        echo "  $TAG: ok (${SECS}s)"
        echo "$TAG,$D,ok,$SECS" >> "$CSV"
    else
        echo "  $TAG: FAIL (${SECS}s) — see $OUT/${TAG}.err"
        echo "$TAG,$D,FAIL,$SECS" >> "$CSV"
    fi
done

echo "=== done: $(ls "$OUT"/*.fs 2>/dev/null | wc -l)/$(echo $DLYS | wc -w) bitstreams -> $OUT ==="
# quick differential summary vs the dly0 baseline (the tap field for each value)
BASE="$OUT/dly0.fs"
if [ -f "$BASE" ]; then
    echo "=== tap bits vs dly0 (diff_bits.py) ==="
    for F in "$OUT"/dly*.fs; do
        [ "$F" = "$BASE" ] && continue
        N=$("$PY" "$HERE/diff_bits.py" "$BASE" "$F" 2>/dev/null | "$PY" -c 'import sys,json; print(json.load(sys.stdin)["n_bits"])' 2>/dev/null)
        echo "  $(basename "$F" .fs): ${N:-?} bits"
    done
fi

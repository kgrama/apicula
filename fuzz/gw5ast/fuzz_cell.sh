#!/usr/bin/env bash
# fuzz_cell.sh — fuzz ONE primitive end-to-end: build empty baseline + cell design,
# run both through the oracle, diff -> the cell's fuse bits.
# Usage: ./fuzz_cell.sh <cellname>   (expects cells/<cellname>.v with module `top`)
# Output: $FUZZ_WORK/<cellname>/bits.json  (set-bit coords) + the two .fs
set -euo pipefail
. "$(dirname "$0")/env.sh"
HERE="$(dirname "$0")"
CELL="$1"
CELLV="$HERE/cells/$CELL.v"
[ -f "$CELLV" ] || { echo "no cell fixture: $CELLV" >&2; exit 2; }

OUT="$FUZZ_WORK/$CELL"; mkdir -p "$OUT"

# 1. empty baseline (cells/empty.v) — built once, cached
if [ ! -f "$OUT/../empty.fs" ]; then
  EFS="$("$HERE/oracle.sh" "$HERE/cells/empty.v" top)"
  install -m 644 "$EFS" "$OUT/../empty.fs"   # oracle .fs is read-only; install resets mode
fi

# 2. the cell design
CFS="$("$HERE/oracle.sh" "$CELLV" top)"
install -m 644 "$CFS" "$OUT/cell.fs"

# 3. diff -> fuse bits
"$PY" "$HERE/diff_bits.py" "$OUT/../empty.fs" "$OUT/cell.fs" > "$OUT/bits.json"
echo "== $CELL ==> $OUT/bits.json"
"$PY" -c "import json;d=json.load(open('$OUT/bits.json'));print('  set bits:',d['n_bits'])"

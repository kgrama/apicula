#!/usr/bin/env bash
# enum_sites.sh — place N instances of a cell so the Gowin placer spreads them across
# ALL sites, then gridwalk each instance to map every site location.
# Usage: ./enum_sites.sh <cellname> <N>   (uses cells/<cellname>_multi template if present,
#        else generates a generate-loop wrapper around cells/<cellname>.v's primitive)
# Output: $FUZZ_WORK/<cellname>_multi/sites.json  (list of (tile_row, tile_col) per instance)
set -euo pipefail
. "$(dirname "$0")/env.sh"
HERE="$(dirname "$0")"
CELL="$1"; N="${2:-16}"
MULTI="$HERE/cells/${CELL}_multi.v"
[ -f "$MULTI" ] || { echo "need a multi-instance fixture: $MULTI" >&2; exit 2; }

OUT="$FUZZ_WORK/${CELL}_multi"; mkdir -p "$OUT"
# baseline reused
[ -f "$FUZZ_WORK/empty.fs" ] || { EFS="$("$HERE/oracle.sh" "$HERE/cells/empty.v" top)"; install -m644 "$EFS" "$FUZZ_WORK/empty.fs"; }
CFS="$("$HERE/oracle.sh" "$MULTI" top)"
install -m644 "$CFS" "$OUT/cell.fs"
"$PY" "$HERE/diff_bits.py" "$FUZZ_WORK/empty.fs" "$OUT/cell.fs" > "$OUT/bits.json"
echo "== ${CELL} x${N} ==> $OUT/bits.json"
"$PY" "$HERE/cluster_sites.py" "$OUT/bits.json"

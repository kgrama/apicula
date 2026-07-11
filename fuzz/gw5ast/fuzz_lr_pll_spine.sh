#!/usr/bin/env bash
# fuzz_lr_pll_spine.sh — extract per-site L/R PLL output-spine routing for GW5AST-138C.
#
# For each of the 8 L/R PLL sites (PLL_L[0..3], PLL_R[0..3]) build the SAME PLL design
# (CLKOUT1 drives fabric registers so the vendor commits a real global-spine clock route),
# pinned to that site via INS_LOC.  The resulting .fs bitstreams differ ONLY in the
# site-specific placement + the spine wire each site's CLKOUT drives.  Pairwise XOR-diff
# (diff_bits.py) isolates those bits; the analysis step maps them to {TL,TR}PLL{0,1}CLK.
#
# Usage: ./fuzz_lr_pll_spine.sh          -> builds all 8, saves .fs into $OUT, prints paths
set -uo pipefail
. "$(dirname "$0")/env.sh"

FIX="${FIX:-/tmp/claude-1000/-home-evd-workspace-sbp-mc-walk/8ec92b22-e952-4065-824c-9dc4bc21519d/scratchpad/pll_spine.v}"
OUT="${OUT:-$FUZZ_WORK/lr_spine}"
mkdir -p "$OUT"
[ -f "$FIX" ] || { echo "missing fixture $FIX" >&2; exit 2; }

SITES="PLL_L[0] PLL_L[1] PLL_L[2] PLL_L[3] PLL_R[0] PLL_R[1] PLL_R[2] PLL_R[3]"

build_one() {
  local site="$1"
  local tag="${site//[\[\]]/}"           # PLL_L[0] -> PLL_L0
  local cst="$OUT/$tag.cst"
  printf 'INS_LOC "u_pll/PLL_inst" %s;\n' "$site" > "$cst"
  local fs
  fs=$(timeout 300 "$(dirname "$0")/oracle.sh" "$FIX" top "$cst" 2>"$OUT/$tag.err")
  if [ -n "$fs" ] && [ -f "$fs" ]; then
    cp "$fs" "$OUT/$tag.fs"
    echo "OK   $site -> $OUT/$tag.fs"
  else
    echo "FAIL $site (see $OUT/$tag.err)"
  fi
}

echo "=== building 8 L/R PLL sites into $OUT ==="
for s in $SITES; do build_one "$s"; done
echo "=== done. bitstreams: ==="
ls -la "$OUT"/*.fs 2>/dev/null | awk '{print $5, $9}'

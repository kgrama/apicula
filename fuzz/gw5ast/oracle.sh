#!/usr/bin/env bash
# oracle.sh — run ONE Gowin synth+PnR headless, produce a bitstream.
# Usage: ./oracle.sh <design.v> <top_module> [<extra.cst>]  -> prints path to .fs
# Reuses the proven headless invocation from env.sh.  This is the atomic fuzz step:
# feed a primitive instantiation, get back the bitstream to diff.
set -euo pipefail
. "$(dirname "$0")/env.sh"

VFILE="$1"; TOP="${2:-top}"; CST="${3:-}"
[ -f "$VFILE" ] || { echo "no such verilog: $VFILE" >&2; exit 2; }

RUN="$FUZZ_WORK/run_$(basename "$VFILE" .v)_$$"
mkdir -p "$RUN"
cp "$VFILE" "$RUN/design.v"
[ -n "$CST" ] && cp "$CST" "$RUN/design.cst"

cat > "$RUN/run.tcl" <<EOF
set_device $GW5AST_PART -name $GW5AST_NAME
add_file -type verilog design.v
$( [ -n "$CST" ] && echo "add_file -type cst design.cst" )
set_option -top_module $TOP -output_base_name out \\
           -synthesis_tool gowinsynthesis -verilog_std sysv2017
run all
EOF

( cd "$RUN" && QT_QPA_PLATFORM=minimal "$GWSH" run.tcl >gwsh.log 2>&1 ) || true
FS="$RUN/impl/pnr/out.fs"
if [ -f "$FS" ]; then
  echo "$FS"
else
  echo "ORACLE_FAILED (no .fs) — see $RUN/gwsh.log" >&2
  tail -5 "$RUN/gwsh.log" >&2 || true
  exit 1
fi

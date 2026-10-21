#!/usr/bin/env bash
# env.sh — single source of truth for the GW5AST apicula fuzzing environment.
# Source this (`. env.sh`) before any fuzz script.  Encodes the PROVEN headless setup
# (see memory gowin-headless-oracle) so nothing is re-derived each run.

# --- apicula repo root (this file lives in <root>/fuzz/gw5ast) ---
export APICULA_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# --- Gowin: DATA layout vs WORKING oracle are DIFFERENT installs ---
#   GOWINHOME -> the install with IDE/share/device/<dev>/<dev>.fse  (chipdb_builder reads this)
#   GWSH      -> the gw_sh that actually runs headless (the /opt flat one; ~/tools has a
#                fontconfig symbol bug).  codegen.py honors $GWSH.
export GOWINHOME="${GOWINHOME:-/home/evd/tools/gowin}"
export GWSH="${GWSH:-/opt/gowin-eda-ide/bin/gw_sh}"

# --- gw_sh is a TCL shell: run headless with minimal Qt (offscreen->GLX coredump) ---
export QT_QPA_PLATFORM=minimal

# --- python: use apicula venv (has msgpack/msgspec; system pip is PEP668-blocked) ---
export PY="$APICULA_ROOT/.venv/bin/python"

# --- target part ---
export GW5AST_DEV="${GW5AST_DEV:-GW5AST-138C}"
export GW5AST_PART="${GW5AST_PART:-GW5AST-LV138PG484AC1/I0}"
export GW5AST_NAME="${GW5AST_NAME:-GW5AST-138B}"   # -name for set_device

# --- de-vendored primitive sources (spec + fixtures) ---
export DEVENDORED="/home/evd/workspace/sbp-mc-walk/hdl/generators"
# AUTHORITATIVE GW5A primitive port/param spec — grep this for exact ports when a fixture
# hits "Cannot find port" (e.g. DCS uses CLKIN0..3 not CLK0..3; SDPB has one RESET).
#   awk '/^module DQS/,/;/' $GW5A_PRIM
export GW5A_PRIM="/opt/gowin-eda-ide/simlib/gw5a/prim_sim.v"

# --- output/work dirs ---
export FUZZ_DIR="$APICULA_ROOT/fuzz/gw5ast"
export FUZZ_WORK="${FUZZ_WORK:-/tmp/gw5ast_fuzz}"
mkdir -p "$FUZZ_WORK"

fuzz_env_check() {
  echo "APICULA_ROOT=$APICULA_ROOT"
  echo "GOWINHOME=$GOWINHOME  (fse: $( [ -f "$GOWINHOME/IDE/share/device/$GW5AST_DEV/$GW5AST_DEV.fse" ] && echo OK || echo MISSING ))"
  echo "GWSH=$GWSH  ($( [ -x "$GWSH" ] && echo OK || echo MISSING ))"
  echo "PY=$PY  ($( "$PY" -c 'import msgpack' 2>/dev/null && echo 'msgpack OK' || echo 'msgpack MISSING' ))"
  echo "DEV=$GW5AST_DEV PART=$GW5AST_PART"
}

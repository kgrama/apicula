# GW5AST fuzz harness

Reusable scripts to fuzz GW5AST-138 primitives for apicula. Built once so the proven
headless environment isn't re-derived each run. See memory `gowin-headless-oracle`.

## One-time
```
. fuzz/gw5ast/env.sh          # exports GOWINHOME/GWSH/PY/QT_QPA_PLATFORM/part
fuzz_env_check                # sanity: fse present, gw_sh runs, msgpack importable
```

## Files
- `env.sh`       — single source of truth for env (GOWINHOME=~/tools data, GWSH=/opt oracle,
                   QT_QPA_PLATFORM=minimal, venv PY, part numbers, de-vendored path).
- `oracle.sh`    — run ONE Gowin synth+PnR headless on a .v -> prints the .fs path.
- `diff_bits.py` — XOR two .fs -> set-bit coords (the fuse extraction; reuses apycula.bslib).
- `fuzz_cell.sh` — fuzz one cell end-to-end: empty baseline + cell -> bits.json.
- `chipdb_inspect.py`   — report chipdb coverage (which target cells HAVE vs MISSING).
- `cells/*.v`    — primitive fixtures. `empty.v` = baseline. Others instantiate ONE hard
                   primitive using the de-vendored port lists (hdl/generators) as spec.

## Fuzz a cell
```
. env.sh
./fuzz_cell.sh dqs            # -> $FUZZ_WORK/dqs/bits.json
```
To sweep a parameter (e.g. DQS_MODE X4 vs X2_DDR3), make cells/dqs_x4.v / dqs_x2.v and
diff their bits.json to isolate the mode bits.

## Cell fixtures source
The de-vendored controller (hdl/generators/pin-compat/ddr3_1_4code_hs-u.v ~line 5082) and
DDR3_TOP.v give vendor-correct port lists + legal param values for DQS/DDRDLL/IODELAY —
use those verbatim in cells/*.v (no doc guesswork).

## Target cells (MISSING in 138C chipdb)
DHCEN, DCS, IODELAY(thin), DDRDLL, DQS, SDPB BSRAM. Prereq for DQS/DDR-PHY: HCLK->FCLK map.

## Verify
After folding fuzzed bits into the chipdb (`make GW5AST-138C` style), compare a
nextpnr-built bitstream of the de-vendored controller against a silicon-proven vendor .fs
(the BPLL re-bake method).

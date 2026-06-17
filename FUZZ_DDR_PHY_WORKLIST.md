# Fuzz DDR-PHY primitives for GW5AST-138C — worklist

Pipeline VERIFIED working headless (see memory gowin-headless-oracle):
  GOWINHOME=~/tools/gowin  GWSH=/opt/gowin-eda-ide/bin/gw_sh  .venv/bin/python -m apycula.chipdb_builder GW5AST-138C
Patched apycula/codegen.py to use GWSH env + QT_QPA_PLATFORM=minimal (no Xvfb).

## Root gap
- `SDRAM_PARAMS` (chipdb_builder.py:71) covers only GW1NS-4/GW1N-9/GW1N-9C/GW2A-18C,
  and those are **HyperRAM** (O_hpram_*), NOT DDR3.  GW5AST-138C has NO entry -> the
  SDRAM/DQS fuzz path (`run_sdram_script`, gated `device in SDRAM_PARAMS`) never runs
  for our part -> DQS/DDRDLL/IODELAY unfuzzed.

## Missing primitives (chipdb_builder/chipdb.py "for now" gaps) relevant to our design
- DQS / DDRDLL / IODELAY  (DDR3 PHY)        <- de-vendored ctrl provides spec+fixture
- DCS   (chipdb.py:1324 "no DCS for now")   <- controller uses clock mux
- DHCEN (chipdb.py:2435 "No DHCEN for now")
- SDPB BSRAM (apicula 0 files)             <- the srcbuf / framebuffer BRAM

## De-vendored sources = spec + fixture (hdl/generators/pin-compat)
- DDR3_TOP.v: DDR3 pin list (IO_ddr_dqs[DQS_WIDTH], IO_ddr_dqs_n, ddr_addr/ba/dq/dm...)
- ddr3_1_4code_hs-u.v ~line 5082: DQS primitive instance —
    .DQS_MODE("X4"/"X2_DDR3") .DQSIN .READ .RCLKSEL .DLLSTEP .WSTEP .DQSR90 .DQSW0 .DQSW270
  -> exact ports + modes to sweep; vendor-correct (no doc guesswork).

## Steps (autonomous)
1. Add GW5AST-138C entry to SDRAM_PARAMS with the DDR3 pin list (from DDR3_TOP.v +
   the board .cst pin assignments for ddr_dqs/dq/addr).
2. Teach run_sdram_script the DDR3 DQS topology (X4 byte-lanes) — adapt from the
   HyperRAM path; DQS strobes per byte-lane vs hpram rwds.
3. Add DQS/DDRDLL/IODELAY bels: clone the IODELAY/IOLOGIC pack pattern; sweep DQS_MODE
   + delay params; diff bitstreams via the oracle to extract fuses.
4. Fold into chipdb; rebuild GW5AST-138C.msgpack.xz.
5. VERIFY vs silicon-proven vendor bitstream of the de-vendored controller (the BPLL
   method: re-bake/compare against a known-good vendor .fs).

## Verify oracle works for a cell-fuzz (already proven for full chipdb build)
A trivial set_device GW5AST-138B + run all -> tiny.fs completed rc=0 headless.

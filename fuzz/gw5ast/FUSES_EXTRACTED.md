# Extracted fuse tables (differential fuzz) — GW5AST-138C

Method: two designs differing by ONE param, XOR cancels routing -> exact fuse (row,col).
Fixtures from de-vendored controller (vendor-correct ports). All confirmed.

## DDRDLL  (bel @ rows 18-19, bottom I/O region)
- DIV_SEL  0->1            : [18,479]
- CODESCAL bit (111->101)  : [18,455]
- CODESCAL 111->000 (3-bit): [18,455],[18,503],[18,599]   (field cols 455/503/599)
- SCAL_EN  FALSE->true     : [19,583]
- DLL_FORCE FALSE->true    : [19,495]

## IODELAY (bel @ row 1511)
- C_STATIC_DLY: **7-bit BINARY-weighted** field (NOT thermometer — earlier note was wrong),
  row 1511, bit-cols 135495..135543 step 8 (= byte-cols 16936..16942, bitpos 7; tile-local row 20
  cols 0..6).  **LSB is at the HIGHEST col**: 135543=w1, 135535=w2, 135527=w4, 135519=w8,
  135511=w16, 135503=w32, 135495=w64.
  PROVEN by the 20-point sweep (fuzz_ddr_iodelay.sh): diff-bit COUNT == popcount(value) at every
  point — dly1/2/4/8/16/32/64 -> 1 bit each; dly3/6/12 -> 2; dly127 -> all 7 (exact match).
  0->40  : cols 135503,135519          (= w32 + w8 = 40 ✓ binary, not a thermometer prefix)
  0->127 : cols 135495,135503,135511,135519,135527,135535,135543 (full field)

## DQS (bel @ row 1011)
- DQS_MODE     X4->X2_DDR3 : [1011,639]
- HWL          true->false : [1011,831]
- FIFO_MODE_SEL 0->1       : [1011,295],[1011,543]

## OSER10 (HDMI TMDS serializer) — 299 bits full diff; param sweep TODO (D-slot map)
## ELVDS_OBUF (HDMI diff pads) — 163 bits full diff; localizes the diff-buffer bel

## PLL TRIM (lock-quality fuses!) — bel @ row 386.  Fixes the auto-calc-wrong lock bug.
- ICP_SEL (charge pump, 6-bit): row 386, cols 1423,1431,1439,1447,1455,1463 (step 8)
- LPF_RES (loop-filter R, 3-bit): row 386, cols 1583,1591,1599 (step 8)
- LPF_CAP (2-bit): 00->11 moved 0 bits (unused on this variant, or 00==11 phys; sweep 01/10 to confirm)
- USE: set ICP_SEL/LPF_RES directly from a silicon-proven .fs instead of trusting vendor
  auto-calc (the 4.71MHz lock failure was wrong trim).  Sweep 6-bit ICP x 3-bit LPF for
  best lock margin at the actual VCO.

## NOTES
- DQS requires a DDRDLL driving DLLSTEP (Gowin PR0015) — fixture instantiates both.
- These are PARAM fuses. Full bel modeling also needs: bel location (row above),
  portmap (input/output wire names — trace from the ~200/2499-bit full diff), and the
  routing pips to reach the cell. Param fuses + location are the chipdb fuse-table core.

## USB_CDR_SERDES (soft-PHY CDR, ttyp-227 @ (82,89)) — HS/FS + op_mode axes (2026-07-10)
Fixtures hdl/usb2-soft-console/fuzz_softphy/run_v0..v4 (utmi_op_mode_i / utmi_xcvrselect_i swept
as constants). diff_bits.py XOR:
- op_mode  v0-v1: **172 bits**, TIGHT (22 bit-rows, 1157-1167).  v0-v2: 224.
- HS<->FS  v0-v4: **27250 bits**, 126 rows, concentrated 1173-1180 (~1000 bits/row) + fabric tail.
INTERPRETATION: both are UTMI **runtime INPUT PINS** (op_mode[1:0], xcvrselect[1:0]), NOT bel
param-fuses.  xcvrselect (HS/FS) re-synthesizes soft-PHY DATAPATH logic -> the diff is fabric-
dominated, so it must NOT be modeled as an attr-fuse mode.  op_mode's small localized diff is the
constant tie-off + normal/loopback mux.  Recorded in chipdb.py USB_CDR_SERDES['utmi_pins'] as the
control axis + measured locality; the pin<->wire portmap trace is the remaining refinement.

## TODO next
- OSER10, ELVDS_OBUF (HDMI), DCS, DHCEN, SDPB.
- Fold (attr -> {bits}) into apycula chipdb fuse tables for these cells, keyed by tile.
- Verify vs silicon-proven vendor .fs.

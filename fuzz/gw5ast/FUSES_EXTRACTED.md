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
- C_STATIC_DLY: thermometer field, row 1511, cols 135495..135543 step 8 (7 bits)
  0->40  : cols 135503,135519
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

## TODO next
- OSER10, ELVDS_OBUF (HDMI), DCS, DHCEN, SDPB.
- Fold (attr -> {bits}) into apycula chipdb fuse tables for these cells, keyed by tile.
- Verify vs silicon-proven vendor .fs.

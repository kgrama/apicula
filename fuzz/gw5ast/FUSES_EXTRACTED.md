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
- DYN_DLY_EN / ADAPT_EN: **NOT config fuses — do not try to reduce them to (row,col,bit) attrs.**
  fuzz_ddr_iodelay_params.sh (cells/iodelay_params.v, one param flipped at a time vs a baseline):
  FALSE->TRUE moves **189 / 191 bits across 32 rows (1419..1512)** — i.e. flipping them
  INSTANTIATES the dynamic/adaptive delay HARDWARE (tap counters/FSM, ~90 bits clustered at rows
  1491-1500), not a mode bit.  Location-only/no-attr is the CORRECT model.  Not on the open-DDR
  critical path either: the open LiteDRAM GW5DDRPHY hardwires DYN_DLY_EN="FALSE" ADAPT_EN="FALSE"
  and ties DLYSTEP/SDTAP/VALUE to 0 (gw5ddrphy.py:211-217) — only C_STATIC_DLY (=cmd_delay) varies.
  => the IODELAY bel model is COMPLETE for real use.

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

## DDRDLL LOCK / DQS exit (2026-10-21)
- `cells/ddrdll_lock_ff.v` (DDRDLL LOCK into DFF): **680 set bits**. Source: `/tmp/gw5ast_fuzz/ddrdll_lock_ff/bits.json`.
- `cells/dqs_exit_ff.v` (DQS RVALID, RBURST, RFLAG, WFLAG into DFFs): **1915 set bits**. Source: `/tmp/gw5ast_fuzz/dqs_exit_ff/bits.json`.
- Both counts are whole-fixture totals. Coordinates per exit wire (RVALID, RBURST, RFLAG, WFLAG, LOCK) are not yet split out.
- Before this section, this file had no LOCK or RBURST/RVALID/RFLAG/WFLAG entries.

## GTR12_QUAD (top-row SerDes)
- `cells/gtr_fabric_ff.v` and `cells/gtr_upar_ff.v` fail in the vendor synth/PnR step with `ERROR (CM2031) : csr file should be specified.` Logs: `/tmp/gw5ast_fuzz/run_gtr_fabric_ff_1501907/gwsh.log:844` and `/tmp/gw5ast_fuzz/run_gtr_upar_ff_1501959/gwsh.log:848`.
- `oracle.sh` accepts only a `.cst` extra argument, with no CSR option. No `.fs` was produced, so there are no bit counts.
- Status: blocked on the gw_sh option name for the CSR file. A search for "SerDesCSR" found only Python SDK module names (gw5at_sdk), not a gw_sh option.

## USB pins (pico_usb)
Per-pin harness: `usb_pins/` holds `usb_base.v`/`.cst` (baseline) and one `.v`/`.cst` per pin for usb_rxdp_i, usb_rxdn_i, usb_pullup_en_o, usb_dxp_io, usb_dxn_io, usb_term_dp_io, usb_term_dn_io.

| Port | Pad | Site | IO type | Dir |
|---|---|---|---|---|
| usb_rxdp_i | H17 | IOR96[A] | LVDS25 | in |
| usb_rxdn_i | H20 | IOR69[A] | LVDS25 | in |
| usb_pullup_en_o | M17 | IOR73[A] | LVCMOS33 | out |
| usb_dxp_io | J19 | IOR80[A] | LVPECL33E | io |
| usb_dxn_io | H19 | IOR80[B] | LVPECL33E | io |
| usb_term_dp_io | J16 | IOR109[A] | LVCMOS33 | io |
| usb_term_dn_io | AB16 | IOB80[A] (bank 5) | LVCMOS33 | io |

Source: pico_usb pin report `impl/pnr/pico_usb.pin.html` (identical rows in `pico_usb_axi_system.pin.html`), under `io_base/sbp-computational`. The report comes from an earlier pnr run.

Caveats (from `pico_usb.cst`):
- usb_dxn_io has no separate IO_LOC. Only the usb_dxp_io line lists J19,H19.
- The rxdp/rxdn LVDS pairs list a second pad in the cst (H18, G20) that the pin reports do not show.

Per-pin set-bit counts (XOR vs `usb_base`, Gowin oracle GW5AST-138C, one run each, 2026-10-21; `/tmp/usb_pins/summary.txt` ends with DONE):

| Pin | Set bits |
|---|---|
| usb_rxdp_i | 6 |
| usb_rxdn_i | 6 |
| usb_pullup_en_o | 71 |
| usb_dxp_io | 75 |
| usb_dxn_io | 75 |
| usb_term_dp_io | 55 |
| usb_term_dn_io | 29 |

Caveat: each count includes pad IO config and any routing the pin adds; no per-bit split yet. The oracle runs printed `RuntimeWarning: Unexpected value in sys.prefix` (venv path resolved via `fuzz/gw5ast/../..`).

Chirp (`/home/evd/workspace/sbp-computational/docs/PICO_USB_UTMI_OBSERVABILITY_2026-10-15.md`; not a source of fuse data): device chirp K 72001 cycles = 1200.016 us; chirp K end to highspeed 15517 cycles = 258.617 us. These are UTMI-level records, not pad measurements of usb_dxp_io/usb_dxn_io.

## TODO next
- OSER10, ELVDS_OBUF (HDMI), DCS, DHCEN, SDPB.
- Fold (attr -> {bits}) into apycula chipdb fuse tables for these cells, keyed by tile.
- Verify vs silicon-proven vendor .fs.

# GW5AST-138C site enumeration (multi-instance placement fuzz)

Method: place N copies of a cell (enum_sites.sh) so the Gowin placer spreads them across
all sites; cluster_sites.py clusters the diff bits into distinct site tiles. Cross-check
the discovered site count against the datasheet (gw5ast-138-resources memory).

Datasheet targets: 12 PLL, 24 HCLK, 340 BSRAM, 16 global-clock, 298 DSP, 8 transceivers.

## BSRAM (SDPB) — datasheet 340 ✅ VALIDATED
Grid analysis (ttyp-40 = BSRAM head): **116 head tiles on 6 BRAM rows (9/27/45/63/81/99)**,
~18-20 heads/row. 116 x 2.93 ~= 340 -> each head ~= 3 logical BSRAM blocks (datasheet 340
counts sub-blocks; 116 heads = the PLACEMENT grid).
VALIDATION (framebuffer.fs XOR empty.fs): **58/116 BSRAM tiles carry REAL distinct content**
(137-215 bits each) = the BRAMs the framebuffer actually uses (pico RAM + line buffers + FIFOs).
Confirms ttyp-40 tiles are genuine BSRAM sites. My single SDPB fuzz landed @ (99,161)=ttyp40 ✓.
Folded all 116 via sites_ttyp=40 (grid-derived).

## PLL — datasheet 12 ✅ ALL 12 MAPPED
12-instance fuzz + grid ttyp analysis: the 12 PLL sites = 4 left-edge + 4 right-edge + 4 bottom:
- LEFT  (ttyp 74/75/76 + 268/270): rows 27,45,63,81 @ col 0-1 -> (27,1)(45,0)(63,0)(81,1)
- RIGHT (ttyp 77/78/79 + 269/271): rows 27,45,63,81 @ col 177-181 -> (27,177)(45,181)(63,181)(81,177)
- BOTTOM (ttyp 182 BPLL heads): (108,28)(108,32)(108,146)(108,150) [each a 182/183/184/185 quad]
4+4+4 = 12 = datasheet. CONFIRMED. The fuzzed trim fuses (ICP/LPF @ (27,1) local row20) apply
to each site (identical hard PLL block; tile-local offset is the same per site).
Only (108,146) was previously modeled (fse_create_bottom_plls); the other 11 are new.

## FRACTIONAL PLL fuses (CRITICAL — the design uses them via frac_pll5.v)
The GW5AST PLL is fractional-N. Integer trim (ICP/LPF) is NOT enough; fractional configs need:
- MDIV_FRAC_SEL (fractional multiplier .XXX), ODIV0_FRAC_SEL (fractional output divider)
- MDSEL_FRAC / ODSEL0_FRAC (3-bit dynamic fractional selects)
- SSC_EN + SSCMDSEL_FRAC (spread-spectrum; frac_pll5 has SSC capability)
apicula partially knows SSCMDSEL_FRAC0/1/2 + MDCLK/MDOPC (chipdb.py:4528) but the
fractional-DIVIDER fuses are unfuzzed. Fuzzing MDIV_FRAC_SEL/ODIV0_FRAC_SEL/SSC_EN now
(differential vs integer pll_trim baseline). Fixtures: cells/pll_mfrac/ofrac/ssc.v.

## DSP — datasheet 298 ✅ EXACT MATCH
ttyp-20 = the DSP bel. Grid has EXACTLY **298 ttyp-20 tiles** on rows 18/36/54/72/90
== datasheet 298 (no extrapolation, 1:1). Clean constant-operand fuzz landed on ttyp-20.
SILICON-VALIDATED: 164/298 used by the framebuffer (.fs XOR empty) = pico ENABLE_MUL + SBP.
Folded all 298 via sites_ttyp=20. (Companion tiles ttyp 21/22 are the DSP macro body;
ttyp 20 head is the placeable bel — use a real ttyp-20 head as io_tile to avoid +1.)

## HCLK — datasheet 24 ✅ already modeled (393 hclk_pip tiles, gw5_make_hclk_pips)
## Global clock — datasheet 16 (clock spine; clkdiv/clkdiv2/clock_gates in chipdb)

## L/R PLL FUSE FRAME (Jun 2026, L0-vs-empty diff, oracle bpll_L0.fs @ PLL_L[0])
LEFT PLL placed at HEAD tile **(27,1) ttyp 74** + companions (27,2) ttyp75 + (27,3) ttyp76
(3-tile group, HORIZONTAL col+1,+2 — vs bottom's 4-tile 182/183/184/185).
Config fuses: local **rows 2-9** (base/power-up) + **row 20** (divider register) — analogous
to bottom (rows 2-9 + row 21) but row 20 not 21, and 3 tiles not 4. ~124/125/126 bits/tile.
Row-20 divider-bit columns captured per tile (see session). RIGHT mirrors at (27,177) ttyp
77/78/79. Rows 45/63 shift in by 1 col (head ttyp 75 not 74 at those rows — see grid dump).
NOTE: frame is SIMILAR to bottom but NOT identical -> needs its own bit_table + a same-site
param-sweep to map row-20 cols -> IDIV/FBDIV/MDIV/ODIV (like _BPLL_DIV_TABLE was for bottom).
Spine: L/R CLKOUT0..3 feed TLPLL{0,1}/TRPLL{0,1} groups (clknames_5ast138c) — 4 groups,
PLL0+PLL1 each, vs bottom's 2 (BL/BR PLL0-only).

## L-PLL MDIV FUSE MAPPING (proven differential, Jun 2026)
Same-site (PLL_L[0]) MDIV=18 vs MDIV=24 diff = **5 bits, ALL in tile (27,1) ttyp74 local row 20**,
bit-cols **351, 359, 367, 655, 663**. Confirms: L-PLL head/config tile = (27,1), divider register
= local row 20. Differential extraction pipeline PROVEN for L/R (build 2 same-site param variants ->
diff_bits.py -> ~5 clean bits). Oracle TCLs: hdl/test-c/build_lsw_{18,24}.tcl (lpll_m{18,24}.v +
lpll_sweep.cst INS_LOC PLL_L[0]). Repeat for IDIV/FBDIV/ODIV0 to complete the L div_table; mirror
for PLL_R[0]. Base/lock bits from bpll_L0-vs-empty filtered to tiles (27,1/2/3).

## L-PLL DIVIDER FUSE MAP (same-site PLL_L[0] differentials, all in tile (27,1) row20)
- MDIV 18->24: 5 bits, cols 351,359,367,655,663
- FBDIV 1->4 : 4 bits, cols 191,199,351,367  (cols 351/367 shared w/ MDIV -> packed field, like bottom _BPLL_DIV_TABLE)
- IDIV: 1->3 invalid (PFD<19MHz); redo IDIV=2 (PFD25/VCO900)
- ODIV0 20->8: 0 FUSE bits (.fs md5 differs but read_bitstream matrix identical -> ODIV0 not in
  the placed-CLKOUT0 fuse frame for single-output config; doesn't affect LOCK, only output freq).
Base/lock bits: 226 (rows 2-9 across the 3 tiles) from bpll_L0-vs-empty (saved /tmp/L_base_bits.json).
=> For a LOCKING L-PLL at a fixed config, base(226)+MDIV+FBDIV+IDIV fields suffice (VCO-determining).

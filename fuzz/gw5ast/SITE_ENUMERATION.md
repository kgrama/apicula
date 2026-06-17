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

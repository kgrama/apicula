# GW5AST-138C site enumeration (multi-instance placement fuzz)

Method: place N copies of a cell (enum_sites.sh) so the Gowin placer spreads them across
all sites; cluster_sites.py clusters the diff bits into distinct site tiles. Cross-check
the discovered site count against the datasheet (gw5ast-138-resources memory).

Datasheet targets: 12 PLL, 24 HCLK, 340 BSRAM, 16 global-clock, 298 DSP, 8 transceivers.

## BSRAM (SDPB) — datasheet 340
64-instance fuzz revealed the **4 BRAM-row structure**: rows **45, 63, 81, 99**
(dense site counts 22/20/19/111; the 111 is row-99 inflated by routing into adjacent tiles).
340 / 4 rows = **~85 BRAM per row**. The 64-instance run under-fills (only 64 placed); to map
all 340, place 340 instances OR read the column stride within a row and extrapolate.
Site cols within a row span ~58..175 (the BRAM column band).
TODO: tighten clustering (filter routing-adjacent tiles by ttyp == BRAM-head only).

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

## HCLK — datasheet 24  (TODO)
## Global clock — datasheet 16 (TODO)

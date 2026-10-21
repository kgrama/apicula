# L/R PLL output-spine fuzz results (fuzz_lr_pll_spine.sh, 2026-07-10)

8 sites built (PLL_L[0..3], PLL_R[0..3]) via INS_LOC "u_pll/PLL_inst" PLL_{L,R}[k], fixture
cells/pll_spine.v (CLKOUT1 drives fabric regs -> vendor commits a real spine route).
Pairwise diff_bits.py (.fs XOR). Intra-group pairs = placement bits only (MIN); inter-group
pairs add the spine-route delta.

LEFT (all head col 1, cleanest):
  L0-L1=1101  L2-L3=1103   <- intra-group MIN
  L0-L2=1138 L0-L3=1135 L1-L2=1131 L1-L3=1128  <- inter-group
  => {L0,L1}=group0, {L2,L3}=group1
RIGHT: R0-R1=1074 (intra group0).  (R2/R3 at head col 178 vs 177 add placement noise.)

CONCLUSION: rows 27/45 -> PLL spine group 0, rows 63/81 -> group 1.  Folded as pll_idx in
chipdb.py _LRPLL_SITES + gw5ast_lr_pll.py _LRPLL_SPINE_GROUP.  .fs kept in /tmp/gw5ast_fuzz/lr_spine.

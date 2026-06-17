# GW5AST-138C fuzzed cell BEL locations (gridwalk + common-tile filter)

Method: diff_bits.py now reports [row, bitcol, BYTECOL, bitpos] (bitmap cols are BYTE-cols
== chipdb tile.width units; the old bug treated unpacked bit-cols as byte-cols). gridwalk.py
maps (row, bytecol) -> tile via cumulative tile heights/widths. Then filter the 27 "common"
tiles (shared by >=4 cells = config/IO-bank artifacts) to isolate each cell's UNIQUE bel.

Grid: 109 rows x 182 cols; bitmap 1517 x 21872 bytes.

## Bel tiles (cell-unique densest, common tiles removed)
| cell        | bel tile     | ttyp | bits | confidence |
|-------------|--------------|------|------|------------|
| DQS         | (81, 5)      | 241  | 148  | STRONG     |
| SDPB        | (99, 161)    | 40   | 118  | STRONG     |
| DCS         | (81, 88)     | 211  | 32   | STRONG     |
| ELVDS_OBUF  | (52, 181)    | 245  | 15   | good (right edge = diff pads) |
| IODELAY     | (108, 141)   | 247  | 20   | good (bottom edge = I/O) |
| DDRDLL      | (1, 1)       | 17   | 20   | moderate   |
| OSER10      | (81, 181)    | 276  | 2    | WEAK (config mostly in common tiles; needs differential re-fuzz) |
| PLL (trim)  | (27, 40)ish  | 233  | -    | a mid-array PLL site (1 of 12); NOT the modeled bottom bpll(108,146) |

## Iterate plan (per user)
1. Fold the STRONG-anchor cells first (DQS/SDPB/DCS) into chipdb at their tiles.
2. Once placed, the known anchors + ttyp adjacency help disambiguate the weak ones
   (OSER10 especially — its param-differential fuses, not the full diff, give the real bel).
3. Param-differential fuses (FUSES_EXTRACTED.md) are the clean config bits to fold; the
   full-diff bel tile here gives the LOCATION.

## Note on PLL
12 PLL sites, only bottom bpll(108,146) modeled. My trim fuzz hit a mid-array site (~row27).
For "1 PLL site" fold: re-fuzz forcing placement at the modeled bpll, or add a bel for the
fuzzed site. Trim fuses (ICP_SEL 6b / LPF_RES 3b) are extracted regardless.

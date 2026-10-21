#!/usr/bin/env bash
# fuzz_ddr_clkwalk.sh — 48-variant clock-network + placement walk (see cells/ddr_clkwalk.v).
# Rebuilds the SAME fixture with generated cst+sdc across two axes:
#   CLK axis   : CLOCK_LOC "wclk" -> BUFG / BUFG[0|3|7|11|15] / LOCAL_CLOCK / (none)
#   REGION axis: GRP_LOC of the 64-FF walk_sr bank -> none/TL/TR/BL/BR/CENTER
# Produces: $FUZZ_WORK/ddr_clkwalk/<clk>_<region>.fs   (48 bitstreams for spine-fuse diffing)
#           $FUZZ_WORK/ddr_clkwalk/results.csv         (variant, ok|FAIL|WEDGE, secs, last phase)
# Per-variant timeout 600 s: a wedge IS data (the Routing Phase 1 signature) — recorded, not fatal.
# Usage: ./fuzz_ddr_clkwalk.sh            # full 48
#        ONLY_CLK=b3 ./fuzz_ddr_clkwalk.sh  # one clock variant x 6 regions (spot check)
set -uo pipefail
# NOTE: sourcing env.sh must precede gen_region_csts ($PY)
. "$(dirname "$0")/env.sh"
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$FUZZ_WORK/ddr_clkwalk"; mkdir -p "$OUT"
CSV="$OUT/results.csv"
[ -f "$CSV" ] || echo "variant,clk,region,status,secs,last_phase" > "$CSV"

# ── variant tables ──
declare -A CLKV=(
  [none]=""
  [bufg]='CLOCK_LOC "wclk" BUFG = CLK;'
  [b0]='CLOCK_LOC "wclk" BUFG[0] = CLK;'
  [b3]='CLOCK_LOC "wclk" BUFG[3] = CLK;'
  [b7]='CLOCK_LOC "wclk" BUFG[7] = CLK;'
  [b11]='CLOCK_LOC "wclk" BUFG[11] = CLK;'
  [b15]='CLOCK_LOC "wclk" BUFG[15] = CLK;'
  [local]='CLOCK_LOC "wclk" LOCAL_CLOCK = CLK;'
)
# REGION axis = per-FF INS_LOC site lists (GRP_LOC WEDGES gw_sh 1.9.12 in constraint
# processing — proven: even explicit-name GROUP+GRP_LOC hangs on a 100-cell design).
# Sites are picked from the chipdb: 64 ttyp-17 (CLS fabric) tiles spread across each
# region rect, emitted as region_<name>.cst once and appended per variant.
declare -A REGV=(
  [none]=""
  [tl]="1 30 1 60"
  [tr]="1 30 120 180"
  [bl]="79 108 1 60"
  [br]="79 108 120 180"
  [ctr]="40 70 70 110"
)
gen_region_csts() {
  "$PY" - "$OUT" <<'PYEOF'
import sys, os
sys.path.insert(0, os.environ["APICULA_ROOT"])
from apycula.chipdb import load_chipdb
out = sys.argv[1]
db = load_chipdb(f"{os.environ['APICULA_ROOT']}/apycula/GW5AST-138C.msgpack.xz")
regions = {"tl": (1,30,1,60), "tr": (1,30,120,180), "bl": (79,108,1,60),
           "br": (79,108,120,180), "ctr": (40,70,70,110)}
for name, (r0, r1, c0, c1) in regions.items():
    sites = [(r, c) for r in range(r0, min(r1, db.rows-1)+1)
                    for c in range(c0, min(c1, db.cols-1)+1)
                    if db.grid[r][c] == 17]
    step = max(1, len(sites)//64)
    picks = sites[::step][:64]
    assert len(picks) == 64, (name, len(picks))
    with open(f"{out}/region_{name}.cst", "w") as f:
        for n, (r, c) in enumerate(picks):
            # CST R/C are 1-based tile coords (chipdb grid is 0-based)
            f.write(f'INS_LOC "walk_sr_{n}_s0" R{r+1}C{c+1}[0][A];\n')
print("region csts written:", ", ".join(sorted(regions)))
PYEOF
}
CLK_ORDER="none bufg b0 b3 b7 b11 b15 local"
REG_ORDER="none tl tr bl br ctr"
[ -n "${ONLY_CLK:-}" ] && CLK_ORDER="$ONLY_CLK"

# ── fixed cst: pins (clk/din/dout LVCMOS33 + the ddr3_pic SSTL15 DDR bank, dqs/ck omitted) ──
base_cst() { cat <<'EOF'
IO_LOC "clkin" V22;
IO_PORT "clkin" IO_TYPE=LVCMOS33 PULL_MODE=UP;
IO_LOC "din" V14;
IO_PORT "din" IO_TYPE=LVCMOS33 PULL_MODE=UP;
IO_LOC "dout" U15;
IO_PORT "dout" IO_TYPE=LVCMOS33 DRIVE=8;
IO_LOC "ddr_a[13]" K1;
IO_LOC "ddr_a[12]" K4;
IO_LOC "ddr_a[11]" H3;
IO_LOC "ddr_a[10]" L1;
IO_LOC "ddr_a[9]" H5;
IO_LOC "ddr_a[8]" J5;
IO_LOC "ddr_a[7]" J1;
IO_LOC "ddr_a[6]" G3;
IO_LOC "ddr_a[5]" H2;
IO_LOC "ddr_a[4]" J2;
IO_LOC "ddr_a[3]" J4;
IO_LOC "ddr_a[2]" G2;
IO_LOC "ddr_a[1]" K2;
IO_LOC "ddr_a[0]" M1;
IO_LOC "ddr_ba[2]" M6;
IO_LOC "ddr_ba[1]" P2;
IO_LOC "ddr_ba[0]" P5;
IO_LOC "ddr_we" M5;
IO_LOC "ddr_cas" L4;
IO_LOC "ddr_ras" L5;
IO_LOC "ddr_cke" K6;
IO_LOC "ddr_odt" M2;
IO_LOC "ddr_cs" P4;
IO_LOC "ddr_rstn" L6;
IO_LOC "ddr_dm[1]" V7;
IO_LOC "ddr_dm[0]" AA4;
IO_LOC "dq_o[7]" AB1;
IO_LOC "dq_o[6]" AB5;
IO_LOC "dq_o[5]" AB2;
IO_LOC "dq_o[4]" AA1;
IO_LOC "dq_o[3]" V4;
IO_LOC "dq_o[2]" AA5;
IO_LOC "dq_o[1]" AB3;
IO_LOC "dq_o[0]" Y4;
IO_LOC "dq_i[7]" Y9;
IO_LOC "dq_i[6]" AB6;
IO_LOC "dq_i[5]" W9;
IO_LOC "dq_i[4]" AB8;
IO_LOC "dq_i[3]" Y7;
IO_LOC "dq_i[2]" AB7;
IO_LOC "dq_i[1]" Y8;
IO_LOC "dq_i[0]" AA8;
EOF
  # SSTL15 IO_PORT for every DDR-bank port (same attrs as ddr3_pic.cst)
  for p in 'ddr_a[13]' 'ddr_a[12]' 'ddr_a[11]' 'ddr_a[10]' 'ddr_a[9]' 'ddr_a[8]' \
           'ddr_a[7]' 'ddr_a[6]' 'ddr_a[5]' 'ddr_a[4]' 'ddr_a[3]' 'ddr_a[2]' \
           'ddr_a[1]' 'ddr_a[0]' 'ddr_ba[2]' 'ddr_ba[1]' 'ddr_ba[0]' ddr_we ddr_cas \
           ddr_ras ddr_cke ddr_odt ddr_cs ddr_rstn 'ddr_dm[1]' 'ddr_dm[0]' \
           'dq_o[7]' 'dq_o[6]' 'dq_o[5]' 'dq_o[4]' 'dq_o[3]' 'dq_o[2]' 'dq_o[1]' 'dq_o[0]'; do
    echo "IO_PORT \"$p\" IO_TYPE=SSTL15 PULL_MODE=NONE DRIVE=8 BANK_VCCIO=1.5;"
  done
  # inputs: DRIVE is illegal (CT1108) — IO_TYPE/PULL only
  for p in 'dq_i[7]' 'dq_i[6]' 'dq_i[5]' 'dq_i[4]' 'dq_i[3]' 'dq_i[2]' 'dq_i[1]' 'dq_i[0]'; do
    echo "IO_PORT \"$p\" IO_TYPE=SSTL15 PULL_MODE=NONE BANK_VCCIO=1.5;"
  done
}

base_sdc() { cat <<'EOF'
create_clock -name clk50 -period 20 -waveform {0 10} [get_ports {clkin}]
// wclk = PLL(27M) -> CLKDIV/2: generated clock ON THE CLKDIV PIN (the TA1132-correct idiom)
create_generated_clock -name wclk -source [get_ports {clkin}] -master_clock clk50 -divide_by 4 [get_pins {u_clkdiv/CLKOUT}]
set_clock_groups -asynchronous -group [get_clocks {clk50}] -group [get_clocks {wclk}]
EOF
}

# design.v = fixture + the self-contained PLL module
DESIGN="$OUT/design.v"
cat "$HERE/cells/ddr_clkwalk.v" "$HERE/cells/pll_mod_base.v" > "$DESIGN"
ls "$OUT"/region_tl.cst >/dev/null 2>&1 || gen_region_csts

total=0; done_n=0
for c in $CLK_ORDER; do for r in $REG_ORDER; do total=$((total+1)); done; done

for c in $CLK_ORDER; do
  for r in $REG_ORDER; do
    V="${c}_${r}"
    done_n=$((done_n+1))
    if [ -f "$OUT/$V.fs" ] || grep -q "^$V," "$CSV" 2>/dev/null; then
      echo "SKIP $V (already run)"; continue
    fi
    RUN="$OUT/run_$V"; rm -rf "$RUN"; mkdir -p "$RUN"
    cp "$DESIGN" "$RUN/design.v"
    { base_cst
      [ -n "${CLKV[$c]}" ] && printf '%s\n' "${CLKV[$c]}"
      [ -n "${REGV[$r]}" ] && cat "$OUT/region_$r.cst"
    } > "$RUN/design.cst"
    base_sdc > "$RUN/design.sdc"
    cat > "$RUN/run.tcl" <<EOF
set_device $GW5AST_PART -name $GW5AST_NAME
add_file -type verilog design.v
add_file -type cst design.cst
add_file -type sdc design.sdc
set_option -top_module top -output_base_name out \\
           -synthesis_tool gowinsynthesis -verilog_std sysv2017
run all
EOF
    t0=$SECONDS
    ( cd "$RUN" && timeout 600 env QT_QPA_PLATFORM=minimal "$GWSH" run.tcl >gwsh.log 2>&1 )
    rc=$?
    secs=$((SECONDS - t0))
    last="$(grep -E "Phase|ERROR|completed" "$RUN/gwsh.log" 2>/dev/null | tail -1 | tr ',' ';' | cut -c1-90)"
    if [ -f "$RUN/impl/pnr/out.fs" ]; then
      install -m 644 "$RUN/impl/pnr/out.fs" "$OUT/$V.fs"
      st=ok
    elif [ $rc -eq 124 ]; then st=WEDGE
    else st=FAIL
    fi
    echo "$V,$c,$r,$st,$secs,$last" >> "$CSV"
    echo "[$done_n/$total] $V -> $st (${secs}s)"
    [ "$st" = ok ] && rm -rf "$RUN"   # keep failed run dirs for forensics
  done
done
echo "SWEEP DONE: $(grep -c ',ok,' "$CSV") ok / $(grep -c ',WEDGE,' "$CSV") wedge / $(grep -c ',FAIL,' "$CSV") fail"
echo "results: $CSV"

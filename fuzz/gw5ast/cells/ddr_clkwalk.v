// ddr_clkwalk.v — clock-network + placement WALK fixture (DDR-shaped clock chain).
// Models the DDR app-clock topology that walls the pic+DDR router: PLL -> CLKDIV -> fabric
// clock (wclk) driving a 64-FF bank + real DDR pins (SSTL15, the ddr3_pic.cst bank).
// The sweep (fuzz_ddr_clkwalk.sh) rebuilds this SAME design 48x with generated cst/sdc:
//   axis 1: CLOCK_LOC "wclk" {BUFG, BUFG[0|3|7|11|15], LOCAL_CLOCK, none}  (spine walk)
//   axis 2: GRP_LOC of the FF bank {none, TL, TR, BL, BR, CENTER}          (region walk)
// Outputs: 48 .fs for spine-fuse diffing (gridwalk) + a route-behavior map (which combos
// route / how long / which wedge) — the empirical answer to the Routing Phase 1 wall.
// dqs diff pairs intentionally omitted (ELVDS fixture is separate); dq split out[7:0]/in[15:8]
// so both OBUF and IBUF paths appear at real DDR pins.
// Needs pll_27_MOD concatenated (cells/pll_mod_base.v) — the runner does this.
module top (
    input  wire        clkin,      // V22, 50 MHz
    input  wire        din,        // V14 (keeps the FF bank live)
    output wire        dout,       // U15
    output wire [13:0] ddr_a,
    output wire [2:0]  ddr_ba,
    output wire        ddr_we,
    output wire        ddr_cas,
    output wire        ddr_ras,
    output wire        ddr_cke,
    output wire        ddr_odt,
    output wire        ddr_cs,
    output wire        ddr_rstn,
    output wire [1:0]  ddr_dm,
    output wire [7:0]  dq_o,       // dq[7:0] pins, OBUF side
    input  wire [7:0]  dq_i        // dq[15:8] pins, IBUF side
);
    // DDR-like clock chain: 50 -> PLL (27 MHz clkout0) -> CLKDIV/2 -> wclk (the walked net)
    wire lock, c0, c1, c2, c3;
    pll_27_MOD u_pll (
        .lock(lock), .clkout0(c0), .clkout1(c1), .clkout2(c2), .clkout3(c3),
        .clkin(clkin), .reset(1'b0),
        .icpsel(6'b0), .lpfres(3'b0), .lpfcap(2'b0));
    wire wclk;
    CLKDIV #(.DIV_MODE("2")) u_clkdiv (
        .HCLKIN(c0), .RESETN(1'b1), .CALIB(1'b0), .CLKOUT(wclk));

    // the walked FF bank: 64-bit shift register, all on wclk (GROUP "*walk_sr*" in the cst)
    reg [63:0] walk_sr = 64'd0;
    reg [7:0]  dq_cap  = 8'd0;
    always @(posedge wclk) begin
        dq_cap  <= dq_i;                              // IBUF path live at real dq pins
        walk_sr <= {walk_sr[62:0], din ^ (^dq_cap)};  // input mix keeps everything un-foldable
    end
    assign dout     = walk_sr[63];
    assign ddr_a    = walk_sr[13:0];
    assign ddr_ba   = walk_sr[16:14];
    assign ddr_we   = walk_sr[17];
    assign ddr_cas  = walk_sr[18];
    assign ddr_ras  = walk_sr[19];
    assign ddr_cke  = walk_sr[20];
    assign ddr_odt  = walk_sr[21];
    assign ddr_cs   = walk_sr[22];
    assign ddr_rstn = walk_sr[23];
    assign ddr_dm   = walk_sr[25:24];
    assign dq_o     = walk_sr[33:26];
endmodule

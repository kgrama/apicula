// ddrdll_step_ff.v — DDRDLL fuzz fixture with STEP[7:0] driving fabric DFFs, so gw_sh
// commits real routing for the output (unlike ddrdll.v/ddrdll_div1.v, whose STEP output
// is unconnected/dangling and generates zero fabric pips). Isolates the STEP[i] -> real
// tile wire mapping at DDRDLL's confirmed real site (0,0).
module top(
    input  clkin,
    input  stop,
    input  reset,
    input  updncntl,
    input  d,
    output [7:0] q
);
    wire [7:0] step;
    DDRDLL #(
        .DLL_FORCE ("FALSE"),
        .CODESCAL  ("111"),
        .SCAL_EN   ("FALSE"),
        .DIV_SEL   (1'b1)
    ) u_dll (
        .CLKIN    (clkin),
        .STOP     (stop),
        .RESET    (reset),
        .UPDNCNTL (updncntl),
        .STEP     (step),
        .LOCK     ()
    );
    genvar i;
    generate for (i = 0; i < 8; i = i + 1) begin : g
        DFF u_dff (.CLK(clkin), .D(step[i] ^ d), .Q(q[i]));
    end endgenerate
endmodule

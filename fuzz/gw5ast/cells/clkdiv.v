// clkdiv.v — CLKDIV (HCLK divider) fuzz fixture, default DIV_MODE="2".
// Ports verbatim from $GW5A_PRIM.  (Variants clkdiv_div*.v sweep DIV_MODE.)
module top(
    input  hclkin,
    input  resetn,
    input  calib,
    output clkout
);
    CLKDIV #(.DIV_MODE("2")) u_clkdiv (
        .HCLKIN (hclkin),
        .RESETN (resetn),
        .CALIB  (calib),
        .CLKOUT (clkout)
    );
endmodule

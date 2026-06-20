// clkdiv2.v — CLKDIV2 (fixed /2 HCLK divider) fuzz fixture.  Ports verbatim from $GW5A_PRIM.
module top(
    input  hclkin,
    input  resetn,
    output clkout
);
    CLKDIV2 u_clkdiv2 (
        .HCLKIN (hclkin),
        .RESETN (resetn),
        .CLKOUT (clkout)
    );
endmodule

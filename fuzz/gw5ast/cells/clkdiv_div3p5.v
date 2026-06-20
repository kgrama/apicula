// clkdiv_div3p5.v — CLKDIV with the fractional DIV_MODE="3.5" (distinct fuse vs "2").
module top(
    input  hclkin,
    input  resetn,
    input  calib,
    output clkout
);
    CLKDIV #(.DIV_MODE("3.5")) u_clkdiv (
        .HCLKIN (hclkin),
        .RESETN (resetn),
        .CALIB  (calib),
        .CLKOUT (clkout)
    );
endmodule

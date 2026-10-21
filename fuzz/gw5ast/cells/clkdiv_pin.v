// clkdiv_pin.v — single CLKDIV, output clocking a fabric DFF, instance name "u_clkdiv"
// so INS_LOC "u_clkdiv" LEFTSIDE[n]/RIGHTSIDE[n]/BOTTOMSIDE[n] can pin its site (n=0..7
// per side per SUG1018 Table 2-2 for GW5A(S)(T)-138). Sweeping all 24 gives a clean
// single-source-per-build differential to decode the HCLK Bridge SPINE mux (unlike
// clkdiv_gclk_ff_multi.v's 24-simultaneous-sources sweep, which can't disambiguate which
// fuse bits belong to which of the 24 concurrent sources).
module top(
    input  hclkin,
    input  resetn,
    input  calib,
    input  d,
    output q
);
    wire sclk;
    reg qr;
    CLKDIV #(.DIV_MODE("2")) u_clkdiv (
        .HCLKIN (hclkin),
        .RESETN (resetn),
        .CALIB  (calib),
        .CLKOUT (sclk)
    );
    always @(posedge sclk) qr <= d;
    assign q = qr;
endmodule

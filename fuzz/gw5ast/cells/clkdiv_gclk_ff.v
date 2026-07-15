// clkdiv_gclk_ff.v — CLKDIV output CLOCKING fabric DFFs: forces gw_sh to route the
// divider output off the HCLK network onto the global clock spine (the HCLK->GCLK
// exit apicula does not model yet).  Diff against clkdiv.v's baseline to isolate the
// spine-entry fuses (center bridge tiles ttyp 80-85 + the HCLK control tile).
module top(
    input  hclkin,
    input  resetn,
    input  calib,
    input  d,
    output reg q
);
    wire sclk;
    CLKDIV #(.DIV_MODE("2")) u_clkdiv (
        .HCLKIN (hclkin),
        .RESETN (resetn),
        .CALIB  (calib),
        .CLKOUT (sclk)
    );
    always @(posedge sclk) q <= d;
endmodule

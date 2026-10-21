// clkdiv_gclk_ff_multi.v — 24x CLKDIV instances (one per HCLK region x line, if the
// placer spreads them that far), each output clocking a fabric DFF (behavioral, so
// gw_sh's own inference picks the DFF, matching the working clkdiv_gclk_ff.v pattern)
// so it commits a real global-spine route for every one. Companion to
// clkdiv_gclk_ff.v (which isolated region 3's HCLK->SPINE27 chain); this sweeps for
// the other 5 regions' assignments via enum_sites.sh-style post-hoc gridwalk.
module top(
    input  hclkin,
    input  resetn,
    input  calib,
    input  d,
    output [23:0] q
);
    genvar i;
    generate for (i = 0; i < 24; i = i + 1) begin : g
        wire sclk;
        reg qr;
        CLKDIV #(.DIV_MODE("2")) u_clkdiv (
            .HCLKIN (hclkin),
            .RESETN (resetn),
            .CALIB  (calib),
            .CLKOUT (sclk)
        );
        always @(posedge sclk) qr <= d;
        assign q[i] = qr;
    end endgenerate
endmodule

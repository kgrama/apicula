// elvds_obuf.v — ELVDS_OBUF (emulated LVDS differential output buffer) fuzz fixture.
// Drives the TMDS differential pads.  Ports from the de-vendored framebuffer
// (ddr3_framebuffer.v:346, tmds_bufds).  I -> O/OB (true/complement diff pair).
module top(
    input  i,
    output o,
    output ob
);
    ELVDS_OBUF u_buf (
        .I  (i),
        .O  (o),
        .OB (ob)
    );
endmodule

// oser10.v — OSER10 (10:1 TMDS serializer) fuzz fixture. Ports from the de-vendored
// HDMI serializer (hdl/.../hdmi2/serializer.sv:115, gwSer0).  D0..D9 + PCLK/FCLK/RESET
// -> Q.  This is the cell that drives the 371.25MHz TMDS serial output for HDMI.
module top(
    input  [9:0] d,
    input        pclk, fclk, reset,
    output       q
);
    OSER10 gwSer (
        .Q  (q),
        .D0 (d[0]), .D1 (d[1]), .D2 (d[2]), .D3 (d[3]), .D4 (d[4]),
        .D5 (d[5]), .D6 (d[6]), .D7 (d[7]), .D8 (d[8]), .D9 (d[9]),
        .PCLK (pclk),
        .FCLK (fclk),
        .RESET(reset)
    );
endmodule

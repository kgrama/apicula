// sdpb.v — SDPB (semi dual-port BRAM) fixture. Ports from gw5a prim_sim.v (single RESET).
module top(input clka, clkb, reset, input [13:0] ada, adb, input di, output [31:0] dout);
    SDPB #(.BIT_WIDTH_0(1), .BIT_WIDTH_1(32), .READ_MODE(1'b1)) u_sdpb (
        .CLKA(clka), .CLKB(clkb), .RESET(reset),
        .CEA(1'b1), .CEB(1'b1), .OCE(1'b1), .BLKSELA(3'b0), .BLKSELB(3'b0),
        .ADA(ada), .ADB(adb), .DI({31'b0, di}), .DO(dout)
    );
endmodule

// sdpb_multi.v — N SDPB instances so the placer spreads across BSRAM sites.
module top(input clka, clkb, reset, input [13:0] ad, input [63:0] di, output [63:0] dout);
  genvar i;
  generate for(i=0;i<64;i=i+1) begin:b
    SDPB #(.BIT_WIDTH_0(1), .BIT_WIDTH_1(1)) u(
      .CLKA(clka), .CLKB(clkb), .RESET(reset),
      .CEA(1'b1), .CEB(1'b1), .OCE(1'b1), .BLKSELA(3'b0), .BLKSELB(3'b0),
      .ADA(ad), .ADB(ad), .DI(di[i]), .DO(dout[i]));
  end endgenerate
endmodule

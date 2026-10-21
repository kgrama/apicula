module top(input [9:0] d, input pclk, fclk, reset, output [15:0] q);
  genvar i;
  generate for(i=0;i<16;i=i+1) begin:s
    OSER10 u(.Q(q[i]),.D0(d[0]),.D1(d[1]),.D2(d[2]),.D3(d[3]),.D4(d[4]),
             .D5(d[5]),.D6(d[6]),.D7(d[7]),.D8(d[8]),.D9(d[9]),
             .PCLK(pclk),.FCLK(fclk),.RESET(reset));
  end endgenerate
endmodule

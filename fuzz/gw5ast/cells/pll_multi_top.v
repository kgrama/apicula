module top(input ci, input rst, output [11:0] lk);
  genvar i;
  generate for(i=0;i<12;i=i+1) begin:p
    pll_27_MOD u(.lock(lk[i]),.clkout0(),.clkout1(),.clkin(ci),.reset(rst),
                 .icpsel(6'b0),.lpfres(3'b0),.lpfcap(2'b0));
  end endgenerate
endmodule

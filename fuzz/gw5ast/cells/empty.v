// empty.v — baseline design (one trivial reg so PnR has something), the XOR reference.
module top(input clk, input a, output reg o);
    always @(posedge clk) o <= a;
endmodule

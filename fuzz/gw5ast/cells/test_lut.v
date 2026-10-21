module top(input clk, input a, input b, output reg o);
    always @(posedge clk) o <= a ^ b;
endmodule

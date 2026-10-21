// usb_base.v — baseline: clk/a/o on fixed pads; XOR reference for the per-USB-pin fixtures.
module top(input clk, input a, output reg o);
    always @(posedge clk) o <= a;
endmodule

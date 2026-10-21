// usb_pullup_en_o.v — usb_pullup_en_o (LVCMOS33) driven by a DFF; base logic as usb_base.v.
module top(input clk, input a, output reg o, output reg usb_pullup_en_o);
    always @(posedge clk) begin o <= a; usb_pullup_en_o <= a; end
endmodule

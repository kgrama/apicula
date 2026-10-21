// usb_rxdn_i.v — usb_rxdn_i (LVDS25) read into a DFF; base logic as usb_base.v.
module top(input clk, input a, output reg o, input usb_rxdn_i);
    reg r;
    always @(posedge clk) begin o <= a; r <= usb_rxdn_i; end
endmodule

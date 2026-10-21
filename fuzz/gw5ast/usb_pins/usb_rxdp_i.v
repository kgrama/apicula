// usb_rxdp_i.v — usb_rxdp_i (LVDS25) read into a DFF; base logic as usb_base.v.
module top(input clk, input a, output reg o, input usb_rxdp_i);
    reg r;
    always @(posedge clk) begin o <= a; r <= usb_rxdp_i; end
endmodule

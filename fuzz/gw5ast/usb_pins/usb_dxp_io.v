// usb_dxp_io.v — usb_dxp_io (LVPECL33E) inout, driven by a DFF; base logic as usb_base.v.
module top(input clk, input a, output reg o, inout usb_dxp_io);
    reg r;
    always @(posedge clk) begin o <= a; r <= a; end
    assign usb_dxp_io = r;
endmodule

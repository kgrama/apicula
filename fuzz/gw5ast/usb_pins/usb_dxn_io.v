// usb_dxn_io.v — usb_dxn_io (LVPECL33E) inout, driven by a DFF; base logic as usb_base.v.
module top(input clk, input a, output reg o, inout usb_dxn_io);
    reg r;
    always @(posedge clk) begin o <= a; r <= a; end
    assign usb_dxn_io = r;
endmodule

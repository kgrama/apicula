// usb_term_dp_io.v — usb_term_dp_io (LVCMOS33) inout, driven by a DFF; base logic as usb_base.v.
module top(input clk, input a, output reg o, inout usb_term_dp_io);
    reg r;
    always @(posedge clk) begin o <= a; r <= a; end
    assign usb_term_dp_io = r;
endmodule

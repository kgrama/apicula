// dsp.v — MULT27X36 with CONSTANT operands so only the DSP cell differs (no I/O flood).
module top(input clk, ce, reset, output o);
    wire [62:0] dout;
    MULT27X36 #(.AREG_CLK("BYPASS")) u_dsp (
        .DOUT(dout), .A(27'd12345), .B(36'd6789), .D(63'd0),
        .PSEL(1'b0), .PADDSUB(1'b0), .CLK(clk), .CE(ce), .RESET(reset)
    );
    assign o = ^dout;   // reduce to 1 bit so synth keeps the DSP but minimal I/O
endmodule

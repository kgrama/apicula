// dsp.v — MULT27X36 (main GW5A DSP multiplier) fixture. Ports from gw5a prim_sim.
module top(input [26:0] a, input [35:0] b, input [62:0] d,
           input psel, paddsub, clk, ce, reset, output [62:0] dout);
    MULT27X36 #(.AREG_CLK("CLK0")) u_dsp (
        .DOUT(dout), .A(a), .B(b), .D(d),
        .PSEL(psel), .PADDSUB(paddsub), .CLK(clk), .CE(ce), .RESET(reset)
    );
endmodule

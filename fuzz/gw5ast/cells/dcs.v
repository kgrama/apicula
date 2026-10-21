// dcs.v — DCS (glitch-free clock mux) fixture. Ports from gw5a prim_sim.v.
module top(input clk0, clk1, clk2, clk3, input [3:0] clksel, input selforce, output clkout);
    DCS #(.DCS_MODE("RISING")) u_dcs (
        .CLKIN0(clk0), .CLKIN1(clk1), .CLKIN2(clk2), .CLKIN3(clk3),
        .CLKSEL(clksel), .SELFORCE(selforce), .CLKOUT(clkout)
    );
endmodule

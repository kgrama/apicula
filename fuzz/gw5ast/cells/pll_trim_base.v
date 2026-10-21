// pll_trim.v — PLL trim fuzz fixture: wraps the de-vendored pll_27_MOD (a complete,
// silicon-proven PLL config).  Sweeping ICP_SEL/LPF_RES/LPF_CAP via param-variants
// isolates the LOOP-FILTER / CHARGE-PUMP fuses that determine LOCK quality.
module top(input clkin, input reset, output clk0, output clk1, output lock);
    pll_27_MOD u_pll(
        .lock(lock), .clkout0(clk0), .clkout1(clk1),
        .clkin(clkin), .reset(reset),
        .icpsel(6'b0), .lpfres(3'b0), .lpfcap(2'b0)
    );
endmodule

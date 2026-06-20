// dce.v — DCE (Dynamic Clock Enable, global-clock gate) fuzz fixture.
// Ports verbatim from $GW5A_PRIM (prim_sim.v).  All I/O to top pins so synth keeps the cell.
module top(
    input  clkin,
    input  ce,
    output clkout
);
    DCE u_dce (
        .CLKIN  (clkin),
        .CE     (ce),
        .CLKOUT (clkout)
    );
endmodule

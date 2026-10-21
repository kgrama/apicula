// dhce.v — DHCE (Dynamic HCLK Enable) fuzz fixture.  Ports verbatim from $GW5A_PRIM.
module top(
    input  clkin,
    input  cen,
    output clkout
);
    DHCE u_dhce (
        .CLKIN  (clkin),
        .CEN    (cen),
        .CLKOUT (clkout)
    );
endmodule

// ddrdll_lock_ff.v — DDRDLL LOCK drives a fabric DFF so gw_sh commits the LOCK->fabric
// route (an unconnected LOCK generates no pips). Same DDRDLL params as dqs.v.
module top(
    input  clkin,
    input  stop,
    input  reset,
    input  updncntl,
    input  d,
    output q
);
    wire [7:0] step;
    wire lock;
    DDRDLL #(
        .DLL_FORCE ("FALSE"),
        .CODESCAL  ("111"),
        .SCAL_EN   ("FALSE"),
        .DIV_SEL   (1'b0)
    ) u_dll (
        .CLKIN    (clkin),
        .STOP     (stop),
        .RESET    (reset),
        .UPDNCNTL (updncntl),
        .STEP     (step),
        .LOCK     (lock)
    );
    DFF u_dff (.CLK(clkin), .D(lock ^ d), .Q(q));
endmodule

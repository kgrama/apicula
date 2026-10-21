// ddrdll_step_ff_iso_base.v — baseline for ddrdll_step_ff_iso.v: SAME DDRDLL instance
// and SAME 8 fabric DFFs, but the DFFs latch a constant instead of STEP, so the design
// shape (placement pressure, DFF count) matches while STEP's fanout pips are absent.
// This isolates STEP's OWN pips via bit-diff without noise from DDRDLL's own baseline
// config/placement footprint (already common to both).
module top(
    input  clkin,
    input  stop,
    input  reset,
    input  updncntl,
    output [7:0] q
);
    wire [7:0] step;
    DDRDLL #(
        .DLL_FORCE ("FALSE"),
        .CODESCAL  ("111"),
        .SCAL_EN   ("FALSE"),
        .DIV_SEL   (1'b1)
    ) u_dll (
        .CLKIN    (clkin),
        .STOP     (stop),
        .RESET    (reset),
        .UPDNCNTL (updncntl),
        .STEP     (step),
        .LOCK     ()
    );
    reg [7:0] qr;
    always @(posedge clkin) qr <= 8'b0;
    assign q = qr;
endmodule

// ddrdll_step_ff_iso.v — DDRDLL fuzz fixture, STEP[7:0] driving fabric DFFs, WITHOUT any
// other competing logic (unlike dqs.v's coupled DDRDLL, this has zero other placed cells
// so a bits diff against ddrdll_div1.v -- which has STEP entirely dangling/unrouted --
// isolates STEP's fanout pips cleanly without unrelated-placement noise).
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
    always @(posedge clkin) qr <= step;
    assign q = qr;
endmodule

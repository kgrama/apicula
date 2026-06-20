// dlldly.v — DLLDLY (DLL-controlled clock delay line) fuzz fixture, default params.
// Ports/params verbatim from $GW5A_PRIM.  (Variants sweep DLY_ADJ/DYN_DLY_EN/ADAPT_EN.)
module top(
    input        clkin,
    input  [7:0] dllstep,
    input  [7:0] cstep,
    input        loadn,
    input        move,
    output       clkout,
    output       flag
);
    DLLDLY #(
        .DLY_SIGN   (1'b0),
        .DLY_ADJ    (0),
        .DYN_DLY_EN ("FALSE"),
        .ADAPT_EN   ("FALSE"),
        .STEP_SEL   (1'b0)
    ) u_dlldly (
        .CLKIN   (clkin),
        .DLLSTEP (dllstep),
        .CSTEP   (cstep),
        .LOADN   (loadn),
        .MOVE    (move),
        .CLKOUT  (clkout),
        .FLAG    (flag)
    );
endmodule

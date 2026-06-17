// ddrdll.v — DDRDLL fuzz fixture. Ports/params verbatim from the de-vendored
// controller (hdl/generators/pin-compat/ddr3_1_4code_hs-u.v:6155). All I/O brought to
// top pins so synth keeps the primitive.
module top(
    input  clkin,
    input  stop,
    input  reset,
    input  updncntl,
    output [7:0] step,
    output lock
);
    DDRDLL #(
        .DLL_FORCE ("true"),
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
endmodule

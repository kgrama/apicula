// ddrdll_step_pin.v — DDRDLL with STEP[7:0] driving fabric DFFs, instance name "u_dll"
// so INS_LOC "u_dll" DDRDLLM_TL/DDRDLLM_TR; can pin its site. Companion .cst files sweep
// both real DDRDLL sites (only 2 exist per SUG1018 table: DDRDLLM_TL, DDRDLLM_TR).
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

// iodelay.v — IODELAY fuzz fixture. Ports/params from the de-vendored controller
// (hdl/generators/pin-compat/ddr3_1_4code_hs-u.v:4291, the iodelay_ck instance).
// C_STATIC_DLY is the static delay tap (0..127) — the main fuzzable param.
module top(
    input        di,
    input  [7:0] dlystep,
    input        sdtap,
    input        value,
    output       dout,
    output       dflag
);
    IODELAY #(
        .C_STATIC_DLY(40)
    ) u_iodelay (
        .DO     (dout),
        .DF     (dflag),
        .DI     (di),
        .SDTAP  (sdtap),
        .DLYSTEP(dlystep),
        .VALUE  (value)
    );
endmodule

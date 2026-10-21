// iodelay_params.v — IODELAY fixture with ALL THREE params explicit, so a sweep can flip exactly
// one at a time for a clean differential:
//   C_STATIC_DLY  0..127  static delay tap — SOLVED: 7-bit BINARY field, local row 20 cols 0..6,
//                         LSB at the HIGHEST col (see fuzz_ddr_iodelay.sh / FUSES_EXTRACTED.md)
//   DYN_DLY_EN    FALSE/TRUE  dynamic delay enable (gates the DLYSTEP[7:0] stepping port)
//   ADAPT_EN      FALSE/TRUE  adaptive delay
// Baseline here = what the open LiteDRAM GW5DDRPHY actually emits (gw5ddrphy.py:211):
// C_STATIC_DLY=0, DYN_DLY_EN="FALSE", ADAPT_EN="FALSE", SDTAP/DLYSTEP/VALUE tied off.
module top(
    input        di,
    input  [7:0] dlystep,
    input        sdtap,
    input        value,
    output       dout,
    output       dflag
);
    IODELAY #(
        .C_STATIC_DLY(0),
        .DYN_DLY_EN("FALSE"),
        .ADAPT_EN("FALSE")
    ) u_iodelay (
        .DO     (dout),
        .DF     (dflag),
        .DI     (di),
        .SDTAP  (sdtap),
        .DLYSTEP(dlystep),
        .VALUE  (value)
    );
endmodule

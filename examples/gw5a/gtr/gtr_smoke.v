// GW5AST-138C GTR12_QUAD open-flow smoke test.
//
// Instantiates one GTR12_QUAD (apicula bel, anchor tile (0,27)) and routes a handful of
// the .dat-confirmed INET tap bits to top-level IO so the full open flow exercises:
//   yosys (GTR12_QUAD as blackbox) -> nextpnr-himbaechel (place GTR bel + route taps)
//   -> gowin_pack --serdes_csr (CSR config) -> .fs
//
// Config rides the CSR (serdes.csr), exactly as the vendor JESD204B oracle does; the
// FABRIC_* functional ports bind on the bel but are not routed here (their fabric routing
// is the remaining ECP5-DCU-parity gap). We drive/observe only the bridged INET bits.

(* blackbox *)
module GTR12_QUAD (
    inout  [91:0]  INET_Q0_Q1,
    inout  [531:0] INET_Q_PMAC,
    inout  [227:0] INET_Q_TEST,
    inout  [420:0] INET_Q_UPAR
);
    parameter POSITION = "Q0";
endmodule

module top (
    input  wire       clk,
    input  wire [3:0] din,
    output wire [3:0] dout
);
    // Drive a few routed INPUT taps from registered pad inputs; observe a few routed
    // OUTPUT taps on pads. Registers keep yosys from constant-folding the cell away.
    reg  [3:0] din_r;
    always @(posedge clk) din_r <= din;

    wire [91:0]  q0_q1;
    wire [531:0] pmac;
    wire [227:0] test;
    wire [420:0] upar;

    // Routed INPUT taps (fabric -> GTR), from the .dat DBIns subset.
    assign upar[0]  = din_r[0];   // INET_Q_UPAR[0]  (UparDBIns bit 0)
    assign upar[2]  = din_r[1];   // INET_Q_UPAR[2]
    assign test[200] = din_r[2];  // INET_Q_TEST[200] (QuadDBIns2 subset)
    assign pmac[0]  = din_r[3];   // INET_Q_PMAC[0]   (PmacDBIns bit 0)

    GTR12_QUAD #(.POSITION("Q0")) gtr_i (
        .INET_Q0_Q1 (q0_q1),
        .INET_Q_PMAC(pmac),
        .INET_Q_TEST(test),
        .INET_Q_UPAR(upar)
    );

    // Routed OUTPUT taps (GTR -> fabric), from the .dat DBOuts subset, registered to pads.
    reg [3:0] dout_r;
    always @(posedge clk)
        dout_r <= {q0_q1[0], test[0], upar[85], pmac[10]};
    assign dout = dout_r;
endmodule

// gtr_fabric_ff.v — GTR12_QUAD FABRIC_* status outputs each drive a fabric DFF so gw_sh
// commits the GTR -> fabric route (unconnected outputs generate no pips). Inputs tied off.
module top(
    input  clk,
    input  d,
    output [5:0] q
);
    wire rxdet0, rxdet1, rxdet2, rxdet3, cm0_det, cm1_det;
    GTR12_QUAD #(.POSITION("Q0")) u_gtr (
        .FABRIC_LN0_RXDET_RESULT        (rxdet0),
        .FABRIC_LN1_RXDET_RESULT        (rxdet1),
        .FABRIC_LN2_RXDET_RESULT        (rxdet2),
        .FABRIC_LN3_RXDET_RESULT        (rxdet3),
        .FABRIC_PMA_CM0_DR_REFCLK_DET_O (cm0_det),
        .FABRIC_PMA_CM1_DR_REFCLK_DET_O (cm1_det),
        .FABRIC_REFCLK_INPUT_SEL_I      (3'b000),
        .FABRIC_PMA_PD_REFHCLK_I        (1'b0)
    );
    DFF u_q0 (.CLK(clk), .D(rxdet0 ^ d), .Q(q[0]));
    DFF u_q1 (.CLK(clk), .D(rxdet1 ^ d), .Q(q[1]));
    DFF u_q2 (.CLK(clk), .D(rxdet2 ^ d), .Q(q[2]));
    DFF u_q3 (.CLK(clk), .D(rxdet3 ^ d), .Q(q[3]));
    DFF u_q4 (.CLK(clk), .D(cm0_det ^ d), .Q(q[4]));
    DFF u_q5 (.CLK(clk), .D(cm1_det ^ d), .Q(q[5]));
endmodule

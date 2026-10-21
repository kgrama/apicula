// gtr_upar_ff.v — GTR12_QUAD INET_Q_UPAR (CSR bus) bits 0 and 85 driven/observed by fabric
// DFFs, bit positions from the .dat UparDBIns/UparDBOuts tables (as in examples/gw5a/gtr).
// Config still rides serdes.csr; this fixture measures only the fabric<->UPAR route.
module top(
    input  clk,
    input  d,
    output [1:0] q
);
    wire [420:0] upar;
    reg  din_r;
    always @(posedge clk) din_r <= d;
    assign upar[0] = din_r;
    GTR12_QUAD #(.POSITION("Q0")) u_gtr (
        .INET_Q0_Q1  (),
        .INET_Q_PMAC (),
        .INET_Q_TEST (),
        .INET_Q_UPAR (upar)
    );
    DFF u_q0 (.CLK(clk), .D(upar[85]), .Q(q[0]));
    DFF u_q1 (.CLK(clk), .D(upar[0] ^ d), .Q(q[1]));
endmodule

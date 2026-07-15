// oser4_mem.v — OSER4_MEM (DDR3 write-side 4:1 SerDes, 1:2 PHY ratio) fuzz fixture.
// The -4 variant the OPEN LiteDRAM GW5DDRPHY actually instantiates (vs the -8 the encrypted
// Gowin IP used).  Port/param map verified against litedram/phy/gw5ddrphy.py:370 — D0..D3,
// TX0..TX1, Q0/Q1, TCLK_SOURCE/TXCLK_POL.  Same DDRDLL -> DQS -> OSER4_MEM chain (PR0015
// coupling: TCLK is the write-DQS strobe DQSW270).
module top(
    input        pclk, fclk, reset,
    input  [3:0] d,
    input  [1:0] tx,
    input        dqsin, read,
    input  [2:0] rclksel,
    input        wstep, rloadn, rmove, rdir, wloadn, wmove, wdir, hold,
    input        dll_clkin, dll_stop, dll_updn,
    inout        dq_pad,
    output       dqsw0_o, dqsw270_o, dll_lock
);
    wire [7:0] dll_step;
    wire       dqsr90, dqsw0, dqsw270;
    wire [2:0] rpoint, wpoint;
    wire       q0, q1;

    DDRDLL #(
        .DLL_FORCE ("FALSE"), .CODESCAL ("111"), .SCAL_EN ("FALSE"), .DIV_SEL (1'b0)
    ) u_dll (
        .CLKIN(dll_clkin), .STOP(dll_stop), .RESET(reset),
        .UPDNCNTL(dll_updn), .STEP(dll_step), .LOCK(dll_lock)
    );
    DQS #(
        .FIFO_MODE_SEL (1'b0), .DQS_MODE ("X4"), .HWL ("true")
    ) u_dqs (
        .DQSIN(dqsin), .PCLK(pclk), .FCLK(fclk), .RESET(reset), .READ(read),
        .RCLKSEL(rclksel), .DLLSTEP(dll_step), .WSTEP(wstep),
        .RLOADN(rloadn), .RMOVE(rmove), .RDIR(rdir),
        .WLOADN(wloadn), .WMOVE(wmove), .WDIR(wdir), .HOLD(hold),
        .DQSR90(dqsr90), .DQSW0(dqsw0), .DQSW270(dqsw270),
        .RPOINT(rpoint), .WPOINT(wpoint),
        .RVALID(), .RBURST(), .RFLAG(), .WFLAG()
    );
    // write-side 4:1 SerDes, TCLK from the write-DQS strobe (DQSW270), as in the open PHY.
    OSER4_MEM #(
        .TCLK_SOURCE ("DQSW270"), .TXCLK_POL (1'b0)
    ) u_oser4_mem (
        .Q0(q0), .Q1(q1),
        .D0(d[0]), .D1(d[1]), .D2(d[2]), .D3(d[3]),
        .TX0(tx[0]), .TX1(tx[1]),
        .PCLK(pclk), .FCLK(fclk), .TCLK(dqsw270), .RESET(reset)
    );
    IOBUF u_dq (.O(), .IO(dq_pad), .I(q0), .OEN(q1));
    assign dqsw0_o = dqsw0;  assign dqsw270_o = dqsw270;
endmodule

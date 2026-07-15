// ides4_mem.v — IDES4_MEM (DDR3 read-side 1:4 SerDes, 1:2 PHY ratio) fuzz fixture.
// The -4 variant the OPEN LiteDRAM GW5DDRPHY instantiates.  Port map verified against
// litedram/phy/gw5ddrphy.py:417 — Q0..Q3, PCLK/FCLK/ICLK, RADDR/WADDR, D, CALIB.  ICLK is the
// read-DQS strobe DQSR90; needs the DDRDLL -> DQS chain (PR0015 coupling), same as the -8.
module top(
    input        pclk, fclk, reset, d, calib,
    input        dqsin, read,
    input  [2:0] rclksel,
    input        wstep, rloadn, rmove, rdir, wloadn, wmove, wdir, hold,
    input        dll_clkin, dll_stop, dll_updn,
    output [3:0] q,
    output       dll_lock
);
    wire [7:0] dll_step;
    wire       dqsr90, dqsw0, dqsw270;
    wire [2:0] rpoint, wpoint;

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
    // read-side 1:4 SerDes, ICLK from the read-DQS strobe (DQSR90), as in the open PHY.
    IDES4_MEM u_ides4_mem (
        .Q0(q[0]), .Q1(q[1]), .Q2(q[2]), .Q3(q[3]),
        .PCLK(pclk), .D(d), .ICLK(dqsr90), .FCLK(fclk), .RESET(reset),
        .CALIB(calib), .WADDR(wpoint[2:0]), .RADDR(rpoint[2:0])
    );
endmodule

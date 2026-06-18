// ides8_mem.v — IDES8_MEM (DDR3 read-side 1:8 SerDes) fuzz fixture.
// Ports verbatim from the de-vendored controller's iserdes_gen/u_ides8_mem
// (ddr3_memory_interface.v).  IDES8_MEM's ICLK is the read-DQS strobe DQSR90 and its
// WADDR/RADDR come from the DQS write/read pointers, so (like OSER8_MEM) it needs the
// DDRDLL -> DQS chain (PR0015 coupling).  Same chain the controller uses.
module top(
    input        pclk, fclk, reset, d, calib,
    input        dqsin, read,
    input  [2:0] rclksel,
    input        wstep, rloadn, rmove, rdir, wloadn, wmove, wdir, hold,
    input        dll_clkin, dll_stop, dll_updn,
    output [7:0] q,
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
    // read-side 1:8 SerDes, ICLK from the read-DQS strobe (DQSR90), as in the ctrl.
    IDES8_MEM u_ides8_mem (
        .Q0(q[0]), .Q1(q[1]), .Q2(q[2]), .Q3(q[3]),
        .Q4(q[4]), .Q5(q[5]), .Q6(q[6]), .Q7(q[7]),
        .PCLK(pclk), .D(d), .ICLK(dqsr90), .FCLK(fclk), .RESET(reset),
        .CALIB(calib), .WADDR(wpoint[2:0]), .RADDR(rpoint[2:0])
    );
endmodule

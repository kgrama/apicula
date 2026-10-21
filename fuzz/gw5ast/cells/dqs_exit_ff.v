// dqs_exit_ff.v — DQS RBURST/RVALID/RFLAG/WFLAG each drive a fabric DFF so gw_sh commits
// the exit routes (dqs.v leaves them on top-level ports). DDRDLL -> DQS coupling as in dqs.v.
module top(
    input        dqsin, pclk, fclk, reset, read,
    input  [2:0] rclksel,
    input        wstep, rloadn, rmove, rdir, wloadn, wmove, wdir, hold,
    input        dll_clkin, dll_stop, dll_updn,
    input        d,
    output [3:0] q
);
    wire [7:0] dll_step;
    wire rvalid, rburst, rflag, wflag;
    DDRDLL #(
        .DLL_FORCE ("FALSE"), .CODESCAL ("111"), .SCAL_EN ("FALSE"), .DIV_SEL (1'b0)
    ) u_dll (
        .CLKIN(dll_clkin), .STOP(dll_stop), .RESET(reset),
        .UPDNCNTL(dll_updn), .STEP(dll_step), .LOCK()
    );
    DQS #(
        .FIFO_MODE_SEL (1'b0),
        .DQS_MODE      ("X4"),
        .HWL           ("true")
    ) u_dqs (
        .DQSIN(dqsin), .PCLK(pclk), .FCLK(fclk), .RESET(reset), .READ(read),
        .RCLKSEL(rclksel), .DLLSTEP(dll_step), .WSTEP(wstep),
        .RLOADN(rloadn), .RMOVE(rmove), .RDIR(rdir),
        .WLOADN(wloadn), .WMOVE(wmove), .WDIR(wdir), .HOLD(hold),
        .DQSR90(), .DQSW0(), .DQSW270(),
        .RPOINT(), .WPOINT(),
        .RVALID(rvalid), .RBURST(rburst), .RFLAG(rflag), .WFLAG(wflag)
    );
    DFF u_q0 (.CLK(pclk), .D(rvalid ^ d), .Q(q[0]));
    DFF u_q1 (.CLK(pclk), .D(rburst ^ d), .Q(q[1]));
    DFF u_q2 (.CLK(pclk), .D(rflag  ^ d), .Q(q[2]));
    DFF u_q3 (.CLK(pclk), .D(wflag  ^ d), .Q(q[3]));
endmodule

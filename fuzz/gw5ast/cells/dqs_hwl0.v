// dqs.v — DQS fuzz fixture. Gowin synth REQUIRES DQS.DLLSTEP to be driven by a real
// DDRDLL (PR0015) — the cells are topologically coupled in HW.  So we instantiate
// DDRDLL -> DQS, exactly as the de-vendored controller does (ddr3_1_4code_hs-u.v).
// Ports/params verbatim from u_dqs (line 5078) + u_dll (line 6155).
module top(
    input        dqsin, pclk, fclk, reset, read,
    input  [2:0] rclksel,
    input        wstep, rloadn, rmove, rdir, wloadn, wmove, wdir, hold,
    input        dll_clkin, dll_stop, dll_updn,
    output       dqsr90, dqsw0, dqsw270,
    output [2:0] rpoint, wpoint,
    output       rvalid, rburst, rflag, wflag,
    output       dll_lock
);
    wire [7:0] dll_step;
    DDRDLL #(
        .DLL_FORCE ("FALSE"), .CODESCAL ("111"), .SCAL_EN ("FALSE"), .DIV_SEL (1'b0)
    ) u_dll (
        .CLKIN(dll_clkin), .STOP(dll_stop), .RESET(reset),
        .UPDNCNTL(dll_updn), .STEP(dll_step), .LOCK(dll_lock)
    );
    DQS #(
        .FIFO_MODE_SEL (1'b0),
        .DQS_MODE      ("X4"),
        .HWL           ("false")
    ) u_dqs (
        .DQSIN(dqsin), .PCLK(pclk), .FCLK(fclk), .RESET(reset), .READ(read),
        .RCLKSEL(rclksel), .DLLSTEP(dll_step), .WSTEP(wstep),
        .RLOADN(rloadn), .RMOVE(rmove), .RDIR(rdir),
        .WLOADN(wloadn), .WMOVE(wmove), .WDIR(wdir), .HOLD(hold),
        .DQSR90(dqsr90), .DQSW0(dqsw0), .DQSW270(dqsw270),
        .RPOINT(rpoint), .WPOINT(wpoint),
        .RVALID(rvalid), .RBURST(rburst), .RFLAG(rflag), .WFLAG(wflag)
    );
endmodule

// oser8_mem.v — OSER8_MEM (DDR3 write-side 8:1 SerDes) fuzz fixture.
// Ports/params verbatim from the de-vendored controller's u_dqs_gen / oserdes_gen
// (ddr3_memory_interface.v).  OSER8_MEM's TCLK is the write-DQS strobe; in HW it is
// fed by a DQS cell's DQSW/DQSW270 output (TCLK_SOURCE selects which), and the DQS
// in turn needs a DDRDLL (PR0015 topological coupling).  So we instantiate the same
// DDRDLL -> DQS -> OSER8_MEM chain the controller uses, and expose the OSER8_MEM
// params (HWL / TCLK_SOURCE / TXCLK_POL) for differential fuzzing.
module top(
    input        pclk, fclk, reset,
    input  [7:0] d,
    input  [3:0] tx,
    input        dqsin, read,
    input  [2:0] rclksel,
    input        wstep, rloadn, rmove, rdir, wloadn, wmove, wdir, hold,
    input        dll_clkin, dll_stop, dll_updn,
    inout        dq_pad,                         // OSER8_MEM Q0 -> DDR tristate data pad
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
    // write-side 8:1 SerDes, TCLK from the write-DQS strobe (DQSW270), as in the ctrl.
    OSER8_MEM #(
        .HWL ("false"), .TCLK_SOURCE ("DQSW270"), .TXCLK_POL (1'b1)
    ) u_oser8_mem (
        .Q0(q0), .Q1(q1),
        .D0(d[0]), .D1(d[1]), .D2(d[2]), .D3(d[3]),
        .D4(d[4]), .D5(d[5]), .D6(d[6]), .D7(d[7]),
        .TX0(tx[0]), .TX1(tx[1]), .TX2(tx[2]), .TX3(tx[3]),
        .PCLK(pclk), .FCLK(fclk), .TCLK(dqsw270), .RESET(reset)
    );
    // OSER8_MEM data output (Q0) + tristate (Q1) drive a real DDR IO pad — synth pads
    // OSER8_MEM.Q only through a DDR-capable IOBUF, not a plain OBUF (CK0011 otherwise).
    IOBUF u_dq (.O(), .IO(dq_pad), .I(q0), .OEN(q1));
    assign dqsw0_o = dqsw0;  assign dqsw270_o = dqsw270;
endmodule

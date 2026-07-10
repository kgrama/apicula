`default_nettype none
module top(input wire clk, output wire led_lock, output wire led_clkout);
    wire lock, clkout0, clkout1, clkout2, clkout3, clkout4, clkout5, clkout6, g;
    assign g = 1'b0;
    PLL #(
        .FCLKIN("50"),
        .DYN_IDIV_SEL("FALSE"),
        .IDIV_SEL(1),
        .DYN_FBDIV_SEL("FALSE"),
        .FBDIV_SEL(1),
        .DYN_MDIV_SEL("FALSE"),
        .MDIV_SEL(18),
        .DYN_ODIV0_SEL("FALSE"),
        .ODIV0_SEL(36),
        .CLKOUT0_EN("TRUE"),
        .CLKFB_SEL("INTERNAL"),
        .ODIV1_SEL(32),
        .CLKOUT1_EN("TRUE"),
        .ODIV2_SEL(16),
        .CLKOUT2_EN("TRUE")
    ) pll_inst (
        .CLKOUT0(clkout0), .CLKOUT1(clkout1), .CLKOUT2(clkout2), .CLKOUT3(clkout3),
        .CLKOUT4(clkout4), .CLKOUT5(clkout5), .CLKOUT6(clkout6),
        .LOCK(lock), .CLKIN(clk), .CLKFB(g),
        .ENCLK0(1'b1), .ENCLK1(1'b1), .ENCLK2(1'b1), .ENCLK3(1'b1), .ENCLK4(1'b1), .ENCLK5(1'b1), .ENCLK6(1'b1),
        .FBDSEL({6{g}}), .IDSEL({6{g}}), .MDSEL({7{g}}), .MDSEL_FRAC({3{g}}),
        .ODSEL0({7{g}}), .ODSEL0_FRAC({3{g}}), .ODSEL1({7{g}}), .ODSEL2({7{g}}), .ODSEL3({7{g}}),
        .ODSEL4({7{g}}), .ODSEL5({7{g}}), .ODSEL6({7{g}}),
        .DT0({4{g}}), .DT1({4{g}}), .DT2({4{g}}), .DT3({4{g}}),
        .PSSEL({3{g}}), .PSDIR(g), .PSPULSE(g), .SSCPOL(g), .SSCON(g),
        .SSCMDSEL({7{g}}), .SSCMDSEL_FRAC({3{g}}), .ICPSEL({6{g}}), .LPFRES({3{g}}), .LPFCAP({2{g}}),
        .RESET(g), .PLLPWD(g), .RESET_I(g), .RESET_O(g)
    );
    assign led_lock = lock;
    assign led_clkout = clkout0 ^ clkout1 ^ clkout2 ^ clkout3 ^ clkout4 ^ clkout5 ^ clkout6;
endmodule

module clock_keeping #(parameter
	BAUD_RATE = 115200
) (
	input i_clk,
	input i_reset,
	output o_middle,
	output o_end
);

// The clock period is 40ns
localparam CYCLES_PER_BAUD = 1000000000 / BAUD_RATE / `CLOCK_PERIOD;

reg [$clog2(CYCLES_PER_BAUD)-1:0] r_clk_count = 0;
reg r_reset = 0;

always @(posedge i_clk) begin
	// Reset on posedge of i_reset only
	if (i_reset && !r_reset)
		r_clk_count <= 0;
	else if (o_end)
		r_clk_count <= 0;
	else
		r_clk_count <= r_clk_count + 1;

	r_reset <= i_reset;
end

assign o_middle = r_clk_count == CYCLES_PER_BAUD / 2 - 1;
assign o_end = r_clk_count == CYCLES_PER_BAUD - 1;

endmodule

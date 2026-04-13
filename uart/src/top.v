`include "constants.v"

module top #(parameter
	BAUD_RATE = 115200,
	DATA_BITS = 8,
	STOP_BITS = 1,
	PARITY_MODE = `EVEN_PARITY
) (
	input i_clk,
	input i_rx_data,
	output o_tx_data,
);

reg r_ready = 0;
wire w_ready, w_done, w_start;
wire [DATA_BITS-1:0] w_data;

uart_recv #(
	.BAUD_RATE(BAUD_RATE),
	.DATA_BITS(DATA_BITS),
	.STOP_BITS(STOP_BITS),
	.PARITY_MODE(PARITY_MODE)
) uart_rx(
	.i_clk(i_clk),
	.i_rx_data(i_rx_data),
	.o_data(w_data),
	.o_ready(w_ready)
);

uart_send #(
	.BAUD_RATE(BAUD_RATE),
	.DATA_BITS(DATA_BITS),
	.STOP_BITS(STOP_BITS),
	.PARITY_MODE(PARITY_MODE)
) uart_tx(
	.i_clk(i_clk),
	.i_data(w_data),
	.i_start(w_start),
	.o_tx_data(o_tx_data),
	.o_done(w_done)
);

always @(posedge i_clk) begin
	r_ready <= w_ready;
end

assign w_start = w_ready && !r_ready;

endmodule

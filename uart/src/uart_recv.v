`include "constants.v"

module uart_recv #(parameter
	BAUD_RATE = 115200,
	DATA_BITS = 8,
	STOP_BITS = 1,
	PARITY_MODE = `NONE_PARITY
) (
	input i_clk,
	input i_rx_data,
	output [DATA_BITS-1:0] o_data,
	output o_ready
);

localparam IDLE = 3'b0, IDLE_TO_DATA = 3'b1, DATA = 3'b10, PARITY = 3'b11;
localparam STOP = 3'b100;

reg [DATA_BITS-1:0] r_inflight_data = 0;

reg [2:0] r_state = IDLE;
reg [$clog2(DATA_BITS)-1:0] r_data_bit_pos = 0;

reg [DATA_BITS-1:0] r_data = 0;
reg r_ready = 0;

reg [1:0] r_stop_bits = 0;

wire w_good, w_middle, w_end;
wire w_reset;

parity #(
	.WIDTH(DATA_BITS),
	.PARITY_MODE(PARITY_MODE)
) par(
	.i_data(r_inflight_data),
	.i_parity(i_rx_data),
	.o_good(w_good)
);

clock_keeping #(
	.BAUD_RATE(BAUD_RATE)
) clk_keeping(
	.i_clk(i_clk),
	.i_reset(w_reset),
	.o_middle(w_middle),
	.o_end(w_end)
);

always @(posedge i_clk) begin
	case (r_state)
		IDLE: begin
			r_ready <= 0;
			if (i_rx_data == 0)
				r_state <= IDLE_TO_DATA;
		end
		IDLE_TO_DATA: begin
			if (w_end)
				r_state <= DATA;
		end
		DATA: begin
			if (w_middle)
				r_inflight_data[r_data_bit_pos] <= i_rx_data;

			if (w_end) begin
				if (r_data_bit_pos == DATA_BITS - 1) begin
					r_data_bit_pos <= 0;
					if (PARITY_MODE != `NONE_PARITY)
						r_state <= PARITY;
					else begin
						r_ready <= 1;
						r_data <= r_inflight_data;
						r_state <= (STOP_BITS == 0) ? IDLE : STOP;
					end
				end
				else
					r_data_bit_pos <= r_data_bit_pos + 1;
			end
		end
		PARITY: begin
			if (w_middle)
				if (w_good) begin
					r_ready <= 1;
					r_data <= r_inflight_data;
				end

			if (w_end)
				r_state <= (STOP_BITS == 0) ? IDLE : STOP;
		end
		STOP: begin
			if (w_end) begin
				if (r_stop_bits == STOP_BITS - 1) begin
					r_state <= IDLE;
					r_stop_bits <= 0;
				end
				else
					r_stop_bits <= r_stop_bits + 1;
			end
		end
		default: begin
		end
	endcase
end

assign o_data = r_data;
assign o_ready = r_ready;
assign w_reset = (i_rx_data == 0) && (r_state == IDLE);

initial begin
	$display("[%0t] Tracing to uart_recv.vcd...\n", $time);
	$dumpfile("uart_recv.vcd");
	$dumpvars();
end

endmodule

`include "constants.v"

module uart_send #(parameter
	BAUD_RATE = 115200,
	DATA_BITS = 8,
	STOP_BITS = 1,
	PARITY_MODE = `NONE_PARITY
) (
	input i_clk,
	input [DATA_BITS-1:0] i_data,
	input i_start,
	output o_tx_data,
	output o_done
);

localparam IDLE = 3'b0, IDLE_TO_DATA = 3'b1, DATA = 3'b10, PARITY = 3'b11;
localparam STOP = 3'b100;

reg [2:0] r_state = IDLE;
wire w_reset;

reg [$clog2(DATA_BITS)-1:0] r_data_pos;
reg [1:0] r_stop_bits;

wire w_middle, w_end;
wire w_parity;

reg r_tx_data = 1'b1;
reg r_done;

clock_keeping #(.BAUD_RATE(BAUD_RATE)) clk_keeping(
	.i_clk(i_clk),
	.i_reset(w_reset),
	.o_middle(w_middle),
	.o_end(w_end)
);

generate_parity #(
	.WIDTH(DATA_BITS),
	.PARITY_MODE(PARITY_MODE)
) gen(
	.i_data(i_data),
	.o_parity(w_parity)
);

always @(posedge i_clk) begin
	case (r_state)
		IDLE: begin
			if (i_start) begin
				r_tx_data <= 0;
				r_state <= IDLE_TO_DATA;
				r_data_pos <= 0;
				r_done <= 0;
			end
			else
				r_tx_data <= 1;
		end
		IDLE_TO_DATA: begin
			if (w_end) begin
				r_state <= DATA;
				r_tx_data <= i_data[r_data_pos];
			end
		end
		DATA: begin
			r_tx_data <= i_data[r_data_pos];
			if (w_end) begin
				if (r_data_pos == DATA_BITS - 1) begin
					if (PARITY_MODE != `NONE_PARITY) begin
						r_state <= PARITY;
						r_tx_data <= w_parity;
					end
					else
						r_state <= (STOP_BITS == 0) ? IDLE : STOP;
				end
				else begin
					r_data_pos <= r_data_pos + 1;
					r_tx_data <= i_data[r_data_pos + 1];
				end
			end
		end
		PARITY: begin
			r_tx_data <= w_parity;
			if (w_end) begin
				r_state <= (STOP_BITS == 0) ? IDLE : STOP;
				r_tx_data <= 1;
			end
		end
		STOP: begin
			r_tx_data <= 1;
			if (w_end) begin
				if (r_stop_bits == STOP_BITS - 1) begin
					r_state <= IDLE;
					r_stop_bits <= 0;
					r_done <= 1;
				end
				else
					r_stop_bits <= r_stop_bits + 1;
			end
		end
		default: begin
		end
	endcase
end

assign o_tx_data = r_tx_data;
assign o_done = r_done;
assign w_reset = i_start && r_state == IDLE;

initial begin
	$display("[%0t] Tracing to uart_send.vcd...\n", $time);
	$dumpfile("uart_send.vcd");
	$dumpvars();
end

endmodule

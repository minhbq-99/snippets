`include "constants.v"

module top #(parameter
	BAUD_RATE = 115200,
	STOP_BITS = 1,
	PARITY_MODE = `EVEN_PARITY
) (
	input i_clk,
	input i_rx_data,
	input i_reset,
	output o_tx_data
);

localparam DATA_BITS = 8;

reg r_ready = 0;
wire w_ready;
wire [DATA_BITS-1:0] w_rx_data;

// 2 rx states:
// - RX_META: 8 bytes which stores number of transferred bytes
// - RX_DATA: actual data transfer
localparam RX_META = 0, RX_DATA = 1;
reg r_rx_state = RX_META;

localparam META_BITS = 64, META_BYTES = 8;
reg [META_BITS-1:0] r_rx_meta;
reg [META_BITS-1:0] r_rx_data_byte_pos = 0;
reg [$clog2(META_BYTES)-1:0] r_rx_meta_byte_pos = 0;
reg [DATA_BITS-1:0] r_rx_data;

localparam CRC_IDLE = 0, CRC_PENDING = 1;
reg r_crc_state = CRC_IDLE;
reg [$clog2(DATA_BITS)-1:0] r_crc_bit_pos = 0;
wire w_crc_data, w_crc_enable;
reg r_crc_reset = 1;

localparam CRC_WIDTH = 32;
wire [CRC_WIDTH-1:0] w_crc;

// We assume CRC32
localparam TX_BYTES = 4;
wire [META_BITS-1:0] w_tx_meta = TX_BYTES;
// FIXME: Is it a better way here?
reg signed [$clog2(META_BYTES):0] r_tx_meta_byte_pos = -1;
reg [$clog2(TX_BYTES)-1:0] r_tx_data_byte_pos = 0;

localparam TX_IDLE = 2'b00, TX_META = 2'b01, TX_DATA = 2'b10;
reg [2:0] r_tx_state = TX_IDLE;
wire [DATA_BITS-1:0] w_tx_data;
wire w_tx_done;
reg r_tx_start = 0;

uart_recv #(
	.BAUD_RATE(BAUD_RATE),
	.DATA_BITS(DATA_BITS),
	.STOP_BITS(STOP_BITS),
	.PARITY_MODE(PARITY_MODE)
) uart_rx(
	.i_clk(i_clk),
	.i_rx_data(i_rx_data),
	.o_data(w_rx_data),
	.o_ready(w_ready)
);

uart_send #(
	.BAUD_RATE(BAUD_RATE),
	.DATA_BITS(DATA_BITS),
	.STOP_BITS(STOP_BITS),
	.PARITY_MODE(PARITY_MODE)
) uart_tx(
	.i_clk(i_clk),
	.i_data(w_tx_data),
	.i_start(r_tx_start),
	.o_tx_data(o_tx_data),
	.o_done(w_tx_done)
);

crc crc32(
	.i_clk(i_clk),
	.i_data(w_crc_data),
	.i_enable(w_crc_enable),
	.i_reset(r_crc_reset),
	.o_crc(w_crc)
);

always @(posedge i_clk) begin
	if (i_reset) begin
		r_ready <= 0;
		r_rx_state <= RX_META;
		r_rx_data_byte_pos <= 0;
		r_rx_meta_byte_pos <= 0;
		r_crc_state <= CRC_IDLE;
		r_crc_reset <= 1;
		r_crc_bit_pos <= 0;
		r_tx_state <= TX_IDLE;
		r_tx_meta_byte_pos <= -1;
		r_tx_data_byte_pos <= 0;
	end
	else begin

	// RX handler
	r_ready <= w_ready;
	// FIXME: We don't handle new data until completing TX transfer.
	// We need to expose this information to user.
	if (w_ready && !r_ready && r_tx_state == TX_IDLE) begin
		r_crc_reset <= 0;
		if (r_rx_state == RX_META) begin
			r_rx_meta[((r_rx_meta_byte_pos+1)*DATA_BITS-1) -: DATA_BITS] <= w_rx_data;
			if (r_rx_meta_byte_pos == META_BYTES - 1) begin
				r_rx_meta_byte_pos <= 0;
				if (r_rx_meta != 0)
					r_rx_state <= RX_DATA;
			end
			else
				r_rx_meta_byte_pos <= r_rx_meta_byte_pos + 1;
		end
		else begin
			r_rx_data <= w_rx_data;
			r_crc_state <= CRC_PENDING;
			if (r_rx_data_byte_pos == r_rx_meta - 1) begin
				r_rx_data_byte_pos <= 0;
				r_rx_state <= RX_META;
				r_tx_state <= TX_META;
			end
			else
				r_rx_data_byte_pos  <= r_rx_data_byte_pos + 1;
		end
	end

	if (r_crc_state == CRC_PENDING) begin
		if (r_crc_bit_pos == DATA_BITS - 1) begin
			r_crc_state <= CRC_IDLE;
			r_crc_bit_pos = 0;
		end
		else
			r_crc_bit_pos <= r_crc_bit_pos + 1;
	end

	// TX handler
	if (w_tx_done) begin
		if (r_tx_state == TX_META) begin
			r_tx_start <= 1;
			if (r_tx_meta_byte_pos == META_BYTES - 1) begin
				r_tx_meta_byte_pos <= -1;
				r_tx_state <= TX_DATA;
			end
			else
				r_tx_meta_byte_pos <= r_tx_meta_byte_pos + 1;
		end
		else if (r_tx_state == TX_DATA) begin
			// We assume at this time the CRC computation has completed
			if (r_tx_data_byte_pos == TX_BYTES - 1) begin
				r_tx_data_byte_pos <= 0;
				r_tx_state <= TX_IDLE;
				r_crc_reset <= 1;
				r_tx_start <= 0;
			end
			else
			begin
				r_tx_data_byte_pos <= r_tx_data_byte_pos + 1;
				r_tx_start <= 1;
			end
		end
	end
	else
		// Reset when a byte is being transferred
		r_tx_start <= 0;

	end
end

assign w_crc_data = r_rx_data[r_crc_bit_pos];
assign w_crc_enable = r_crc_state == CRC_PENDING;
assign w_tx_data = (r_tx_state == TX_META) ?
	w_tx_meta[((r_tx_meta_byte_pos+1)*DATA_BITS-1) -: DATA_BITS] :
	w_crc[((r_tx_data_byte_pos+1)*DATA_BITS-1) -: DATA_BITS];

initial begin
	$display("[%0t] Tracing to top.vcd...\n", $time);
	$dumpfile("top.vcd");
	$dumpvars();
end

endmodule

module crc(
	input i_data,
	input i_clk,
	input i_enable,
	input i_reset,
	output [32-1:0] o_crc
);

reg [32-1:0] r_crc;
wire w_temp;

always @(posedge i_clk or posedge i_reset) begin
	if (i_reset)
		r_crc <= {32{1'b1}};
	else begin
		if (i_enable) begin
			r_crc[31] <= w_temp;
			r_crc[30] <= r_crc[31] ^ w_temp;
			r_crc[29] <= r_crc[30] ^ w_temp;
			r_crc[28] <= r_crc[29];
			r_crc[27] <= r_crc[28] ^ w_temp;
			r_crc[26] <= r_crc[27] ^ w_temp;
			r_crc[25] <= r_crc[26];
			r_crc[24] <= r_crc[25] ^ w_temp;
			r_crc[23] <= r_crc[24] ^ w_temp;
			r_crc[22] <= r_crc[23];
			r_crc[21] <= r_crc[22] ^ w_temp;
			r_crc[20] <= r_crc[21] ^ w_temp;
			r_crc[19] <= r_crc[20] ^ w_temp;
			r_crc[18] <= r_crc[19];
			r_crc[17] <= r_crc[18];
			r_crc[16] <= r_crc[17];
			r_crc[15] <= r_crc[16] ^ w_temp;
			r_crc[14] <= r_crc[15];
			r_crc[13] <= r_crc[14];
			r_crc[12] <= r_crc[13];
			r_crc[11] <= r_crc[12];
			r_crc[10] <= r_crc[11];
			r_crc[9] <= r_crc[10] ^ w_temp;
			r_crc[8] <= r_crc[9] ^ w_temp;
			r_crc[7] <= r_crc[8];
			r_crc[6] <= r_crc[7];
			r_crc[5] <= r_crc[6] ^ w_temp;
			r_crc[4] <= r_crc[5];
			r_crc[3] <= r_crc[4];
			r_crc[2] <= r_crc[3];
			r_crc[1] <= r_crc[2];
			r_crc[0] <= r_crc[1];

		end
	end
end

assign w_temp = r_crc[0] ^ i_data;
assign o_crc = ~r_crc;

initial begin
	$display("[%0t] Tracing to crc.vcd...\n", $time);
	$dumpfile("crc.vcd");
	$dumpvars();
end

endmodule

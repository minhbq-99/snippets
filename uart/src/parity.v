`include "constants.v"

module parity #(parameter
	WIDTH = 8,
	PARITY_MODE = `EVEN_PARITY // EVEN_PARITY or ODD_PARITY
) (
	input [WIDTH-1:0] i_data,
	input i_parity,
	output o_good
);

wire w_ret = (^i_data) ^ i_parity;
assign o_good = (PARITY_MODE == `ODD_PARITY && w_ret == 1) ||
	(PARITY_MODE == `EVEN_PARITY && w_ret == 0);

endmodule


module generate_parity #(parameter
	WIDTH = 8,
	PARITY_MODE = `EVEN_PARITY
) (
	input [WIDTH-1:0] i_data,
	output o_parity
);

wire w_ret = (^i_data);
assign o_parity = (PARITY_MODE == `ODD_PARITY) ? ~w_ret : w_ret;

endmodule

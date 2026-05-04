#include <verilated.h>
#include <verilated_vcd_c.h>

#include "obj_dir/Vtop.h"

// 1 baud 2 clock cycles
void two_clock_cycles(Vtop &top)
{
	for (int i = 0; i < 2; i++) {
		Verilated::timeInc(1);
		top.i_clk = 0;
		top.eval();

		Verilated::timeInc(1);
		top.i_clk = 1;

		if (i < 1)
			top.eval();
	}
}

void send_byte(Vtop &top, unsigned char input)
{
	// Start bit
	two_clock_cycles(top);
	top.i_rx_data = 0;
	top.eval();

	// Send data
	for (int i = 0; i < 8; i++) {
		two_clock_cycles(top);
		top.i_rx_data = input & 1;
		top.eval();

		input = input >> 1;
	}

	two_clock_cycles(top);
	top.i_rx_data = 1;
	top.eval();
}

int main(int argc, char **argv)
{
	Verilated::commandArgs(argc, argv);
	Verilated::traceEverOn(true);

	Vtop top = Vtop();

	Verilated::timeInc(1);
	top.i_rx_data = 1;
	top.i_clk = 1;
	top.eval();

	for (int j = 0; j < 2; j++) {
		send_byte(top, '\x01');
		for (int i = 0; i < 7; i++)
			send_byte(top, '\x00');

		send_byte(top, '\x01');

		for (int i = 0; i < 200; i++) {
			two_clock_cycles(top);
			top.eval();
		}
	}

	top.final();

	return 0;
}

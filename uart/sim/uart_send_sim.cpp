#include <verilated.h>
#include <verilated_vcd_c.h>

#include <iostream>

#include "obj_dir/Vuart_send.h"

// 1 baud 2 clock cycles
void two_clock_cycles(Vuart_send &uart)
{
	for (int i = 0; i < 2; i++) {
		Verilated::timeInc(1);
		uart.i_clk = 0;
		uart.eval();

		Verilated::timeInc(1);
		uart.i_clk = 1;
		uart.eval();
	}
}

int main(int argc, char **argv)
{
	Verilated::commandArgs(argc, argv);
	Verilated::traceEverOn(true);

	Vuart_send uart = Vuart_send();

	Verilated::timeInc(1);
	uart.i_clk = 1;
	uart.eval();

	Verilated::timeInc(1);
	uart.i_clk = 0;
	uart.eval();

	{
		Verilated::timeInc(1);
		uart.i_data = 0x1;
		uart.i_start = 1;
		uart.i_clk = 1;
		uart.eval();

		for (int i = 0; i < 10; i++) {
			two_clock_cycles(uart);
		}
	}

	uart.final();

	return 0;
}

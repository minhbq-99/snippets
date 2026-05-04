#include <verilated.h>
#include <verilated_vcd_c.h>

#include <iostream>

#include "obj_dir/Vuart_recv.h"

// 1 baud 2 clock cycles
void two_clock_cycles(Vuart_recv &uart)
{
	for (int i = 0; i < 2; i++) {
		Verilated::timeInc(1);
		uart.i_clk = 0;
		uart.eval();

		Verilated::timeInc(1);
		uart.i_clk = 1;

		if (i < 1)
			uart.eval();
	}
}

int main(int argc, char **argv)
{
	Verilated::commandArgs(argc, argv);
	Verilated::traceEverOn(true);

	Vuart_recv uart = Vuart_recv();

	Verilated::timeInc(1);
	uart.i_rx_data = 1;
	uart.i_clk = 1;
	uart.eval();

	Verilated::timeInc(1);
	uart.i_clk = 0;
	uart.eval();

	{
		Verilated::timeInc(1);
		uart.i_clk = 1;
		// Start bit
		uart.i_rx_data = 0;
		uart.eval();

		// Send data = 50 = ord('2') = 0b00110010
		int data[8] = {0, 1, 0, 0, 1, 1, 0, 0};
		for (int i = 0; i < 8; i++) {
			two_clock_cycles(uart);
			uart.i_rx_data = data[i];
			uart.eval();
		}

		two_clock_cycles(uart);
		// Parity bit
		uart.i_rx_data = 1;
		uart.eval();

		two_clock_cycles(uart);
		// Stop bit
		uart.i_rx_data = 1;
		uart.eval();

		two_clock_cycles(uart);
		uart.eval();

		two_clock_cycles(uart);
		uart.eval();

		unsigned int output = uart.o_data;
		std::cout << "Output: " << output << std::endl;
	}

	{
		// New data
		// Start bit
		two_clock_cycles(uart);
		uart.i_rx_data = 0;
		uart.eval();

		// Send data = 51 = ord('3') = 0b00110011
		int data[8] = {1, 1, 0, 0, 1, 1, 0, 0};
		for (int i = 0; i < 8; i++) {
			two_clock_cycles(uart);
			uart.i_rx_data = data[i];
			uart.eval();
		}

		two_clock_cycles(uart);
		// Parity bit
		uart.i_rx_data = 0;
		uart.eval();

		two_clock_cycles(uart);
		// Stop bit
		uart.i_rx_data = 1;
		uart.eval();

		two_clock_cycles(uart);
		uart.eval();

		unsigned int output = uart.o_data;
		std::cout << "Output: " << output << std::endl;
	}

	uart.final();

	return 0;
}

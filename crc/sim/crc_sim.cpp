#include <verilated.h>
#include <verilated_vcd_c.h>

#include <iostream>
#include <format>

#include "obj_dir/Vcrc.h"

int main(int argc, char **argv)
{
	unsigned int output;

	Verilated::commandArgs(argc, argv);
	Verilated::traceEverOn(true);

	{
		Vcrc crc = Vcrc();

		Verilated::timeInc(1);
		crc.i_clk = 1;
		crc.eval();
		Verilated::timeInc(1);
		crc.i_clk = 0;
		crc.eval();

		Verilated::timeInc(1);
		crc.i_reset = 1;
		crc.i_clk = 1;
		crc.eval();

		Verilated::timeInc(1);
		crc.i_reset = 0;
		crc.i_clk = 0;
		crc.eval();

		unsigned int input = 0x41414141;
		for (int i = 0; i < 32; i++) {
			Verilated::timeInc(1);
			crc.i_data = input & 1;
			crc.i_enable = 1;
			crc.i_clk = 1;
			crc.eval();

			Verilated::timeInc(1);
			crc.i_clk = 0;
			crc.eval();

			input = input >> 1;
		}

		Verilated::timeInc(1);
		crc.i_enable = 0;
		crc.i_clk = 1;
		crc.eval();

		crc.final();

		output = crc.o_crc;
	}

	std::cout << std::format("{:08X}", output) << std::endl;
	assert(output == 0x9B0D08F1);
	return 0;
}

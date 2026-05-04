sim-compile:
	for sim in uart_send uart_recv crc top; do \
		g++ -std=c++20 -I/usr/local/share/verilator/include \
			$${sim}_sim.cpp -o $${sim}_sim \
			-L./obj_dir -lV$${sim} -lverilated; \
	done

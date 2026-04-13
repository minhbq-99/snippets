sim-compile:
	for sim in send recv; do \
		g++ -I/usr/local/share/verilator/include \
			uart_$${sim}_sim.cpp -o uart_$${sim}_sim \
			-L./obj_dir -lVuart_$${sim} -lverilated; \
	done

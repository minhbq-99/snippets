PROJ = top
FPGA_PKG = vq100
FPGA_TYPE = hx1k

ADD_SRC = src/uart_recv.v src/uart_send.v src/parity.v src/clock_keeping.v \
	src/crc.v

crc-generate:
	uv run script/crc_generate.py

%.json: src/%.v $(ADD_SRC) crc-generate
	yosys -ql $(basename $@)-yosys.log -p \
		'synth_ice40 -top $(basename $@) -json $@' $< $(ADD_SRC)

%.asc: %.json
	nextpnr-ice40 --${FPGA_TYPE} --package ${FPGA_PKG} \
		--json $< --pcf config/${PROJ}.pcf --sdc config/${PROJ}.sdc \
		--asc $@

%.rpt: %.asc
	icetime -d ${FPGA_TYPE} -mtr $@ $<

%.bin: %.asc
	icepack $< $@

compile: ${PROJ}.bin ${PROJ}.rpt

program: compile
	sudo iceprog ${PROJ}.bin

sim-generate: crc-generate
	for sim in uart_send uart_recv; do \
		verilator --trace-vcd --Mdir sim/obj_dir -Isrc \
			-Wno-WIDTHEXPAND -Wno-WIDTHTRUNC \
			-cc src/$${sim}.v src/parity.v \
			--top-module $${sim} \
			-GBAUD_RATE=12500000 -GPARITY_MODE=1; \
		make -C sim/obj_dir -f V$${sim}.mk; \
	done

	verilator --trace-vcd --Mdir sim/obj_dir -Isrc \
		-Wno-WIDTHEXPAND -Wno-WIDTHTRUNC \
		-cc src/crc.v \
		--top-module crc
	make -C sim/obj_dir -f Vcrc.mk;

	verilator --trace-vcd --Mdir sim/obj_dir -Isrc \
		-Wno-WIDTHEXPAND -Wno-WIDTHTRUNC \
		-cc src/top.v src/parity.v \
		--top-module top \
		-GBAUD_RATE=12500000 -GPARITY_MODE=2 -GSTOP_BITS=0;
	make -C sim/obj_dir -f Vtop.mk; \

sim-compile: sim-generate
	make -C sim sim-compile

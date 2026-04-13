PROJ = top
FPGA_PKG = vq100
FPGA_TYPE = hx1k

ADD_SRC = src/uart_recv.v src/uart_send.v src/parity.v src/clock_keeping.v

%.json: src/%.v $(ADD_SRC)
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

program:
	sudo iceprog ${PROJ}.bin

sim-generate:
	for sim in send recv; do \
		verilator --trace-vcd --Mdir sim/obj_dir -Isrc \
			-Wno-WIDTHEXPAND -Wno-WIDTHTRUNC \
			-cc src/uart_$${sim}.v \
			-GBAUD_RATE=12500000 -GPARITY_MODE=1; \
		make -C sim/obj_dir -f Vuart_$${sim}.mk; \
	done

sim-compile:
	make -C sim sim-compile

# dependencies = [
#	"pyserial",
# ]
import serial
import struct

def read_exact(ser: serial.Serial, length: int) -> bytes:
	ret = b""
	while len(ret) != length:
		ret += ser.read()

	return ret

def read(ser: serial.Serial) -> bytes:
	inp = read_exact(ser, 8)
	length = struct.unpack('<Q', inp)[0]
	return read_exact(ser, length)

def send(ser: serial.Serial, output: bytes):
	length = len(output)
	meta = struct.pack('<Q', length)
	ser.write(meta + output)

ser = serial.Serial("/dev/ttyUSB1", 115200, parity=serial.PARITY_EVEN)
print(ser)

data = b"AAAABBBB"
send(ser, data)
crc = read(ser)
crc_num = struct.unpack("<I", crc)[0]
print(f"CRC32: 0x{crc_num:08X}")

send(ser, data + crc)
crc = read(ser)
crc_num = struct.unpack("<I", crc)[0]
# With every data, the result must always be equal to this residue
assert crc_num == 0x2144DF1C

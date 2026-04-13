# dependencies = [
#	"pyserial",
# ]
import serial

def write(ser):
	inp = input("Enter a number: ")
	number = int(inp)
	if number < 0 or number >= 256:
		print("Invalid input")
		return False
	else:
		ser.write(number.to_bytes(1))
		return True

def read(ser):
	byte = ser.read()
	print(f"Recv: {byte[0]}")

ser = serial.Serial("/dev/ttyUSB1", 115200, parity=serial.PARITY_EVEN)
print(ser)

try:
	while True:
		if write(ser):
			read(ser)
except KeyboardInterrupt:
	ser.close()

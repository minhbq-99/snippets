# dependencies = [
#	"jinja2",
# ]

import jinja2
import os

CRC_ALGO = (32, 0x04c11db7)

def generate_code(width, polynomial):
	ret = ""
	ret += f"\t\t\tr_crc[{width-1}] <= w_temp;\n"
	polynomial = polynomial >> 1
	for i in range(width-2, -1, -1):
		if polynomial & 1 != 0:
			ret += f"\t\t\tr_crc[{i}] <= r_crc[{i+1}] ^ w_temp;\n"
		else:
			ret += f"\t\t\tr_crc[{i}] <= r_crc[{i+1}];\n"
		polynomial = polynomial >> 1

	return ret

dirname = os.path.dirname(os.path.abspath(__file__))
template_dir = os.path.join(dirname, "templates")
loader = jinja2.FileSystemLoader(searchpath=template_dir)
env = jinja2.Environment(loader=loader)

template = env.get_template("crc.v.j2")
data = {"width": str(CRC_ALGO[0]), "data": generate_code(CRC_ALGO[0], CRC_ALGO[1])}
output = template.render(data)

source_dir = os.path.join(os.path.dirname(dirname), "src")
output_filename = os.path.join(source_dir, "crc.v")
with open(output_filename, "w") as file:
	file.write(output)

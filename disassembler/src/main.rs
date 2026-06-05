use std::{fs, env};

mod disassembler;

use disassembler::disassembler::Reader;

// 64-bit mode disassembler
fn main() {
    let mut args = env::args();
    if args.len() != 2 && args.len() != 3 {
        panic!("Usage: {} file [base]", args.next().unwrap());
    }

    let prog_args: Vec<String> = args.collect();
    let file_name = &prog_args[1];
    let mut base = 0;
    if prog_args.len() == 3 {
        base = u64::from_str_radix(&prog_args[2], 16).unwrap();
    }

    let content = fs::read_to_string(file_name).unwrap();
    let data = content.trim();

    if data.len() % 2 != 0 {
        panic!("Invalid hex string, length: {}", data.len())
    }

    let mut reader = Reader::new_with_base(data, base);
    println!("{}", reader.disassemble().unwrap());
}

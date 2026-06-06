use std::{collections::HashMap, sync::LazyLock};

use num_enum::{TryFromPrimitive, IntoPrimitive};
use paste::paste;

#[derive(Debug)]
pub struct Reader {
    data: String,
    /// The current position in the hex string
    current: usize,
    base_address: u64,
}

macro_rules! gen_parse_reg_rm {
    ($op_name: literal, $bit8_mr: literal, $bit8_rm: literal, $bit32_mr: literal, $bit32_rm: literal) => {

        paste! {
            fn [<parse_ $op_name _reg_rm>] (&mut self, opcode: u8, prefix: &Prefix) -> String {
                let byte = self.u8().unwrap();
                let operand_size;
                let operand_encoding;
                match opcode {
                    $bit8_mr | $bit8_rm => {
                        operand_size = OperandSize::Size8Bit;
                        if opcode == $bit8_mr {
                            operand_encoding = OperandEncoding::MR;
                        } else {
                            operand_encoding = OperandEncoding::RM;
                        }
                    }
                    $bit32_mr | $bit32_rm => {
                        operand_size = prefix.get_operand_size();
                        if opcode == $bit32_mr {
                            operand_encoding = OperandEncoding::MR;
                        } else {
                            operand_encoding = OperandEncoding::RM;
                        }
                    },
                    _ => unreachable!(),
                };

                let (reg, rm) = self.parse_modrm(byte, &prefix, operand_size);
                Self::format_instruction($op_name, &reg, &rm, operand_encoding)
            }
        }
    };
}

macro_rules! gen_read {
    ($size: literal) => {
        paste! {
            fn  [<peek_u $size>](&self) -> Result<[<u $size>], String> {
                if self.current + $size / 8 * 2 > self.data.len() {
                    return Err(String::from("EOF"))
                }

                let raw = &self.data[self.current..self.current+($size / 8 * 2)];
                match [<u $size>]::from_str_radix(raw, 16) {
                    Ok(mut num) => {
                        // The number is in little endian so we need to fix it
                        let mut ret: [<u $size>] = 0;
                        for _ in 0..($size / 8) {
                            // In case gen_read!(8), the following shifts will overflow.
                            // But it's fine, use wrapping to avoid panic.
                            ret = ret.wrapping_shl(8) | (num & 0xff);
                            num = num.wrapping_shr(8);
                        }
                        Ok(ret)
                    },
                    Err(err) => Err(format!("{}: input: {}", err, raw)),
                }
            }

            fn [<u $size>](&mut self) -> Result<[<u $size>], String> {
                let ret = self.[<peek_u $size>]()?;
                self.current += ($size / 8 * 2);
                return Ok(ret);
            }
        }
    };
}

impl Reader {
    #[allow(dead_code)]
    pub fn new(data: &str) -> Self {
        Self {
            data: String::from(data),
            current: 0,
            base_address: 0,
        }
    }

    #[allow(dead_code)]
    pub fn new_with_base(data: &str, base_address: u64) -> Self {
        Self {
            data: String::from(data),
            current: 0,
            base_address: base_address,
        }
    }

    fn current_byte(&self) -> usize {
        self.current / 2
    }

    fn change_current_byte(&mut self, delta: i64) {
        self.current = ((self.current as i64) + delta) as usize;
    }

    gen_parse_reg_rm!("mov", 0x88, 0x8a, 0x89, 0x8b);
    gen_parse_reg_rm!("sub", 0x28, 0x2a, 0x29, 0x2b);
    gen_parse_reg_rm!("xor", 0x30, 0x32, 0x31, 0x33);
    gen_read!(8);
    gen_read!(16);
    gen_read!(32);
    gen_read!(64);

    pub fn disassemble(&mut self) -> Result <String, String> {
	let mut result = String::new();
        let mut prefix = Prefix::new();
        loop {
            let byte = self.u8();

            let mut end_of_instruction = true;
            let mut instruction = String::new();
            match byte {
                Ok(byte) => {
                    match byte {
                        0x0f => {
                            let opcode = self.u8().unwrap();
                            match opcode {
                                0x80..=0x8f => {
                                    instruction = self.parse_jcc_rel32(opcode);
                                }
                                other => todo!("Unimplemented opcode: 0x0f {:#02x}", other)
                            }
                        }
                        0x2e | 0x36 | 0x3e | 0x26 | 0x64 | 0x65 => {
                            prefix.parse_segment_override(byte);
                            end_of_instruction = false;
                        }
                        0x28..=0x2b => {
                            instruction = self.parse_sub_reg_rm(byte, &prefix)
                        }
                        0x30..=0x33=> {
                            instruction = self.parse_xor_reg_rm(byte, &prefix)
                        }
                        0x40..=0x4F => {
                            // FIXME: REX prefix is valid only if it immediately
                            // precedes opcode byte
                            prefix.rex = Some(parse_rex(byte));
                            end_of_instruction = false;
                        }
                        0x50..=0x58 => {
                            instruction = self.parse_push_reg(byte, &prefix);
                        }
                        // Operand size prefix to switch between 16-bit and 32-bit
                        0x66 => {
                            prefix.has_0x66 = true;
                            end_of_instruction = false;
                        }
                        // Address size prefix to switch between 32-bit and 64-bit
                        0x67 => {
                            prefix.has_0x67 = true;
                            end_of_instruction = false;
                        }
                        0x70..=0x7f => {
                            instruction = self.parse_jcc_rel8(byte);
                        }
                        0x83 => {
                            instruction = self.parse_alu_rm_imm8(&prefix);
                        }
                        0x88..=0x8c | 0x8e => {
                            instruction = self.parse_mov_reg_rm(byte, &prefix);
                        }
                        0x8d => {
                            instruction = self.parse_lea(&prefix);
                        }
                        0xb0..=0xbf => {
                            instruction = self.parse_mov_rm_imm(byte, &prefix);
                        }
                        0xc3 => {
                            instruction = String::from("ret");
                        }
                        0xc9 => {
                            instruction = String::from("leave");
                        }
                        0xe8 => {
                            instruction = self.parse_call();
                        }
                        0xf0..=0xf3 => {
                            prefix.group1_prefix = Some(byte);
                            end_of_instruction = false;

                            // FIXME: dirty hack because I'm too lazy
                            if byte == 0xf3 {
                                let next1 = self.u8();
                                let next2 = self.u8();
                                let next3 = self.u8();
                                if next1.is_ok() && next2.is_ok() && next3.is_ok() {
                                    if next1.unwrap() == 0x0f && next2.unwrap() == 0x1e && next3.unwrap() == 0xfa {
                                        instruction = String::from("endbr64");
                                        end_of_instruction = true;
                                    } else {
                                        self.change_current_byte(-3);
                                    }
                                } else {
                                    self.change_current_byte(-3);
                                }
                            }
                        }
                        other => todo!("Unimplemented opcode: 0x{:02x}", other)
                    }
                },
                Err(err) => {
                    if err == "EOF" {
                        return Ok(result);
                    } else {
                        return Err(format!("Failed to parse byte, err: {}", err));
                    }
                }
            }

            if end_of_instruction {
                result = Self::push_instruction(result, instruction);
                prefix = Prefix::new();
            }
        }
    }

    fn push_instruction(mut result: String, instruction: String) -> String {
        if result != "" {
            result += "\n";
        }

        result += &instruction;
        result
    }

    fn get_imm(&mut self, num_of_bits: u8) -> i64 {
        match num_of_bits {
            8 => {
                (self.u8().unwrap() as i8) as i64
            }
            16 => {
                (self.u16().unwrap() as i16) as i64
            }
            32 => {
                (self.u32().unwrap() as i32) as i64
            }
            64 => {
                self.u64().unwrap() as i64
            }
            _ => unreachable!()
        }
    }

    fn format_instruction(opcode: &str, reg: &str, rm: &str, encoding: OperandEncoding) -> String {
        if encoding == OperandEncoding::MR {
            format!("{} {}, {}", opcode, rm, reg)
        } else {
            // OperandEncoding::RM
            format!("{} {}, {}", opcode, reg, rm)
        }
    }

    /// parse_modrm returns register and rm
    fn parse_modrm(&mut self, byte: u8, prefix: &Prefix, operand_size: OperandSize) -> (String, String) {
        let mut modrm = ModRM::new(byte);
        if let Some(ref rex) = prefix.rex {
            modrm.reg = (rex.r << 3) | modrm.reg;
            modrm.rm = (rex.b << 3) | modrm.rm;
        }

        let has_rex = prefix.rex.is_some();
        let reg = get_reg(modrm.reg, operand_size, has_rex);

        let address_size;
        if prefix.has_0x67 {
            address_size = OperandSize::Size32Bit;
        } else {
            address_size = OperandSize::Size64Bit;
        }

        let mut rm: String;
        if modrm.addressing_mode != AddressingMode::RegisterToRegister {
            // The correct way might be check if modrm.rm is 0b111 before
            // appending rex.b to it. But we have already appended rex.b
            // at this point, so check 3 LSb only.
            match modrm.rm & 0b111 {
                0b100 => {
                    // SIB byte addressing
                    let sib_byte = self.u8().unwrap();
                    let sib = parse_sib(sib_byte, &prefix, modrm.addressing_mode, address_size);

                    let mut displacement = 0;
                    // [scaled index] + disp32
                    if sib.base_reg == "" && modrm.addressing_mode == AddressingMode::Memory {
                        displacement = self.get_imm(32);
                    } else {
                        let bits = modrm.addressing_mode.displacement_bits();
                        if bits != 0 {
                            displacement = self.get_imm(bits);
                        }
                    }

                    let mut scale_str = String::from("");
                    if sib.index_reg != "" {
                        scale_str = format!("{}*{}", sib.index_reg, sib.scale);
                    }

                    let mut displacement_str = String::from("");
                    if displacement < 0 {
                        displacement_str = format!("-{:#x}", displacement.abs());
                    } else if displacement > 0 {
                        displacement_str = format!("{:+#x}", displacement.abs());
                    }

                    rm = sib.base_reg;
                    if scale_str != "" {
                        if rm == "" {
                            rm += &format!("{}", scale_str);
                        } else {
                            rm += &format!("+{}", scale_str);
                        }
                    }

                    if rm == "" {
                        // Remove +/- when only displacement exists
                        rm = format!("{}[{}]", prefix.segment_override, rm + &displacement_str[1..]);
                    } else {
                        rm = format!("{}[{}]", prefix.segment_override, rm + &displacement_str);
                    }
                }
                _ => {
                    // RIP relative addressing
                    if modrm.rm == 0b101 && modrm.addressing_mode == AddressingMode::Memory {
                        let offset = self.u32().unwrap() as i32;
                        if offset < 0 {
                            rm = format!("[rip-{:#x}]", offset.abs());
                        } else {
                            rm = format!("[rip{:+#x}]", offset);
                        }
                    } else {
                        let bits = modrm.addressing_mode.displacement_bits();
                        let mut displacement = 0;
                        if bits != 0 {
                            displacement = self.get_imm(bits);
                        }

                        // has_rex is irrelevant here
                        let base_reg = get_reg(modrm.rm, address_size, has_rex);

                        if displacement < 0 {
                            rm = format!("{}[{}-{:#x}]", prefix.segment_override, base_reg, displacement.abs());
                        } else if displacement > 0 {
                            rm = format!("{}[{}{:+#x}]", prefix.segment_override, base_reg, displacement);
                        } else {
                            rm = format!("{}[{}]", prefix.segment_override, base_reg);
                        }
                }
                }
            }
        } else {
            rm = get_reg(modrm.rm, operand_size, has_rex);
        }

        return (reg, rm)
    }

    fn parse_mov_rm_imm(&mut self, opcode: u8, prefix: &Prefix) -> String {
        let operand_size: OperandSize;
        let mut reg;

        if opcode < 0xb8 {
            reg = opcode - 0xb0;
            operand_size = OperandSize::Size8Bit;
        } else {
            reg = opcode - 0xb8;
            operand_size = prefix.get_operand_size();
        }

        if let Some(ref rex) = prefix.rex {
            reg = (rex.b << 3) | reg;
        }

        let reg_str = get_reg(reg, operand_size, prefix.rex.is_some());
        let imm = self.get_imm(operand_size.imm_bits(true));

        format!("mov {}, {:#x}", reg_str, imm)
    }

    fn parse_lea(&mut self, prefix: &Prefix) -> String {
        let operand_size = prefix.get_operand_size();
        let byte = self.u8().unwrap();
        let (reg, rm) = self.parse_modrm(byte, prefix, operand_size);

        format!("lea {}, {}", reg, rm)
    }

    fn parse_alu_rm_imm8(&mut self, prefix: &Prefix) -> String {
        static ALU_MAP: LazyLock<HashMap<u8, String>> = LazyLock::new(|| {
            let mut map = HashMap::new();
            map.insert(0, String::from("add"));
            map.insert(1, String::from("or"));
            map.insert(2, String::from("adc"));
            map.insert(3, String::from("sbb"));
            map.insert(4, String::from("and"));
            map.insert(5, String::from("sub"));
            map.insert(6, String::from("xor"));
            map.insert(7, String::from("cmp"));
            map
        });

        let operand_size = prefix.get_operand_size();
        let byte = self.u8().unwrap();
        let mut modrm = ModRM::new(byte);

        if let Some(ref rex) = prefix.rex {
            modrm.rm  = (rex.b << 3) | modrm.rm;
        }
        let rm = get_reg(modrm.rm, operand_size, prefix.rex.is_some());

        let opcode = ALU_MAP.get(&modrm.reg).unwrap();
        let imm = self.get_imm(8) as i8;
        // FIXME: The output might be confusing when imm is negative
        format!("{} {}, {:#02x}", opcode, rm, imm)
    }

    fn parse_call(&mut self) -> String {
        let displacement = self.get_imm(32) as u64;
        let pc = (self.base_address + (self.current_byte()  as u64)).wrapping_add(displacement);

        format!("call {:#x}", pc)
    }

    fn parse_jcc_rel32(&mut self, opcode: u8) -> String {
        let op = opcode - 0x80;
        let displacement = self.get_imm(32) as u64;
        let pc = self.base_address + (self.current_byte() as u64) + displacement;
        let op_str = JCC_MAP.get(&op).unwrap();

        format!("{} {:#x}", op_str, pc)
    }

    fn parse_jcc_rel8(&mut self, opcode: u8) -> String {
        let op = opcode - 0x70;
        let displacement = self.get_imm(8) as u64;
        let pc = self.base_address + (self.current_byte() as u64) + displacement;
        let op_str = JCC_MAP.get(&op).unwrap();

        format!("{} {:#x}", op_str, pc)
    }

    fn parse_push_reg(&mut self, opcode: u8, prefix: &Prefix) -> String {
        // Only 64-bit, we don't have push r32 (e.g. push eax)
        let operand_size;
        if prefix.has_0x66 {
            operand_size = OperandSize::Size16Bit;
        } else {
            operand_size = OperandSize::Size64Bit;
        }

        let mut reg = opcode - 0x50;
        if let Some(ref rex) = prefix.rex {
            reg = (rex.b << 3) | reg;
        }

        let reg_str = get_reg(reg, operand_size, prefix.rex.is_some());
        format!("push {}", reg_str)
    }
}

static JCC_MAP: LazyLock<HashMap<u8, String>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(0x0, String::from("jo"));
    map.insert(0x1, String::from("jno"));
    map.insert(0x2, String::from("jb"));
    map.insert(0x3, String::from("jnb"));
    map.insert(0x4, String::from("je"));
    map.insert(0x5, String::from("jne"));
    map.insert(0x6, String::from("jbe"));
    map.insert(0x7, String::from("ja"));
    map.insert(0x8, String::from("js"));
    map.insert(0x9, String::from("jns"));
    map.insert(0xa, String::from("jp"));
    map.insert(0xb, String::from("jnp"));
    map.insert(0xc, String::from("jl"));
    map.insert(0xd, String::from("jge"));
    map.insert(0xe, String::from("jle"));
    map.insert(0xf, String::from("jg"));
    map
});

//  7                            0
// +---+---+---+---+---+---+---+---+
// |  mod  |    reg    |     rm    |
// +---+---+---+---+---+---+---+---+
const MODRM_MOD_POS: u8 = 6;
const MODRM_REG_POS: u8 = 3;

#[derive(Debug, Clone, Copy)]
struct ModRM {
    addressing_mode: AddressingMode,
    reg: u8,
    rm: u8,
}

impl ModRM {
    fn new(byte: u8) -> Self {
        let addressing_mode_raw = (byte & (0b11 << MODRM_MOD_POS)) >> MODRM_MOD_POS;
        let addressing_mode = AddressingMode::try_from(addressing_mode_raw).unwrap();

        let reg = (byte & (0b111 << MODRM_REG_POS)) >> MODRM_REG_POS;
        let rm = byte & 0b111;

        ModRM {
            addressing_mode: addressing_mode,
            reg: reg,
            rm: rm,
        }
    }
}

#[derive(TryFromPrimitive, PartialEq, Debug, Copy, Clone)]
#[repr(u8)]
enum AddressingMode {
    Memory,
    MemoryWith8BitDisp,
    MemoryWith32BitDisp,
    RegisterToRegister,
}

impl AddressingMode {
    fn displacement_bits(&self) -> u8 {
        match self {
            AddressingMode::MemoryWith8BitDisp => 8,
            AddressingMode::MemoryWith32BitDisp => 32,
            _ => 0,
        }
    }
}

#[derive(Debug)]
struct Prefix {
    group1_prefix: Option<u8>,
    segment_override: String,
    /// Override the default operand size
    has_0x66: bool,
    /// Override the default memory size
    has_0x67: bool,
    rex: Option<Rex>,
}

static SEGMENT_OVERRIDE: LazyLock<HashMap<u8, String>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(0x2e, String::from("cs:"));
    map.insert(0x36, String::from("ss:"));
    map.insert(0x3e, String::from("ds:"));
    map.insert(0x26, String::from("es:"));
    map.insert(0x64, String::from("fs:"));
    map.insert(0x65, String::from("gs:"));
    map
});

impl Prefix {
    fn new() -> Self {
        Self {
            group1_prefix: None,
            segment_override: String::from(""),
            has_0x66: false,
            has_0x67: false,
            rex: None,
        }
    }

    fn parse_segment_override(&mut self, byte: u8) {
        let segment_override = SEGMENT_OVERRIDE.get(&byte).unwrap();
        self.segment_override = segment_override.clone();
    }

    /// 16-bit, 32-bit and 64-bit usually use the same opcode, we
    /// differentiate between them by looking at the prefixes
    fn get_operand_size(&self) -> OperandSize {
        if self.has_0x66 {
            OperandSize::Size16Bit
        } else if let Some(ref rex) = self.rex && rex.w == 1 {
            OperandSize::Size64Bit
        } else {
            OperandSize::Size32Bit
        }
    }
}

//  7                            0
// +---+---+---+---+---+---+---+---+
// | 0   1   0   0 | W | R | X | B |
// +---+---+---+---+---+---+---+---+
const REX_W_POS: u8 = 3;
const REX_R_POS: u8 = 2;
const REX_X_POS: u8 = 1;

#[derive(Debug)]
struct Rex {
    /// 64-bit operand
    w: u8,
    /// an extension to modrm.reg
    r: u8,
    /// an extension to sib.index
    x: u8,
    /// an extension to modrm.rm or sib.base
    b: u8,
}

fn parse_rex(byte: u8) -> Rex {
    let w = (byte & (1 << REX_W_POS)) >> REX_W_POS;
    let r = (byte & (1 << REX_R_POS)) >> REX_R_POS;
    let x = (byte & (1 << REX_X_POS)) >> REX_X_POS;
    let b = byte & 1;

    Rex { w, r, x, b }
}

#[derive(Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Clone, Copy)]
#[repr(u8)]
enum OperandSize {
    Size64Bit,
    Size32Bit,
    Size16Bit,
    Size8Bit,
}

impl OperandSize {
    /// Sometimes, the immediate is only 32-bit even if
    /// the operand is 64-bit
    fn imm_bits(&self, can_64bit: bool) -> u8 {
        match self {
            OperandSize::Size8Bit => 8,
            OperandSize::Size16Bit => 16,
            OperandSize::Size32Bit => 32,
            OperandSize::Size64Bit => {
                if can_64bit {
                    64
                } else {
                    32
                }
            },
        }
    }
}

#[derive(Debug, TryFromPrimitive, PartialEq, Clone, Copy)]
#[repr(u8)]
enum OperandEncoding {
    MR,
    RM,
}

const REGISTER_MAP: [[&str; 4]; 16] = [
    ["rax", "eax", "ax", "al"],
    ["rcx", "ecx", "cx", "cl"],
    ["rdx", "edx", "dx", "dl"],
    ["rbx", "ebx", "bx", "bl"],
    ["rsp", "esp", "sp", "ah"],
    ["rbp", "ebp", "bp", "ch"],
    ["rsi", "esi", "si", "dh"],
    ["rdi", "edi", "di", "bh"],
    ["r8", "r8d", "r8w", "r8b"],
    ["r9", "r9d", "r9w", "r9b"],
    ["r10", "r10d", "r10w", "r10b"],
    ["r11", "r11d", "r11w", "r11b"],
    ["r12", "r12d", "r12w", "r12b"],
    ["r13", "r13d", "r13w", "r13b"],
    ["r14", "r14d", "r14w", "r14b"],
    ["r15", "r15d", "r15w", "r15b"],
];

// With a REX prefix in 64-bit mode, attempts to access AH, BH, CH, or DH will
// instead access SPL, DIL, BPL, or SIL, respectively
fn fix_up_8_bit_reg(reg: &str) -> &str {
    match reg {
        "ah" => "spl",
        "bh" => "dil",
        "ch" => "bpl",
        "dh" => "sil",
        _ => reg,
    }
}

fn get_reg(reg_index: u8, operand_size: OperandSize, has_rex: bool) -> String {
    let size = operand_size as usize;
    let mut reg = REGISTER_MAP[reg_index as usize][size];

    if operand_size == OperandSize::Size8Bit && has_rex {
        reg = fix_up_8_bit_reg(reg);
    }

    String::from(reg)
}

//  7                            0
// +---+---+---+---+---+---+---+---+
// | scale |   index   |    base   |
// +---+---+---+---+---+---+---+---+
#[derive(Debug, Clone)]
struct Sib {
    scale: u8,
    index_reg: String,
    base_reg: String,
}

const SIB_SCALE_POS: u8 = 6;
const SIB_INDEX_POS: u8 = 3;
fn parse_sib(byte: u8, prefix: &Prefix, addressing_mode: AddressingMode, address_size: OperandSize) -> Sib {
    let scale = (byte & (0b11 << SIB_SCALE_POS)) >> SIB_SCALE_POS;
    let mut index = (byte & (0b111 << SIB_INDEX_POS)) >> SIB_INDEX_POS;
    let mut base = byte & 0b111;

    if let Some(ref rex) = prefix.rex {
        index = (rex.x << 3) | index;
        base = (rex.b << 3) | base;
    }

    let mut sib = Sib {
        scale: 2_u8.pow(scale as u32),
        index_reg: String::from(""),
        base_reg: String::from(""),
    };

    if index != 0b100 {
        // has_rex is not relevant here
        sib.index_reg = get_reg(index, address_size, true);
    }

    // If this is not [scaled index] + disp32 case
    // So if we don't want to have base register in SIB form,
    // we need to use AddressingMode::Memory with base = 0b101.
    if base != 0b101 || addressing_mode != AddressingMode::Memory {
        sib.base_reg = get_reg(base, address_size, true);
    }

    sib
}

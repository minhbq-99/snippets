use super::disassembler::Reader;

#[test]
fn test_mov() {
    let pairs = vec!(
        // MR encoding
        ("88FC", "mov ah, bh"),
        ("4088EC", "mov spl, bpl"),
        ("4588F0", "mov r8b, r14b"),
        ("4889F7", "mov rdi, rsi"),
        ("664189CA", "mov r10w, cx"),
        ("4189E9", "mov r9d, ebp"),

        // RM encoding
        ("418BE9", "mov ebp, r9d"),
        ("408AEC", "mov bpl, spl"),

        // Memory operand
        ("8B02", "mov eax, [rdx]"),
        ("67488B2E", "mov rbp, [esi]"),
        ("408A35CCEDFFFF", "mov sil, [rip-0x1234]"),
        ("880578563412", "mov [rip+0x12345678], al"),

        // Has SIB byte
        ("67498B0499", "mov rax, [r9d+ebx*4]"),
        // Interestingly, there is no away to encode 8-bit displacement with
        // no base register, so here we need to use 32-bit displacement for
        // 0.
        ("67488B049D00000000", "mov rax, [ebx*4]"),
        ("4C8B3C2500004000", "mov r15, [0x400000]"),
        ("67478B54E308", "mov r10d, [r11d+r12d*8+0x8]"),
        ("67478B94E378563412", "mov r10d, [r11d+r12d*8+0x12345678]"),
        ("6766478994E378563412", "mov [r11d+r12d*8+0x12345678], r10w"),
        ("67478894E378563412", "mov [r11d+r12d*8+0x12345678], r10b"),
        ("64488b042528000000", "mov rax, fs:[0x28]"),
        ("64488b442528", "mov rax, fs:[rbp+0x28]"),

        // mov reg, imm
        ("B012", "mov al, 0x12"),
        ("B712", "mov bh, 0x12"),
        ("6641B93412", "mov r9w, 0x1234"),
        ("41B978563412", "mov r9d, 0x12345678"),
        ("49B9F0DEBC9A78563412", "mov r9, 0x123456789abcdef0"),
    );

    for pair in &pairs {
        let mut reader = Reader::new(pair.0);
        assert_eq!(reader.disassemble().unwrap(), pair.1);
    }
}

#[test]
fn test_lea() {
    let input = "488d05590e0000";
    let output = "lea rax, [rip+0xe59]";

    let mut reader = Reader::new(input);
    assert_eq!(reader.disassemble().unwrap(), output);
}

#[test]
fn test_alu_imm8() {
    let pairs = vec!(
        ("4883EC20", "sub rsp, 0x20"),
        ("4983C2ff", "add r10, 0xff"),
    );

    for pair in &pairs {
        let mut reader = Reader::new(pair.0);
        assert_eq!(reader.disassemble().unwrap(), pair.1);
    }
}

#[test]
fn test_push_reg() {
    let pairs = vec!(
        ("4151", "push r9"),
        ("50", "push rax"),
        ("6651", "push cx"),
        ("664152", "push r10w"),
    );

    for pair in &pairs {
        let mut reader = Reader::new(pair.0);
        assert_eq!(reader.disassemble().unwrap(), pair.1);
    }
}

#[test]
fn test_hello_world() {
    // The test program
    //  #include <stdio.h>
    //  int main()
    //  {
    //      char name[20];
    //      printf("What's your name? ");
    //      scanf("%19s", name);
    //      printf("Hello %s\n", name);
    //      return 0;
    //  }

    let input = "f30f1efa554889e54883ec2064488b042528000000488945f831c0488d05590e00004889c7b800000000e8c8feffff488d45\
                    e04889c6488d05510e00004889c7b800000000e8bdfeffff488d45e04889c6488d053b0e00004889c7b800000000e892fe\
                    ffffb800000000488b55f864482b1425280000007405e869feffffc9c3";

    let output = "\
endbr64
push rbp
mov rbp, rsp
sub rsp, 0x20
mov rax, fs:[0x28]
mov [rbp-0x8], rax
xor eax, eax
lea rax, [rip+0xe59]
mov rdi, rax
mov eax, 0x0
call 0x1080
lea rax, [rbp-0x20]
mov rsi, rax
lea rax, [rip+0xe51]
mov rdi, rax
mov eax, 0x0
call 0x1090
lea rax, [rbp-0x20]
mov rsi, rax
lea rax, [rip+0xe3b]
mov rdi, rax
mov eax, 0x0
call 0x1080
mov eax, 0x0
mov rdx, [rbp-0x8]
sub rdx, fs:[0x28]
je 0x1207
call 0x1070
leave
ret";

    let mut reader = Reader::new_with_base(input, 0x1189);
    assert_eq!(reader.disassemble().unwrap(), output);
}

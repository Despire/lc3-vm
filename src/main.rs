use std::env;

use lc3_vm::cpu::CPU;

#[repr(i32)]
enum ErrCode {
    InvalidArgs = 0x1,
    MissingArgs = 0x2,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("lc3-vm <image-file1> ...");
        std::process::exit(ErrCode::MissingArgs as i32);
    }

    args.iter().skip(1).for_each(|arg| {
        if !read_image(arg) {
            println!("failed to load image: {}", arg);
            std::process::exit(ErrCode::InvalidArgs as i32);
        }
    });

    let mut cpu = CPU::new();

    loop {
        let instr = mem_read(cpu.pc);
        cpu.next_instruction();
        cpu.exec(instr);
    }
}

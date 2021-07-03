use std::env;

use lc3_vm::Cli;
use lc3_vm::ErrCode;
use lc3_vm::cpu::CPU;
use lc3_vm::memory::Memory;

fn main() {
    let cli = Cli::new(env::args().collect());
    let mut memory = Memory::new();

    cli.args().iter().skip(1).for_each(|arg| {
        if let Err(e) = lc3_vm::read_image(arg, &mut memory) {
            println!("failed to load image: {}", arg);
            println!("Error: {}", e);
            std::process::exit(ErrCode::InvalidArgs as i32);
        }
    });

    CPU::new(&mut memory).run();
}



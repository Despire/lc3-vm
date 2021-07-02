use std::env;
use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;

use lc3_vm::cpu::CPU;
use lc3_vm::memory::Memory;

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

    let mut memory = Memory::new();

    args.iter().skip(1).for_each(|arg| {
        if let Err(e) = read_image(arg, &mut memory) {
            println!("failed to load image: {}", arg);
            println!("Error: {}", e);
            std::process::exit(ErrCode::InvalidArgs as i32);
        }
    });

    CPU::new(&mut memory).run();
}

fn read_image(path: &String, m: &mut Memory) -> io::Result<()> {
    let path = Path::new(path);

    let mut raw_file_contents = Vec::new();
    fs::File::open(path)?.read_to_end(&mut raw_file_contents)?;

    let mut origin = u16::from_be_bytes([raw_file_contents[0], raw_file_contents[1]]).swap_bytes();

    for b in raw_file_contents.chunks(2).skip(1) {
        m.memory_write(origin, u16::from_be_bytes([b[0], b[1]]).swap_bytes());
        origin += 1;
    }

    Ok(())
}

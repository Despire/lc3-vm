use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;

use memory::Memory;

pub mod cpu;
pub mod memory;

#[repr(i32)]
pub enum ErrCode {
    InvalidArgs = 0x1,
    MissingArgs = 0x2,
    Halt = 0x3,
}

pub struct Cli {
    args: Vec<String>
}

impl Cli {
    pub fn new(args: Vec<String>) -> Self {
        if args.len() < 2 {
            println!("usage: lc3-vm <image-file1>");
            std::process::exit(ErrCode::MissingArgs as i32);
        }

        Cli { args }
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }
}

pub fn read_image(path: &String, m: &mut Memory) -> io::Result<()> {
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


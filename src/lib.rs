use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;

use cpu::CPU;
use memory::Memory;

use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

pub mod cpu;
pub mod memory;

#[repr(i32)]
pub enum ErrCode {
    InvalidArgs = 0x1,
    MissingArgs = 0x2,
    Halt = 0x3,
}

struct Console {
    base: Termios,
    modified: Termios,
    stdin: i32,
}

pub struct VM {
    args: Vec<String>,
    console: Console,
    memory: Memory,
    cpu: CPU,
}

impl VM {
    pub fn new(args: Vec<String>) -> Self {
        if args.len() < 2 {
            println!("usage: lc3-vm <image-file1>");
            std::process::exit(ErrCode::MissingArgs as i32);
        }

        VM {
            args,
            memory: Memory::new(),
            cpu: CPU::new(),
            console: {
                // unix specific to disable input buffering / echoing.
                let stdin = 0;
                let mut console = Console {
                    stdin,
                    base: Termios::from_fd(stdin).expect("failed attach termios to stdin"),
                    modified: Termios::from_fd(stdin)
                        .expect("failed to attach modified termios to stdin"),
                };

                console.modified.c_lflag &= !(ICANON | ECHO);
                tcsetattr(console.stdin, TCSANOW, &mut console.modified)
                    .expect("failed to modified termios");

                console
            },
        }
        .process_args()
    }

    pub fn run(&mut self) -> ! {
        self.cpu.run(&mut self.memory)
    }

    fn process_args(&mut self) -> Self {
        self.args.iter().skip(1).for_each(|arg| {
            if let Err(e) = read_image(arg, &mut self.memory) {
                println!("failed to load image: {}", arg);
                println!("Error: {}", e);
                std::process::exit(ErrCode::InvalidArgs as i32);
            }
        });

        *self
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        tcsetattr(self.console.stdin, TCSANOW, &self.console.base)
            .expect("failed to reload base console");
    }
}

pub fn read_image(path: &String, m: &mut Memory) -> io::Result<()> {
    let path = Path::new(path);

    let mut raw_file_contents = Vec::new();
    fs::File::open(path)?.read_to_end(&mut raw_file_contents)?;

    let mut origin = u16::from_be_bytes([raw_file_contents[0], raw_file_contents[1]]);

    for b in raw_file_contents.chunks(2).skip(1) {
        m.memory_write(origin, u16::from_be_bytes([b[0], b[1]]));
        origin += 1;
    }

    Ok(())
}

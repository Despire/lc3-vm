use std::io::{self, Read};

#[repr(u16)]
pub enum MemRegister {
    KBSR = 0xFE00, // keyboard status
    KBDR = 0xFE02, // keyboard data
}

pub struct Memory([u16; 1 << 16]);

impl Memory {
    pub fn new() -> Self {
        Memory([0 as u16; 1 << 16])
    }

    pub fn raw(&self) -> &[u16] {
        &self.0
    }

    pub fn raw_mut(&mut self) -> &mut [u16] {
        &mut self.0
    }

    pub fn memory_write(&mut self, addr: u16, val: u16) {
        self.0[addr as usize] = val;
    }

    pub fn memory_read(&mut self, addr: u16) -> u16 {
        if addr == MemRegister::KBSR as u16 {
            let mut buffer = [0 as u8; 1];
            io::stdin()
                .read_exact(&mut buffer)
                .expect("failed to read stdin");

            if buffer[0] != 0 {
                self.0[MemRegister::KBSR as usize] = 1 << 15;
                self.0[MemRegister::KBDR as usize] = buffer[0] as u16;
            } else {
                self.0[MemRegister::KBSR as usize] = 0;
            }
        }

        self.0[addr as usize]
    }
}

use super::memory::Memory;

use std::io::{self, Read};

#[repr(u16)]
pub enum Trap {
    GetC = 0x20,  // get character from keyboard
    Out = 0x21,   // output character
    PutS = 0x22,  // output word string
    In = 0x23,    // get characater from keyboard echoes onto the terminal
    PutSp = 0x24, // output a byte string
    Halt = 0x25,  // halt the program
}

impl Trap {
    pub fn from(v: u16) -> Trap {
        match v {
            0x20 => Trap::GetC,
            0x21 => Trap::Out,
            0x22 => Trap::PutS,
            0x23 => Trap::In,
            0x24 => Trap::PutSp,
            0x25 => Trap::Halt,
            _ => panic!("invalid trap: {}", v),
        }
    }
}

#[repr(u16)]
pub enum Condition {
    FlPos = 1 << 0, // pos
    FlZro = 1 << 1, // zero
    FlNeg = 1 << 2, // neg
}

impl Condition {
    pub fn from(v: u16) -> Condition {
        match v {
            1 => Condition::FlPos,
            2 => Condition::FlZro,
            4 => Condition::FlNeg,
            _ => panic!("invalid condition flag: {}", v),
        }
    }
}

#[repr(u8)]
pub enum Instruction {
    OpBr,   // branch
    OpAdd,  // add
    OpLd,   // load
    OpSt,   // store
    OpJsr,  // jump register
    OpAnd,  // bitwise and
    OpLdr,  // load register
    OpStr,  // store register
    OpRti,  // unused
    OpNot,  // bitwise not
    OpLdi,  // load indirect
    OpSti,  // store indirect
    OpJmp,  // jump
    OpRes,  // reserverd (unused)
    OpLea,  // load effectire address
    OpTrap, // execute trap
}

impl Instruction {
    pub fn from(v: u8) -> Instruction {
        match v {
            0x0 => Instruction::OpBr,
            0x1 => Instruction::OpAdd,
            0x2 => Instruction::OpLd,
            0x3 => Instruction::OpSt,
            0x4 => Instruction::OpJsr,
            0x5 => Instruction::OpAnd,
            0x6 => Instruction::OpLdr,
            0x7 => Instruction::OpStr,
            0x8 => Instruction::OpRti,
            0x9 => Instruction::OpNot,
            0xA => Instruction::OpLdi,
            0xB => Instruction::OpSti,
            0xC => Instruction::OpJmp,
            0xD => Instruction::OpRes,
            0xE => Instruction::OpLea,
            0xF => Instruction::OpTrap,
            _ => panic!("unsupported instruction 0x{:x}", v),
        }
    }

    pub fn value(&self) -> u8 {
        match self {
            Instruction::OpBr => 0x0,
            Instruction::OpAdd => 0x1,
            Instruction::OpLd => 0x2,
            Instruction::OpSt => 0x3,
            Instruction::OpJsr => 0x4,
            Instruction::OpAnd => 0x5,
            Instruction::OpLdr => 0x6,
            Instruction::OpStr => 0x7,
            Instruction::OpRti => 0x8,
            Instruction::OpNot => 0x9,
            Instruction::OpLdi => 0xA,
            Instruction::OpSti => 0xB,
            Instruction::OpJmp => 0xC,
            Instruction::OpRes => 0xD,
            Instruction::OpLea => 0xE,
            Instruction::OpTrap => 0xF,
        }
    }
}

pub struct CPU<'a> {
    // general purpose registers.
    pub r0: u16,
    pub r1: u16,
    pub r2: u16,
    pub r3: u16,
    pub r4: u16,
    pub r5: u16,
    pub r6: u16,
    pub r7: u16,

    // program counter register.
    pub pc: u16,

    // condition register.
    pub cond: u16,

    // memory reference.
    pub memory: &'a mut Memory,
}

impl<'a> CPU<'a> {
    pub fn new(memory: &'a mut Memory) -> Self {
        CPU {
            r0: 0,
            r1: 0,
            r2: 0,
            r3: 0,
            r4: 0,
            r5: 0,
            r6: 0,
            r7: 0,
            pc: 0x3000, // 0x3000 is the default position.
            cond: 0,
            memory,
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            let instr = self.memory.memory_read(self.pc);
            self.next_instruction();
            self.exec(instr);
        }
    }

    pub fn exec(&mut self, instruction: u16) {
        let op_code = instruction >> 12;

        match Instruction::from(op_code as u8) {
            Instruction::OpBr => self.branch(instruction),
            Instruction::OpAdd => self.add(instruction),
            Instruction::OpLd => self.load(instruction),
            Instruction::OpSt => self.store(instruction),
            Instruction::OpJsr => self.jump_register(instruction),
            Instruction::OpAnd => self.bitwise_and(instruction),
            Instruction::OpLdr => self.load_register(instruction),
            Instruction::OpStr => self.store_register(instruction),
            Instruction::OpRti => panic!("OP_RTI unsupported instruction"),
            Instruction::OpNot => self.bitwise_not(instruction),
            Instruction::OpLdi => self.load_indirect(instruction),
            Instruction::OpSti => self.store_indirect(instruction),
            Instruction::OpJmp => self.jump(instruction),
            Instruction::OpRes => panic!("OP_RES unsupported instruction"),
            Instruction::OpLea => self.load_effective_address(instruction),
            Instruction::OpTrap => match Trap::from(instruction & 0xFF) {
                Trap::GetC => self.get_c(),
                Trap::Out => self.out(),
                Trap::PutS => self.put_s(),
                Trap::In => self.read(),
                Trap::PutSp => self.put_sp(),
                Trap::Halt => self.halt(),
            },
        }
    }

    pub fn next_instruction(&mut self) {
        self.pc += 1
    }

    pub fn set_cond(&mut self, r: u16) {
        if r == 0 {
            self.cond = Condition::FlZro as u16;
        } else if r >> 15 & 0x1 == 1 {
            self.cond = Condition::FlNeg as u16;
        } else {
            self.cond = Condition::FlPos as u16;
        }
    }

    fn sign_extend(v: &mut u16, c: u32) {
        if (*v >> (c - 1)) & 0x1 == 1 {
            *v |= 0xFFFF << c;
        }
    }

    fn register_from_mut(&mut self, i: u8) -> &mut u16 {
        match i {
            0 => &mut self.r0,
            1 => &mut self.r1,
            2 => &mut self.r2,
            3 => &mut self.r3,
            4 => &mut self.r4,
            5 => &mut self.r5,
            6 => &mut self.r6,
            7 => &mut self.r7,
            8 => &mut self.pc,
            9 => &mut self.cond,
            _ => panic!("unknown register: {}", i),
        }
    }

    fn register_from(&self, i: u8) -> &u16 {
        match i {
            0 => &self.r0,
            1 => &self.r1,
            2 => &self.r2,
            3 => &self.r3,
            4 => &self.r4,
            5 => &self.r5,
            6 => &self.r6,
            7 => &self.r7,
            8 => &self.pc,
            9 => &self.cond,
            _ => panic!("unknown register: {}", i),
        }
    }
}

// cpu instructions.
impl<'a> CPU<'a> {
    pub fn add(&mut self, instr: u16) {
        // dst register
        let dst = ((instr >> 9) & 0x7) as u8;

        // left side.
        let left = *self.register_from(((instr >> 6) & 0x7) as u8);

        // is imm
        if (instr >> 5) & 0x1 == 1 {
            let mut imm_5 = instr & 0x1F;
            CPU::sign_extend(&mut imm_5, 5);
            *self.register_from_mut(dst) = left.wrapping_add(imm_5);
        } else {
            let right = *self.register_from((instr & 0x7) as u8);
            *self.register_from_mut(dst) = left.wrapping_add(right);
        }

        self.set_cond(*self.register_from(dst));
    }

    pub fn load_indirect(&mut self, instr: u16) {
        let dst = ((instr >> 9) & 0x7) as u8;

        let mut offset = instr & 0x1FF;
        CPU::sign_extend(&mut offset, 9);

        let loc = self.memory.memory_read(self.pc + offset);
        *self.register_from_mut(dst) = self.memory.memory_read(loc);
        self.set_cond(*self.register_from(dst));
    }

    pub fn bitwise_and(&mut self, instr: u16) {
        let dst = ((instr >> 9) & 0x7) as u8;

        let left = *self.register_from(((instr >> 6) & 0x7) as u8);

        if (instr >> 5) & 0x1 == 1 {
            let mut imm_5 = instr & 0x1F;
            CPU::sign_extend(&mut imm_5, 5);
            *self.register_from_mut(dst) = left & imm_5;
        } else {
            let right = *self.register_from((instr & 0x7) as u8);
            *self.register_from_mut(dst) = left & right;
        }

        self.set_cond(*self.register_from(dst));
    }

    pub fn bitwise_not(&mut self, instr: u16) {
        let dst = ((instr >> 9) & 0x7) as u8;
        let left = *self.register_from(((instr >> 6) & 0x7) as u8);

        *self.register_from_mut(dst) = !left;

        self.set_cond(*self.register_from(dst));
    }

    pub fn branch(&mut self, instr: u16) {
        let mut offset = instr & 0x1FF;
        CPU::sign_extend(&mut offset, 9);

        let cond = Condition::from(((instr >> 9) & 0x7) as u16);

        if (self.cond & cond as u16) > 0x0 {
            self.pc = self.pc + offset;
        }
    }

    pub fn jump(&mut self, instr: u16) {
        let reg = ((instr >> 6) & 0x7) as u8;
        self.pc = *self.register_from(reg);
    }

    pub fn jump_register(&mut self, instr: u16) {
        self.r7 = self.pc;

        if (instr >> 11) & 0x1 == 0x1 {
            let mut offset = instr & 0x7FF;
            CPU::sign_extend(&mut offset, 11);

            self.pc = self.pc + offset;
        } else {
            let base = ((instr >> 6) & 0x7) as u8;
            self.pc = *self.register_from(base);
        }
    }

    pub fn load(&mut self, instr: u16) {
        let dest = ((instr >> 9) & 0x7) as u8;
        let mut offset = instr & 0x1FF;
        CPU::sign_extend(&mut offset, 9);

        *self.register_from_mut(dest) = self.memory.memory_read(self.pc + offset);

        self.set_cond(*self.register_from(dest));
    }

    pub fn load_register(&mut self, instr: u16) {
        let dest = ((instr >> 9) & 0x7) as u8;
        let base_reg = ((instr >> 6) & 0x7) as u8;
        let mut offset = instr & 0x3F;
        CPU::sign_extend(&mut offset, 6);

        *self.register_from_mut(dest) = self
            .memory
            .memory_read(*self.register_from(base_reg) + offset);

        self.set_cond(*self.register_from(dest));
    }

    pub fn load_effective_address(&mut self, instr: u16) {
        let dest = ((instr >> 9) & 0x7) as u8;
        let mut offset = instr & 0x1FF;
        CPU::sign_extend(&mut offset, 9);

        *self.register_from_mut(dest) = self.pc + offset;
        self.set_cond(*self.register_from(dest))
    }

    pub fn store(&mut self, instr: u16) {
        let src = ((instr >> 9) & 0x7) as u8;
        let mut offset = instr & 0x1FF;
        CPU::sign_extend(&mut offset, 9);

        self.memory
            .memory_write(self.pc + offset, *self.register_from(src));
    }

    pub fn store_indirect(&mut self, instr: u16) {
        let src = ((instr >> 9) & 0x7) as u8;
        let mut offset = instr & 0x1FF;
        CPU::sign_extend(&mut offset, 9);

        let loc = self.memory.memory_read(self.pc + offset);
        self.memory.memory_write(loc, *self.register_from(src));
    }

    pub fn store_register(&mut self, instr: u16) {
        let src = ((instr >> 9) & 0x7) as u8;
        let base_reg = ((instr >> 6) & 0x7) as u8;
        let mut offset = instr & 0x3F;
        CPU::sign_extend(&mut offset, 6);

        self.memory.memory_write(
            *self.register_from(base_reg) + offset,
            *self.register_from(src),
        );
    }
}

// trap instructions
impl<'a> CPU<'a> {
    pub fn put_s(&mut self) {
        let mut it = self.r0;
        let mut c = self.memory.memory_read(it);

        while c != 0x0 {
            print!("{}", c as u8 as char);
            it += 1;
            c = self.memory.memory_read(it);
        }
    }

    pub fn get_c(&mut self) {
        let mut buff = [0 as u8; 1];
        io::stdin()
            .read_exact(&mut buff)
            .expect("failed to read char from stdin");
        self.r0 = buff[0] as u16;
    }

    pub fn out(&self) {
        print!("{}", self.r0 as u8 as char);
    }

    pub fn read(&mut self) {
        print!("Enter a character: ");

        let mut buff = [0 as u8; 1];
        io::stdin()
            .read_exact(&mut buff)
            .expect("failed to read char from stdin");

        print!("{}", buff[0] as char);

        self.r0 = buff[0] as u16;
    }

    pub fn put_sp(&mut self) {
        let mut it = self.r0;
        let mut c = self.memory.memory_read(it);

        while c != 0x0 {
            let c1 = c & 0xFF;

            print!("{}", c1 as u8 as char);

            let c2 = c >> 8;
            if c2 != 0x0 {
                print!("{}", c2 as u8 as char);
            }

            it += 1;
            c = self.memory.memory_read(it);
        }
    }

    pub fn halt(&self) {
        print!("HALT");
        std::process::abort();
    }
}

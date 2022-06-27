#![allow(dead_code)]
use std::{self, fmt};

#[cfg(test)]
mod tests;

const MEMORY_SIZE: usize = 0x4000;



pub struct Em8080 {

    // Registers
    a: u8,

    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    sp: u16,
    pc: u16,

    memory: [u8; MEMORY_SIZE],
}

impl std::default::Default for Em8080 {
    fn default() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,

            memory: [0; MEMORY_SIZE],
        }
    }
}

impl fmt::Debug for Em8080 {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "{:>4} {:>4} {:>4} {:>4} {:>4} {:>4} {:>4}",
                 "a", "bc", "de", "hl", "pc", "sp", "flags")?;

        write!(f,
               "{:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {}",
               0, //self.a,
               self.get_bc(),
               self.get_de(),
               0, //self.hl(),
               self.pc,
               self.sp,
               0, //self.flags,
        )
    }
}

impl Em8080 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn emulate(&mut self /* , io_state: &mut IOState */) -> u64 {
        let op_code = self.read_byte(self.pc);

        //if cfg!(feature="logging") && self.pc != 0xada && self.pc != 0xadd && self.pc != 0xade {
        //    println!("{}", self);
        //}

        println!("Running op_code: {}", op_code);

        let (op_length, cycles) = match op_code {
            // NOP
            0x00 | 0x20 => (1, 4),

            // LXI
            0x01 => { // LXI B
                self.set_bc(self.read_next_word());
                (3, 10)
            },
            0x11 => { // LXI D
                self.set_de(self.read_next_word());
                (3, 10)
            },
            0x21 => { // LXI H
                self.set_hl(self.read_next_word());
                (3, 10)
            },
            0x31 => { // LXI SP
                self.sp = self.read_next_word();
                (3, 10)
            },

            // MVI
            0x3E => { // MVI A, d8
                self.a = self.read_next_byte();
                (2, 7)
            },
            
            0x06 => { // MVI B, d8
                self.b = self.read_next_byte();
                (2, 7)
            },

            0x0E => { // MVI C, d8
                self.c = self.read_next_byte();
                (2, 7)
            },

            0x16 => { // MVI D, d8
                self.d = self.read_next_byte();
                (2, 7)
            },

            0x1E => { // MVI E, d8
                self.e = self.read_next_byte();
                (2, 7)
            },

            0x26 => { // MVI H, d8
                self.h = self.read_next_byte();
                (2, 7)
            },

            0x2E => { // MVI L, d8
                self.l = self.read_next_byte();
                (2, 7)
            },

            0x36 => { // MVI M, d8
                self.write_byte(self.get_hl(), self.read_next_byte());
                (2, 7)
            },
            
            // MOV
            0x40 => { // MOV B, B
                self.b = self.b;
                (1, 5)
            },

            0x41 => { // MOV B, C
                self.b = self.c;
                (1, 5)
            },

            0x42 => { // MOV B, D
                self.b = self.d;
                (1, 5)
            },

            0x43 => { // MOV B, E
                self.b = self.e;
                (1, 5)
            },

            0x44 => { // MOV B, H
                self.b = self.h;
                (1, 5)
            },

            0x45 => { // MOV B, L
                self.b = self.l;
                (1, 5)
            },

            0x46 => { // MOV B, M
                self.b = self.get_m();
                (1, 7)
            },

            0x47 => { // MOV B, A
                self.b = self.a;
                (1, 5)
            },

            0x48 => { // MOV C, B
                self.c = self.b;
                (1, 5)
            },

            0x49 => { // MOV C, C
                self.c = self.c;
                (1, 5)
            },

            0x4A => { // MOV C, D
                self.c = self.d;
                (1, 5)
            },

            0x4B => { // MOV C, E
                self.c = self.e;
                (1, 5)
            },

            0x4C => { // MOV C, H
                self.c = self.h;
                (1, 5)
            },

            0x4D => { // MOV C, L
                self.c = self.l;
                (1, 5)
            },

            0x4E => { // MOV C, M
                self.c = self.get_m();
                (1, 7)
            },

            0x4F => { // MOV C, A
                self.c = self.a;
                (1, 5)
            },

            // MOV, ROW 2

            0x50 => { // MOV D, B
                self.d = self.b;
                (1, 5)
            },

            0x51 => { // MOV D, C
                self.d = self.c;
                (1, 5)
            },

            0x52 => { // MOV D, D
                self.d = self.d;
                (1, 5)
            },

            0x53 => { // MOV D, E
                self.d = self.e;
                (1, 5)
            },

            0x54 => { // MOV D, H
                self.d = self.h;
                (1, 5)
            },

            0x55 => { // MOV D, L
                self.d = self.l;
                (1, 5)
            },

            0x56 => { // MOV D, M
                self.d = self.get_m();
                (1, 7)
            },

            0x57 => { // MOV D, A
                self.d = self.a;
                (1, 5)
            },

            0x58 => { // MOV E, B
                self.e = self.b;
                (1, 5)
            },

            0x59 => { // MOV E, C
                self.e = self.c;
                (1, 5)
            },

            0x5A => { // MOV E, D
                self.e = self.d;
                (1, 5)
            },

            0x5B => { // MOV E, E
                self.e = self.e;
                (1, 5)
            },

            0x5C => { // MOV E, H
                self.e = self.h;
                (1, 5)
            },

            0x5D => { // MOV E, L
                self.e = self.l;
                (1, 5)
            },

            0x5E => { // MOV E, M
                self.e = self.get_m();
                (1, 7)
            },

            0x5F => { // MOV E, A
                self.e = self.a;
                (1, 5)
            },

            // MOV, Row 3

            0x60 => { // MOV H, B
                self.h = self.b;
                (1, 5)
            },

            0x61 => { // MOV H, C
                self.h = self.c;
                (1, 5)
            },

            0x62 => { // MOV H, D
                self.h = self.d;
                (1, 5)
            },

            0x63 => { // MOV H, E
                self.h = self.e;
                (1, 5)
            },

            0x64 => { // MOV H, H
                self.h = self.h;
                (1, 5)
            },

            0x65 => { // MOV H, L
                self.h = self.l;
                (1, 5)
            },

            0x66 => { // MOV H, M
                self.h = self.get_m();
                (1, 7)
            },

            0x67 => { // MOV H, A
                self.h = self.a;
                (1, 5)
            },

            0x68 => { // MOV L, B
                self.l = self.b;
                (1, 5)
            },

            0x69 => { // MOV L, C
                self.l = self.c;
                (1, 5)
            },

            0x6A => { // MOV L, D
                self.l = self.d;
                (1, 5)
            },

            0x6B => { // MOV L, E
                self.l = self.e;
                (1, 5)
            },

            0x6C => { // MOV L, H
                self.l = self.h;
                (1, 5)
            },

            0x6D => { // MOV L, L
                self.l = self.l;
                (1, 5)
            },

            0x6E => { // MOV L, M
                self.l = self.get_m();
                (1, 7)
            },

            0x6F => { // MOV L, A
                self.l = self.a;
                (1, 5)
            },

            // MOV Row 4

            0x70 => { // MOV M, B
                self.set_m(self.b);
                (1, 7)
            },

            0x71 => { // MOV M, C
                self.set_m(self.c);
                (1, 7)
            },

            0x72 => { // MOV M, D
                self.set_m(self.d);
                (1, 7)
            },

            0x73 => { // MOV M, E
                self.set_m(self.e);
                (1, 7)
            },

            0x74 => { // MOV M, H
                self.set_m(self.h);
                (1, 7)
            },

            0x75 => { // MOV M, L
                self.set_m(self.l);
                (1, 7)
            },

            0x77 => { // MOV M, A
                self.set_m(self.a);
                (1, 7)
            },

            0x78 => { // MOV A, B
                self.a = self.b;
                (1, 5)
            },

            0x79 => { // MOV A, C
                self.a = self.c;
                (1, 5)
            },

            0x7A => { // MOV A, D
                self.a = self.d;
                (1, 5)
            },

            0x7B => { // MOV A, E
                self.a = self.e;
                (1, 5)
            },

            0x7C => { // MOV A, H
                self.a = self.h;
                (1, 5)
            },

            0x7D => { // MOV A, L
                self.a = self.l;
                (1, 5)
            },

            0x7E => { // MOV A, M
                self.a = self.get_m();
                (1, 7)
            },

            0x7F => { // MOV A, A
                self.a = self.a;
                (1, 5)
            },            

            // Unimplemented
            _ => {
                println!(
                    "Unimplemented instruction: {:04x} {:02x} {}",
                    self.pc,
                    op_code,
                    self.next_opcode()
                );
                panic!("Unimplemented instruction: {:04x} {:02x} {}",
                self.pc,
                op_code,
                self.next_opcode())
            }            
        };

        self.pc += op_length;
        cycles        
    }

    fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }
    
    fn read_word(&self, address: u16) -> u16 {
        (self.read_byte(address + 1) as u16) << 8 | (self.read_byte(address) as u16)
    }

    // Reads next word from memory
    fn read_next_word(&self) -> u16 {
        self.read_word(self.pc + 1)
    }

    // Reads next word from memory
    fn read_next_byte(&self) -> u8 {
        self.read_byte(self.pc + 1)
    }

    fn write_byte(&mut self, address: u16, val: u8) {
        self.memory[address as usize] = val;
    }

    fn write_word(&mut self, address: u16, word: u16) {
        self.write_byte(address, (word & 0xFF) as u8);
        self.write_byte(address + 1, (word >> 8) as u8);
    }    

    fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }

    fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | (self.c as u16)
    }

    fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | (self.e as u16)
    }

    fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | (self.l as u16)
    }

    fn get_m(&self) -> u8 {
        self.read_byte(self.get_hl())
    }

    fn set_m(&mut self, value: u8) {
        self.write_byte(self.get_hl(), value)
    }

    fn next_opcode(&self) -> String {
        self.op_name(self.pc)
    }
    
    fn op_name(&self, address: u16) -> String {
        return match self.read_byte(address) {
            0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 => "NOP".into(),

            _ => "ERR".into(),
        };
    }
}
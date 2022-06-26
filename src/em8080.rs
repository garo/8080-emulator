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
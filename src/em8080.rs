#![allow(dead_code)]
use std::{self, fmt};

#[cfg(test)]
mod tests;

mod flags;
use flags::Flags;

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

    // Flags
    flags: Flags,
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

            flags: Flags {
                zero: false,
                sign: false,
                parity: false,
                carry: false,
                aux_carry: false,
            },            
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
               self.flags,
        )
    }
}

impl Em8080 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn flags(&self) -> &Flags {
        &self.flags
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
            0x40 => { self.b = self.b; (1, 5) },
            0x41 => { self.b = self.c; (1, 5) },
            0x42 => { self.b = self.d; (1, 5) },
            0x43 => { self.b = self.e; (1, 5) },
            0x44 => { self.b = self.h; (1, 5) },
            0x45 => { self.b = self.l; (1, 5) },
            0x46 => { self.b = self.get_m(); (1, 7) },
            0x47 => { self.b = self.a; (1, 5) },

            0x48 => { self.c = self.b; (1, 5) },
            0x49 => { self.c = self.c; (1, 5) },
            0x4A => { self.c = self.d; (1, 5) },
            0x4B => { self.c = self.e; (1, 5) },
            0x4C => { self.c = self.h; (1, 5) },
            0x4D => { self.c = self.l; (1, 5) },
            0x4E => { self.c = self.get_m(); (1, 7) },
            0x4F => { self.c = self.a; (1, 5) },

            // MOV, ROW 2
            0x50 => { self.d = self.b; (1, 5) },
            0x51 => { self.d = self.c; (1, 5) },
            0x52 => { self.d = self.d; (1, 5) },
            0x53 => { self.d = self.e; (1, 5) },
            0x54 => { self.d = self.h; (1, 5) },
            0x55 => { self.d = self.l; (1, 5) },
            0x56 => { self.d = self.get_m(); (1, 7) },
            0x57 => { self.d = self.a; (1, 5) },
            0x58 => { self.e = self.b; (1, 5) },
            0x59 => { self.e = self.c; (1, 5) },
            0x5A => { self.e = self.d; (1, 5) },
            0x5B => { self.e = self.e; (1, 5) },
            0x5C => { self.e = self.h; (1, 5) },
            0x5D => { self.e = self.l; (1, 5) },
            0x5E => { self.e = self.get_m(); (1, 7) },
            0x5F => { self.e = self.a; (1, 5) },

            // MOV, Row 3
            0x60 => { self.h = self.b; (1, 5) },
            0x61 => { self.h = self.c; (1, 5) },
            0x62 => { self.h = self.d; (1, 5) },
            0x63 => { self.h = self.e; (1, 5) },
            0x64 => { self.h = self.h; (1, 5) },
            0x65 => { self.h = self.l; (1, 5) },
            0x66 => { self.h = self.get_m(); (1, 7) },
            0x67 => { self.h = self.a; (1, 5) },

            0x68 => { self.l = self.b; (1, 5) },
            0x69 => { self.l = self.c; (1, 5) },
            0x6A => { self.l = self.d; (1, 5) },
            0x6B => { self.l = self.e; (1, 5) },
            0x6C => { self.l = self.h; (1, 5) },
            0x6D => { self.l = self.l; (1, 5) },
            0x6E => { self.l = self.get_m(); (1, 7) },
            0x6F => { self.l = self.a; (1, 5) },

            // MOV Row 4

            0x70 => { self.set_m(self.b); (1, 7) },
            0x71 => { self.set_m(self.c); (1, 7) },
            0x72 => { self.set_m(self.d); (1, 7) },
            0x73 => { self.set_m(self.e); (1, 7) },
            0x74 => { self.set_m(self.h); (1, 7) },
            0x75 => { self.set_m(self.l); (1, 7) },
            0x77 => { self.set_m(self.a); (1, 7) },

            0x78 => { self.a = self.b; (1, 5) },
            0x79 => { self.a = self.c; (1, 5) },
            0x7A => { self.a = self.d; (1, 5) },
            0x7B => { self.a = self.e; (1, 5) },
            0x7C => { self.a = self.h; (1, 5) },
            0x7D => { self.a = self.l; (1, 5) },
            0x7E => { self.a = self.get_m(); (1, 7) },
            0x7F => { self.a = self.a; (1, 5) },

            // INR

            0x04 => { self.b = self.inr(self.b); (1, 5) },
            0x0C => { self.c = self.inr(self.c); (1, 5) },
            0x14 => { self.d = self.inr(self.d); (1, 5) },
            0x1C => { self.e = self.inr(self.e); (1, 5) },
            0x24 => { self.h = self.inr(self.h); (1, 5) },
            0x2C => { self.l = self.inr(self.l); (1, 5) },
            0x34 => {
                let value = self.inr(self.get_m());
                self.set_m(value);
                (1, 10)
            },
            0x3C => { self.a = self.inr(self.a); (1, 5) },

            // DCR

            0x05 => { self.b = self.dcr(self.b); (1, 5) },
            0x0D => { self.c = self.dcr(self.c); (1, 5) },
            0x15 => { self.d = self.dcr(self.d); (1, 5) },
            0x1D => { self.e = self.dcr(self.e); (1, 5) },
            0x25 => { self.h = self.dcr(self.h); (1, 5) },
            0x2D => { self.l = self.dcr(self.l); (1, 5) },
            0x35 => {
                let value = self.dcr(self.get_m()); 
                self.set_m(value);
                (1, 10)
            },
            0x3D => { self.a = self.dcr(self.a); (1, 5) },

            // INX
            0x03 => { self.set_bc(self.get_bc() + 1); (1, 5) },
            0x13 => { self.set_de(self.get_de() + 1); (1, 5) },
            0x23 => { self.set_hl(self.get_hl() + 1); (1, 5) },
            0x33 => { self.sp += 1; (1, 5) },

            // DCX
            0x0B => { self.set_bc(self.get_bc() - 1); (1, 5) },
            0x1B => { self.set_de(self.get_de() - 1); (1, 5) },
            0x2B => { self.set_hl(self.get_hl() - 1); (1, 5) },
            0x3B => { self.sp -= 1; (1, 5) },

            // ADD
            0x80 => { self.add(self.b); (1, 4) },
            0x81 => { self.add(self.c); (1, 4) },
            0x82 => { self.add(self.d); (1, 4) },
            0x83 => { self.add(self.e); (1, 4) },
            0x84 => { self.add(self.h); (1, 4) },
            0x85 => { self.add(self.l); (1, 4) },
            0x86 => { self.add(self.get_m()); (1, 7) },
            0x87 => { self.add(self.a); (1, 4) },

            // ADC
            0x88 => { self.adc(self.b); (1, 4) },
            0x89 => { self.adc(self.c); (1, 4) },
            0x8A => { self.adc(self.d); (1, 4) },
            0x8B => { self.adc(self.e); (1, 4) },
            0x8C => { self.adc(self.h); (1, 4) },
            0x8D => { self.adc(self.l); (1, 4) },
            0x8E => { self.adc(self.get_m()); (1, 7) },
            0x8F => { self.adc(self.a); (1, 4) },

            // SUB
            0x90 => { self.sub(self.b); (1, 4) },
            0x91 => { self.sub(self.c); (1, 4) },
            0x92 => { self.sub(self.d); (1, 4) },
            0x93 => { self.sub(self.e); (1, 4) },
            0x94 => { self.sub(self.h); (1, 4) },
            0x95 => { self.sub(self.l); (1, 4) },
            0x96 => { self.sub(self.get_m()); (1, 7) },
            0x97 => { self.sub(self.a); (1, 4) },

            // SBB
            0x98 => { self.sbb(self.b); (1, 4) },
            0x99 => { self.sbb(self.c); (1, 4) },
            0x9A => { self.sbb(self.d); (1, 4) },
            0x9B => { self.sbb(self.e); (1, 4) },
            0x9C => { self.sbb(self.h); (1, 4) },
            0x9D => { self.sbb(self.l); (1, 4) },
            0x9E => { self.sbb(self.get_m()); (1, 7) },
            0x9F => { self.sbb(self.a); (1, 4) },

            // ANA (bitwise and)
            0xA0 => { self.and(self.b); (1, 4) },
            0xA1 => { self.and(self.c); (1, 4) },
            0xA2 => { self.and(self.d); (1, 4) },
            0xA3 => { self.and(self.e); (1, 4) },
            0xA4 => { self.and(self.h); (1, 4) },
            0xA5 => { self.and(self.l); (1, 4) },
            0xA6 => { self.and(self.get_m()); (1, 7) },
            0xA7 => { self.and(self.a); (1, 4) },
            
            // XRA (bitwise xor)
            0xA8 => { self.xor(self.b); (1, 4) },
            0xA9 => { self.xor(self.c); (1, 4) },
            0xAA => { self.xor(self.d); (1, 4) },
            0xAB => { self.xor(self.e); (1, 4) },
            0xAC => { self.xor(self.h); (1, 4) },
            0xAD => { self.xor(self.l); (1, 4) },
            0xAE => { self.xor(self.get_m()); (1, 7) },
            0xAF => { self.xor(self.a); (1, 4) },
            
            // ORA (bitwise xor)
            0xB0 => { self.or(self.b); (1, 4) },
            0xB1 => { self.or(self.c); (1, 4) },
            0xB2 => { self.or(self.d); (1, 4) },
            0xB3 => { self.or(self.e); (1, 4) },
            0xB4 => { self.or(self.h); (1, 4) },
            0xB5 => { self.or(self.l); (1, 4) },
            0xB6 => { self.or(self.get_m()); (1, 7) },
            0xB7 => { self.or(self.a); (1, 4) },

            // CMP
            0xB8 => { self.cmp(self.b); (1, 4) },
            0xB9 => { self.cmp(self.c); (1, 4) },
            0xBA => { self.cmp(self.d); (1, 4) },
            0xBB => { self.cmp(self.e); (1, 4) },
            0xBC => { self.cmp(self.h); (1, 4) },
            0xBD => { self.cmp(self.l); (1, 4) },
            0xBE => { self.cmp(self.get_m()); (1, 7) },
            0xBF => { self.cmp(self.a); (1, 4) },

            // JNZ
            0xC2 => {
                if self.flags.zero {
                    (3, 10)
                } else {
                    self.jmp(self.read_next_word());
                    (0, 10)
                }
            }

            // JNC
            0xD2 => {
                if self.flags.carry {
                    (3, 10)
                } else {
                    self.jmp(self.read_next_word());
                    (0, 10)
                }
            }

            // JPO
            0xE2 => {
                if self.flags.parity {
                    (3, 10)
                } else {
                    self.jmp(self.read_next_word());
                    (0, 10)
                }
            }

            // JP
            0xF2 => {
                if self.flags.sign {
                    (3, 10)
                } else {
                    self.jmp(self.read_next_word());
                    (0, 10)
                }
            }

            // JMP
            0xC3 => {
                self.jmp(self.read_next_word());
                (0, 10)
            }

            // CNZ adr
            0xC4 => {
                if self.flags.zero {
                    (3, 11)
                } else {
                    self.call(self.read_next_word());
                    (0, 17)
                }
            }

            // CNC adr
            0xD4 => {
                if self.flags.carry {
                    (3, 11)
                } else {
                    self.call(self.read_next_word());
                    (0, 17)
                }
            }

            // CPO adr
            0xE4 => {
                if self.flags.parity {
                    (3, 11)
                } else {
                    self.call(self.read_next_word());
                    (0, 17)
                }
            }            

            // CP adr
            0xF4 => {
                if self.flags.sign {
                    (3, 11)
                } else {
                    self.call(self.read_next_word());
                    (0, 17)
                }
            }

            // PUSH 
            0xC5 => { self.push(self.get_bc()); (1, 11) }
            0xD5 => { self.push(self.get_de()); (1, 11) }
            0xE5 => { self.push(self.get_hl()); (1, 11) }
            0xF5 => { self.push(self.get_af()); (1, 11) }

            // POP
            0xC1 => { let v = self.pop(); self.set_bc(v); (1, 10) }
            0xD1 => { let v = self.pop(); self.set_de(v); (1, 10) }
            0xE1 => { let v = self.pop(); self.set_hl(v); (1, 10) }
            0xF1 => { let v = self.pop(); self.set_af(v); (1, 10) }

            // XTHL
            0xE3 => {
                let tmp = self.get_hl();
                let from_stack = self.pop();
                self.set_hl(from_stack);
                self.push(tmp);
                (1, 18)
            }

            // ADI D8
            0xC6 => {
                self.add(self.read_next_byte());
                (2, 7)
            }

            // SUI D8
            0xD6 => {
                self.sub(self.read_next_byte());
                (2, 7)
            }

            // ANI D8
            0xE6 => {
                self.and(self.read_next_byte());
                (2, 7)
            }

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

    /// Increments `operand`
    fn inr(&mut self, operand: u8) -> u8 {
        let result = operand.wrapping_add(1);
        self.flags.set_all_but_carry(result);
        result
    }

    /// Decrements `operand`
    fn dcr(&mut self, operand: u8) -> u8 {
        let result = operand.wrapping_sub(1);
        self.flags.set_all_but_carry(result);
        result
    }

    /// Add `operand` to A
    fn add(&mut self, operand: u8) {
        let result = (self.a as u16).wrapping_add(operand as u16);
        self.flags.set_all(result, (self.a & 0xf).wrapping_add(operand & 0xf));
        self.a = result as u8;
    }

    /// Add `operand` + carry to A
    fn adc(&mut self, operand: u8) {
        let result = (self.a as u16).wrapping_add(operand as u16).wrapping_add(self.flags.carry as u16);
        self.flags.set_all(result, (self.a & 0xf).wrapping_add(operand.wrapping_add(self.flags.carry as u8) & 0xf));
        self.a = result as u8;
    }

    /// Subtract `operand` from A
    fn sub(&mut self, operand: u8) {
        let result = (self.a as u16).wrapping_sub(operand as u16);
        self.flags.set_all(result, (self.a & 0xf).wrapping_sub(operand & 0xf));
        self.a = result as u8;
    }

    /// Subtract `operand` from A with borrow
    fn sbb(&mut self, operand: u8) {
        let result = (self.a as u16).wrapping_sub(operand as u16).wrapping_sub(self.flags.carry as u16);
        self.flags.set_all(result, (self.a & 0xf).wrapping_sub(operand.wrapping_sub(self.flags.carry as u8) & 0xf));
        self.a = result as u8;
    }
    
    /// Bitwise AND between A and `operand`
    fn and(&mut self, operand: u8) {
        self.a &= operand;
        self.flags.set_all(self.a as u16, self.a);
    }

    /// Bitwise OR between A and `operand`
    fn or(&mut self, operand: u8) {
        self.a |= operand;
        self.flags.set_all_but_aux_carry(self.a as u16);
    }

    /// Bitwise XOR between A and `operand`
    fn xor(&mut self, operand: u8) {
        self.a ^= operand;
        self.flags.set_all(self.a as u16, self.a);
        self.flags.carry = false;
    }

    /// Compare `operand` to A
    fn cmp(&mut self, operand: u8) {
        self.flags.set_all((self.a as u16).wrapping_sub(operand as u16), (self.a & 0xf).wrapping_sub(operand & 0xf));
    }

    fn jmp(&mut self, adr: u16) {
        self.pc = adr;
    }

    fn call(&mut self, adr: u16) {
        self.push(self.pc + 3);
        self.pc = adr;
    }

    fn ret(&mut self) {
        self.pc = self.pop();
    }    

    fn pop(&mut self) -> u16 {
        self.sp += 2;
        self.read_word(self.sp - 2)
    }

    fn push(&mut self, value: u16) {
        self.sp -= 2;
        self.write_word(self.sp, value);
    }    

    pub fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | self.flags.psw() as u16
    }

    fn set_af(&mut self, value: u16) {
        self.flags.set_psw(value as u8);
        self.a = (value >> 8) as u8;
    }

    fn op_name(&self, address: u16) -> String {
        return match self.read_byte(address) {
            0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 => "NOP".into(),

            _ => "ERR".into(),
        };
    }
}
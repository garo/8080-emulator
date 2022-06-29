#![allow(dead_code)]
use std::{self, fmt};

#[cfg(test)]
mod tests;
mod test_int;


mod flags;
use flags::Flags;

// This file borrows from https://github.com/alexandrejanin/rust-8080/tree/master/srcv

const MEMORY_SIZE: usize = 0x4000;

/// Interface between the emulator's IO functions and the machine state
pub trait IOState {
    fn input(&self, port: u8) -> u8;
    fn output(&mut self, port: u8, value: u8);
}

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

    halted : bool,
    interrupts_enabled : bool,
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
            
            halted : false,
            interrupts_enabled : true,
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

    pub fn interrupt(&mut self, interrupt_num: u16) {
        if self.interrupts_enabled {
            self.push(self.pc);
            self.pc = 8 * interrupt_num;
            self.interrupts_enabled = false;
        }
    }

    pub fn emulate(&mut self, io_state: &mut dyn IOState ) -> u64 {
        let op_code = self.read_byte(self.pc);

        //if cfg!(feature="logging") && self.pc != 0xada && self.pc != 0xadd && self.pc != 0xade {
        //    println!("{}", self);
        //}

        println!("PC:{:04X}, SP:{:04X}. op: {:2X} ({})", self.pc, self.sp, op_code, self.op_name(self.pc));

        let (op_length, cycles) = match op_code {
            // NOP
            0x00 | 0x10 | 0x20 | 0x30 | 0x08 | 0x18 | 0x28 | 0x38 => (1, 4),
            
            // LXI
            0x01 => { // LXI B
                self.set_bc(self.read_next_word());
                (3, 10)
            },

            // STAX B
            0x02 => {
                self.write_byte(self.get_bc(), self.a);
                (1, 7)
            }

            // STAX D
            0x12 => {
                self.write_byte(self.get_de(), self.a);
                (1, 7)
            }

            // LDAX B
            0x0A => {
                self.a = self.read_byte(self.get_bc());
                (1, 7)
            }

            // LDAX D
            0x1A => {
                self.a = self.read_byte(self.get_de());
                (1, 7)
            }

            // RRC
            0x0F => {
                let bit0: u8 = self.a & 1;
                self.a >>= 1;
                self.a |= bit0 << 7;
                self.flags.carry = bit0 != 0;
                (1, 4)
            }
            
            // RAR
            0x1f => {
                let bit0: u8 = self.a & 1;
                self.a >>= 1;
                if self.flags.carry { self.a |= 1 << 7; }
                self.flags.carry = bit0 != 0;
                (1, 4)
            }
            
            // CMA
            0x2F => {
                self.a = !self.a;
                (1, 4)
            }

            // CMC
            0x3F => {
                self.flags.carry = !self.flags.carry;
                (1, 4)
            }            

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

            // RLC
            0x07 => {
                let bit7: u8 = self.a & (1 << 7);
                self.a <<= 1;
                self.a |= bit7 >> 7;
                self.flags.carry = bit7 != 0;
                (1, 4)
            }

            // RAL
            0x17 => {
                let bit7: u8 = self.a & (1 << 7);
                self.a <<= 1;
                self.a |= self.flags.carry as u8;
                self.flags.carry = bit7 != 0;
                (1, 4)
            }
            
            

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

            // DAA
            0x27 => {
                self.daa();
                (1, 4)
            }            

            0x2E => { // MVI L, d8
                self.l = self.read_next_byte();
                (2, 7)
            },

            0x36 => { // MVI M, d8
                self.write_byte(self.get_hl(), self.read_next_byte());
                (2, 7)
            },

            // STC
            0x37 => { self.flags.carry = true; (1, 4) }
            
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

            // SHLD adr
            0x22 => {
                self.write_word(self.read_next_word(), self.get_hl());
                (3, 16)
            }
            // LHLD adr
            0x2A => {
                let v = self.read_word(self.read_next_word());
                self.set_hl(v);
                (3, 16)
            }            

            // STA adr
            0x32 => {
                self.write_byte(self.read_next_word(), self.a);
                (3, 13)
            }

            // LDA adr
            0x3A => {
                self.a = self.read_byte(self.read_next_word());
                (3, 13)
            }

            // HLT
            0x76 => {
                println!("HLT instruction received");
                self.halted = true;
                (1, 7)
            }

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
                    (3, 10)
                }
            }

            // JZ
            0xCA => {
                if self.flags.zero {
                    self.jmp(self.read_next_word());
                    (3, 10)
                } else {
                    (3, 10)
                }
            }

            // JC adr
            0xDA => {
                if self.flags.carry {
                    self.jmp(self.read_next_word());
                    (3, 10)
                } else {
                    (3, 10)
                }
            }

            // JPE adr
            0xEA => {
                if self.flags.parity {
                    self.jmp(self.read_next_word());
                    (3, 10)
                } else {
                    (3, 10)
                }
            }

            // JM adr
            0xFA => {
                if self.flags.sign {
                    self.jmp(self.read_next_word());
                    (3, 10)
                } else {
                    (3, 10)
                }
            }

            // SPHL
            0xF9 => {
                self.sp = self.get_hl();
                (1, 5)
            }

            // CZ adr
            0xCC => {
                if self.flags.zero {
                    self.call(self.read_next_word());
                    (3, 17)
                } else {
                    (3, 11)
                }
            }            

            // JNC
            0xD2 => {
                if self.flags.carry {
                    (3, 10)
                } else {
                    self.jmp(self.read_next_word());
                    (3, 10)
                }
            }

            // JPO
            0xE2 => {
                if self.flags.parity {
                    (3, 10)
                } else {
                    self.jmp(self.read_next_word());
                    (3, 10)
                }
            }

            // JP
            0xF2 => {
                if self.flags.sign {
                    (3, 10)
                } else {
                    self.jmp(self.read_next_word());
                    (3, 10)
                }
            }

            // JMP
            0xC3 | 0xCB  => {
                self.jmp(self.read_next_word());
                (3, 10)
            }

            // CC adr
            0xDC => {
                if self.flags.zero {
                    self.call(self.read_next_word());
                    (3, 17)
                } else {
                    (3, 11)
                }
            }

            // CNZ adr
            0xC4 => {
                if self.flags.zero {
                    (3, 11)
                } else {
                    self.call(self.read_next_word());
                    (3, 17)
                }
            }
            
            // CNC adr
            0xD4 => {
                if self.flags.carry {
                    (3, 11)
                } else {
                    self.call(self.read_next_word());
                    (3, 17)
                }
            }

            // CPE adr
            0xEC => {
                if self.flags.parity {
                    self.call(self.read_next_word());
                    (3, 17)
                } else {
                    (3, 11)
                }
            }

            // CPO adr
            0xE4 => {
                if self.flags.parity {
                    (3, 11)
                } else {
                    self.call(self.read_next_word());
                    (3, 17)
                }
            }

            // CM adr
            0xFC => {
                if self.flags.sign {
                    self.call(self.read_next_word());
                    (3, 17)
                } else {
                    (3, 11)
                }
            }

            // CALL adr
            0xCD | 0xDD | 0xED | 0xFD => {
                self.call(self.read_next_word());
                (3, 17)
            }            

            // CP adr
            0xF4 => {
                if self.flags.sign {
                    (3, 11)
                } else {
                    self.call(self.read_next_word());
                    (3, 17)
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
            0xC6 => { self.add(self.read_next_byte()); (2, 7) }

            // SUI D8
            0xD6 => { self.sub(self.read_next_byte()); (2, 7) }

            // ANI D8
            0xE6 => { self.and(self.read_next_byte()); (2, 7) }

            // ORI D8
            0xF6 => { self.or(self.read_next_byte()); (2, 7) }

            // ADI d8
            0xCE => { self.adc(self.read_next_byte()); (2, 7) },

            // SBI d8
            0xDE => { self.sbb(self.read_next_byte()); (2, 7) },

            // XRI d8
            0xEE => { self.xor(self.read_next_byte()); (2, 7) },

            // CPI d8
            0xFE => { self.cmp(self.read_next_byte()); (2, 7) },

            // RNZ
            0xC0 => {
                if self.flags.zero {
                    (1, 5)
                } else {
                    self.ret();
                    (0, 11)
                }
            }

            // RNC
            0xD0 => {
                if self.flags.carry {
                    (1, 5)
                } else {
                    self.ret();
                    (0, 11)
                }
            }

            // RPO
            0xE0 => {
                if self.flags.parity {
                    (1, 5)
                } else {
                    self.ret();
                    (1, 11)
                }
            }

            // RP
            0xF0 => {
                if self.flags.sign {
                    (1, 5)
                } else {
                    self.ret();
                    (1, 11)
                }
            }

            // RZ
            0xC8 => {
                if self.flags.zero {
                    self.ret();
                    (1, 11)
                } else {
                    (1, 5)
                }
            }

            // RC
            0xD8 => {
                if self.flags.carry {
                    self.ret();
                    (1, 11)
                } else {
                    (1, 5)
                }
            }

            // RPE
            0xE8 => {
                if self.flags.parity {
                    self.ret();
                    (1, 11)
                } else {
                    (1, 5)
                }
            }

            // RM
            0xF8 => {
                if self.flags.sign {
                    self.ret();
                    (1, 11)
                } else {
                    (1, 5)
                }
            }

            // RET
            0xC9 | 0xD9 => {
                self.ret();
                (1, 10)
            }

            0xE9 => {
                self.jmp(self.get_hl());
                (1, 5)
            }            

            // RST
            0xC7 => { self.call(0x00); (1, 11) }
            0xCF => { self.call(0x08); (1, 11) }
            0xD7 => { self.call(0x10); (1, 11) }
            0xDF => { self.call(0x18); (1, 11) }
            0xE7 => { self.call(0x20); (1, 11) }
            0xEF => { self.call(0x28); (1, 11) }
            0xF7 => { self.call(0x30); (1, 11) }
            0xFF => { self.call(0x38); (1, 11) }

            // DAD B
            0x09 => { self.dad(self.get_bc()); (1, 10) }
            0x19 => { self.dad(self.get_de()); (1, 10) }
            0x29 => { self.dad(self.get_hl()); (1, 10) }
            0x39 => { self.dad(self.sp); (1, 10) }

            // XCHG
            0xEB => {
                let de = self.get_de();
                let hl = self.get_hl();
                self.set_de(hl);
                self.set_hl(de);
                (1, 5)
            }

            // XCHG
            0xeb => {
                let de = self.get_de();
                let hl = self.get_hl();
                self.set_de(hl);
                self.set_hl(de);
                (1, 5)
            }            

            // OUT D8
            0xD3 => {
                io_state.output(self.read_next_byte(), self.a);
                (2, 10)
            }            

            // IN D8
            0xDB => {
                self.a = io_state.input(self.read_next_byte());
                (2, 10)
            }  
            
            // DI
            0xF3 => { self.interrupts_enabled = false; (1, 4) }

            // EN
            0xFB => { self.interrupts_enabled = true; (1, 4) }
/*
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
            } */
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

    /// Add `operand` to HL
    fn dad(&mut self, operand: u16) {
        let result = (self.get_hl() as u32).wrapping_add(operand as u32);
        self.flags.set_carry(result as u16);
        self.set_hl(result as u16);
    }

    fn daa(&mut self) {
        let mut result = self.a as u16;

        let lsb = result & 0xf;

        if self.flags.aux_carry || lsb > 9 {
            result += 6;

            if result & 0xf < lsb {
                self.flags.aux_carry = true;
            }
        }

        let lsb = result & 0xf;
        let mut msb = (result >> 4) & 0xf;

        if self.flags.carry || msb > 9 {
            msb += 6;
        }

        let result = (msb << 4) | lsb;
        self.flags.set_all_but_aux_carry(result);

        self.a = result as u8;
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

    pub fn from_rom(rom: &[u8], rom_start: usize, pc_start: u16) -> Self {
        let mut new = Self::new();
        new.load_rom(rom, rom_start);
        new.pc = pc_start;
        new
    }

    pub fn load_rom(&mut self, rom: &[u8], rom_start: usize) {
        self.memory[rom_start..rom_start + rom.len()].clone_from_slice(rom);
    }    

    /// Returns the name of the instruction at the specified address in memory
    fn op_name(&self, address: u16) -> String {
        match self.read_byte(address) {
            0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 => "NOP".into(),
            0x01 => format!("LXI B, ${:04x}", self.read_word(address + 1)),
            0x02 => "STAX B".into(),
            0x03 => "INX B".into(),
            0x04 => "INR B".into(),
            0x05 => "DCR B".into(),
            0x06 => format!("MVI B, ${:02x}", self.read_byte(address + 1)),
            0x07 => "RLC".into(),
            0x09 => "DAD B".into(),
            0x0a => "LDAX B".into(),
            0x0b => "DCX B".into(),
            0x0c => "INR C".into(),
            0x0d => "DCR C".into(),
            0x0e => format!("MVI C, ${:02x}", self.read_byte(address + 1)),
            0x0f => "RRC".into(),
            0x11 => format!("LXI D, ${:04x}", self.read_word(address + 1)),
            0x12 => "STAX D".into(),
            0x13 => "INX D".into(),
            0x14 => "INR D".into(),
            0x15 => "DCR D".into(),
            0x16 => format!("MVI D, ${:02x}", self.read_byte(address + 1)),
            0x17 => "RAL".into(),
            0x19 => "DAD D".into(),
            0x1a => "LDAX D".into(),
            0x1b => "DCX D".into(),
            0x1c => "INR E".into(),
            0x1d => "DCR E".into(),
            0x1e => format!("MVI E, ${:02x}", self.read_byte(address + 1)),
            0x1f => "RAR".into(),
            0x21 => format!("LXI H, ${:04x}", self.read_word(address + 1)),
            0x22 => format!("SHLD ${:04x}", self.read_word(address + 1)),
            0x23 => "INX H".into(),
            0x24 => "INR H".into(),
            0x25 => "DCR H".into(),
            0x26 => format!("MVI H, ${:02x}", self.read_byte(address + 1)),
            0x27 => "DAA".into(),
            0x29 => "DAD H".into(),
            0x2a => format!("LHLD ${:04x}", self.read_word(address + 1)),
            0x2b => "DCX H".into(),
            0x2c => "INR L".into(),
            0x2d => "DCR L".into(),
            0x2e => format!("MVI L, ${:02x}", self.read_byte(address + 1)),
            0x2f => "CMA".into(),
            0x31 => format!("LXI SP, ${:04x}", self.read_word(address + 1)),
            0x32 => format!("STA ${:04x}", self.read_word(address + 1)),
            0x33 => "INX SP".into(),
            0x34 => "INR M".into(),
            0x35 => "DCR M".into(),
            0x36 => format!("MVI M, ${:02x}", self.read_byte(address + 1)),
            0x37 => "STC".into(),
            0x39 => "DAD SP".into(),
            0x3a => format!("LDA ${:04x}", self.read_word(address + 1)),
            0x3b => "DCX SP".into(),
            0x3c => "INR A".into(),
            0x3d => "DCR A".into(),
            0x3e => format!("MVI A, ${:02x}", self.read_byte(address + 1)),
            0x3f => "CMC".into(),
            0x40 => "MOV B,B".into(),
            0x41 => "MOV B,C".into(),
            0x42 => "MOV B,D".into(),
            0x43 => "MOV B,E".into(),
            0x44 => "MOV B,H".into(),
            0x45 => "MOV B,L".into(),
            0x46 => "MOV B,M".into(),
            0x47 => "MOV B,A".into(),
            0x48 => "MOV C,B".into(),
            0x49 => "MOV C,C".into(),
            0x4a => "MOV C,D".into(),
            0x4b => "MOV C,E".into(),
            0x4c => "MOV C,H".into(),
            0x4d => "MOV C,L".into(),
            0x4e => "MOV C,M".into(),
            0x4f => "MOV C,A".into(),
            0x50 => "MOV D,B".into(),
            0x51 => "MOV D,C".into(),
            0x52 => "MOV D,D".into(),
            0x53 => "MOV D,E".into(),
            0x54 => "MOV D,H".into(),
            0x55 => "MOV D,L".into(),
            0x56 => "MOV D,M".into(),
            0x57 => "MOV D,A".into(),
            0x58 => "MOV E,B".into(),
            0x59 => "MOV E,C".into(),
            0x5a => "MOV E,D".into(),
            0x5b => "MOV E,E".into(),
            0x5c => "MOV E,H".into(),
            0x5d => "MOV E,L".into(),
            0x5e => "MOV E,M".into(),
            0x5f => "MOV E,A".into(),
            0x60 => "MOV H,B".into(),
            0x61 => "MOV H,C".into(),
            0x62 => "MOV H,D".into(),
            0x63 => "MOV H,E".into(),
            0x64 => "MOV H,H".into(),
            0x65 => "MOV H,L".into(),
            0x66 => "MOV H,M".into(),
            0x67 => "MOV H,A".into(),
            0x68 => "MOV L,B".into(),
            0x69 => "MOV L,C".into(),
            0x6a => "MOV L,D".into(),
            0x6b => "MOV L,E".into(),
            0x6c => "MOV L,H".into(),
            0x6d => "MOV L,L".into(),
            0x6e => "MOV L,M".into(),
            0x6f => "MOV L,A".into(),
            0x70 => "MOV M,B".into(),
            0x71 => "MOV M,C".into(),
            0x72 => "MOV M,D".into(),
            0x73 => "MOV M,E".into(),
            0x74 => "MOV M,H".into(),
            0x75 => "MOV M,L".into(),
            0x76 => "HLT".into(),
            0x77 => "MOV M,A".into(),
            0x78 => "MOV A,B".into(),
            0x79 => "MOV A,C".into(),
            0x7a => "MOV A,D".into(),
            0x7b => "MOV A,E".into(),
            0x7c => "MOV A,H".into(),
            0x7d => "MOV A,L".into(),
            0x7e => "MOV A,M".into(),
            0x7f => "MOV A,A".into(),
            0x80 => "ADD B".into(),
            0x81 => "ADD C".into(),
            0x82 => "ADD D".into(),
            0x83 => "ADD E".into(),
            0x84 => "ADD H".into(),
            0x85 => "ADD L".into(),
            0x86 => "ADD M".into(),
            0x87 => "ADD A".into(),
            0x88 => "ADC B".into(),
            0x89 => "ADC C".into(),
            0x8a => "ADC D".into(),
            0x8b => "ADC E".into(),
            0x8c => "ADC H".into(),
            0x8d => "ADC L".into(),
            0x8e => "ADC M".into(),
            0x8f => "ADC A".into(),
            0x90 => "SUB B".into(),
            0x91 => "SUB C".into(),
            0x92 => "SUB D".into(),
            0x93 => "SUB E".into(),
            0x94 => "SUB H".into(),
            0x95 => "SUB L".into(),
            0x96 => "SUB M".into(),
            0x97 => "SUB A".into(),
            0x98 => "SBB B".into(),
            0x99 => "SBB C".into(),
            0x9a => "SBB D".into(),
            0x9b => "SBB E".into(),
            0x9c => "SBB H".into(),
            0x9d => "SBB L".into(),
            0x9e => "SBB M".into(),
            0x9f => "SBB A".into(),
            0xa0 => "ANA B".into(),
            0xa1 => "ANA C".into(),
            0xa2 => "ANA D".into(),
            0xa3 => "ANA E".into(),
            0xa4 => "ANA H".into(),
            0xa5 => "ANA L".into(),
            0xa6 => "ANA M".into(),
            0xa7 => "ANA A".into(),
            0xa8 => "XRA B".into(),
            0xa9 => "XRA C".into(),
            0xaa => "XRA D".into(),
            0xab => "XRA E".into(),
            0xac => "XRA H".into(),
            0xad => "XRA L".into(),
            0xae => "XRA M".into(),
            0xaf => "XRA A".into(),
            0xb0 => "ORA B".into(),
            0xb1 => "ORA C".into(),
            0xb2 => "ORA D".into(),
            0xb3 => "ORA E".into(),
            0xb4 => "ORA H".into(),
            0xb5 => "ORA L".into(),
            0xb6 => "ORA M".into(),
            0xb7 => "ORA A".into(),
            0xb8 => "CMP B".into(),
            0xb9 => "CMP C".into(),
            0xba => "CMP D".into(),
            0xbb => "CMP E".into(),
            0xbc => "CMP H".into(),
            0xbd => "CMP L".into(),
            0xbe => "CMP M".into(),
            0xbf => "CMP A".into(),
            0xc0 => "RNZ".into(),
            0xc1 => "POP B".into(),
            0xc2 => format!("JNZ ${:04x}", self.read_word(address + 1)),
            0xc3 | 0xcb => format!("JMP ${:04x}", self.read_word(address + 1)),
            0xc4 => format!("CNZ ${:04x}", self.read_word(address + 1)),
            0xc5 => "PUSH B".into(),
            0xc6 => format!("ADI ${:02x}", self.read_byte(address + 1)),
            0xc7 => "RST 0".into(),
            0xc8 => "RZ".into(),
            0xc9 | 0xd9 => "RET".into(),
            0xca => format!("JZ ${:04x}", self.read_word(address + 1)),
            0xcc => format!("CZ ${:04x}", self.read_word(address + 1)),
            0xcd | 0xdd | 0xed | 0xfd => format!("CALL ${:04x}", self.read_word(address + 1)),
            0xce => format!("ACI ${:02x}", self.read_byte(address + 1)),
            0xcf => "RST 1".into(),
            0xd0 => "RNC".into(),
            0xd1 => "POP D".into(),
            0xd2 => format!("JNC ${:04x}", self.read_word(address + 1)),
            0xd3 => format!("OUT ${:02x}", self.read_byte(address + 1)),
            0xd4 => format!("CNC ${:04x}", self.read_word(address + 1)),
            0xd5 => "PUSH D".into(),
            0xd6 => format!("SUI ${:02x}", self.read_byte(address + 1)),
            0xd7 => "RST 2".into(),
            0xd8 => "RC".into(),
            0xda => format!("JC ${:04x}", self.read_word(address + 1)),
            0xdb => format!("IN ${:02x}", self.read_byte(address + 1)),
            0xdc => format!("CC ${:04x}", self.read_word(address + 1)),
            0xde => "SBI D8".into(),
            0xdf => "RST 3".into(),
            0xe0 => "RPO".into(),
            0xe1 => "POP H".into(),
            0xe2 => format!("JPO ${:04x}", self.read_word(address + 1)),
            0xe3 => "XTHL".into(),
            0xe4 => format!("CPO ${:04x}", self.read_word(address + 1)),
            0xe5 => "PUSH H".into(),
            0xe6 => format!("ANI ${:02x}", self.read_byte(address + 1)),
            0xe7 => "RST 4".into(),
            0xe8 => "RPE".into(),
            0xe9 => "PCHL".into(),
            0xea => format!("JPE ${:04x}", self.read_word(address + 1)),
            0xeb => "XCHG".into(),
            0xec => format!("CPE ${:04x}", self.read_word(address + 1)),
            0xee => format!("XRI ${:02x}", self.read_byte(address + 1)),
            0xef => "RST 5".into(),
            0xf0 => "RP".into(),
            0xf1 => "POP AF".into(),
            0xf2 => format!("JP ${:04x}", self.read_word(address + 1)),
            0xf3 => "DI".into(),
            0xf4 => format!("CP ${:04x}", self.read_word(address + 1)),
            0xf5 => "PUSH AF".into(),
            0xf6 => format!("ORI ${:02x}", self.read_byte(address + 1)),
            0xf7 => "RST 6".into(),
            0xf8 => "RM".into(),
            0xf9 => "SPHL".into(),
            0xfa => format!("JM ${:04x}", self.read_word(address + 1)),
            0xfb => "EI".into(),
            0xfc => format!("CM ${:04x}", self.read_word(address + 1)),
            0xfe => format!("CPI ${:02x}", self.read_byte(address + 1)),
            0xff => "RST 7".into(),
        }
    }    
}
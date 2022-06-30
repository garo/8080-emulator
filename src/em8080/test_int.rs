use std::{num::ParseIntError};

use crate::em8080::Em8080;
use crate::em8080::IOState;

// Many (but not all) test cases are coming from
// this old 8080 programmers manual
// https://altairclone.com/downloads/manuals/8080%20Programmers%20Manual.pdf


#[test]
fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}

struct TestIO {
    io : [u8; 0xFF],

    test_finished : bool,
}

impl IOState for TestIO {
    fn input(&self, _cpu : &Em8080, port: u8) -> u8 {
        self.io[port as usize]
    }

    fn output(&mut self, cpu : &Em8080, port: u8, value: u8) {
        //println!("OUT {} -> {:02x} {}", port, value, value as char);
        self.io[port as usize] = value;

        if port == 0 {
            self.test_finished = true;
        } else if port == 1 {
            let operation = cpu.c;

            if operation == 0x02 {
                // Print a character stored in E
                // This is presumably a CP/M "syscall"
                print!("{}", cpu.e as char);

            } else if operation == 9 {
                // Print from memory at DE until '$' char
                let mut addr = cpu.get_de();
                loop {
                    print!("{}", cpu.read_byte(addr) as char);
                    addr += 1;
                    if cpu.read_byte(addr) as char == '$' { break; }
                }
            }
        }

/*
  if (port == 0) {
    test_finished = 1;
  } else if (port == 1) {
    uint8_t operation = c->c;

    if (operation == 2) { // print a character stored in E
      printf("%c", c->e);
    } else if (operation == 9) { // print from memory at (DE) until '$' char
      uint16_t addr = (c->d << 8) | c->e;
      do {
        printf("%c", rb(c, addr++));
      } while (rb(c, addr) != '$');
    }
  }
*/
    }    
}

impl TestIO {
    pub fn new() -> Self {
        Self {
            io: [0; 0xFF],

            test_finished : false,
        }
    }    
}

#[test]
fn test_cpm_integration() {
    let program = include_bytes!("../../test_data/TST8080.COM");
    //let program = include_bytes!("../../test_data/8080PRE.COM");

    let mut sys = Em8080::from_rom(program, 0x100, 0x100);
    
    let mut io = TestIO::new();
        
    // inject "out 0,a" at 0x0000 (signal to stop the test)
    sys.memory[0x0000] = 0xD3;
    sys.memory[0x0001] = 0x00;

    // inject "out 1,a" at 0x0005 (signal to output some characters)
    sys.memory[0x0005] = 0xD3;
    sys.memory[0x0006] = 0x01;
    sys.memory[0x0007] = 0xC9;

    let mut c : u64 = 0;
    //sys.trace = true;
    while sys.halted == false {
        c = c + 1;

        /*
        if sys.pc >= 0x0294 && sys.pc < 0x02A2 {
            sys.print_op();
            println!("{:#?}", sys);
        }
        */
        sys.emulate(&mut io);

        if io.test_finished {
            break;
        }
    }

    println!("\nInstructions executed: {}", c);

}
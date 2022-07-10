
mod em8080;

use crate::em8080::Em8080;

pub const SCREEN_WIDTH: usize = 224;
pub const SCREEN_HEIGHT: usize = 256;

use minifb;

struct InvadersIO {
    shift_register: u16,
    shift_amount: u8,
    port0: u8,
    port1: u8,
    port2: u8,
}

impl IOState for InvadersIO {
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

impl InvadersIO {
    pub fn new() -> Self {
        Self {
            io: [0; 0xFF],

            test_finished : false,
        }
    }    
}


fn main() {

    println!("Hello, world!");

    // Create window
    let mut window = minifb::Window::new(
        "8080-emulator",
        224,
        256,
        minifb::WindowOptions {
            borderless: false,
            title: true,
            resize: false,
            scale: minifb::Scale::X2,
            scale_mode: minifb::ScaleMode::Stretch,
            topmost: false,
            transparency: false,
            none: false,
        },
    ).expect("Could not create window");    
    
    let mut sys = Em8080::new();
    println!("{:#?}", sys);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(16));        
    }
}

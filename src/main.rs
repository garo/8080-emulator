
mod em8080;

use crate::em8080::{Em8080, IOState};

pub const SCREEN_WIDTH: usize = 224;
pub const SCREEN_HEIGHT: usize = 256;

use minifb;

#[derive(Clone, Copy)]
#[repr(C)]
pub union RegisterPair {
    both: u16,
    one: (u8, u8),
}

impl RegisterPair {
    pub fn new() -> Self {
        Self { both: 0 }
    }

    pub fn both(self) -> u16 {
        unsafe { self.both }
    }

    pub fn both_mut(&mut self) -> &mut u16 {
        unsafe { &mut self.both }
    }

    /// Least significant byte
    pub fn lsb(self) -> u8 {
        unsafe { self.one.0 }
    }

    /// Least significant byte
    pub fn lsb_mut(&mut self) -> &mut u8 {
        unsafe { &mut self.one.0 }
    }

    /// Most significant byte
    pub fn msb(self) -> u8 {
        unsafe { self.one.1 }
    }

    /// Most significant byte
    pub fn msb_mut(&mut self) -> &mut u8 {
        unsafe { &mut self.one.1 }
    }
}

struct InvadersIO {
    shift_register: RegisterPair,
    shift_amount: u8,
    port0: u8,
    port1: u8,
    port2: u8,
}

impl IOState for InvadersIO {
    fn input(&self, _cpu: &Em8080, port: u8) -> u8 {
        match port {
            1 => {
                self.port1
            },
            2 => self.port2,
            3 => (self.shift_register.both() >> (8 - self.shift_amount)) as u8,
            _ => panic!("Cannot read port: {}", port),
        }
    }

    fn output(&mut self, cpu : &Em8080, port: u8, value: u8) {
        match port {
            2 => self.shift_amount = value & 0b111,
            4 => {
                *self.shift_register.lsb_mut() = self.shift_register.msb();
                *self.shift_register.msb_mut() = value;
            }
            3 | 5 | 6 => {}
            _ => panic!("Cannot write to port: {}", port),
        }
    }
    
}

impl InvadersIO {
    pub fn new() -> Self {
        Self {
            shift_register: RegisterPair::new(),
            shift_amount: 0,
            port0: 0b0111_0000,
            port1: 0b0001_0000,
            port2: 0b0000_0000,
        }
    }

    fn update_input(&mut self, window: &minifb::Window) {
        // Credit
        Self::set_key(&mut self.port1, 0, window.is_key_down(minifb::Key::C));
        // P2 Start
        Self::set_key(&mut self.port1, 1, window.is_key_down(minifb::Key::W));
        // P1 Start
        Self::set_key(&mut self.port1, 2, window.is_key_down(minifb::Key::Q));
        // Always 1
        Self::set_key(&mut self.port1, 3, true);

        // P1 Fire
        Self::set_key(&mut self.port1, 4, window.is_key_down(minifb::Key::Space));
        // P1 Left
        Self::set_key(&mut self.port1, 5, window.is_key_down(minifb::Key::A));
        // P1 Right
        Self::set_key(&mut self.port1, 6, window.is_key_down(minifb::Key::D));

        // P2 Fire
        Self::set_key(&mut self.port2, 4, window.is_key_down(minifb::Key::Enter));
        // P2 Left
        Self::set_key(&mut self.port2, 5, window.is_key_down(minifb::Key::Left));
        // P2 Right
        Self::set_key(&mut self.port2, 6, window.is_key_down(minifb::Key::Right));
    }

    fn set_key(port: &mut u8, bit: u8, on: bool) {
        if on {
            *port |= 1 << bit
        } else {
            *port &= !(1 << bit)
        }
    }    
}

pub struct SpaceInvaders {
    cpu: Em8080,
    io_state: InvadersIO,
    window_buffer: [u32; 224 * 256],
    instructions: u64,
    cycles: u64,
    frames: u64,
}

impl SpaceInvaders {
    const CYCLES_PER_FRAME: u64 = 4_000_000 / 60;
    pub const SCREEN_WIDTH: usize = 224;
    pub const SCREEN_HEIGHT: usize = 256;

    pub fn new() -> Self {
        Self::from_rom(include_bytes!("invaders.rom"))
    }

    pub fn from_rom(rom: &[u8]) -> Self {
        Self {
            cpu: Em8080::from_rom(rom, 0, 0),
            io_state: InvadersIO::new(),
            window_buffer: [0; 224 * 256],
            instructions: 0,
            cycles: 0,
            frames: 0,
        }
    }

    // Proceeds one frame of the emulator
    pub fn step(&mut self, window: &mut minifb::Window) {
        self.half_step(window, true);
        self.half_step(window, false);

        self.frames += 1;

        // Lastly, update input
        self.io_state.update_input(window);

        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    fn half_step(&mut self, window: &mut minifb::Window, top_half: bool) {
        let mut cycles_spent = 0;
        while cycles_spent < Self::CYCLES_PER_FRAME / 2 {
            let cycles = self.cpu.emulate(&mut self.io_state);

            cycles_spent += cycles;

            // For monitoring/debug purposes
            self.instructions += 1;
            self.cycles += cycles;
        }

        // Render half of the screen
        window.update_with_buffer(&self.screen(top_half), SCREEN_WIDTH, SCREEN_HEIGHT)
              .unwrap_or_else(|e| println!("Failed to update window buffer: {}", e));

        // Middle/end of frame interrupt
        self.cpu.interrupt(if top_half { 1 } else { 2 });
    }

    fn screen(&mut self, top_half: bool) -> &[u32] {
        let (start_memory, start_pixel) = if top_half {
            (0x2400, 0)
        } else {
            (0x3200, 0x7000)
        };

        // Iterate half the screen
        for offset in 0..0xE00 {
            let byte = self.cpu.memory[start_memory + offset];

            for bit in 0..8 {
                let color: u32 = if byte & (1 << bit) == 0 {
                    0x00_00_00_00
                } else {
                    0xff_ff_ff_ff
                };

                let x = (start_pixel + 8 * offset + bit) / Self::SCREEN_HEIGHT;
                let y = Self::SCREEN_HEIGHT - 1 - (start_pixel + 8 * offset + bit) % Self::SCREEN_HEIGHT;
                self.window_buffer[x + y * Self::SCREEN_WIDTH] = color;
            }
        }

        &self.window_buffer
    }
}

fn main() {

    println!("Space Invaders. Keys:");
    println!("C to add credits. Q: Start with 1 player, W: start with 2 players");
    println!("Player 1 move A and D, fire Space");

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
    
    let mut invaders = SpaceInvaders::new();
//    invaders.cpu.set_sp(0x4000);
   // invaders.cpu.trace = true;


    while window.is_open() {
        invaders.step(&mut window);
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

}

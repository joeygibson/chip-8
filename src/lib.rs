use std::fs::File;
use std::io;
use std::io::{Error, Read};

// 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
// 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
// 0x200-0xFFF - Program ROM and work RAM
pub struct Chip8 {
    op_code: u16,
    memory: [u8; 4096],
    v: [u8; 16],
    i: usize,
    pc: usize,
    gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    sp: usize,
    key: [u8; 16],
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            op_code: 0,
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            gfx: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            key: [0; 16],
        };

        // Load fonts0et
        for i in 1..80 {
            // chip8.memory[i] = chip8_fontset[i];
        }

        chip8
    }

    fn load_program(&mut self, file_name: &str) -> io::Result<()> {
        let mut f = File::open(file_name)?;
        let mut buffer = Vec::new();

        // read the whole file
        f.read_to_end(&mut buffer)?;

        for (i, b) in buffer.iter().enumerate() {
            self.memory[i + 512] = *b;
        }

        Ok(())
    }

    fn cycle(&mut self) {
        self.op_code = (self.memory[self.pc] << 8 | self.memory[self.pc + 1]).into();

        match self.op_code & 0xF000 {
            0x0000 => {
                // two special opcodes that can't be determined by the
                // top four bits
                match self.op_code & 0x000F {
                    0x0000 => { // 0x00E0; clear the screen
                    }
                    0x000E => { // 0x00EE; returns from subroutine
                    }
                    _ => panic!("unknown opcode [0x0000]: 0x{:#X?}", self.op_code),
                }
            }
            0xA000 => {
                // 0xANNN: sets I to the address NNN
                self.i = (self.op_code & 0x0FFF) as usize;
                self.pc += 2;
            }
            0x1000 => { // 0x1NNN: jumps to address NNN
            }
            0x2000 => {
                // 0x2NNN: calls subroutine at NNN
                self.stack[self.sp] = self.pc as u16;
                self.sp += 1;
                self.pc = (self.op_code & 0x0FFF) as usize;
            }

            // more opcodes...
            0x0004 => {
                // 0x8XY4: adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
                let y = ((self.op_code & 0x00F0) >> 4) as usize;
                let x = ((self.op_code & 0x0F00) >> 8) as usize;

                if self.v[y] > (0xFF - self.v[x]) {
                    self.v[0xF] = 1; // carry the 1
                } else {
                    self.v[0xF] = 0;
                }

                self.v[x] += self.v[y];
                self.pc += 2;
            }

            0x0033 => {
                // 0xFX33:
                let index: usize = ((self.op_code & 0x0F00) >> 8) as usize;
                self.memory[self.i] = self.v[index] / 100;
                self.memory[self.i + 1] = (self.v[index] / 10) % 10;
                self.memory[self.i + 2] = (self.v[index] % 100) / 10;
                self.pc += 2;
            }

            // more opcodes
            _ => panic!("unknown opcode"),
        }

        // update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP!");
            }

            self.sound_timer -= 1;
        }
    }
}

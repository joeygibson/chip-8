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
    i: u16,
    pc: usize,
    gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    sp: u16,
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
            // 0xANNN: sets I to the address NNN
            0xA000 => {
                self.i = self.op_code & 0x0FFF;
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

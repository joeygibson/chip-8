use std::fs::File;
use std::io;
use std::io::Read;

// 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
// 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
// 0x200-0xFFF - Program ROM and work RAM
pub struct Chip8 {
    memory: [u8; 4096], // program memory
    v: [u8; 16],        // registers
    i: u16,             // index register
    pc: u16,            // program counter
    gfx: [u8; 64 * 32], // graphics display
    delay_timer: u8,    // delay timer
    sound_timer: u8,    // sound timer
    stack: [u16; 16],   // program stack
    sp: u8,             // stack pointer
    key: [u8; 16],      // keyboard
    draw_flag: bool,    // drawing flag
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
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
            draw_flag: false,
        };

        // Load fontset
        for i in 1..80 {
            chip8.memory[i] = CHIP8_FONTSET[i];
        }

        chip8
    }

    pub fn load_program(&mut self, file_name: &str) -> io::Result<()> {
        let mut f = File::open(file_name)?;
        let mut buffer = Vec::new();

        // read the whole file
        f.read_to_end(&mut buffer)?;

        for (i, b) in buffer.iter().enumerate() {
            self.memory[i + 512] = *b;
        }

        Ok(())
    }

    pub fn execute_cycle(&mut self) {
        let opcode = read_word(self.memory, self.pc);

        self.process_opcode(opcode);

        self.update_program_counter()
    }

    fn update_program_counter(&mut self) {
        self.pc += 2;
    }

    fn process_opcode(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let nnn = opcode & 0x0FFF;
        let nn = (opcode & 0x00FF) as u8;
        let n = (opcode & 0x000F) as u8;

        match opcode & 0xF000 {
            0x0000 => {
                // two special opcodes that can't be determined by the
                // top four bits
                match opcode & 0x000F {
                    0x0000 => {
                        // 0x00E0; clear the screen
                        for i in 0..2048 {
                            self.gfx[i as usize] = 0;
                        }

                        self.draw_flag = true;
                    }
                    0x000E => {
                        // 0x00EE; returns from subroutine
                        self.sp -= 1;
                        self.pc = self.stack[self.sp as usize];
                    }
                    _ => panic!("unknown opcode [0x0000]: 0x{:#X?}", opcode),
                }
            }

            0x1000 => {
                // 0x1NNN: jumps to address NNN
                self.pc = nnn;
            }

            0x2000 => {
                // 0x2NNN: calls subroutine at NNN
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            }

            0x3000 => {
                // 0x3XNN: Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to skip a code block)
                if self.v[x] == nn {
                    self.pc += 2;
                }
            }

            0x4000 => {
                // 0x4XNN: Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a jump to skip a code block)
                if self.v[x] != nn {
                    self.pc += 2;
                }
            }

            0x5000 => {
                // 0x5XY0: Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to skip a code block)
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }

            0x6000 => {
                // 0x6XNN: Sets VX to NN.
                self.v[x] = nn;
            }

            0x7000 => {
                // 0x7XNN: Adds NN to VX. (Carry flag is not changed)
                self.v[x] += nn;
            }

            0x8000 => {
                match n {
                    0x0 => {
                        // 0x8XY0: Sets VX to the value of VY.
                        self.v[x] = self.v[y];
                    }
                    0x1 => {
                        // 0x8XY1: Sets VX to VX or VY. (Bitwise OR operation)
                        self.v[x] |= self.v[y];
                    }
                    0x2 => {
                        // 0x8XY2: Sets VX to VX and VY. (Bitwise AND operation)
                        self.v[x] &= self.v[y];
                    }
                    0x3 => {
                        // 0x8XY3: Sets VX to VX xor VY.
                        self.v[x] ^= self.v[y];
                    }
                    0x4 => {
                        // 0x8XY4: Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
                        if self.v[y] > (0xFF - self.v[x]) {
                            self.v[0xF] = 1; // carry the 1
                        } else {
                            self.v[0xF] = 0;
                        }

                        self.v[x] += self.v[y];
                    }
                    0x5 => {
                        // 0x8XY5: VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
                        if self.v[y] > (self.v[x]) {
                            self.v[0xF] = 0; // carry the 1
                        } else {
                            self.v[0xF] = 1;
                        }

                        self.v[x] -= self.v[y];
                    }
                    0x6 => {
                        // 0x8XY6: Stores the least significant bit of VX in VF and then shifts VX to the right by 1.
                        self.v[0xF] = self.v[x] & 0x1;
                        self.v[x] >>= 1;
                    }
                    0x7 => {
                        // 0x8XY7: Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
                        if self.v[x] > self.v[y] {
                            self.v[0xF] = 0;
                        } else {
                            self.v[0xF] = 1;
                        }

                        self.v[x] = self.v[y] - self.v[x];
                    }
                    0xE => {
                        // 0x8XYE: Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
                        self.v[0xF] = self.v[x] >> 7;
                        self.v[x] <<= 1;
                    }
                    _ => panic!("unknown opcode [0x0000]: 0x{:#X?}", opcode),
                }
            }

            0x9000 => {
                // 0x9XY0: Skips the next instruction if VX doesn't equal VY. (Usually the next instruction is a jump to skip a code block)
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }

            0xA000 => {
                // 0xANNN: sets I to the address NNN
                self.i = nnn;
            }

            0xB000 => {
                // 0xBNNN: Jumps to the address NNN plus V0.
                self.pc = nnn + self.v[0] as u16;
            }

            0xC000 => {
                // 0xCXNN: Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
                let r: u8 = rand::random();
                self.v[x] = r | nn;
            }

            0xD000 => {
                // 0xDXYN: Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels.
                let height = n;

                self.v[0xF] = 0;

                for yline in 0..height {
                    let pixel = self.memory[(self.i + yline as u16) as usize];

                    for xline in 0..8 {
                        if (pixel & (0x80 >> xline)) != 0 {
                            if self.gfx[(x + xline as usize + ((y + yline as usize) * 64))] == 1 {
                                self.v[0xF] = 1;
                            }

                            self.gfx[x + xline as usize + ((y + yline as usize) * 64)] ^= 1;
                        }
                    }
                }

                self.draw_flag = true;
            }

            0xE000 => {
                match opcode & 0x00FF {
                    0x009E => {
                        // 0xEX9E: Skips the next instruction if the key stored in VX is pressed. (Usually the next instruction is a jump to skip a code block)
                        if self.key[x] != 0 {
                            self.pc += 2;
                        }
                    }
                    0x00A1 => {
                        // 0xEXA1: Skips the next instruction if the key stored in VX isn't pressed. (Usually the next instruction is a jump to skip a code block)
                        if self.key[x] == 0 {
                            self.pc += 2;
                        }
                    }
                    _ => panic!("unknown opcode [0x0000]: 0x{:#X?}", opcode),
                }
            }

            0xF000 => {
                match opcode & 0x00FF {
                    0x0007 => {
                        // 0xFX07: Sets VX to the value of the delay timer.
                        self.v[x] = self.delay_timer;
                    }

                    0x000A => {
                        // 0xFX0A: A key press is awaited, and then stored in VX. (Blocking Operation. All instruction halted until next key event)
                        let mut key_pressed = false;

                        for i in 0..16 {
                            if self.key[i] != 0 {
                                self.v[x] = i as u8;
                                key_pressed = true;
                            }
                        }

                        if !key_pressed {
                            return; // needed?
                        }
                    }

                    0x0015 => {
                        // 0xFX15: Sets the delay timer to VX.
                        self.delay_timer = self.v[x];
                    }

                    0x0018 => {
                        // 0xFX18: Sets the sound timer to VX.
                        self.sound_timer = self.v[x];
                    }

                    0x001E => {
                        // 0xFX1E: Adds VX to I. VF is set to 1 when there is a range overflow (I+VX>0xFFF), and to 0 when there isn't.
                        if self.i + self.v[x] as u16 > 0xFFF {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }

                        self.i += self.v[x] as u16;
                    }

                    0x0029 => {
                        // 0xFX29: Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.
                        self.i = (self.v[x] * 0x5) as u16;
                    }

                    0x0033 => {
                        // 0xFX33: Stores the binary-coded decimal representation of VX, with the most significant of three digits at the address in I, the middle digit at I plus 1, and the least significant digit at I plus 2.
                        self.memory[self.i as usize] = self.v[x] / 100;
                        self.memory[(self.i + 1) as usize] = self.v[x] / 10 % 10;
                        self.memory[(self.i + 2) as usize] = self.v[x] % 10;
                    }

                    0x0055 => {
                        // 0xFX55: Stores V0 to VX (including VX) in memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.
                        for i in 0..x {
                            self.memory[(self.i + i as u16) as usize] = self.v[i];
                        }
                    }

                    0x0065 => {
                        // 0xFX65: Fills V0 to VX (including VX) with values from memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.
                        for i in 0..x {
                            self.v[i] = self.memory[(self.i + i as u16) as usize];
                        }
                    }
                    _ => panic!("unknown opcode [0x0000]: 0x{:#X?}", opcode),
                }
            }

            _ => panic!("unknown opcode [0x0000]: 0x{:#X?}", opcode),
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

static CHIP8_FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

fn read_word(memory: [u8; 4096], index: u16) -> u16 {
    (memory[index as usize] as u16) << 8 | memory[(index + 1) as usize] as u16
}

pub fn run_loop(chip8: &mut Chip8) {
    loop {
        chip8.execute_cycle();
        draw_graphics(chip8);
        // chip8.set_keys();
    }
}

pub fn draw_graphics(chip8: &mut Chip8) -> io::Result<()> {
    Ok(())
}

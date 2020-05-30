use std::error::Error;

use errors::ProgramTooLargeError;

mod errors;

const MEMORY_SIZE: usize = 4096;
const LOWER_MEMORY_BOUNDARY: usize = 512;
const GRAPHICS_COLUMNS: usize = 64;
const GRAPHICS_ROWS: usize = 32;
const GRAPHICS_ARRAY_SIZE: usize = GRAPHICS_COLUMNS * GRAPHICS_ROWS;
const STACK_SIZE: usize = 16;
const KEYBOARD_ARRAY_SIZE: usize = 16;
const REGISTERS: usize = 16;

// 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
// 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
// 0x200-0xFFF - Program ROM and work RAM
pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],          // program memory
    v: [u8; REGISTERS],                 // registers
    i: u16,                             // index register
    pc: u16,                            // program counter
    pub gfx: [u8; GRAPHICS_ARRAY_SIZE], // graphics display
    delay_timer: u8,                    // delay timer
    pub sound_timer: u8,                // sound timer
    stack: [u16; STACK_SIZE],           // program stack
    sp: u8,                             // stack pointer
    pub key: [u8; KEYBOARD_ARRAY_SIZE], // keyboard
    pub draw_flag: bool,                // drawing flag
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Chip8 {
            memory: [0; MEMORY_SIZE],
            v: [0; REGISTERS],
            i: 0,
            pc: 0x200,
            gfx: [0; GRAPHICS_COLUMNS * GRAPHICS_ROWS],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; STACK_SIZE],
            sp: 0,
            key: [0; KEYBOARD_ARRAY_SIZE],
            draw_flag: false,
        };

        // Load fontset
        for i in 0..79 {
            chip8.memory[i] = CHIP8_FONTSET[i];
        }

        chip8
    }

    pub fn load_program(&mut self, program: Vec<u8>) -> Result<(), Box<dyn Error>> {
        if program.len() + LOWER_MEMORY_BOUNDARY > MEMORY_SIZE {
            return Err(Box::new(ProgramTooLargeError));
        }

        for (i, b) in program.iter().enumerate() {
            self.memory[i + 512] = *b;
        }

        Ok(())
    }

    pub fn execute_cycle(&mut self) {
        let opcode = read_word(self.memory, self.pc);

        self.process_opcode(opcode);

        self.update_timers();
    }

    fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // println!("BEEP!");
            }

            self.sound_timer -= 1;
        }
    }

    fn process_opcode(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let vx = self.v[x] as u16;
        let vy = self.v[y] as u16;
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
                        self.pc += 2;
                    }
                    0x000E => {
                        // 0x00EE; returns from subroutine
                        self.sp -= 1;
                        self.pc = self.stack[self.sp as usize];
                        self.pc += 2;
                    }
                    _ => {
                        // 0x0NNN: Calls RCA 1802 program at address NNN. Not necessary for most ROMs.
                        self.pc = nnn;
                    }
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
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }

            0x4000 => {
                // 0x4XNN: Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a jump to skip a code block)
                if self.v[x] != nn {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }

            0x5000 => {
                // 0x5XY0: Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to skip a code block)
                if self.v[x] == self.v[y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }

            0x6000 => {
                // 0x6XNN: Sets VX to NN.
                self.v[x] = nn;
                self.pc += 2;
            }

            0x7000 => {
                // 0x7XNN: Adds NN to VX. (Carry flag is not changed)
                self.v[x] = ((self.v[x] as u16 + nn as u16) & 0xff) as u8;
                self.pc += 2;
            }

            0x8000 => {
                match n {
                    0x0 => {
                        // 0x8XY0: Sets VX to the value of VY.
                        self.v[x] = self.v[y];
                        self.pc += 2;
                    }
                    0x1 => {
                        // 0x8XY1: Sets VX to VX or VY. (Bitwise OR operation)
                        self.v[x] |= self.v[y];
                        self.pc += 2;
                    }
                    0x2 => {
                        // 0x8XY2: Sets VX to VX and VY. (Bitwise AND operation)
                        self.v[x] &= self.v[y];
                        self.pc += 2;
                    }
                    0x3 => {
                        // 0x8XY3: Sets VX to VX xor VY.
                        self.v[x] ^= self.v[y];
                        self.pc += 2;
                    }
                    0x4 => {
                        // 0x8XY4: Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
                        if self.v[y] > (0xFF - self.v[x]) {
                            self.v[0xF] = 1; // carry the 1
                        } else {
                            self.v[0xF] = 0;
                        }

                        self.v[x] = ((self.v[x] as u16 + self.v[y] as u16) & 0xff) as u8;
                        self.pc += 2;
                    }
                    0x5 => {
                        // 0x8XY5: VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
                        if self.v[y] > (self.v[x]) {
                            self.v[0xF] = 0; // carry the 1
                        } else {
                            self.v[0xF] = 1;
                        }

                        let tx = self.v[x];
                        let ty = self.v[y];

                        let tz = if ty > tx {
                            ((tx as i16 - ty as i16).abs() as u8) - 1
                        } else {
                            tx - ty
                        };

                        self.v[x] = tz;
                        self.pc += 2;
                    }
                    0x6 => {
                        // 0x8XY6: Stores the least significant bit of VX in VF and then shifts VX to the right by 1.
                        self.v[0xF] = self.v[x] & 0x1;
                        self.v[x] >>= 1;
                        self.pc += 2;
                    }
                    0x7 => {
                        // 0x8XY7: Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
                        if vx > vy {
                            self.v[0xF] = 0;
                        } else {
                            self.v[0xF] = 1;
                        }

                        let tz = if vx > vy {
                            ((vy as i16 - vx as i16).abs() as u8) - 1
                        } else {
                            (vy - vx) as u8
                        };

                        self.v[x] = tz;
                        self.pc += 2;
                    }
                    0xE => {
                        // 0x8XYE: Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
                        self.v[0xF] = self.v[x] >> 7;
                        self.v[x] <<= 1;
                        self.pc += 2;
                    }
                    _ => panic!("unknown 0x8000 opcode: {:#X?}", opcode),
                }
            }

            0x9000 => {
                // 0x9XY0: Skips the next instruction if VX doesn't equal VY. (Usually the next instruction is a jump to skip a code block)
                if self.v[x] != self.v[y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }

            0xA000 => {
                // 0xANNN: sets I to the address NNN
                self.i = nnn;
                self.pc += 2;
            }

            0xB000 => {
                // 0xBNNN: Jumps to the address NNN plus V0.
                self.pc = nnn + self.v[0] as u16;
            }

            0xC000 => {
                // 0xCXNN: Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255) and NN.
                let r: u8 = rand::random();
                self.v[x] = r | nn;
                self.pc += 2;
            }

            0xD000 => {
                // 0xDXYN: Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels
                // and a height of N pixels.
                let height = n;

                self.v[0xF] = 0;

                for yline in 0..height {
                    let pixel = self.memory[(self.i + yline as u16) as usize];

                    for xline in 0..8 {
                        if (pixel & (0x80 >> xline)) != 0 {
                            let x_coord = (vx + xline as u16) % GRAPHICS_COLUMNS as u16;
                            let y_coord = (vy + yline as u16) % GRAPHICS_ROWS as u16;
                            let pixel_index =
                                ((y_coord * GRAPHICS_COLUMNS as u16) + x_coord) as usize;

                            if self.gfx[pixel_index] == 0x01 {
                                self.v[0xF] = 1;
                            }

                            self.gfx[pixel_index] ^= 0x01;
                        }
                    }
                }

                self.draw_flag = true;
                self.pc += 2;
            }

            0xE000 => {
                match opcode & 0x00FF {
                    0x009E => {
                        // 0xEX9E: Skips the next instruction if the key stored in VX is pressed. (Usually the next instruction is a jump to skip a code block)
                        if self.key[vx as usize] != 0 {
                            // since we can't get key released events, let's clear it out
                            self.key[vx as usize] = 0;
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    }
                    0x00A1 => {
                        // 0xEXA1: Skips the next instruction if the key stored in VX isn't pressed. (Usually the next instruction is a jump to skip a code block)
                        if self.key[vx as usize] == 0 {
                            self.pc += 4;
                        } else {
                            self.key[vx as usize] = 0;
                            self.pc += 2;
                        }
                    }
                    _ => panic!("unknown 0xE000 opcode: {:#X?}", opcode),
                }
            }

            0xF000 => {
                match opcode & 0x00FF {
                    0x0007 => {
                        // 0xFX07: Sets VX to the value of the delay timer.
                        self.v[x] = self.delay_timer;
                        self.pc += 2;
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
                            // Since we didn't get a key press, we do not upate the
                            // program counter, so the same instruciton will
                            // get executed again, effectively waiting forever
                            // for a keypress
                            return;
                        }

                        self.pc += 2;
                    }

                    0x0015 => {
                        // 0xFX15: Sets the delay timer to VX.
                        self.delay_timer = self.v[x];
                        self.pc += 2;
                    }

                    0x0018 => {
                        // 0xFX18: Sets the sound timer to VX.
                        self.sound_timer = self.v[x];
                        self.pc += 2;
                    }

                    0x001E => {
                        // 0xFX1E: Adds VX to I. VF is set to 1 when there is a range overflow (I+VX>0xFFF), and to 0 when there isn't.
                        if self.i + self.v[x] as u16 > 0xFFF {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }

                        self.i += self.v[x] as u16;
                        self.pc += 2;
                    }

                    0x0029 => {
                        // 0xFX29: Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font.
                        self.i = (self.v[x] * 0x5) as u16;
                        self.pc += 2;
                    }

                    0x0033 => {
                        // 0xFX33: Stores the binary-coded decimal representation of VX, with the most significant of three digits at the address in I, the middle digit at I plus 1, and the least significant digit at I plus 2.
                        self.memory[self.i as usize] = self.v[x] / 100;
                        self.memory[(self.i + 1) as usize] = self.v[x] / 10 % 10;
                        self.memory[(self.i + 2) as usize] = self.v[x] % 10;
                        self.pc += 2;
                    }

                    0x0055 => {
                        // 0xFX55: Stores V0 to VX (including VX) in memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.
                        for i in 0..x {
                            self.memory[(self.i + i as u16) as usize] = self.v[i];
                        }
                        self.pc += 2;
                    }

                    0x0065 => {
                        // 0xFX65: Fills V0 to VX (including VX) with values from memory starting at address I. The offset from I is increased by 1 for each value written, but I itself is left unmodified.
                        for i in 0..x {
                            self.v[i] = self.memory[(self.i + i as u16) as usize];
                        }
                        self.pc += 2;
                    }
                    _ => panic!("unknown 0xF000 opcode: {:#X?}", opcode),
                }
            }

            _ => panic!("unknown opcode: {:#X?}", opcode),
        }
    }

    pub fn clear_keys(&mut self) {
        for i in 0..16 as usize {
            self.key[i] = 0;
        }
    }

    pub fn to_string(&self) -> String {
        let mut rows: Vec<String> = vec![];

        for row in self.gfx.chunks(GRAPHICS_COLUMNS) {
            let s: String = row
                .iter()
                .map(|c| if *c == 1 { '*' } else { ' ' })
                .collect();
            rows.push(s.clone());
        }

        rows.join("\n")
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

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{
        Chip8, GRAPHICS_ARRAY_SIZE, GRAPHICS_COLUMNS, GRAPHICS_ROWS, LOWER_MEMORY_BOUNDARY,
    };

    #[test]
    fn test_load_program() {
        let program: Vec<u8> = [0; 512].to_vec();

        let chip8 = create_and_load(&program);
        assert!(chip8.is_ok())
    }

    #[test]
    fn test_load_program_that_is_too_big() {
        let program: Vec<u8> = [0; 8192].to_vec();

        let chip8 = create_and_load(&program);
        assert!(chip8.is_err())
    }

    #[test]
    fn test_clear_screen() {
        // 0x00E0; clear the screen
        let program: Vec<u8> = vec![0xF, 0x0];

        let mut chip8 = create_and_load(&program).unwrap();

        for i in 0..GRAPHICS_ARRAY_SIZE {
            chip8.gfx[i] = 1;
        }

        chip8.execute_cycle();

        let all_empty = chip8.gfx.iter().all(|b| *b == 0);

        assert!(all_empty);
        assert!(chip8.draw_flag);
        assert_eq!(chip8.pc, (LOWER_MEMORY_BOUNDARY + 2) as u16);
    }

    #[test]
    fn test_return_from_subroutine() {
        // 0x00EE; returns from subroutine
        // placeholder test
    }

    #[test]
    fn test_jump_to_address() {
        // 0x1NNN: jumps to address NNN
        let program: Vec<u8> = vec![0x10, 0xDC];

        let mut chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.memory[0xDC as usize], 0);
        chip8.memory[0xDC as usize] = 1;
        assert_eq!(chip8.memory[0xDC as usize], 1);

        chip8.execute_cycle();

        assert_eq!(chip8.pc, 0xDC);
        assert_eq!(chip8.memory[chip8.pc as usize], 1);
    }

    #[test]
    fn test_call_subroutine_at_nnn() {
        // 0x2NNN: calls subroutine at NNN
        let program: Vec<u8> = vec![0x20, 0xDC];

        let mut chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.sp, 0);

        chip8.execute_cycle();

        assert_eq!(chip8.pc, 0xDC);
        assert_eq!(chip8.sp, 1);
        assert_eq!(chip8.stack[0], 512);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_equals_nn_positive() {
        // 0x3XNN: Skips the next instruction if VX equals NN.
        let program: Vec<u8> = vec![0x34, 0x17];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0x17;

        let orig_pc = chip8.pc;

        chip8.execute_cycle();

        assert_eq!(chip8.pc, orig_pc + 4);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_equals_nn_negative() {
        // 0x3XNN: Skips the next instruction if VX equals NN.
        let program: Vec<u8> = vec![0x34, 0x17];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0x23;

        let orig_pc = chip8.pc;

        chip8.execute_cycle();

        assert_eq!(chip8.pc, orig_pc + 2);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_does_not_equal_nn_positive() {
        // 0x4XNN: Skips the next instruction if VX doesn't equal NN.
        let program: Vec<u8> = vec![0x44, 0x17];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0x23;

        let orig_pc = chip8.pc;

        chip8.execute_cycle();

        assert_eq!(chip8.pc, orig_pc + 4);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_does_not_equal_nn_negative() {
        // 0x4XNN: Skips the next instruction if VX doesn't equal NN.
        let program: Vec<u8> = vec![0x44, 0x17];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0x17;

        let orig_pc = chip8.pc;

        chip8.execute_cycle();

        assert_eq!(chip8.pc, orig_pc + 2);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_equals_vy_positive() {
        // 0x5XY0: Skips the next instruction if VX equals VY.
        let program: Vec<u8> = vec![0x54, 0x60];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0x17;
        chip8.v[6] = 0x17;

        let orig_pc = chip8.pc;

        chip8.execute_cycle();

        assert_eq!(chip8.pc, orig_pc + 4);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_equals_vy_negative() {
        // 0x5XY0: Skips the next instruction if VX equals VY.
        let program: Vec<u8> = vec![0x54, 0x60];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0x17;
        chip8.v[6] = 0x23;

        let orig_pc = chip8.pc;

        chip8.execute_cycle();

        assert_eq!(chip8.pc, orig_pc + 2);
    }

    #[test]
    fn test_set_vx_to_nn() {
        // 0x6XNN: Sets VX to NN.
        let program: Vec<u8> = vec![0x64, 0xAA];

        let chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.v[4], 0);

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.execute_cycle();

        assert_eq!(chip8.v[4], 0xAA);
    }

    #[test]
    fn test_add_nn_to_vx() {
        // 0x7XNN: Adds NN to VX. (Carry flag is not changed)
        let program: Vec<u8> = vec![0x74, 0xAA];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0x10;

        chip8.execute_cycle();

        assert_eq!(chip8.v[4], 0xBA);
    }

    #[test]
    fn test_add_nn_to_vx_wrapping() {
        // 0x7XNN: Adds NN to VX. (Carry flag is not changed)
        let program: Vec<u8> = vec![0x74, 0xAA];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0xBA;

        chip8.execute_cycle();

        assert_eq!(chip8.v[4], 0x64);
    }

    #[test]
    fn test_set_vx_to_value_of_vy() {
        // 0x8XY0: Sets VX to the value of VY.
        let program: Vec<u8> = vec![0x84, 0x50];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0xBA;
        chip8.v[5] = 0xDD;

        chip8.execute_cycle();

        assert_eq!(chip8.v[4], 0xDD);
        assert_eq!(chip8.v[5], 0xDD);
    }

    #[test]
    fn test_set_vx_to_vx_or_vy() {
        // 0x8XY1: Sets VX to VX or VY. (Bitwise OR operation)
        let program: Vec<u8> = vec![0x84, 0x51];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0xBA;
        chip8.v[5] = 0xCC;

        chip8.execute_cycle();

        assert_eq!(chip8.v[4], 0xFE);
        assert_eq!(chip8.v[5], 0xCC);
    }

    #[test]
    fn test_set_vx_to_vx_and_vy() {
        // 0x8XY2: Sets VX to VX and VY. (Bitwise AND operation)
        let program: Vec<u8> = vec![0x84, 0x52];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0xBA;
        chip8.v[5] = 0xCC;

        chip8.execute_cycle();

        assert_eq!(chip8.v[4], 0x88);
        assert_eq!(chip8.v[5], 0xCC);
    }

    #[test]
    fn test_set_vx_to_vx_xor_vy() {
        // 0x8XY3: Sets VX to VX xor VY.
        let program: Vec<u8> = vec![0x84, 0x53];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0xBA;
        chip8.v[5] = 0xCC;

        chip8.execute_cycle();

        assert_eq!(chip8.v[4], 0x76);
        assert_eq!(chip8.v[5], 0xCC);
    }

    #[test]
    fn test_add_vy_to_vx_with_carry() {
        // 0x8XY4: Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
        let program: Vec<u8> = vec![0x84, 0x54];

        let mut chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.v[0xF], 0);

        chip8.v[4] = 0xBA;
        chip8.v[5] = 0xCC;

        chip8.execute_cycle();

        assert_eq!(chip8.v[4], 0x86);
        assert_eq!(chip8.v[0xF], 1);
    }

    #[test]
    fn test_add_vy_to_vx_without_carry() {
        // 0x8XY4: Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
        let program: Vec<u8> = vec![0x84, 0x54];

        let mut chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.v[0xF], 0);

        chip8.v[4] = 0xBA;
        chip8.v[5] = 0x10;

        chip8.execute_cycle();

        assert_eq!(chip8.v[4], 0xCA);
        assert_eq!(chip8.v[0xF], 0);
    }

    #[test]
    fn test_subtract_vy_from_vx_with_borrow() {
        // 0x8XY5: VY is subtracted from VX. VF is set to 0 when there's a borrow,
        // and 1 when there isn't.
        let program: Vec<u8> = vec![0x84, 0x55];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0xBA;
        chip8.v[5] = 0xCC;

        chip8.execute_cycle();

        assert_eq!(chip8.v[4], 0x11);
        assert_eq!(chip8.v[0xF], 0);
    }

    #[test]
    fn test_store_least_significant_bit_of_vx_in_vf_and_shift_vx_right_by_1() {
        // 0x8XY6: Stores the least significant bit of VX in VF and then shifts VX to
        // the right by 1.
        let program: Vec<u8> = vec![0x84, 0x56];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0xBB;
        chip8.v[0xF] = 0x0;

        chip8.execute_cycle();

        assert_eq!(chip8.v[4], 0x5D);
        assert_eq!(chip8.v[0xF], 1);
    }

    #[test]
    fn test_set_vx_to_vy_minus_vx_with_borrow() {
        // 0x8XY7: Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1
        // when there isn't.
        let program: Vec<u8> = vec![0x84, 0x57];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0xCC;
        chip8.v[5] = 0xBA;

        chip8.execute_cycle();

        assert_eq!(chip8.v[4], 0x11);
        assert_eq!(chip8.v[0xF], 0);
    }

    #[test]
    fn test_store_most_significant_bit_of_vx_in_vf_and_shift_vx_right_by_1() {
        // 0x8XYE: Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
        let program: Vec<u8> = vec![0x84, 0x5E];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0xF0;
        chip8.v[0xF] = 0x0;

        chip8.execute_cycle();

        assert_eq!(chip8.v[4], 0xE0);
        assert_eq!(chip8.v[0xF], 1);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_does_not_equal_vy_positive() {
        // 0x9XY0: Skips the next instruction if VX doesn't equal VY. (Usually the next
        // instruction is a jump to skip a code block)
        let program: Vec<u8> = vec![0x94, 0x60];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0x23;
        chip8.v[6] = 0x17;

        let orig_pc = chip8.pc;

        chip8.execute_cycle();

        assert_eq!(chip8.pc, orig_pc + 4);
    }

    #[test]
    fn test_skip_next_instruction_if_vx_does_not_equal_vy_negative() {
        // 0x9XY0: Skips the next instruction if VX doesn't equal VY. (Usually the next
        // instruction is a jump to skip a code block)
        let program: Vec<u8> = vec![0x94, 0x60];

        let mut chip8 = create_and_load(&program).unwrap();

        chip8.v[4] = 0x17;
        chip8.v[6] = 0x17;

        let orig_pc = chip8.pc;

        chip8.execute_cycle();

        assert_eq!(chip8.pc, orig_pc + 2);
    }

    #[test]
    fn test_set_i_to_address_nnn() {
        // 0xANNN: sets I to the address NNN
        let program: Vec<u8> = vec![0xA0, 0xDC];

        let mut chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.i, 0);

        chip8.execute_cycle();

        assert_eq!(chip8.i, 0xDC);
    }

    #[test]
    fn test_jump_to_nnn_plus_v0() {
        // 0xBNNN: Jumps to the address NNN plus V0.
        let program: Vec<u8> = vec![0xB0, 0xDC];

        let mut chip8 = create_and_load(&program).unwrap();

        assert_eq!(chip8.i, 0);

        chip8.v[0] = 0x17;

        chip8.execute_cycle();

        assert_eq!(chip8.pc, 0xF3);
    }

    #[test]
    fn test_draw_sprite_at_x_y_with_height_n_with_no_collision() {
        // 0xDXYN: Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels
        // and a height of N pixels.
        let height = 5;
        let start_x = 10;
        let start_y = 10;

        // Draw the 0 pixel at (10, 10)
        let program: Vec<u8> = vec![0xD4, 0x65];

        let mut chip8 = create_and_load(&program).unwrap();

        let how_many_ones = chip8.gfx.iter().filter(|b| **b == 1).count();

        assert_eq!(how_many_ones, 0);
        assert_eq!(chip8.v[0xF], 0);

        // set i to the first sprite in the font set (the number 0)
        chip8.i = 0;
        chip8.v[4] = start_x;
        chip8.v[6] = start_y;

        chip8.execute_cycle();

        let x_coord = (start_x % GRAPHICS_ROWS as u8) as usize;
        let y_coord = (start_y % GRAPHICS_COLUMNS as u8) as usize;

        let start_pixel = ((y_coord * GRAPHICS_COLUMNS) + x_coord) as usize;
        let end_pixel = start_pixel + (GRAPHICS_COLUMNS * height);

        let how_many_ones = chip8.gfx[start_pixel..end_pixel]
            .iter()
            .filter(|b| **b == 1)
            .count();

        assert_eq!(how_many_ones, 14);
        assert_eq!(chip8.v[0xF], 0);
        assert!(chip8.draw_flag);
    }

    #[test]
    fn test_draw_sprite_at_x_y_with_height_n_with_collision() {
        // 0xDXYN: Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels
        // and a height of N pixels.
        let height = 5;
        let start_x = 10;
        let start_y = 10;

        // Draw the 0 pixel at (10, 10), twice, which should result in
        // 0 pixels being set to 1, and the `chip8.v[0xF]` should be set to 1,
        // indicating a collistion
        let program: Vec<u8> = vec![0xD4, 0x65, 0xD4, 0x65];

        let mut chip8 = create_and_load(&program).unwrap();

        let how_many_ones = chip8.gfx.iter().filter(|b| **b == 1).count();

        assert_eq!(how_many_ones, 0);
        assert_eq!(chip8.v[0xF], 0);

        // set i to the first sprite in the font set (the number 0)
        chip8.i = 0;
        chip8.v[4] = start_x;
        chip8.v[6] = start_y;

        // This will draw the `0`
        chip8.execute_cycle();

        let x_coord = (start_x % GRAPHICS_ROWS as u8) as usize;
        let y_coord = (start_y % GRAPHICS_COLUMNS as u8) as usize;

        let start_pixel = ((y_coord * GRAPHICS_COLUMNS) + x_coord) as usize;
        let end_pixel = start_pixel + (GRAPHICS_COLUMNS * height);

        let how_many_ones = chip8.gfx[start_pixel..end_pixel]
            .iter()
            .filter(|b| **b == 1)
            .count();

        assert_eq!(how_many_ones, 14);
        assert_eq!(chip8.v[0xF], 0);
        assert!(chip8.draw_flag);

        // This will redraw the `0`, which should erase the previous one
        chip8.execute_cycle();

        let x_coord = (start_x % GRAPHICS_ROWS as u8) as usize;
        let y_coord = (start_y % GRAPHICS_COLUMNS as u8) as usize;

        let start_pixel = ((y_coord * GRAPHICS_COLUMNS) + x_coord) as usize;
        let end_pixel = start_pixel + (GRAPHICS_COLUMNS * height);

        let how_many_ones = chip8.gfx[start_pixel..end_pixel]
            .iter()
            .filter(|b| **b == 1)
            .count();

        assert_eq!(how_many_ones, 0);
        assert_eq!(chip8.v[0xF], 1);
        assert!(chip8.draw_flag);
    }

    #[test]
    fn test_skip_next_instruction_if_key_in_vx_is_pressed_positive() {
        // 0xEX9E: Skips the next instruction if the key stored in VX is pressed.
        let key_index: u8 = 0x4;
        let program: Vec<u8> = vec![0xE4, 0x9E];

        let mut chip8 = create_and_load(&program).unwrap();
        let keys_pressed = chip8.key.iter().filter(|k| **k == 1).count();

        assert_eq!(keys_pressed, 0);

        let orig_pc = chip8.pc;

        chip8.v[4] = key_index;
        chip8.key[key_index as usize] = 1;

        chip8.execute_cycle();

        let keys_pressed = chip8.key.iter().filter(|k| **k == 1).count();

        assert_eq!(keys_pressed, 0);
        assert_eq!(chip8.pc, orig_pc + 4);
    }

    #[test]
    fn test_skip_next_instruction_if_key_in_vx_is_pressed_negative() {
        // 0xEX9E: Skips the next instruction if the key stored in VX is pressed.
        let key_index: u8 = 0x4;
        let program: Vec<u8> = vec![0xE4, 0x9E];

        let mut chip8 = create_and_load(&program).unwrap();
        let keys_pressed = chip8.key.iter().filter(|k| **k == 1).count();

        assert_eq!(keys_pressed, 0);

        let orig_pc = chip8.pc;

        chip8.v[4] = key_index;

        chip8.execute_cycle();

        let keys_pressed = chip8.key.iter().filter(|k| **k == 1).count();

        assert_eq!(keys_pressed, 0);
        assert_eq!(chip8.pc, orig_pc + 2);
    }

    #[test]
    fn test_skip_next_instruction_if_key_in_vx_is_not_pressed_positive() {
        // 0xEXA1: Skips the next instruction if the key stored in VX isn't pressed.
        let key_index: u8 = 0x4;
        let program: Vec<u8> = vec![0xE4, 0xA1];

        let mut chip8 = create_and_load(&program).unwrap();
        let keys_pressed = chip8.key.iter().filter(|k| **k == 1).count();

        assert_eq!(keys_pressed, 0);

        let orig_pc = chip8.pc;

        chip8.v[4] = key_index;

        chip8.execute_cycle();

        let keys_pressed = chip8.key.iter().filter(|k| **k == 1).count();

        assert_eq!(keys_pressed, 0);
        assert_eq!(chip8.pc, orig_pc + 4);
    }

    #[test]
    fn test_skip_next_instruction_if_key_in_vx_is_not_pressed_negative() {
        // 0xEXA1: Skips the next instruction if the key stored in VX isn't pressed.
        let key_index: u8 = 0x4;
        let program: Vec<u8> = vec![0xE4, 0xA1];

        let mut chip8 = create_and_load(&program).unwrap();
        let keys_pressed = chip8.key.iter().filter(|k| **k == 1).count();

        assert_eq!(keys_pressed, 0);

        let orig_pc = chip8.pc;

        chip8.v[4] = key_index;
        chip8.key[key_index as usize] = 1;

        chip8.execute_cycle();

        let keys_pressed = chip8.key.iter().filter(|k| **k == 1).count();

        assert_eq!(keys_pressed, 0);
        assert_eq!(chip8.pc, orig_pc + 2);
    }



    fn create_and_load(program: &Vec<u8>) -> Result<Chip8, Box<dyn Error>> {
        let mut chip8 = Chip8::new();

        chip8.load_program(program.clone())?;

        Ok(chip8)
    }
}

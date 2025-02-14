use core::panic;
use rand;

use crate::fontset::FONTSET;
use std::cmp::min;

const PROGRAM_START: u16 = 0x200;
const MEMORY_SIZE: usize = 4096;

/*
   Notes on Sprites:
   * One byte corresponds to one row of a sprite
   * 1 means white
   * All sprites are drawn using XOR mode
     * During drawing, the sprite data is XOR'd with the current screen
     * To erase a sprite, draw it again
*/

#[derive(Debug)]
pub struct CPU {
    pub opcode: u16,

    // Registers
    pub v: [u8; 16],

    // Timers
    // Both count down at 60 hz
    pub delay_timer: u8, // read/write
    pub sound_timer: u8, // write only

    // Memory
    pub memory: [u8; MEMORY_SIZE],

    pub i: u16,  // index register. Max = 0xfff
    pub pc: u16, // program counter. Max = 0xfff

    // Display
    // 2048 pixels total, binary state black or white
    pub gfx: [u8; 64 * 32],

    // Stack
    pub stack: [u16; 16],
    pub sp: u16, // stack pointer

    // Keybuoard
    pub keys: [u8; 16],
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            opcode: 0,
            v: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            memory: [0; MEMORY_SIZE],
            i: 0,
            pc: 0,
            gfx: [0; 64 * 32],
            stack: [0; 16],
            sp: 0,
            keys: [0; 16],
        }
    }

    pub fn initialize(&mut self) {
        self.pc = PROGRAM_START;
        self.opcode = 0;
        self.i = 0;
        self.sp = 0;
        self.v = [0; 16];
    }

    pub fn load(&mut self, input: Vec<u8>) {
        let size = input.len();
        if (PROGRAM_START + size as u16) as usize > MEMORY_SIZE {
            panic!("Program is too large to load.");
        }
        for (i, &byte) in input.iter().enumerate() {
            let idx: usize = (PROGRAM_START + i as u16) as usize;
            self.memory[idx] = byte;
        }
    }

    pub fn cycle(&mut self) {
        // Fetch opcode
        // TODO: Does opcode need to be a member variable?
        self.opcode = (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[self.pc as usize + 1] as u16);
        let opcode = self.opcode;

        match opcode & 0xF000 {
            0x0 => match opcode & 0x00FF {
                0xE0 => {
                    // Clear the screen
                    self.gfx = [0; 64 * 32];
                    self.pc += 2;
                }
                // TODO: Stack pushing popping tests
                0xEE => {
                    println!("return");
                    self.pc = self.stack[self.sp as usize];
                    self.sp -= 1;
                }
                _ => {
                    println!("Unknown opcode: 0x{:x}", opcode);
                }
            },
            0x1000 => {
                // Jump to address NNN
                println!("1NNN");
                self.pc = opcode & 0x0FFF;
            }
            0x2000 => {
                // Call subroutine at NNN
                println!("2NNN");
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = opcode & 0x0FFF;
            }
            0x3000 => {
                // Skip next instruction if vX == NN
                println!("3XNN");
                if self.v[(opcode & 0x0F00) as usize >> 8] == (opcode & 0x00FF) as u8 {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            0x4000 => {
                // Skip next instruction if vX != NN
                println!("4XNN");
                if self.v[(opcode & 0x0F00) as usize >> 8] != (opcode & 0x00FF) as u8 {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            0x5000 => {
                // Skip next instruction if VX == VY
                println!("5XY0");
                self.pc += 2;
                if self.v[(opcode & 0x0F00) as usize >> 8]
                    == self.v[(opcode & 0x00F0) as usize >> 4]
                {
                    self.pc += 2;
                }
            }
            0x6000 => {
                // Store NN in vX
                println!("6XNN");
                self.v[(opcode & 0x0F00) as usize >> 8] = (opcode & 0x00FF) as u8;
                self.pc += 2;
            }
            0x7000 => {
                println!("7XNN");
                self.v[(opcode & 0x0F00) as usize >> 8] += (opcode & 0x00FF) as u8;
                self.pc += 2;
            }
            0x8000 => match opcode & 0xF {
                0x0 => {
                    // Store VY in VX
                    println!("8XY0");
                    self.v[(opcode & 0x0F00) as usize >> 8] =
                        self.v[(opcode & 0x00F0) as usize >> 4];
                    self.pc += 2;
                }
                0x1 => {
                    // store VY | VX in VX
                    println!("8XY1");
                    self.v[(opcode & 0x0F00) as usize >> 8] |=
                        self.v[(opcode & 0x00F0) as usize >> 4];
                    self.pc += 2;
                }
                0x2 => {
                    // store VY & VX in VX
                    println!("8XY2");
                    self.v[(opcode & 0x0F00) as usize >> 8] &=
                        self.v[(opcode & 0x00F0) as usize >> 4];
                    self.pc += 2;
                }
                0x3 => {
                    // store VY xor VX in VX
                    println!("8XY3");
                    self.v[(opcode & 0x0F00) as usize >> 8] ^=
                        self.v[(opcode & 0x00F0) as usize >> 4];
                    self.pc += 2;
                }
                0x4 => {
                    // Add VY to VX
                    // if carry occurred, set vf to 01 else vf to 00
                    println!("8XY4");
                    let x = (opcode & 0x0F00) as usize >> 8;
                    let y = (opcode & 0x00F0) as usize >> 4;
                    let (result, overflow) = self.v[x].overflowing_add(self.v[y]);
                    self.v[x] = result;
                    self.v[0xF] = if overflow { 0x01 } else { 0x00 };
                    self.pc += 2;
                }
                0x5 => {
                    println!("8XY5");
                    // Sub VY from VX
                    // if borrow occurred, set vf to 00 else vf to 01
                    let x = (opcode & 0x0F00) as usize >> 8;
                    let y = (opcode & 0x00F0) as usize >> 4;
                    let (result, borrow) = self.v[x].overflowing_sub(self.v[y]);
                    self.v[x] = result;
                    self.v[0xF] = if borrow { 0x00 } else { 0x01 };
                    self.pc += 2;
                }
                0x6 => {
                    // Store NN in vX
                    println!("8XY6");
                    self.v[(opcode & 0x0F00) as usize >> 8] = (opcode & 0x00FF) as u8;
                    self.pc += 2;
                }
                0x7 => {
                    // Add NN to vX
                    println!("8XY7");
                    self.v[(opcode & 0x0F00) as usize >> 8] =
                        self.v[(opcode & 0x00F0) as usize >> 4] + (opcode & 0x00FF) as u8;
                    self.pc += 2;
                }
                0xE => {
                    // Store vy << 1 in vx. Set vf to most significant bit of vy before shift.
                    println!("8XYE");
                    let x = (opcode & 0x0F00) as usize >> 8;
                    let y = (opcode & 0x00F0) as usize >> 4;
                    self.v[0xF] = self.v[y] >> 7;
                    self.v[x] = self.v[y] << 1;
                    self.pc += 2;
                }
                _ => {
                    println!("Unknown opcode: 0x{:x}", opcode);
                }
            },
            0x9000 => {
                // Skip next instruction if vX != vY
                println!("9XY0");
                let x = (opcode & 0x0F00) as usize >> 8;
                let y: usize = (opcode & 0x00F0) as usize >> 4;
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
                self.pc += 2;
            }
            0xA000 => {
                // ANNN: Sets I to address NNN
                println!("ANNN");
                self.i = opcode & 0x0FFF;
                self.pc += 2;
            }
            0xB000 => {
                // Jump to NNN + v0
                println!("BNNN");
                self.pc = (opcode & 0x0FFF) + self.v[0] as u16;
            }
            0xC000 => {
                // Set vX to random number & NN
                println!("CXNN");
                let x = (opcode & 0x0F00) as usize >> 8;
                let nn = (opcode & 0x00FF) as u8;
                self.v[x] = rand::random::<u8>() & nn;
                self.pc += 2;
            }
            0xD000 => {
                println!("DXYN");
                // Draw sprite DXYN
                // Notes:
                // * Draw at coordinates vX, vY
                // * Use N bytes starting at address I
                // * Set VF to 1 if any set pixels are changed to unset, else 0
                // * To be visible on the screen, the vX register must be
                //      between 00 and 3F. vY must be between 00 and 1F
                let x = self.v[(opcode & 0x0F00) as usize >> 8] as usize;
                let y = self.v[(opcode & 0x00F0) as usize >> 4] as usize;
                let n = (opcode & 0x000F) as usize;
                let addr = self.i as usize;
                // Height is the min of N and 1F - vY
                let height = min(n, 0x1F - y);
                let max_col = min(8, 0x3F - x);
                self.v[0xF] = 0;

                for row in 0..height {
                    let sprite = self.memory[addr + row];
                    for col in 0..max_col {
                        let pixel = sprite & (0x80 >> col);
                        let idx: usize = (x + col + ((y + row) * 64)) as usize;
                        if pixel != 0 {
                            if self.gfx[idx] == 1 {
                                self.v[0xF] = 1;
                            }
                            self.gfx[idx] ^= 1;
                        }
                    }
                }
            }
            0xE000 => match opcode & 0x00FF {
                // TODO: Combine code for these two
                0x9E => {
                    // Skip instruction if key with value vX is pressed
                    println!("EX9E");
                    let val = self.v[(opcode & 0x0F00) as usize >> 8];
                    if self.keys[val as usize] == 1 {
                        self.pc += 2;
                    }
                    self.pc += 2;
                }
                0xA1 => {
                    println!("EXA1");
                    let val = self.v[(opcode & 0x0F00) as usize >> 8];
                    if self.keys[val as usize] == 0 {
                        self.pc += 2;
                    }
                    self.pc += 2;
                }
                _ => {
                    println!("Unknown opcode: 0x{:x}", opcode);
                }
            },
            0xF000 => match opcode & 0x00FF {
                0x07 => {
                    // store delay timer in vX
                    println!("FX07");
                    self.v[(opcode & 0x0F00) as usize >> 8] = self.delay_timer;
                    self.pc += 2;
                }
                0x0A => {
                    // Wait for keypress and store in vX
                    println!("FX0A");
                    let first_pressed_key = self.keys.iter().position(|&x| x == 1);
                    match first_pressed_key {
                        Some(key) => {
                            self.v[(opcode & 0x0F00) as usize >> 8] = key as u8;
                            self.pc += 2;
                        }
                        None => {}
                    }
                }
                0x15 => {
                    // Set delay timer to vX
                    println!("FX15");
                    self.delay_timer = self.v[(opcode & 0x0F00) as usize >> 8];
                }
                0x18 => {
                    // Set sound timer to vX
                    println!("FX18");
                    self.sound_timer = self.v[(opcode & 0x0F00) as usize >> 8];
                }
                0x1E => {
                    // Add vX to I
                    println!("FX1E");
                    self.i += self.v[(opcode & 0x0F00) as usize >> 8] as u16;
                }
                0x29 => {
                    // Set I to location of sprite for digit vX
                    // Notes:
                    // * vX should be between 0 and F
                    // * Fontset is between 0x050-0x0A0
                    // * Each character is 5 bytes long
                    println!("FX29");
                    self.i = self.v[(opcode & 0x0F00) as usize >> 4] as u16 * 5 + 0x50;
                    self.pc += 2;
                }
                0x33 => {
                    // Store binary-coded decimal representation of vX at I, I+1, I+2
                    // Notes:
                    // * Since each register is 8 bits, will have at most 3 decimal digits (0-255)
                    // * Store most significant digit at I, next at I+1, least significant at I+2
                    println!("FX33");
                    let x = (opcode & 0x0F00) as usize >> 8;
                    let val = self.v[x];
                    self.memory[self.i as usize] = val / 100;
                    self.memory[(self.i + 1) as usize] = (val / 10) % 10;
                    self.memory[(self.i + 2) as usize] = val % 10;
                    self.pc += 2;
                }
                0x55 => {
                    // Store v0 to vX in memory starting at I
                    println!("FX55");
                    let x = (opcode & 0x0F00) as usize >> 8;
                    for i in 0..x {
                        self.memory[(self.i + i as u16) as usize] = self.v[i];
                    }
                    self.i = self.i + x as u16 + 1;
                    self.pc += 2;
                }
                0x65 => {
                    // Load v0 to vX from memory starting at I
                    println!("FX65");
                    let x = (opcode & 0x0F00) as usize >> 8;
                    for i in 0..x {
                        self.v[i] = self.memory[(self.i + i as u16) as usize];
                    }
                    self.i = self.i + x as u16 + 1;
                    self.pc += 2;
                }
                _ => {
                    println!("Unknown opcode: 0x{:x}", opcode);
                }
            },

            _ => {
                println!("Unknown opcode: 0x{:x}", opcode);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    fn setup() -> CPU {
        let mut cpu = CPU::new();
        cpu.initialize();
        cpu
    }

    #[test]
    fn test_initialization() {
        let cpu = setup();
        assert_eq!(cpu.pc, PROGRAM_START);
        assert_eq!(cpu.opcode, 0);
        assert_eq!(cpu.i, 0);
        assert_eq!(cpu.sp, 0);
    }

    #[test]
    fn test_load_program() {
        let mut cpu = setup();
        let program = vec![0xA2, 0xB4]; // ANNN instruction
        cpu.load(program.clone());

        // Verify program was loaded at correct address
        assert_eq!(cpu.memory[PROGRAM_START as usize], program[0]);
        assert_eq!(cpu.memory[PROGRAM_START as usize + 1], program[1]);
    }

    #[test]
    fn test_clear_screen() {
        let mut cpu = setup();
        let program = vec![0x00, 0xE0];
        cpu.load(program.clone());
        cpu.gfx[0] = 1;
        cpu.gfx[64] = 1;
        cpu.cycle();
        assert!(cpu.gfx.iter().all(|&x| x == 0));
        assert!(cpu.pc == PROGRAM_START + 2);
    }

    #[test]
    fn test_jump() {
        let mut cpu = setup();
        let program = vec![0x12, 0x34];
        cpu.load(program.clone());
        cpu.cycle();
        assert!(cpu.pc == 0x1234 & 0x0FFF);
    }

    #[test]
    fn test_call_subroutine() {
        let mut cpu = setup();
        let program = vec![0x22, 0x34];
        cpu.load(program.clone());
        cpu.cycle();
        assert!(cpu.sp == 1);
        assert!(cpu.stack[0] == 0x200);
        assert!(cpu.pc == 0x1234 & 0x0FFF);
    }

    #[test]
    fn test_skip_equal() {
        let mut cpu = setup();
        let program = vec![0x30, 0x11];

        cpu.load(program.clone());
        cpu.v[0] = 0x11;
        assert!(cpu.pc == 0x200, "got 0x{:X}", cpu.pc);
        cpu.cycle();
        assert!(cpu.pc == 0x204, "got 0x{:X}", cpu.pc);

        cpu.initialize();
        cpu.load(program.clone());
        cpu.cycle();
        assert!(cpu.pc == 0x202);
    }

    #[test]
    fn test_skip_not_equal() {
        let mut cpu = setup();
        let program = vec![0x40, 0x11];

        cpu.load(program.clone());
        cpu.v[0] = 0x11;
        cpu.cycle();
        assert!(cpu.pc == 0x202);

        cpu.initialize();
        cpu.load(program.clone());
        cpu.cycle();
        assert!(cpu.pc == 0x204);
    }

    #[test]
    fn test_skip_equal_registers() {
        let mut cpu = setup();
        let program = vec![0x51, 0x20];
        cpu.load(program.clone());
        cpu.v[1] = 0x01;
        cpu.v[2] = 0x00;

        cpu.cycle();
        assert!(cpu.pc == 0x202);

        cpu.initialize();
        cpu.load(program.clone());
        cpu.v[1] = 0x01;
        cpu.v[2] = 0x01;
        cpu.cycle();
        assert!(cpu.pc == 0x204);
    }

    #[test]
    fn test_set_vx() {
        let mut cpu = setup();
        let program = vec![0x60, 0x12];
        cpu.load(program.clone());
        cpu.cycle();
        assert!(cpu.v[0] == 0x12);
    }

    #[test]
    fn test_add_vx() {
        let mut cpu = setup();
        let program = vec![0x70, 0x12, 0x70, 0x12];
        cpu.load(program.clone());
        cpu.cycle();
        assert!(cpu.v[0] == 0x12);
        cpu.cycle();
        assert!(cpu.v[0] == 0x12 + 0x12);
    }

    #[test]
    fn test_set_vx_vy() {
        let mut cpu = setup();
        let program = vec![0x80, 0x10];
        cpu.load(program.clone());
        cpu.v[0] = 0x10;
        cpu.v[1] = 0x20;
        cpu.cycle();
        assert!(cpu.v[0] == 0x20);
    }

    #[test]
    fn test_store_vx() {
        let mut cpu = setup();
        let program = vec![0x61, 0x23];
        cpu.load(program.clone());

        cpu.cycle();
        assert!(cpu.v[1] == 0x23);
    }

    #[test]
    fn test_shift_vy_to_vx() {
        let mut cpu = setup();
        let program = vec![0x80, 0x1E];
        cpu.load(program.clone());
        cpu.v[1] = 0b00000100;
        cpu.cycle();
        assert!(cpu.v[0] == 0b00001000);
        assert!(cpu.v[0xF] == 0);
        assert!(cpu.v[1] == 0b00000100);

        cpu.initialize();
        cpu.load(program.clone());
        cpu.v[1] = 0b10000100;
        cpu.cycle();
        assert!(cpu.v[0] == 0b00001000);
        assert!(cpu.v[0xF] == 1);
        assert!(cpu.v[1] == 0b10000100);
    }

    #[test]
    fn test_keypress() {
        let mut cpu = setup();
        let program = vec![0xE0, 0x9E, 0x00, 0x00, 0xE0, 0xA1];
        cpu.load(program.clone());

        cpu.keys[0] = 1;
        cpu.cycle();
        assert!(cpu.pc == PROGRAM_START + 4, "got 0x{:X}", cpu.pc);
        cpu.keys[0] = 0;
        cpu.cycle();
        assert!(cpu.pc == PROGRAM_START + 8, "got 0x{:X}", cpu.pc);
    }

    #[test]
    fn test_get_font_address() {
        let mut cpu = setup();
        let program = vec![0xF0, 0x29];
        cpu.load(program.clone());
        cpu.v[0] = 0x3;
        cpu.cycle();
        assert!(cpu.i == 15);
    }
}

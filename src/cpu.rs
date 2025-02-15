use core::panic;
use rand;

use crate::display::Display;
use crate::fontset::FONTSET;

const PROGRAM_START: u16 = 0x200;
const MEMORY_SIZE: usize = 4096;

const FONTSET_START: usize = 0x50;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

const FRAMERATE: u32 = 120;
const CYCLES_PER_FRAME: u32 = 1000;

const SHIFT_GLITCH: bool = true;
const DRAW_GLITCH: bool = true;

/*
   Notes on Sprites:
   * One byte corresponds to one row of a sprite
   * 1 means white
   * All sprites are drawn using XOR mode
     * During drawing, the sprite data is XOR'd with the current screen
     * To erase a sprite, draw it again
*/

#[derive(Debug)]
pub struct CPU<D: Display> {
    display: D,
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
    pub gfx: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],

    // Stack
    pub stack: [u16; 16],
    pub sp: u16, // stack pointer

    // Keybuoard
    pub keys: [u8; 16],
}

impl<D: Display> CPU<D> {
    pub fn new(display: D) -> CPU<D> {
        CPU {
            display,
            opcode: 0,
            v: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            memory: [0; MEMORY_SIZE],
            i: 0,
            pc: 0,
            gfx: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            stack: [0; 16],
            sp: 0,
            keys: [0; 16],
        }
    }

    pub fn run(&mut self) {
        let frame_time = std::time::Duration::from_micros((1_000_000 / FRAMERATE).into());
        let cycle_time =
            std::time::Duration::from_micros((1_000_000 / (FRAMERATE * CYCLES_PER_FRAME)).into());
        // println!("Frame time: {}us", frame_time.as_micros());
        // println!("Cycle time: {}us", cycle_time.as_micros());
        let mut last_timer_update = std::time::Instant::now();

        let mut num_cycles = 0;
        while self.display.is_open() {
            let cycle_start = std::time::Instant::now();
            num_cycles += 1;

            // Update keys
            for i in 0..16 {
                self.keys[i] = if self.display.is_key_down(i) { 1 } else { 0 };
            }

            let time_since_frame = last_timer_update.elapsed();
            if !self.cycle(time_since_frame.as_micros(), frame_time.as_micros()) {
                break;
            }

            // Update timers at 60Hz
            if last_timer_update.elapsed() >= frame_time {
                // Update display
                self.display.update(&self.gfx, SCREEN_WIDTH, SCREEN_HEIGHT);
                println!("frame: {} cycles", num_cycles);
                num_cycles = 0;
                if self.delay_timer > 0 {
                    self.delay_timer -= 1;
                }
                if self.sound_timer > 0 {
                    self.sound_timer -= 1;
                }
                last_timer_update = std::time::Instant::now();
            }

            // Sleep for remaining cycle time
            if let Some(sleep_time) = cycle_time.checked_sub(cycle_start.elapsed()) {
                println!("Sleeping for {}us", sleep_time.as_micros());
                std::thread::sleep(sleep_time);
            }
        }
    }

    pub fn initialize(&mut self) {
        self.pc = PROGRAM_START;
        self.opcode = 0;
        self.i = 0;
        self.sp = 0;
        self.v = [0; 16];

        // Clear memory and stack
        self.memory = [0; MEMORY_SIZE];
        self.stack = [0; 16];

        // Clear timers
        self.delay_timer = 0;
        self.sound_timer = 0;

        // Clear display
        self.gfx = [0; SCREEN_WIDTH * SCREEN_HEIGHT];

        // Clear keys
        self.keys = [0; 16];

        // Load fontset
        for (i, &byte) in FONTSET.iter().enumerate() {
            self.memory[i + FONTSET_START] = byte;
        }
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

    pub fn cycle(&mut self, time_since_frame: u128, frame_time: u128) -> bool {
        self.opcode = (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[self.pc as usize + 1] as u16);
        let opcode = self.opcode;
        self.pc += 2;
        // println!("\npc: 0x{:x}, opcode: 0x{:x}", self.pc, opcode);

        let op_x = (opcode & 0x0F00) as usize >> 8;
        let op_y = (opcode & 0x00F0) as usize >> 4;
        match opcode & 0xF000 {
            0x0 => match opcode & 0x0FFF {
                0x0E0 => {
                    // Clear the screen
                    // println!("00E0");
                    self.gfx = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
                }
                // TODO: Stack pushing popping tests
                0x0EE => {
                    // println!("return");
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                }
                _ => {
                    println!("Unknown opcode: 0x{:x}", opcode);
                    return false;
                }
            },
            0x1000 => {
                // Jump to address NNN
                // println!("1NNN");
                self.pc = opcode & 0x0FFF;
            }
            0x2000 => {
                // Call subroutine at NNN
                // println!("2NNN");
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = opcode & 0x0FFF;
            }
            0x3000 => {
                // Skip next instruction if vX == NN
                // println!("3XNN");
                if self.v[op_x] == (opcode & 0x00FF) as u8 {
                    self.pc += 2;
                }
            }
            0x4000 => {
                // Skip next instruction if vX != NN
                // println!("4XNN");
                let nn = (opcode & 0x00FF) as u8;
                // println!("vx: {}, nn: {}", self.v[x], nn);
                if self.v[op_x] != nn {
                    self.pc += 2;
                }
            }
            0x5000 => {
                if opcode & 0x000F != 0 {
                    println!("Unknown opcode: 0x{:x}", opcode);
                    return false;
                }
                // Skip next instruction if VX == VY
                // println!("5XY0");
                if self.v[op_x] == self.v[op_y] {
                    self.pc += 2;
                }
            }
            0x6000 => {
                // Store NN in vX
                // println!("6XNN");
                self.v[op_x] = (opcode & 0x00FF) as u8;
            }
            0x7000 => {
                // Add NN to vX
                // println!("7XNN");
                let add = (opcode & 0x00FF) as u8;
                // println!("x: {}, add: {}", x, add);
                // Wrapping add to handle overflow correctly
                self.v[op_x] = self.v[op_x].wrapping_add(add);
            }
            0x8000 => match opcode & 0xF {
                0x0 => {
                    // Store VY in VX
                    // println!("8XY0");
                    self.v[op_x] = self.v[op_y];
                }
                0x1 => {
                    // store VY | VX in VX
                    // println!("8XY1");
                    self.v[op_x] |= self.v[op_y];
                    self.v[0xF] = 0;
                }
                0x2 => {
                    // store VY & VX in VX
                    // println!("8XY2");
                    self.v[op_x] &= self.v[op_y];
                    self.v[0xF] = 0;
                }
                0x3 => {
                    // store VY xor VX in VX
                    // println!("8XY3");
                    self.v[op_x] ^= self.v[op_y];
                    self.v[0xF] = 0;
                }
                0x4 => {
                    // Add VY to VX
                    // if carry occurred, set vf to 01 else vf to 00
                    // println!("8XY4");
                    let (result, overflow) = self.v[op_x].overflowing_add(self.v[op_y]);
                    self.v[op_x] = result;
                    self.v[0xF] = if overflow { 0x01 } else { 0x00 };
                }
                0x5 => {
                    // println!("8XY5");
                    // Sub VY from VX
                    // if borrow occurred, set vf to 00 else vf to 01
                    let (result, borrow) = self.v[op_x].overflowing_sub(self.v[op_y]);
                    self.v[op_x] = result;
                    self.v[0xF] = if borrow { 0x00 } else { 0x01 };
                }
                0x6 => {
                    // Store vy >> 1 in vx. Set vf to LSB of vy before shift
                    // println!("8XY6");
                    if SHIFT_GLITCH {
                        let bit = self.v[op_y] & 0x1;
                        self.v[op_x] = self.v[op_y] >> 1;
                        self.v[0xF] = bit;
                    } else {
                        let bit = self.v[op_x] & 0x1;
                        self.v[op_x] >>= 1;
                        self.v[0xF] = bit;
                    }
                }
                0x7 => {
                    // Set vX = Vy - Vx, set VF to !borrowed
                    // println!("8XY7");
                    let (result, borrow) = self.v[op_y].overflowing_sub(self.v[op_x]);
                    self.v[op_x] = result;
                    self.v[0xF] = if borrow { 0x00 } else { 0x01 };
                }
                0xE => {
                    // Store vy << 1 in vx. Set vf to most significant bit of vy before shift.
                    // println!("8XYE");
                    if SHIFT_GLITCH {
                        let bit = (self.v[op_y] & 0b10000000) >> 7;
                        self.v[op_x] = self.v[op_y] << 1;
                        self.v[0xF] = bit;
                    } else {
                        let bit = (self.v[op_x] & 0b10000000) >> 7;
                        self.v[op_x] <<= 1;
                        self.v[0xF] = bit;
                    }
                }
                _ => {
                    println!("Unknown opcode: 0x{:x}", opcode);
                    return false;
                }
            },
            0x9000 => {
                // Skip next instruction if vX != vY
                // println!("9XY0");
                if opcode & 0x000F != 0x0000 {
                    println!("Unknown opcode: 0x{:x}", opcode);
                    return false;
                }
                if self.v[op_x] != self.v[op_y] {
                    self.pc += 2;
                }
            }
            0xA000 => {
                // ANNN: Sets I to address NNN
                // println!("ANNN");
                self.i = opcode & 0x0FFF;
            }
            0xB000 => {
                // Jump to NNN + v0
                // println!("BNNN");
                self.pc = (opcode & 0x0FFF) + self.v[0] as u16;
            }
            0xC000 => {
                // Set vX to random number & NN
                // println!("CXNN");
                let nn = (opcode & 0x00FF) as u8;
                self.v[op_x] = rand::random::<u8>() & nn;
            }
            0xD000 => {
                // println!("DXYN");
                // Draw sprite DXYN
                // Notes:
                // * Draw at coordinates vX, vY
                // * Use N bytes starting at address I
                // * Set VF to 1 if any set pixels are changed to unset, else 0
                // * To be visible on the screen, the vX register must be
                //      between 00 and 3F. vY must be between 00 and 1F
                if DRAW_GLITCH {
                    if time_since_frame > (frame_time / 30) {
                        self.pc -= 2;
                        return true;
                    }
                }
                let mut x = self.v[op_x] as usize;
                let mut y = self.v[op_y] as usize;
                let n = (opcode & 0x000F) as usize;
                let addr = self.i as usize;
                if y >= SCREEN_HEIGHT {
                    y = y % SCREEN_HEIGHT;
                }
                if x >= SCREEN_WIDTH {
                    x = x % SCREEN_WIDTH;
                }
                // if y >= SCREEN_HEIGHT || x >= SCREEN_WIDTH {
                //     return true;
                // }
                let mut height = n;
                if y + height > SCREEN_HEIGHT {
                    height = SCREEN_HEIGHT - y;
                }
                let mut width = 8;
                if x + 8 > SCREEN_WIDTH {
                    width = SCREEN_WIDTH - x;
                }
                self.v[0xF] = 0;

                for row in 0..height {
                    let sprite = self.memory[addr + row];
                    for col in 0..width {
                        let pixel = sprite & (0x80 >> col);
                        let idx: usize = (x + col + ((y + row) * SCREEN_WIDTH)) as usize;
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
                    // println!("EX9E");
                    let val = self.v[op_x];
                    if self.keys[val as usize] == 1 {
                        self.pc += 2;
                    }
                }
                0xA1 => {
                    // println!("EXA1");
                    let val = self.v[op_x];
                    if self.keys[val as usize] == 0 {
                        self.pc += 2;
                    }
                }
                _ => {
                    println!("Unknown opcode: 0x{:x}", opcode);
                    return false;
                }
            },
            0xF000 => match opcode & 0x00FF {
                0x07 => {
                    // store delay timer in vX
                    // println!("FX07");
                    self.v[op_x] = self.delay_timer;
                }
                0x0A => {
                    // Wait for keypress and store in vX
                    // println!("FX0A");
                    let first_pressed_key = self.keys.iter().position(|&x| x == 1);
                    match first_pressed_key {
                        Some(key) => {
                            self.v[op_x] = key as u8;
                            self.pc += 2;
                        }
                        None => {
                            self.pc -= 2;
                        }
                    }
                }
                0x15 => {
                    // Set delay timer to vX
                    // println!("FX15");
                    self.delay_timer = self.v[op_x];
                }
                0x18 => {
                    // Set sound timer to vX
                    // println!("FX18");
                    self.sound_timer = self.v[op_x];
                }
                0x1E => {
                    // Add vX to I
                    // println!("FX1E");
                    self.i += self.v[op_x] as u16;
                }
                0x29 => {
                    // Set I to location of sprite for digit vX
                    // Notes:
                    // * vX should be between 0 and F
                    // * Fontset is between 0x050-0x0A0
                    // * Each character is 5 bytes long
                    // println!("FX29");
                    self.i = self.v[op_x] as u16 * 5 + FONTSET_START as u16;
                }
                0x33 => {
                    // Store binary-coded decimal representation of vX at I, I+1, I+2
                    // Notes:
                    // * Since each register is 8 bits, will have at most 3 decimal digits (0-255)
                    // * Store most significant digit at I, next at I+1, least significant at I+2
                    // println!("FX33");
                    let x = op_x;
                    let val = self.v[x];
                    self.memory[self.i as usize] = val / 100;
                    self.memory[(self.i + 1) as usize] = (val / 10) % 10;
                    self.memory[(self.i + 2) as usize] = val % 10;
                }
                0x55 => {
                    // Store v0 to vX in memory starting at I
                    // println!("FX55");
                    for i in 0..=op_x {
                        self.memory[(self.i + i as u16) as usize] = self.v[i];
                    }
                    self.i += (op_x + 1) as u16;
                }
                0x65 => {
                    // Load v0 to vX from memory starting at I
                    // println!("FX65");
                    for i in 0..=op_x {
                        self.v[i] = self.memory[(self.i + i as u16) as usize];
                    }
                    self.i += (op_x + 1) as u16;
                }
                _ => {
                    println!("Unknown opcode: 0x{:x}", opcode);
                    return false;
                }
            },

            _ => {
                println!("Unknown opcode: 0x{:x}", opcode);
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {

    use crate::display::NullDisplay;

    use super::*;
    use pretty_assertions::assert_eq;

    fn setup() -> CPU<NullDisplay> {
        let display = NullDisplay::new();
        let mut cpu = CPU::new(display);
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
        assert!(cpu.stack[0] == 0x202);
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
        assert!(cpu.i == (0x3 * 5) + FONTSET_START as u16);
    }

    #[test]
    fn test_initialize() {
        let mut cpu = setup();
        cpu.v[0] = 0x1;
        cpu.i = 0x2;
        cpu.sp = 0x3;
        cpu.stack[0] = 0x4;
        cpu.delay_timer = 0x5;
        cpu.sound_timer = 0x6;
        cpu.memory[0x200] = 0x7;
        cpu.gfx[0] = 0x8;
        cpu.keys[0] = 0x9;
        cpu.pc = 0xA;

        cpu.initialize();

        assert!(cpu.v.iter().all(|&x| x == 0));
        assert!(cpu.i == 0);
        assert!(cpu.sp == 0);
        assert!(cpu.stack.iter().all(|&x| x == 0));
        assert!(cpu.delay_timer == 0);
        assert!(cpu.sound_timer == 0);
        assert!(cpu.gfx.iter().all(|&x| x == 0));
        assert!(cpu.keys.iter().all(|&x| x == 0));
        assert!(cpu.pc == PROGRAM_START);

        // Check various fontset bytes
        assert!(cpu.memory[FONTSET_START as usize] == 0xF0);
        assert!(cpu.memory[FONTSET_START as usize + 5] == 0x20);
        assert!(cpu.memory[FONTSET_START as usize + 79] == 0x80);
    }

    #[test]
    fn test_8xye() {
        let mut cpu = setup();
        let program = vec![0x80, 0x1E];
        cpu.load(program.clone());
        cpu.v[1] = 0b00000100;
        cpu.v[0] = 0b00000000;
        cpu.cycle();
        assert!(cpu.v[0] == 0b00001000);
        assert!(cpu.v[1] == 0b00000100);
        assert!(cpu.v[0xF] == 0);

        cpu.initialize();
        cpu.load(program.clone());
        cpu.v[1] = 0b10000100;
        cpu.v[0] = 0b00000000;
        cpu.cycle();
        assert!(cpu.v[0] == 0b00001000);
        assert!(cpu.v[1] == 0b10000100);
        assert!(cpu.v[0xF] == 1);
    }

    #[test]
    fn test_draw_sprite() {
        let mut cpu = setup();
        // Test drawing sprite
        cpu.initialize();
        let program = vec![0xD0, 0x05, 0xD0, 0x05];
        cpu.load(program.clone());
        cpu.v[0] = 0x00;
        cpu.i = 0x50; // Sprite for 0
        cpu.cycle();
        assert!(cpu.gfx[0] == 1);
        assert!(cpu.gfx[1] == 1);
        assert!(cpu.gfx[2] == 1);
        assert!(cpu.gfx[3] == 1);
        assert!(cpu.gfx[0 + SCREEN_WIDTH] == 1);
        assert!(cpu.gfx[3 + SCREEN_WIDTH] == 1);
        assert!(cpu.gfx[0 + SCREEN_WIDTH * 2] == 1);
        assert!(cpu.gfx[3 + SCREEN_WIDTH * 2] == 1);
        assert!(cpu.gfx[0 + SCREEN_WIDTH * 3] == 1);
        assert!(cpu.gfx[3 + SCREEN_WIDTH * 3] == 1);
        assert!(cpu.gfx[0 + SCREEN_WIDTH * 4] == 1);
        assert!(cpu.gfx[1 + SCREEN_WIDTH * 4] == 1);
        assert!(cpu.gfx[2 + SCREEN_WIDTH * 4] == 1);
        assert!(cpu.gfx[3 + SCREEN_WIDTH * 4] == 1);
        assert!(cpu.v[0xF] == 0);
        cpu.cycle();
        assert!(cpu.gfx.iter().all(|&x| x == 0));
        assert!(cpu.v[0xF] == 1);

        // Test drawing sprite near right edge
        cpu.initialize();
        let program = vec![0xD0, 0x15];
        cpu.load(program.clone());
        cpu.v[0] = SCREEN_WIDTH as u8 - 4; // Just enough to fit sprite
        cpu.v[1] = 0x01;
        cpu.i = 0x50;
        cpu.cycle();
        assert!(cpu.gfx.iter().take(SCREEN_WIDTH).all(|&x| x == 0));
        assert!(cpu.gfx[SCREEN_WIDTH - 4 + SCREEN_WIDTH] == 1);
        assert!(cpu.gfx[SCREEN_WIDTH - 3 + SCREEN_WIDTH] == 1);
        assert!(cpu.gfx[SCREEN_WIDTH - 2 + SCREEN_WIDTH] == 1);
        assert!(cpu.gfx[SCREEN_WIDTH - 1 + SCREEN_WIDTH] == 1);
        assert!(cpu.v[0xF] == 0);

        // Test drawing sprite near top
        cpu.initialize();
        let program = vec![0xD0, 0x15];
        cpu.load(program.clone());
        cpu.v[0] = 0x00;
        cpu.v[1] = SCREEN_HEIGHT as u8 - 1;
        cpu.i = 0x50;
        cpu.cycle();

        assert!(cpu
            .gfx
            .iter()
            .take(SCREEN_WIDTH * (SCREEN_HEIGHT - 1))
            .all(|&x| x == 0));
        assert!(cpu.gfx[0 + SCREEN_WIDTH * (SCREEN_HEIGHT - 1)] == 1);
        assert!(cpu.gfx[1 + SCREEN_WIDTH * (SCREEN_HEIGHT - 1)] == 1);
        assert!(cpu.gfx[2 + SCREEN_WIDTH * (SCREEN_HEIGHT - 1)] == 1);
        assert!(cpu.gfx[3 + SCREEN_WIDTH * (SCREEN_HEIGHT - 1)] == 1);
        assert!(cpu.gfx[4 + SCREEN_WIDTH * (SCREEN_HEIGHT - 1)] == 0);
        assert!(cpu.v[0xF] == 0);
    }
}

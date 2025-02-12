use core::panic;

const PROGRAM_START: u16 = 0x200;
const MEMORY_SIZE: usize = 4096;

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
        self.opcode = (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[self.pc as usize + 1] as u16);

        match self.opcode & 0xF000 {
            0x000 => match self.opcode & 0x00FF {
                0xE0 => {
                    println!("clear");
                }
                0xEE => {
                    println!("return");
                }
                _ => {
                    println!("Unknown opcode: 0x{:x}", self.opcode);
                }
            },
            0x1000 => {
                println!("1NNN")
            }
            0x2000 => {
                println!("2NNN")
            }
            0x3000 => {
                println!("3XNN")
            }
            0x4000 => {
                println!("4XNN")
            }
            0x5000 => {
                println!("5XY0")
            }
            0x6000 => {
                println!("6XNN")
            }
            0x7000 => {
                println!("7XNN")
            }
            0x8000 => match self.opcode & 0xF {
                0x0 => {
                    println!("8XY0")
                }
                0x1 => {
                    println!("8XY1")
                }
                0x2 => {
                    println!("8XY2")
                }
                0x3 => {
                    println!("8XY3")
                }
                0x4 => {
                    println!("8XY4")
                }
                0x5 => {
                    println!("8XY5")
                }
                0x6 => {
                    println!("8XY6")
                }
                0x7 => {
                    println!("8XY7")
                }
                0xE => {
                    println!("8XYE")
                }
                _ => {
                    println!("Unknown opcode: 0x{:x}", self.opcode);
                }
            },
            0x9000 => {
                println!("9XY0")
            }
            0xA000 => {
                // ANNN: Sets I to address NNN
                println!("ANNN")
            }
            0xB000 => {
                println!("BNNN")
            }
            0xC000 => {
                println!("CXNN")
            }
            0xD000 => {
                println!("DXYN")
            }
            0xE000 => match self.opcode & 0x00FF {
                0x9E => {
                    println!("EX9E")
                }
                0xA1 => {
                    println!("EXA1")
                }
                _ => {
                    println!("Unknown opcode: 0x{:x}", self.opcode);
                }
            },
            0xF000 => match self.opcode & 0x00FF {
                0x07 => {
                    println!("FX07")
                }
                0x0A => {
                    println!("FX0A")
                }
                0x15 => {
                    println!("FX15")
                }
                0x18 => {
                    println!("FX18")
                }
                0x1E => {
                    println!("FX1E")
                }
                0x29 => {
                    println!("FX29")
                }
                0x33 => {
                    println!("FX33")
                }
                0x55 => {
                    println!("FX55")
                }
                0x65 => {
                    println!("FX65")
                }
                _ => {
                    println!("Unknown opcode: 0x{:x}", self.opcode);
                }
            },

            _ => {
                println!("Unknown opcode: 0x{:x}", self.opcode);
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
        // Load CLS instruction (0x00E0)
        cpu.load(vec![0x00, 0xE0]);
        
        // Set some pixels to 1
        cpu.gfx[0] = 1;
        cpu.gfx[100] = 1;
        
        cpu.cycle();
        
        // Verify all pixels are cleared
        assert!(cpu.gfx.iter().all(|&pixel| pixel == 0));
        assert_eq!(cpu.pc, PROGRAM_START + 2);
    }
}

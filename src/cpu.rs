use core::panic;

const PROGRAM_START: u16 = 0x200;
const MEMORY_SIZE: usize = 4096;

#[derive(Debug)]
pub struct CPU {
    opcode: u16,

    // Registers
    v: [u8; 16],

    // Timers
    // Both count down at 60 hz
    delay_timer: u8, // read/write
    sound_timer: u8, // write only

    // Memory
    memory: [u8; MEMORY_SIZE],

    i: u16,  // index register. Max = 0xfff
    pc: u16, // program counter. Max = 0xfff

    // Display
    // 2048 pixels total, binary state black or white
    gfx: [u8; 64 * 32],

    // Stack
    stack: [u16; 16],
    sp: u16, // stack pointer

    // Keybuoard
    keys: [u8; 16],
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

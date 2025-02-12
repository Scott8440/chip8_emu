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
            0xA000 => {
                // ANNN: Sets I to address NNN
                println!("ANNN")
            }
            _ => {
                println!("Unknown opcode: 0x{:x}", self.opcode);
            }
        }
    }
}

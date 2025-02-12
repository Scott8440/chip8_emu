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
    memory: [u8; 4096],

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
            memory: [0; 4096],
            i: 0,
            pc: 0,
            gfx: [0; 64 * 32],
            stack: [0; 16],
            sp: 0,
            keys: [0; 16],
        }
    }
}

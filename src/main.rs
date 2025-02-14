mod cpu;
mod display;
mod fontset;

use display::MiniFBDisplay;
use minifb::{Window, WindowOptions};
use std::env;
use std::fs::File;
use std::io::Read;

fn get_window() -> Window {
    let scale = 10;
    let width = 64 * scale;
    let height = 32 * scale;

    Window::new(
        "CHIP-8 Emulator",
        width,
        height,
        WindowOptions {
            resize: false,
            scale: minifb::Scale::X1,
            ..WindowOptions::default()
        },
    )
    .unwrap()
}

fn load_rom(filename: &str) -> Vec<u8> {
    let mut file = File::open(filename).expect("Failed to open ROM file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .expect("Failed to read ROM file");
    buffer
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <rom_file>", args[0]);
        std::process::exit(1);
    }

    let window = get_window();
    let display = MiniFBDisplay::new(window);

    // Create a program to display each fontset character:
    // 1. Set V0 to 0 (first character)
    // 2. Get font location for V0
    // 3. Draw sprite at (10,10) with height 5
    // 4. Wait for 30 frames (~0.5 seconds)
    // 5. Clear screen
    // 6. Increment V0
    // 7. If V0 == 16, end program
    // 8. Jump back to step 2
    let program: Vec<u8> = vec![
        0x60, 0x00, // Set V0 = 0 (current character)
        0x61, 0x0A, // Set V1 = 10 (x position)
        0x62, 0x0A, // Set V2 = 10 (y position)
        0xF0, 0x29, // Set I to font address for V0
        0xD1, 0x25, // Draw sprite at (V1,V2) with height 5
        0x64, 0x1E, // Set V4 = 30 (delay timer value)
        0xF4, 0x15, // Set delay timer to V4 (30/60 = 0.5 seconds)
        0xF5, 0x07, // Get delay timer value into V5
        0x35, 0x00, // Skip if V5 != 0 (wait until timer expires)
        0x12, 0x0E, // Jump back to check timer
        0x00, 0xE0, // Clear screen
        0x65, 0x00, // Reset V5 to 0 for next character
        0x70, 0x01, // Add 1 to V0 (next character)
        0x40, 0x10, // Skip if V0 == 16
        0x60, 0x00, // reset character to 0
        0x12, 0x04, // Jump back to font loading (F029)
    ];

    let mut cpu = cpu::CPU::new(display);
    cpu.initialize();
    cpu.load(program);
    cpu.run();
}

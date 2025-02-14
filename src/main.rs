mod cpu;
mod display;
mod fontset;

use display::MiniFBDisplay;
use minifb::{Window, WindowOptions};

fn get_window() -> Window {
    let width = 100;
    let height = 100;

    Window::new(
        "Test",
        width,
        height,
        WindowOptions {
            resize: true,
            scale: minifb::Scale::X8,
            ..WindowOptions::default()
        },
    )
    .unwrap()
}

fn main() {
    let window = get_window();
    let display = MiniFBDisplay::new(window);

    let mut program: Vec<u8> = vec![
        0x60, 0x0F, // Set V0 = 2
        0x62, 0x02, // Set V0 = 0
        0x61, 0x00, // v1 is x coordinate, set to 0
        0xF0, 0x29, // Load sprite for digit 0 into I
        0xD1, 0x25, // Draw sprite at (v1, v2) with height 5
        // 0x60, 0x03, // Set V0 = 3
        // 0xF0, 0x29, // Load sprite for digit 0 into I
        0x71, 0x05, // v1 is x coordinate, Add 5
        0xD1, 0x25, // Draw sprite at (v1, v2) with height 5
        // 0x00, 0xE0, // Clear screen
        0x12, 0x0A, // Jump to start
    ];

    let mut cpu = cpu::CPU::new(display);
    cpu.initialize();
    cpu.load(program);
    cpu.run();
}

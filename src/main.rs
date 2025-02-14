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
    
    // Create an infinite loop program:
    // 0x200: 0x6001 - Set V0 to 1
    // 0x202: 0x7001 - Add 1 to V0
    // 0x204: 0x1202 - Jump back to 0x202
    let mut program: Vec<u8> = vec![
        0x60, 0x01,  // Set V0 = 1
        0x70, 0x01,  // Add 1 to V0
        0x12, 0x02   // Jump to 0x202
    ];

    let mut cpu = cpu::CPU::new(display);
    cpu.initialize();
    cpu.load(program);
    cpu.run();
}

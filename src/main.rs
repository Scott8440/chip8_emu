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
    
    let mut test_memory: Vec<u8> = Vec::new();
    test_memory.push(0xA2);
    test_memory.push(0xF0);

    let mut cpu = cpu::CPU::new(display);
    cpu.initialize();
    cpu.load(test_memory);
    cpu.run();
}

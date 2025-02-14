mod cpu;
mod fontset;

use minifb::{Key, Window, WindowOptions};

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
    let mut window = get_window();
    let x = 50;
    let y = 50;
    let mut buffer: Vec<u32> = vec![0; 100 * 100];
    buffer[y * 100 + x] = 0xFFFFFFFF; // white pixel
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, 100, 100).unwrap();
    }

    let mut test_memory: Vec<u8> = Vec::new();
    test_memory.push(0xA2);
    test_memory.push(0xF0);

    let mut cpu = cpu::CPU::new();
    cpu.initialize();
    cpu.load(test_memory);
    cpu.cycle();
}

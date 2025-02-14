mod cpu;
mod display;
mod fontset;

use std::env;
use std::fs::File;
use std::io::Read;
use display::MiniFBDisplay;
use minifb::{Window, WindowOptions};

fn get_window() -> Window {
    let width = 64;
    let height = 32;

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

fn load_rom(filename: &str) -> Vec<u8> {
    let mut file = File::open(filename).expect("Failed to open ROM file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Failed to read ROM file");
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
    
    let program = load_rom(&args[1]);

    let mut cpu = cpu::CPU::new(display);
    cpu.initialize();
    cpu.load(program);
    cpu.run();
}

mod cpu;
mod display;
mod fontset;
mod rom_loader;

use crate::rom_loader::Ch8RomLoader;
use crate::rom_loader::HexRomLoader;
use crate::rom_loader::RomLoader;

use display::MiniFBDisplay;
use minifb::{Window, WindowOptions};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <rom_file>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let program;
    if filename.ends_with(".hex") {
        program = HexRomLoader::read(Path::new(filename));
    } else if filename.ends_with(".ch8") || filename.ends_with(".8o") {
        program = Ch8RomLoader::read(Path::new(filename));
    } else {
        eprintln!("Unsupported file type: {}", filename);
        return;
    }
    let window = get_window();
    let display = MiniFBDisplay::new(window);

    let mut cpu = cpu::CPU::new(display);
    cpu.initialize();
    cpu.load(program);
    cpu.run();
}

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub trait RomLoader {
    fn read(file: &Path) -> Vec<u8>;
}

// Rom in format of hex strings, 2 per line
pub struct HexRomLoader;

impl RomLoader for HexRomLoader {
    fn read(file: &Path) -> Vec<u8> {
        let mut buffer = Vec::<u8>::new();
        let mut f = File::open(file).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        for line in s.lines() {
            let mut iter = line.split_whitespace();
            let a = u8::from_str_radix(iter.next().unwrap(), 16).unwrap();
            let b = u8::from_str_radix(iter.next().unwrap(), 16).unwrap();
            buffer.push(a);
            buffer.push(b);
        }
        buffer
    }
}

// Reads .ch8 files which are just the raw bytes
pub struct Ch8RomLoader;

impl RomLoader for Ch8RomLoader {
    fn read(file: &Path) -> Vec<u8> {
        let mut buffer = Vec::<u8>::new();
        let mut f = File::open(file).unwrap();
        f.read_to_end(&mut buffer).unwrap();
        buffer
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_hex_loader() {
        let expected_program = vec![
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
        let read_program = HexRomLoader::read(Path::new("src/programs/font_cycle.hex"));

        assert_eq!(expected_program, read_program);
    }

}

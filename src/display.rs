use minifb::Window;

pub trait Display {
    fn update(&mut self, buffer: &[u8], width: usize, height: usize);
    fn is_open(&self) -> bool;
}

pub struct MiniFBDisplay {
    window: Window,
    buffer: Vec<u32>,
}

impl MiniFBDisplay {
    pub fn new(window: Window) -> Self {
        MiniFBDisplay {
            window,
            buffer: vec![0; 64 * 32], // CHIP-8 display size
        }
    }
}

pub struct NullDisplay;

impl NullDisplay {
    pub fn new() -> Self {
        NullDisplay
    }
}

impl Display for NullDisplay {
    fn update(&mut self, _buffer: &[u8], _width: usize, _height: usize) {}
    fn is_open(&self) -> bool {
        true
    }
}

impl Display for MiniFBDisplay {
    fn update(&mut self, gfx: &[u8], width: usize, height: usize) {
        // Convert CHIP-8 display buffer (0,1) to minifb buffer (0x0, 0xFFFFFFFF)
        for (i, &pixel) in gfx.iter().enumerate() {
            self.buffer[i] = if pixel == 0 { 0 } else { 0xFFFFFFFF };
        }
        self.window
            .update_with_buffer(&self.buffer, width, height)
            .unwrap();
    }

    fn is_open(&self) -> bool {
        self.window.is_open()
    }
}

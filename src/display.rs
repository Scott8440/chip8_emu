use minifb::Window;

pub trait Display {
    fn update(&mut self, buffer: &[u8], width: usize, height: usize);
    fn is_open(&self) -> bool;
    fn is_key_down(&self, key: usize) -> bool;
}

pub struct MiniFBDisplay {
    window: Window,
    buffer: Vec<u32>,
}

impl MiniFBDisplay {
    pub fn new(window: Window) -> Self {
        let size = window.get_size();
        MiniFBDisplay {
            window,
            buffer: vec![0; size.0 * size.1],
        }
    }
}

impl Display for MiniFBDisplay {
    fn update(&mut self, gfx: &[u8], width: usize, height: usize) {
        // Convert CHIP-8 display buffer (0,1) to minifb buffer (0x0, 0xFFFFFFFF)
        let window_size = self.window.get_size();
        let scale_x = window_size.0 / width;
        let scale_y = window_size.1 / height;

        for y in 0..height {
            for x in 0..width {
                let pixel = gfx[y * width + x];
                let color = if pixel == 0 { 0 } else { 0xFFFFFFFF };

                // Scale up the pixel
                for sy in 0..scale_y {
                    for sx in 0..scale_x {
                        let buf_x = x * scale_x + sx;
                        let buf_y = y * scale_y + sy;
                        self.buffer[buf_y * window_size.0 + buf_x] = color;
                    }
                }
            }
        }

        self.window
            .update_with_buffer(&self.buffer, window_size.0, window_size.1)
            .unwrap();
    }

    fn is_open(&self) -> bool {
        self.window.is_open()
    }

    fn is_key_down(&self, key: usize) -> bool {
        use minifb::Key;
        let keys = [
            Key::X,
            Key::Key1,
            Key::Key2,
            Key::Key3,
            Key::Q,
            Key::W,
            Key::E,
            Key::A,
            Key::S,
            Key::D,
            Key::Z,
            Key::C,
            Key::Key4,
            Key::R,
            Key::F,
            Key::V,
        ];
        if key < keys.len() {
            self.window.is_key_down(keys[key])
        } else {
            false
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
    fn is_key_down(&self, key: usize) -> bool {
        false
    }
}

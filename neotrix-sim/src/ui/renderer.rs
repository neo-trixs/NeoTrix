pub struct Renderer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        Renderer {
            width,
            height,
            pixels: vec![0xFF000000; (width * height) as usize],
        }
    }

    pub fn clear(&mut self, color: u32) {
        self.pixels.fill(color);
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
            self.pixels[(y as u32 * self.width + x as u32) as usize] = color;
        }
    }

    pub fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: u32) {
        for dy in 0..h as i32 {
            for dx in 0..w as i32 {
                self.set_pixel(x + dx, y + dy, color);
            }
        }
    }

    pub fn draw_circle(&mut self, cx: i32, cy: i32, r: i32, color: u32) {
        for y in -r..=r {
            for x in -r..=r {
                if x * x + y * y <= r * r {
                    self.set_pixel(cx + x, cy + y, color);
                }
            }
        }
    }

    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;
        loop {
            self.set_pixel(x, y, color);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn present(&self) -> &[u32] {
        &self.pixels
    }
}

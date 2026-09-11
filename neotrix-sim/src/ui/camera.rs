pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
    pub screen_width: f32,
    pub screen_height: f32,
}

impl Camera {
    pub fn new(sw: f32, sh: f32) -> Self {
        Camera {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
            screen_width: sw,
            screen_height: sh,
        }
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.x += dx / self.zoom;
        self.y += dy / self.zoom;
    }

    pub fn zoom(&mut self, factor: f32) {
        self.zoom = (self.zoom * factor).clamp(0.1, 10.0);
    }

    pub fn center_on(&mut self, wx: f32, wy: f32) {
        self.x = wx - self.screen_width / (2.0 * self.zoom);
        self.y = wy - self.screen_height / (2.0 * self.zoom);
    }

    pub fn screen_to_world(&self, sx: f32, sy: f32) -> (f32, f32) {
        (sx / self.zoom + self.x, sy / self.zoom + self.y)
    }

    pub fn world_to_screen(&self, wx: f32, wy: f32) -> (f32, f32) {
        ((wx - self.x) * self.zoom, (wy - self.y) * self.zoom)
    }
}

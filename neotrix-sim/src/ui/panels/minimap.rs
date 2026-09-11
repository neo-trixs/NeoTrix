pub struct MinimapPanel {
    pub x: f32,
    pub y: f32,
    pub size: f32,
}

impl MinimapPanel {
    pub fn new(x: f32, y: f32, size: f32) -> Self {
        MinimapPanel { x, y, size }
    }

    pub fn world_to_minimap(&self, wx: f32, wy: f32, world_size: f32) -> (f32, f32) {
        (
            self.x + wx / world_size * self.size,
            self.y + wy / world_size * self.size,
        )
    }
}

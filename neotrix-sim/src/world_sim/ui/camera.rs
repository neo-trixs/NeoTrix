use crate::world_sim::config::WorldSimConfig;

pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub zoom: f32,
    pub rotation: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub target_zoom: f32,
    pub smoothing: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Camera {
    pub fn new(config: &WorldSimConfig) -> Self {
        Self {
            x: config.world_width as f32 / 2.0,
            y: config.world_height as f32 / 2.0,
            zoom: 1.0,
            rotation: 0.0,
            target_x: config.world_width as f32 / 2.0,
            target_y: config.world_height as f32 / 2.0,
            target_zoom: 1.0,
            smoothing: 0.1,
            min_zoom: 0.2,
            max_zoom: 5.0,
        }
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.target_x += dx / self.zoom;
        self.target_y += dy / self.zoom;
    }

    pub fn zoom_in(&mut self) {
        self.target_zoom = (self.target_zoom * 1.2).min(self.max_zoom);
    }

    pub fn zoom_out(&mut self) {
        self.target_zoom = (self.target_zoom / 1.2).max(self.min_zoom);
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.target_zoom = zoom.clamp(self.min_zoom, self.max_zoom);
    }

    pub fn center_on(&mut self, x: f32, y: f32) {
        self.target_x = x;
        self.target_y = y;
    }

    pub fn update(&mut self, dt: f32) {
        let t = self.smoothing * dt * 60.0;
        self.x += (self.target_x - self.x) * t;
        self.y += (self.target_y - self.y) * t;
        self.zoom += (self.target_zoom - self.zoom) * t;
    }

    pub fn screen_to_world(&self, screen_x: f32, screen_y: f32, screen_width: f32, screen_height: f32) -> (f32, f32) {
        let world_x = (screen_x - screen_width / 2.0) / self.zoom + self.x;
        let world_y = (screen_y - screen_height / 2.0) / self.zoom + self.y;
        (world_x, world_y)
    }

    pub fn world_to_screen(&self, world_x: f32, world_y: f32, screen_width: f32, screen_height: f32) -> (f32, f32) {
        let screen_x = (world_x - self.x) * self.zoom + screen_width / 2.0;
        let screen_y = (world_y - self.y) * self.zoom + screen_height / 2.0;
        (screen_x, screen_y)
    }

    pub fn get_view_bounds(&self, screen_width: f32, screen_height: f32) -> (f32, f32, f32, f32) {
        let half_w = screen_width / 2.0 / self.zoom;
        let half_h = screen_height / 2.0 / self.zoom;
        (
            self.x - half_w,
            self.y - half_h,
            self.x + half_w,
            self.y + half_h,
        )
    }

    pub fn is_visible(&self, x: f32, y: f32, screen_width: f32, screen_height: f32) -> bool {
        let (left, top, right, bottom) = self.get_view_bounds(screen_width, screen_height);
        x >= left && x <= right && y >= top && y <= bottom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_creation() {
        let config = WorldSimConfig::default();
        let camera = Camera::new(&config);
        assert_eq!(camera.x, config.world_width as f32 / 2.0);
        assert_eq!(camera.zoom, 1.0);
    }

    #[test]
    fn test_camera_pan() {
        let config = WorldSimConfig::default();
        let mut camera = Camera::new(&config);
        camera.pan(10.0, 5.0);
        assert!(camera.target_x > 0.0);
    }

    #[test]
    fn test_camera_zoom() {
        let config = WorldSimConfig::default();
        let mut camera = Camera::new(&config);
        camera.zoom_in();
        assert!(camera.target_zoom > 1.0);
        let after_zoom_in = camera.target_zoom;
        camera.zoom_out();
        assert!(camera.target_zoom < after_zoom_in);
        assert!(camera.target_zoom >= camera.min_zoom);
    }

    #[test]
    fn test_camera_screen_to_world() {
        let config = WorldSimConfig::default();
        let camera = Camera::new(&config);
        let (wx, wy) = camera.screen_to_world(400.0, 300.0, 800.0, 600.0);
        assert!(wx > 0.0);
        assert!(wy > 0.0);
    }
}

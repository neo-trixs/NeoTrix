use crate::core::{Vec2, Rect};

/// Axis-aligned bounding box for camera clamping.
pub struct CameraBounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl CameraBounds {
    pub fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self { min_x, min_y, max_x, max_y }
    }

    pub fn from_rect(r: &Rect) -> Self {
        Self::new(r.x, r.y, r.x + r.width, r.y + r.height)
    }
}

/// Full-featured 2D camera with follow, shake, fade, and bounds.
pub struct Camera2D {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub follow_smoothing: f32,
    pub bounds: Option<CameraBounds>,
    pub shake_intensity: f32,
    pub shake_decay: f32,
    pub shake_offset: Vec2,
    pub screen_fade: f32,
    pub fade_speed: f32,
}

impl Camera2D {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            position: Vec2::zero(),
            zoom: 1.0,
            rotation: 0.0,
            viewport_width,
            viewport_height,
            follow_smoothing: 0.1,
            bounds: None,
            shake_intensity: 0.0,
            shake_decay: 8.0,
            shake_offset: Vec2::zero(),
            screen_fade: 0.0,
            fade_speed: 2.0,
        }
    }

    /// Smoothly follow a target position (call once per frame).
    pub fn follow(&mut self, target_pos: Vec2) {
        let desired_x = target_pos.x - self.viewport_width / (2.0 * self.zoom);
        let desired_y = target_pos.y - self.viewport_height / (2.0 * self.zoom);
        self.position.x += (desired_x - self.position.x) * self.follow_smoothing;
        self.position.y += (desired_y - self.position.y) * self.follow_smoothing;
        self.clamp_to_bounds();
    }

    /// Instantly snap to a target position (no smoothing).
    pub fn snap_to(&mut self, target_pos: Vec2) {
        self.position.x = target_pos.x - self.viewport_width / (2.0 * self.zoom);
        self.position.y = target_pos.y - self.viewport_height / (2.0 * self.zoom);
        self.clamp_to_bounds();
    }

    pub fn clamp_to_bounds(&mut self) {
        if let Some(ref bounds) = self.bounds {
            let vw = self.viewport_width / self.zoom;
            let vh = self.viewport_height / self.zoom;
            self.position.x = self.position.x.clamp(bounds.min_x, (bounds.max_x - vw).max(bounds.min_x));
            self.position.y = self.position.y.clamp(bounds.min_y, (bounds.max_y - vh).max(bounds.min_y));
        }
    }

    /// Trigger screen shake.
    pub fn shake(&mut self, intensity: f32) {
        self.shake_intensity = intensity;
    }

    /// Update shake offset (call once per frame). Returns the current offset.
    pub fn update_shake(&mut self, dt: f32) -> Vec2 {
        if self.shake_intensity > 0.1 {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos();
            let rx = ((nanos % 10007) as f32 / 10007.0) * 2.0 - 1.0;
            let ry = (((nanos / 1000) % 10007) as f32 / 10007.0) * 2.0 - 1.0;
            self.shake_offset = Vec2::new(rx * self.shake_intensity, ry * self.shake_intensity);
            self.shake_intensity *= (-self.shake_decay * dt).exp();
            self.shake_offset
        } else {
            self.shake_intensity = 0.0;
            self.shake_offset = Vec2::zero();
            self.shake_offset
        }
    }

    /// Convert screen coordinates to world coordinates.
    pub fn screen_to_world(&self, screen_x: f32, screen_y: f32) -> Vec2 {
        Vec2::new(
            (screen_x - self.viewport_width / 2.0) / self.zoom + self.position.x,
            (screen_y - self.viewport_height / 2.0) / self.zoom + self.position.y,
        )
    }

    /// Convert world coordinates to screen coordinates.
    pub fn world_to_screen(&self, world_x: f32, world_y: f32) -> Vec2 {
        Vec2::new(
            (world_x - self.position.x) * self.zoom + self.viewport_width / 2.0,
            (world_y - self.position.y) * self.zoom + self.viewport_height / 2.0,
        )
    }

    /// The axis-aligned rectangle of world space visible on screen.
    pub fn visible_world_rect(&self) -> Rect {
        let hw = self.viewport_width / (2.0 * self.zoom);
        let hh = self.viewport_height / (2.0 * self.zoom);
        Rect::new(self.position.x - hw, self.position.y - hh, hw * 2.0, hh * 2.0)
    }

    /// Check if a world-space rectangle is at least partially visible.
    pub fn is_visible(&self, rect: &Rect) -> bool {
        self.visible_world_rect().intersects(rect)
    }

    /// Start fading to black. Returns true when fully black.
    pub fn fade_to_black(&mut self, dt: f32) -> bool {
        self.screen_fade = (self.screen_fade + dt * self.fade_speed).min(1.0);
        self.screen_fade >= 1.0
    }

    /// Start fading from black. Returns true when fully clear.
    pub fn fade_from_black(&mut self, dt: f32) -> bool {
        self.screen_fade = (self.screen_fade - dt * self.fade_speed).max(0.0);
        self.screen_fade <= 0.0
    }

    /// Set fade directly (0.0 = clear, 1.0 = black).
    pub fn set_fade(&mut self, alpha: f32) {
        self.screen_fade = alpha.clamp(0.0, 1.0);
    }

    /// Get the effective position including shake offset.
    pub fn effective_position(&self) -> Vec2 {
        self.position + self.shake_offset
    }
}

impl Default for Camera2D {
    fn default() -> Self { Self::new(800.0, 600.0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_camera() {
        let cam = Camera2D::new(800.0, 600.0);
        assert_eq!(cam.zoom, 1.0);
        assert_eq!(cam.position, Vec2::zero());
    }

    #[test]
    fn test_screen_to_world_roundtrip() {
        let cam = Camera2D::new(800.0, 600.0);
        let world = Vec2::new(100.0, 200.0);
        let screen = cam.world_to_screen(world.x, world.y);
        let back = cam.screen_to_world(screen.x, screen.y);
        assert!((back.x - world.x).abs() < 0.01);
        assert!((back.y - world.y).abs() < 0.01);
    }

    #[test]
    fn test_follow() {
        let mut cam = Camera2D::new(800.0, 600.0);
        cam.follow_smoothing = 1.0; // instant
        cam.follow(Vec2::new(100.0, 200.0));
        // follow centers target: pos = target - viewport/2
        assert!((cam.position.x - (100.0 - 400.0)).abs() < 0.01);
        assert!((cam.position.y - (200.0 - 300.0)).abs() < 0.01);
    }

    #[test]
    fn test_bounds_clamp() {
        let mut cam = Camera2D::new(100.0, 100.0);
        cam.bounds = Some(CameraBounds::new(0.0, 0.0, 200.0, 200.0));
        cam.position = Vec2::new(300.0, 300.0);
        cam.clamp_to_bounds();
        assert!(cam.position.x <= 200.0);
        assert!(cam.position.y <= 200.0);
    }

    #[test]
    fn test_visible_rect() {
        let cam = Camera2D::new(800.0, 600.0);
        let r = cam.visible_world_rect();
        assert_eq!(r.width, 800.0);
        assert_eq!(r.height, 600.0);
    }

    #[test]
    fn test_is_visible() {
        let cam = Camera2D::new(100.0, 100.0);
        let inside = Rect::new(10.0, 10.0, 20.0, 20.0);
        let outside = Rect::new(500.0, 500.0, 10.0, 10.0);
        assert!(cam.is_visible(&inside));
        assert!(!cam.is_visible(&outside));
    }

    #[test]
    fn test_fade() {
        let mut cam = Camera2D::new(800.0, 600.0);
        cam.fade_speed = 1.0;
        assert!(!cam.fade_to_black(0.5));  // 0.5
        assert!(cam.fade_to_black(0.5));   // 1.0 — done
        assert!(!cam.fade_from_black(0.5)); // 0.5
        assert!(!cam.fade_from_black(0.4)); // 0.1
        assert!(cam.fade_from_black(0.2));  // 0.0 — done
    }

    #[test]
    fn test_shake() {
        let mut cam = Camera2D::new(800.0, 600.0);
        cam.shake(10.0);
        assert!(cam.shake_intensity > 0.0);
        let _ = cam.update_shake(0.016);
        assert!(cam.shake_intensity < 10.0);
    }

    #[test]
    fn test_snap_to() {
        let mut cam = Camera2D::new(100.0, 100.0);
        cam.snap_to(Vec2::new(50.0, 50.0));
        // snap_to centers target: pos = target - viewport/2
        assert!((cam.position.x - 0.0).abs() < 0.01);
        assert!((cam.position.y - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_effective_position() {
        let mut cam = Camera2D::new(800.0, 600.0);
        cam.position = Vec2::new(100.0, 100.0);
        cam.shake_offset = Vec2::new(5.0, -3.0);
        let ep = cam.effective_position();
        assert!((ep.x - 105.0).abs() < 0.01);
        assert!((ep.y - 97.0).abs() < 0.01);
    }
}

pub struct Sprite {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub width: u32,
    pub height: u32,
    pub color: u32,
    pub scale: f32,
    pub rotation: f32,
    pub visible: bool,
    pub layer: u32,
}

impl Sprite {
    pub fn new(id: &str, x: f32, y: f32, width: u32, height: u32, color: u32) -> Self {
        Self {
            id: id.to_string(),
            x,
            y,
            width,
            height,
            color,
            scale: 1.0,
            rotation: 0.0,
            visible: true,
            layer: 0,
        }
    }

    pub fn scaled_width(&self) -> f32 {
        self.width as f32 * self.scale
    }

    pub fn scaled_height(&self) -> f32 {
        self.height as f32 * self.scale
    }

    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        let sw = self.scaled_width();
        let sh = self.scaled_height();
        px >= self.x && px <= self.x + sw && py >= self.y && py <= self.y + sh
    }
}

pub struct SpriteBatch {
    sprites: Vec<Sprite>,
    dirty: bool,
}

impl SpriteBatch {
    pub fn new() -> Self {
        Self {
            sprites: Vec::new(),
            dirty: false,
        }
    }

    pub fn add(&mut self, sprite: Sprite) {
        self.sprites.push(sprite);
        self.dirty = true;
    }

    pub fn remove(&mut self, id: &str) {
        self.sprites.retain(|s| s.id != id);
        self.dirty = true;
    }

    pub fn update(&mut self, id: &str, f: impl FnOnce(&mut Sprite)) {
        if let Some(sprite) = self.sprites.iter_mut().find(|s| s.id == id) {
            f(sprite);
            self.dirty = true;
        }
    }

    pub fn get_visible(&self) -> Vec<&Sprite> {
        self.sprites.iter().filter(|s| s.visible).collect()
    }

    pub fn get_visible_mut(&mut self) -> Vec<&mut Sprite> {
        self.sprites.iter_mut().filter(|s| s.visible).collect()
    }

    pub fn sort_by_layer(&mut self) {
        self.sprites.sort_by_key(|s| s.layer);
        self.dirty = false;
    }

    pub fn get(&self, id: &str) -> Option<&Sprite> {
        self.sprites.iter().find(|s| s.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Sprite> {
        self.sprites.iter_mut().find(|s| s.id == id)
    }

    pub fn count(&self) -> usize {
        self.sprites.len()
    }

    pub fn visible_count(&self) -> usize {
        self.sprites.iter().filter(|s| s.visible).count()
    }

    pub fn clear(&mut self) {
        self.sprites.clear();
        self.dirty = false;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn get_in_rect(&self, x: f32, y: f32, w: f32, h: f32) -> Vec<&Sprite> {
        self.sprites
            .iter()
            .filter(|s| {
                s.visible
                    && s.x < x + w
                    && s.x + s.scaled_width() > x
                    && s.y < y + h
                    && s.y + s.scaled_height() > y
            })
            .collect()
    }

    pub fn click_at(&self, sx: f32, sy: f32) -> Option<&Sprite> {
        self.sprites
            .iter()
            .rev()
            .find(|s| s.visible && s.contains_point(sx, sy))
    }
}

impl Default for SpriteBatch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_creation() {
        let s = Sprite::new("test", 10.0, 20.0, 32, 32, 0xFF0000FF);
        assert_eq!(s.id, "test");
        assert_eq!(s.x, 10.0);
        assert_eq!(s.scale, 1.0);
        assert!(s.visible);
    }

    #[test]
    fn test_sprite_scaled_size() {
        let mut s = Sprite::new("s", 0.0, 0.0, 10, 20, 0);
        s.scale = 2.0;
        assert_eq!(s.scaled_width(), 20.0);
        assert_eq!(s.scaled_height(), 40.0);
    }

    #[test]
    fn test_sprite_contains_point() {
        let s = Sprite::new("s", 10.0, 10.0, 20, 20, 0);
        assert!(s.contains_point(15.0, 15.0));
        assert!(!s.contains_point(5.0, 5.0));
        assert!(!s.contains_point(35.0, 35.0));
    }

    #[test]
    fn test_batch_add_remove() {
        let mut batch = SpriteBatch::new();
        batch.add(Sprite::new("a", 0.0, 0.0, 10, 10, 0));
        batch.add(Sprite::new("b", 10.0, 10.0, 10, 10, 0));
        assert_eq!(batch.count(), 2);
        batch.remove("a");
        assert_eq!(batch.count(), 1);
    }

    #[test]
    fn test_batch_update() {
        let mut batch = SpriteBatch::new();
        batch.add(Sprite::new("s", 0.0, 0.0, 10, 10, 0));
        batch.update("s", |s| {
            s.x = 50.0;
            s.color = 0x00FF00FF;
        });
        assert_eq!(batch.get("s").unwrap().x, 50.0);
    }

    #[test]
    fn test_batch_get_visible() {
        let mut batch = SpriteBatch::new();
        batch.add(Sprite::new("a", 0.0, 0.0, 10, 10, 0));
        let mut hidden = Sprite::new("b", 10.0, 10.0, 10, 10, 0);
        hidden.visible = false;
        batch.add(hidden);
        assert_eq!(batch.get_visible().len(), 1);
    }

    #[test]
    fn test_batch_sort_by_layer() {
        let mut batch = SpriteBatch::new();
        let mut s1 = Sprite::new("a", 0.0, 0.0, 10, 10, 0);
        s1.layer = 5;
        let mut s2 = Sprite::new("b", 0.0, 0.0, 10, 10, 0);
        s2.layer = 1;
        batch.add(s1);
        batch.add(s2);
        batch.sort_by_layer();
        assert_eq!(batch.get_visible()[0].id, "b");
    }

    #[test]
    fn test_batch_click_at() {
        let mut batch = SpriteBatch::new();
        batch.add(Sprite::new("a", 10.0, 10.0, 20, 20, 0));
        assert!(batch.click_at(15.0, 15.0).is_some());
        assert!(batch.click_at(0.0, 0.0).is_none());
    }

    #[test]
    fn test_batch_get_in_rect() {
        let mut batch = SpriteBatch::new();
        batch.add(Sprite::new("a", 5.0, 5.0, 10, 10, 0));
        batch.add(Sprite::new("b", 100.0, 100.0, 10, 10, 0));
        let found = batch.get_in_rect(0.0, 0.0, 20.0, 20.0);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, "a");
    }

    #[test]
    fn test_batch_clear() {
        let mut batch = SpriteBatch::new();
        batch.add(Sprite::new("a", 0.0, 0.0, 10, 10, 0));
        batch.clear();
        assert_eq!(batch.count(), 0);
    }
}

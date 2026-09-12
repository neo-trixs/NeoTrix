use crate::engine::renderer::{Color, Rect, DrawCommand};
use std::collections::HashMap;

pub struct SpriteBatchExt {
    pub batches: HashMap<String, Vec<BatchEntry>>,
    pub max_batch_size: usize,
}

#[derive(Clone)]
pub struct BatchEntry {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub src_x: f32,
    pub src_y: f32,
    pub src_w: f32,
    pub src_h: f32,
    pub color: Color,
    pub z_order: i32,
}

impl SpriteBatchExt {
    pub fn new(max_batch_size: usize) -> Self {
        Self {
            batches: HashMap::new(),
            max_batch_size,
        }
    }

    pub fn add(&mut self, texture: &str, entry: BatchEntry) {
        self.batches
            .entry(texture.to_string())
            .or_default()
            .push(entry);
    }

    pub fn clear(&mut self) {
        self.batches.clear();
    }

    pub fn to_draw_commands(&self) -> Vec<DrawCommand> {
        let mut cmds = Vec::new();
        for (texture, entries) in &self.batches {
            let mut sorted = entries.clone();
            sorted.sort_by_key(|e| e.z_order);
            for entry in sorted {
                cmds.push(DrawCommand::DrawSprite {
                    texture: texture.clone(),
                    dest: Rect::new(entry.x, entry.y, entry.width, entry.height),
                    color: entry.color,
                    z_index: entry.z_order,
                });
            }
        }
        cmds
    }

    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }

    pub fn entry_count(&self) -> usize {
        self.batches.values().map(|b| b.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.batches.is_empty()
    }

    pub fn textures(&self) -> Vec<&str> {
        self.batches.keys().map(|s| s.as_str()).collect()
    }

    pub fn entries_for(&self, texture: &str) -> Option<&Vec<BatchEntry>> {
        self.batches.get(texture)
    }

    pub fn remove_texture(&mut self, texture: &str) -> bool {
        self.batches.remove(texture).is_some()
    }

    pub fn total_vertices(&self) -> usize {
        self.entry_count() * 4
    }
}

impl Default for SpriteBatchExt {
    fn default() -> Self {
        Self::new(1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(x: f32, z: i32) -> BatchEntry {
        BatchEntry {
            x,
            y: 0.0,
            width: 16.0,
            height: 16.0,
            src_x: 0.0,
            src_y: 0.0,
            src_w: 16.0,
            src_h: 16.0,
            color: Color::white(),
            z_order: z,
        }
    }

    #[test]
    fn test_add_and_count() {
        let mut batch = SpriteBatchExt::new(64);
        batch.add("hero.png", make_entry(0.0, 0));
        batch.add("hero.png", make_entry(32.0, 1));
        batch.add("enemy.png", make_entry(64.0, 0));
        assert_eq!(batch.batch_count(), 2);
        assert_eq!(batch.entry_count(), 3);
    }

    #[test]
    fn test_to_draw_commands_sorted() {
        let mut batch = SpriteBatchExt::new(64);
        batch.add("t.png", make_entry(10.0, 5));
        batch.add("t.png", make_entry(20.0, 1));
        batch.add("t.png", make_entry(30.0, 3));
        let cmds = batch.to_draw_commands();
        assert_eq!(cmds.len(), 3);
        if let DrawCommand::DrawSprite { dest, z_index, .. } = &cmds[0] {
            assert_eq!(dest.x, 20.0);
            assert_eq!(*z_index, 1);
        } else {
            panic!("expected DrawSprite");
        }
    }

    #[test]
    fn test_clear() {
        let mut batch = SpriteBatchExt::new(64);
        batch.add("a.png", make_entry(0.0, 0));
        assert!(!batch.is_empty());
        batch.clear();
        assert!(batch.is_empty());
    }

    #[test]
    fn test_remove_texture() {
        let mut batch = SpriteBatchExt::new(64);
        batch.add("a.png", make_entry(0.0, 0));
        assert!(batch.remove_texture("a.png"));
        assert!(!batch.remove_texture("a.png"));
        assert_eq!(batch.entry_count(), 0);
    }
}

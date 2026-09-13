use crate::engine::renderer::{Color, Rect, DrawCommand};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Texture Atlas — maps sub-textures to regions within a single atlas texture
// ---------------------------------------------------------------------------

/// A region within an atlas texture (UV-like coordinates in pixels)
#[derive(Debug, Clone, Copy)]
pub struct AtlasRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Texture atlas: packs multiple small textures into one large texture to
/// reduce draw calls. Sprites referencing atlas sub-textures are batched
/// together under the atlas texture name.
#[derive(Debug, Clone)]
pub struct TextureAtlas {
    /// Atlas texture name (the single GPU texture)
    pub atlas_texture: String,
    /// Maps sub-texture names to their region within the atlas
    pub regions: HashMap<String, AtlasRegion>,
    /// Atlas dimensions (for validation)
    pub atlas_width: f32,
    pub atlas_height: f32,
}

impl TextureAtlas {
    pub fn new(atlas_texture: &str, width: f32, height: f32) -> Self {
        Self {
            atlas_texture: atlas_texture.to_string(),
            regions: HashMap::new(),
            atlas_width: width,
            atlas_height: height,
        }
    }

    /// Register a sub-texture region within the atlas
    pub fn add_region(&mut self, name: &str, region: AtlasRegion) {
        self.regions.insert(name.to_string(), region);
    }

    /// Look up the atlas region for a texture name
    #[inline]
    pub fn get_region(&self, texture_name: &str) -> Option<&AtlasRegion> {
        self.regions.get(texture_name)
    }

    /// Check if a texture name belongs to this atlas
    #[inline]
    pub fn contains(&self, texture_name: &str) -> bool {
        self.regions.contains_key(texture_name)
    }
}

pub struct SpriteBatchExt {
    pub batches: HashMap<String, Vec<BatchEntry>>,
    pub max_batch_size: usize,
    /// Optional texture atlas — sprites referencing atlas sub-textures are
    /// remapped to the atlas texture and their src coords are adjusted.
    pub atlas: Option<TextureAtlas>,
    /// Tracks total draw calls generated (for perf metrics)
    pub draw_calls: u64,
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
            atlas: None,
            draw_calls: 0,
        }
    }

    /// Attach a texture atlas. Sprites added via `add()` will be automatically
    /// remapped to the atlas texture if their name matches an atlas region.
    pub fn set_atlas(&mut self, atlas: TextureAtlas) {
        self.atlas = Some(atlas);
    }

    pub fn add(&mut self, texture: &str, entry: BatchEntry) {
        // If an atlas is set and contains this texture, remap to atlas texture
        if let Some(ref atlas) = self.atlas {
            if let Some(region) = atlas.get_region(texture) {
                let key = atlas.atlas_texture.clone();
                let remapped = BatchEntry {
                    src_x: region.x,
                    src_y: region.y,
                    src_w: region.width,
                    src_h: region.height,
                    ..entry
                };
                self.batches.entry(key).or_default().push(remapped);
                return;
            }
        }
        self.batches
            .entry(texture.to_string())
            .or_default()
            .push(entry);
    }

    pub fn clear(&mut self) {
        self.batches.clear();
        self.draw_calls = 0;
    }

    /// Convert batches to draw commands. Sprites sharing the same texture
    /// are grouped into a single batch → single draw call.
    pub fn to_draw_commands(&mut self) -> Vec<DrawCommand> {
        let mut cmds = Vec::new();
        for (texture, entries) in &self.batches {
            let mut sorted = entries.clone();
            sorted.sort_by_key(|e| e.z_order);
            // Split into sub-batches if exceeding max_batch_size
            for chunk in sorted.chunks(self.max_batch_size) {
                self.draw_calls += 1;
                for entry in chunk {
                    cmds.push(DrawCommand::DrawSprite {
                        texture: texture.clone(),
                        dest: Rect::new(entry.x, entry.y, entry.width, entry.height),
                        src_rect: Some(Rect::new(entry.src_x, entry.src_y, entry.src_w, entry.src_h)),
                        color: entry.color,
                        alpha: 1.0,
                        flip_x: false,
                        flip_y: false,
                        rotation: 0.0,
                        z_index: entry.z_order,
                    });
                }
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

    /// How many unique textures are being batched (lower = fewer draw calls)
    pub fn unique_texture_count(&self) -> usize {
        self.batches.len()
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

    #[test]
    fn test_texture_atlas_remapping() {
        let mut atlas = TextureAtlas::new("atlas_main.png", 512.0, 512.0);
        atlas.add_region("hero.png", AtlasRegion { x: 0.0, y: 0.0, width: 32.0, height: 32.0 });
        atlas.add_region("enemy.png", AtlasRegion { x: 32.0, y: 0.0, width: 32.0, height: 32.0 });

        let mut batch = SpriteBatchExt::new(64);
        batch.set_atlas(atlas);

        batch.add("hero.png", make_entry(0.0, 0));
        batch.add("enemy.png", make_entry(64.0, 0));
        batch.add("hero.png", make_entry(128.0, 1));

        // All sprites remapped to single atlas texture → 1 batch
        assert_eq!(batch.batch_count(), 1);
        assert_eq!(batch.entry_count(), 3);
        // Check that the batch key is the atlas texture
        assert!(batch.batches.contains_key("atlas_main.png"));
    }

    #[test]
    fn test_atlas_non_atlas_texture_unchanged() {
        let mut atlas = TextureAtlas::new("atlas.png", 256.0, 256.0);
        atlas.add_region("hero.png", AtlasRegion { x: 0.0, y: 0.0, width: 16.0, height: 16.0 });

        let mut batch = SpriteBatchExt::new(64);
        batch.set_atlas(atlas);

        batch.add("hero.png", make_entry(0.0, 0));
        batch.add("background.png", make_entry(64.0, 0)); // not in atlas

        assert_eq!(batch.batch_count(), 2); // atlas batch + separate batch
    }

    #[test]
    fn test_max_batch_size_chunking() {
        let mut batch = SpriteBatchExt::new(2); // max 2 per batch
        for i in 0..5 {
            batch.add("t.png", make_entry(i as f32, 0));
        }
        let cmds = batch.to_draw_commands();
        assert_eq!(cmds.len(), 5);
        assert_eq!(batch.draw_calls, 3); // ceil(5/2) = 3 draw calls
    }
}

use std::collections::HashMap;

/// Typed handle into the asset server.
#[derive(Debug)]
pub struct AssetHandle<T> {
    pub id: u32,
    pub _marker: std::marker::PhantomData<T>,
}

impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self { Self { id: self.id, _marker: std::marker::PhantomData } }
}
impl<T> Copy for AssetHandle<T> {}

impl<T> PartialEq for AssetHandle<T> {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
impl<T> Eq for AssetHandle<T> {}

impl<T> std::hash::Hash for AssetHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state); }
}

/// Texture data stored as RGBA pixels.
#[derive(Debug)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Sound data stored as f32 samples.
pub struct SoundData {
    pub sample_rate: u32,
    pub channels: u16,
    pub data: Vec<f32>,
}

/// Font data with per-glyph metrics.
pub struct FontData {
    pub size: u32,
    pub glyphs: HashMap<char, GlyphData>,
}

/// Metrics for a single glyph.
pub struct GlyphData {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub advance: f32,
}

/// Central asset server — loads, caches, and serves assets by name.
pub struct AssetServer {
    textures: HashMap<String, TextureData>,
    sounds: HashMap<String, SoundData>,
    fonts: HashMap<String, FontData>,
    next_id: u32,
}

impl AssetServer {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            sounds: HashMap::new(),
            fonts: HashMap::new(),
            next_id: 0,
        }
    }

    // -- textures --

    pub fn load_texture(&mut self, name: &str, width: u32, height: u32, pixels: Vec<u8>) -> AssetHandle<TextureData> {
        self.textures.insert(name.to_string(), TextureData { width, height, pixels });
        let id = self.next_id;
        self.next_id += 1;
        AssetHandle { id, _marker: std::marker::PhantomData }
    }

    pub fn get_texture(&self, name: &str) -> Option<&TextureData> {
        self.textures.get(name)
    }

    pub fn get_texture_mut(&mut self, name: &str) -> Option<&mut TextureData> {
        self.textures.get_mut(name)
    }

    pub fn has_texture(&self, name: &str) -> bool { self.textures.contains_key(name) }

    pub fn remove_texture(&mut self, name: &str) -> Option<TextureData> {
        self.textures.remove(name)
    }

    pub fn generate_placeholder_texture(&mut self, name: &str, width: u32, height: u32, color: [u8; 4]) -> AssetHandle<TextureData> {
        let pixels = vec![color[0], color[1], color[2], color[3]]
            .repeat((width * height) as usize);
        self.load_texture(name, width, height, pixels)
    }

    pub fn generate_checkerboard(&mut self, name: &str, size: u32, tile: u32, c1: [u8; 4], c2: [u8; 4]) -> AssetHandle<TextureData> {
        let mut pixels = Vec::with_capacity((size * size * 4) as usize);
        for y in 0..size {
            for x in 0..size {
                let checker = ((x / tile) + (y / tile)) % 2 == 0;
                let c = if checker { c1 } else { c2 };
                pixels.extend_from_slice(&c);
            }
        }
        self.load_texture(name, size, size, pixels)
    }

    pub fn texture_count(&self) -> usize { self.textures.len() }

    // -- sounds --

    pub fn load_sound(&mut self, name: &str, sample_rate: u32, channels: u16, data: Vec<f32>) -> AssetHandle<SoundData> {
        self.sounds.insert(name.to_string(), SoundData { sample_rate, channels, data });
        let id = self.next_id;
        self.next_id += 1;
        AssetHandle { id, _marker: std::marker::PhantomData }
    }

    pub fn get_sound(&self, name: &str) -> Option<&SoundData> { self.sounds.get(name) }
    pub fn has_sound(&self, name: &str) -> bool { self.sounds.contains_key(name) }
    pub fn remove_sound(&mut self, name: &str) -> Option<SoundData> { self.sounds.remove(name) }
    pub fn sound_count(&self) -> usize { self.sounds.len() }

    pub fn generate_silence(&mut self, name: &str, sample_rate: u32, channels: u16, duration_secs: f32) -> AssetHandle<SoundData> {
        let len = (sample_rate as f32 * duration_secs) as usize;
        self.load_sound(name, sample_rate, channels, vec![0.0; len * channels as usize])
    }

    // -- fonts --

    pub fn load_font(&mut self, name: &str, size: u32, glyphs: HashMap<char, GlyphData>) -> AssetHandle<FontData> {
        self.fonts.insert(name.to_string(), FontData { size, glyphs });
        let id = self.next_id;
        self.next_id += 1;
        AssetHandle { id, _marker: std::marker::PhantomData }
    }

    pub fn get_font(&self, name: &str) -> Option<&FontData> { self.fonts.get(name) }
    pub fn has_font(&self, name: &str) -> bool { self.fonts.contains_key(name) }
    pub fn remove_font(&mut self, name: &str) -> Option<FontData> { self.fonts.remove(name) }
    pub fn font_count(&self) -> usize { self.fonts.len() }

    pub fn generate_ascii_font(&mut self, name: &str, size: u32) -> AssetHandle<FontData> {
        let mut glyphs = HashMap::new();
        let glyph_w = size / 2;
        let glyphs_per_row = 16;
        for ch in (32u8..127u8).map(char::from) {
            let idx = (ch as u32) - 32;
            let col = idx % glyphs_per_row;
            let row = idx / glyphs_per_row;
            glyphs.insert(ch, GlyphData {
                x: col * glyph_w,
                y: row * size,
                width: glyph_w,
                height: size,
                advance: glyph_w as f32,
            });
        }
        self.load_font(name, size, glyphs)
    }

    // -- cleanup --

    pub fn clear_all(&mut self) {
        self.textures.clear();
        self.sounds.clear();
        self.fonts.clear();
    }

    pub fn total_asset_count(&self) -> usize {
        self.textures.len() + self.sounds.len() + self.fonts.len()
    }
}

impl Default for AssetServer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_load_and_get() {
        let mut server = AssetServer::new();
        let h = server.load_texture("player", 2, 2, vec![255; 16]);
        assert_eq!(h.id, 0);
        assert_eq!(server.texture_count(), 1);
        let tex = server.get_texture("player").unwrap();
        assert_eq!(tex.width, 2);
        assert_eq!(tex.height, 2);
    }

    #[test]
    fn test_placeholder_texture() {
        let mut server = AssetServer::new();
        server.generate_placeholder_texture("ph", 4, 4, [0, 255, 0, 255]);
        let tex = server.get_texture("ph").unwrap();
        assert_eq!(tex.pixels.len(), 64);
        assert_eq!(tex.pixels[0], 0);
        assert_eq!(tex.pixels[1], 255);
    }

    #[test]
    fn test_checkerboard() {
        let mut server = AssetServer::new();
        server.generate_checkerboard("cb", 4, 2, [255; 4], [0; 4]);
        let tex = server.get_texture("cb").unwrap();
        assert_eq!(tex.pixels.len(), 64);
    }

    #[test]
    fn test_sound_load() {
        let mut server = AssetServer::new();
        let h = server.load_sound("sfx", 44100, 1, vec![0.0; 44100]);
        assert_eq!(server.sound_count(), 1);
        let snd = server.get_sound("sfx").unwrap();
        assert_eq!(snd.sample_rate, 44100);
        assert_eq!(snd.data.len(), 44100);
        let _ = h;
    }

    #[test]
    fn test_font_generate_ascii() {
        let mut server = AssetServer::new();
        server.generate_ascii_font("mono", 16);
        let font = server.get_font("mono").unwrap();
        assert!(font.glyphs.contains_key(&'A'));
        assert!(font.glyphs.contains_key(&'z'));
        assert_eq!(font.size, 16);
    }

    #[test]
    fn test_clear_all() {
        let mut server = AssetServer::new();
        server.load_texture("a", 1, 1, vec![0; 4]);
        server.load_sound("b", 44100, 1, vec![]);
        server.generate_ascii_font("c", 16);
        assert_eq!(server.total_asset_count(), 3);
        server.clear_all();
        assert_eq!(server.total_asset_count(), 0);
    }

    #[test]
    fn test_handle_eq_and_hash() {
        let mut server = AssetServer::new();
        let h1 = server.load_texture("a", 1, 1, vec![0; 4]);
        let h2 = server.load_texture("b", 1, 1, vec![0; 4]);
        assert_ne!(h1, h2);
        let mut map = HashMap::new();
        map.insert(h1, "first");
        assert_eq!(map.get(&h1), Some(&"first"));
    }
}

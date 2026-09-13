use std::collections::HashMap;

use super::renderer::{Color, Vec2};

// ---------------------------------------------------------------------------
// Tile
// ---------------------------------------------------------------------------

/// A single tile on the map.
#[derive(Debug, Clone)]
pub struct Tile {
    /// Tile ID in the tileset.
    pub id: u32,
    /// Per-tile properties (collision, speed modifier, etc).
    pub properties: HashMap<String, TileProperty>,
    /// Whether this tile blocks movement.
    pub collision: bool,
}

impl Tile {
    pub fn new(id: u32) -> Self {
        Self { id, properties: HashMap::new(), collision: false }
    }

    pub fn with_collision(mut self, collides: bool) -> Self {
        self.collision = collides;
        self
    }

    pub fn with_property(mut self, key: &str, value: TileProperty) -> Self {
        self.properties.insert(key.to_string(), value);
        self
    }

    pub fn get_property(&self, key: &str) -> Option<&TileProperty> {
        self.properties.get(key)
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Tile property value.
#[derive(Debug, Clone)]
pub enum TileProperty {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

// ---------------------------------------------------------------------------
// MapLayer
// ---------------------------------------------------------------------------

/// A single layer of tile data.
#[derive(Debug, Clone)]
pub struct MapLayer {
    /// Layer name (e.g. "ground", "walls", "objects").
    pub name: String,
    /// 2D tile data: `tiles[y * width + x]`.
    pub tiles: Vec<u32>,
    pub width: usize,
    pub height: usize,
    /// Whether this layer is visible.
    pub visible: bool,
    /// Opacity (0.0..1.0).
    pub opacity: f32,
    /// Parallax multiplier (1.0 = no scroll, <1 = slower).
    pub parallax: Vec2,
    /// Draw order (lower = earlier).
    pub z_index: i32,
    /// Whether collision is checked on this layer.
    pub collision_enabled: bool,
}

impl MapLayer {
    pub fn new(name: &str, width: usize, height: usize) -> Self {
        Self {
            name: name.to_string(),
            tiles: vec![0; width * height],
            width,
            height,
            visible: true,
            opacity: 1.0,
            parallax: Vec2::new(1.0, 1.0),
            z_index: 0,
            collision_enabled: true,
        }
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<u32> {
        if x < self.width && y < self.height {
            Some(self.tiles[y * self.width + x])
        } else {
            None
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile_id: u32) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = tile_id;
        }
    }

    pub fn fill(&mut self, tile_id: u32) {
        self.tiles.iter_mut().for_each(|t| *t = tile_id);
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.iter().all(|&t| t == 0)
    }
}

// ---------------------------------------------------------------------------
// TileMap
// ---------------------------------------------------------------------------

/// A multi-layer tile map.
#[derive(Debug, Clone)]
pub struct TileMap {
    pub layers: Vec<MapLayer>,
    /// Width in tiles.
    pub width: usize,
    /// Height in tiles.
    pub height: usize,
    /// Size of each tile in world units.
    pub tile_size: f32,
    /// Palette mapping tile IDs to colors (fallback rendering).
    pub palette: HashMap<u32, Color>,
}

impl TileMap {
    pub fn new(width: usize, height: usize, tile_size: f32) -> Self {
        Self {
            layers: Vec::new(),
            width,
            height,
            tile_size,
            palette: HashMap::new(),
        }
    }

    pub fn add_layer(&mut self, layer: MapLayer) {
        self.layers.push(layer);
    }

    pub fn add_layer_at(&mut self, index: usize, layer: MapLayer) {
        self.layers.insert(index.min(self.layers.len()), layer);
    }

    pub fn remove_layer(&mut self, index: usize) -> Option<MapLayer> {
        if index < self.layers.len() {
            Some(self.layers.remove(index))
        } else {
            None
        }
    }

    pub fn get_layer(&self, name: &str) -> Option<&MapLayer> {
        self.layers.iter().find(|l| l.name == name)
    }

    pub fn get_layer_mut(&mut self, name: &str) -> Option<&mut MapLayer> {
        self.layers.iter_mut().find(|l| l.name == name)
    }

    /// Get the tile at (x, y) on a specific layer by index.
    pub fn tile_at(&self, layer: usize, x: usize, y: usize) -> Option<u32> {
        self.layers.get(layer)?.get_tile(x, y)
    }

    /// Set the tile at (x, y) on a specific layer by index.
    pub fn set_tile_at(&mut self, layer: usize, x: usize, y: usize, tile_id: u32) {
        if let Some(l) = self.layers.get_mut(layer) {
            l.set_tile(x, y, tile_id);
        }
    }

    /// Check if a position is walkable across all collision-enabled layers.
    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        for layer in &self.layers {
            if !layer.collision_enabled || !layer.visible {
                continue;
            }
            if let Some(tile_id) = layer.get_tile(x, y) {
                if tile_id != 0 {
                    // Non-zero tile on a collision layer blocks movement
                    return false;
                }
            }
        }
        true
    }

    /// World size in units.
    pub fn world_size(&self) -> Vec2 {
        Vec2::new(
            self.width as f32 * self.tile_size,
            self.height as f32 * self.tile_size,
        )
    }

    /// Convert world position to tile coordinates.
    pub fn world_to_tile(&self, pos: Vec2) -> (usize, usize) {
        let x = (pos.x / self.tile_size).floor() as usize;
        let y = (pos.y / self.tile_size).floor() as usize;
        (x.min(self.width.saturating_sub(1)), y.min(self.height.saturating_sub(1)))
    }

    /// Convert tile coordinates to world position (top-left corner).
    pub fn tile_to_world(&self, x: usize, y: usize) -> Vec2 {
        Vec2::new(x as f32 * self.tile_size, y as f32 * self.tile_size)
    }

    pub fn set_palette_color(&mut self, tile_id: u32, color: Color) {
        self.palette.insert(tile_id, color);
    }

    pub fn get_tile_color(&self, tile_id: u32) -> Color {
        self.palette.get(&tile_id).copied().unwrap_or(Color::rgba(0.3, 0.3, 0.3, 1.0))
    }
}

impl Default for TileMap {
    fn default() -> Self {
        Self::new(32, 32, 16.0)
    }
}

// ---------------------------------------------------------------------------
// AutoTile (13-tile transitions)
// ---------------------------------------------------------------------------

/// Bitmask flags for auto-tile neighbor detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AutoTileFlags(pub u8);

impl AutoTileFlags {
    pub const EMPTY: u8 = 0;
    pub const TOP: u8 = 1 << 0;
    pub const RIGHT: u8 = 1 << 1;
    pub const BOTTOM: u8 = 1 << 2;
    pub const LEFT: u8 = 1 << 3;
    pub const TOP_RIGHT: u8 = 1 << 4;
    pub const BOTTOM_RIGHT: u8 = 1 << 5;
    pub const BOTTOM_LEFT: u8 = 1 << 6;
    pub const TOP_LEFT: u8 = 1 << 7;

    pub fn new() -> Self {
        Self(Self::EMPTY)
    }

    pub fn with(mut self, flag: u8) -> Self {
        self.0 |= flag;
        self
    }

    pub fn has(self, flag: u8) -> bool {
        (self.0 & flag) != 0
    }

    /// Compute the 13-tile auto-tile index from the bitmask.
    /// 0 = isolated, 1-12 = various edge/corner/inner combinations.
    pub fn to_tile_index(self) -> u32 {
        // Simplified 13-tile mapping
        let cardinal = (self.0 & 0x0F) as u32;
        match cardinal {
            0b0000 => 0,  // isolated
            0b0001 => 1,  // top edge
            0b0010 => 2,  // right edge
            0b0100 => 3,  // bottom edge
            0b1000 => 4,  // left edge
            0b0011 => 5,  // top-right corner
            0b0110 => 6,  // bottom-right corner
            0b1100 => 7,  // bottom-left corner
            0b1001 => 8,  // top-left corner
            0b0101 => 9,  // vertical strip
            0b1010 => 10, // horizontal strip
            0b0111 => 11, // T-junction (missing top)
            0b1011 => 12, // T-junction (missing right)
            0b1101 => 13, // T-junction (missing bottom)
            0b1110 => 14, // T-junction (missing left)
            0b1111 => 15, // cross (all neighbors)
        }
    }
}

/// Auto-tile system: recomputes tile IDs based on neighbor presence.
pub struct AutoTileSystem {
    /// The tile ID that triggers auto-tiling.
    pub active_tile_id: u32,
    /// Base tile ID range start in the tileset.
    pub base_tile_id: u32,
}

impl AutoTileSystem {
    pub fn new(active_tile_id: u32, base_tile_id: u32) -> Self {
        Self { active_tile_id, base_tile_id }
    }

    /// Compute auto-tile flags for a position on a layer.
    pub fn compute_flags(&self, layer: &MapLayer, x: usize, y: usize) -> AutoTileFlags {
        let mut flags = AutoTileFlags::new();

        // Check cardinal neighbors
        if y > 0 && layer.get_tile(x, y - 1) == Some(self.active_tile_id) {
            flags = flags.with(AutoTileFlags::TOP);
        }
        if x + 1 < layer.width && layer.get_tile(x + 1, y) == Some(self.active_tile_id) {
            flags = flags.with(AutoTileFlags::RIGHT);
        }
        if y + 1 < layer.height && layer.get_tile(x, y + 1) == Some(self.active_tile_id) {
            flags = flags.with(AutoTileFlags::BOTTOM);
        }
        if x > 0 && layer.get_tile(x - 1, y) == Some(self.active_tile_id) {
            flags = flags.with(AutoTileFlags::LEFT);
        }

        // Check diagonal neighbors
        if y > 0 && x + 1 < layer.width && layer.get_tile(x + 1, y - 1) == Some(self.active_tile_id) {
            flags = flags.with(AutoTileFlags::TOP_RIGHT);
        }
        if y + 1 < layer.height && x + 1 < layer.width && layer.get_tile(x + 1, y + 1) == Some(self.active_tile_id) {
            flags = flags.with(AutoTileFlags::BOTTOM_RIGHT);
        }
        if y + 1 < layer.height && x > 0 && layer.get_tile(x - 1, y + 1) == Some(self.active_tile_id) {
            flags = flags.with(AutoTileFlags::BOTTOM_LEFT);
        }
        if y > 0 && x > 0 && layer.get_tile(x - 1, y - 1) == Some(self.active_tile_id) {
            flags = flags.with(AutoTileFlags::TOP_LEFT);
        }

        flags
    }

    /// Apply auto-tiling to an entire layer.
    pub fn apply_to_layer(&self, layer: &mut MapLayer) {
        for y in 0..layer.height {
            for x in 0..layer.width {
                if layer.get_tile(x, y) == Some(self.active_tile_id) {
                    let flags = self.compute_flags(layer, x, y);
                    let tile_index = flags.to_tile_index();
                    layer.set_tile(x, y, self.base_tile_id + tile_index);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// FogOfWar
// ---------------------------------------------------------------------------

/// State of a fog tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FogState {
    /// Completely hidden (black).
    Hidden,
    /// Previously seen but not currently visible (dim).
    Explored,
    /// Currently visible (clear).
    Visible,
}

impl FogState {
    /// Blending alpha for rendering.
    pub fn alpha(&self) -> f32 {
        match self {
            FogState::Hidden => 1.0,
            FogState::Explored => 0.5,
            FogState::Visible => 0.0,
        }
    }
}

/// Fog of war system.
pub struct FogOfWar {
    /// 2D grid of fog states.
    pub states: Vec<Vec<FogState>>,
    pub width: usize,
    pub height: usize,
    /// Radius (in tiles) of visibility around entities.
    pub visibility_radius: i32,
}

impl FogOfWar {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            states: vec![vec![FogState::Hidden; width]; height],
            width,
            height,
            visibility_radius: 5,
        }
    }

    /// Reveal tiles around a world position (tile coords).
    pub fn reveal_around(&mut self, tx: usize, ty: usize) {
        let r = self.visibility_radius;
        for dy in -r..=r {
            for dx in -r..=r {
                let nx = tx as i32 + dx;
                let ny = ty as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < self.width && (ny as usize) < self.height {
                    if dx * dx + dy * dy <= r * r {
                        let x = nx as usize;
                        let y = ny as usize;
                        if self.states[y][x] != FogState::Visible {
                            self.states[y][x] = FogState::Explored;
                        }
                    }
                }
            }
        }
        // Mark immediate area as visible
        for dy in -r..=r {
            for dx in -r..=r {
                let nx = tx as i32 + dx;
                let ny = ty as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < self.width && (ny as usize) < self.height {
                    if dx * dx + dy * dy <= r * r {
                        self.states[ny as usize][nx as usize] = FogState::Visible;
                    }
                }
            }
        }
    }

    /// Fade all Visible tiles to Explored (call at end of turn/step).
    pub fn fade_visible(&mut self) {
        for row in &mut self.states {
            for state in row.iter_mut() {
                if *state == FogState::Visible {
                    *state = FogState::Explored;
                }
            }
        }
    }

    /// Check if a tile is visible.
    pub fn is_visible(&self, x: usize, y: usize) -> bool {
        self.states.get(y).and_then(|row| row.get(x)) == Some(&FogState::Visible)
    }

    /// Check if a tile has been explored.
    pub fn is_explored(&self, x: usize, y: usize) -> bool {
        self.states.get(y).and_then(|row| row.get(x))
            .map_or(false, |s| *s != FogState::Hidden)
    }

    /// Reset all fog to hidden.
    pub fn reset(&mut self) {
        for row in &mut self.states {
            for state in row.iter_mut() {
                *state = FogState::Hidden;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Minimap Renderer
// ---------------------------------------------------------------------------

/// Renders a minimap from tile map + fog data.
pub struct MinimapRenderer {
    /// Size of each minimap pixel in screen units.
    pub pixel_size: f32,
    /// Border padding.
    pub padding: f32,
    /// Background color.
    pub bg_color: Color,
}

impl MinimapRenderer {
    pub fn new(pixel_size: f32) -> Self {
        Self {
            pixel_size,
            padding: 2.0,
            bg_color: Color::rgba(0.0, 0.0, 0.0, 0.7),
        }
    }

    /// Generate minimap pixel data: Vec of (screen_x, screen_y, color).
    pub fn render(
        &self,
        tilemap: &TileMap,
        fog: Option<&FogOfWar>,
        camera_pos: Vec2,
        viewport_size: Vec2,
    ) -> Vec<(f32, f32, Color)> {
        let mut pixels = Vec::new();
        let map_w = tilemap.width as f32 * self.pixel_size;
        let map_h = tilemap.height as f32 * self.pixel_size;

        // Background
        for y in 0..tilemap.height {
            for x in 0..tilemap.width {
                let sx = self.padding + x as f32 * self.pixel_size;
                let sy = self.padding + y as f32 * self.pixel_size;

                // Check fog
                if let Some(fog) = fog {
                    if fog.states[y][x] == FogState::Hidden {
                        continue;
                    }
                    if fog.states[y][x] == FogState::Explored {
                        // Dim the color
                        pixels.push((sx, sy, Color::rgba(0.15, 0.15, 0.15, 1.0)));
                        continue;
                    }
                }

                // Get tile color from first visible layer
                let color = tilemap.layers.iter()
                    .filter(|l| l.visible)
                    .find_map(|l| l.get_tile(x, y).map(|id| tilemap.get_tile_color(id)))
                    .unwrap_or(Color::rgba(0.1, 0.1, 0.1, 1.0));

                pixels.push((sx, sy, color));
            }
        }

        // Camera viewport indicator
        let cam_tile_x = (camera_pos.x / tilemap.tile_size * self.pixel_size) + self.padding;
        let cam_tile_y = (camera_pos.y / tilemap.tile_size * self.pixel_size) + self.padding;
        let vp_w = viewport_size.x / tilemap.tile_size * self.pixel_size;
        let vp_h = viewport_size.y / tilemap.tile_size * self.pixel_size;

        // Draw viewport rectangle corners
        let view_color = Color::rgba(1.0, 1.0, 1.0, 0.8);
        let corner_len = 3.0;
        // Top-left
        for i in 0..corner_len as usize {
            pixels.push((cam_tile_x + i as f32, cam_tile_y, view_color));
            pixels.push((cam_tile_x, cam_tile_y + i as f32, view_color));
        }
        // Top-right
        for i in 0..corner_len as usize {
            pixels.push((cam_tile_x + vp_w - i as f32, cam_tile_y, view_color));
            pixels.push((cam_tile_x + vp_w, cam_tile_y + i as f32, view_color));
        }

        pixels
    }

    /// World size of the minimap in screen units.
    pub fn minimap_size(&self, tilemap: &TileMap) -> Vec2 {
        Vec2::new(
            tilemap.width as f32 * self.pixel_size + self.padding * 2.0,
            tilemap.height as f32 * self.pixel_size + self.padding * 2.0,
        )
    }
}

impl Default for MinimapRenderer {
    fn default() -> Self {
        Self::new(2.0)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_new() {
        let t = Tile::new(5).with_collision(true);
        assert_eq!(t.id, 5);
        assert!(t.collision);
    }

    #[test]
    fn test_tile_property() {
        let t = Tile::new(1)
            .with_property("speed", TileProperty::Float(0.5))
            .with_property("water", TileProperty::Bool(true));
        assert!(matches!(t.get_property("water"), Some(TileProperty::Bool(true))));
    }

    #[test]
    fn test_map_layer() {
        let mut layer = MapLayer::new("ground", 4, 4);
        layer.set_tile(1, 2, 42);
        assert_eq!(layer.get_tile(1, 2), Some(42));
        assert_eq!(layer.get_tile(0, 0), Some(0));
        assert!(!layer.is_empty());
    }

    #[test]
    fn test_tilemap_basics() {
        let mut map = TileMap::new(8, 8, 16.0);
        map.add_layer(MapLayer::new("ground", 8, 8));
        assert_eq!(map.layers.len(), 1);
        map.set_tile_at(0, 3, 4, 99);
        assert_eq!(map.tile_at(0, 3, 4), Some(99));
    }

    #[test]
    fn test_tilemap_world_size() {
        let map = TileMap::new(10, 20, 32.0);
        let size = map.world_size();
        assert_eq!(size.x, 320.0);
        assert_eq!(size.y, 640.0);
    }

    #[test]
    fn test_world_to_tile_roundtrip() {
        let map = TileMap::new(16, 16, 16.0);
        let pos = Vec2::new(50.0, 80.0);
        let (tx, ty) = map.world_to_tile(pos);
        let back = map.tile_to_world(tx, ty);
        assert!((back.x - 48.0).abs() < 0.01);
        assert!((back.y - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_autotile_flags() {
        let mut f = AutoTileFlags::new();
        f = f.with(AutoTileFlags::TOP | AutoTileFlags::RIGHT);
        assert!(f.has(AutoTileFlags::TOP));
        assert!(f.has(AutoTileFlags::RIGHT));
        assert!(!f.has(AutoTileFlags::BOTTOM));
    }

    #[test]
    fn test_autotile_index() {
        let f = AutoTileFlags(AutoTileFlags::TOP | AutoTileFlags::RIGHT);
        let idx = f.to_tile_index();
        assert_eq!(idx, 5); // top-right corner
    }

    #[test]
    fn test_fog_of_war() {
        let mut fog = FogOfWar::new(16, 16);
        fog.visibility_radius = 2;
        fog.reveal_around(5, 5);
        assert!(fog.is_visible(5, 5));
        assert!(fog.is_visible(4, 5));
        assert!(!fog.is_visible(0, 0)); // too far
        fog.fade_visible();
        assert!(!fog.is_visible(5, 5));
        assert!(fog.is_explored(5, 5));
    }

    #[test]
    fn test_fog_reset() {
        let mut fog = FogOfWar::new(8, 8);
        fog.reveal_around(4, 4);
        fog.reset();
        assert!(!fog.is_explored(4, 4));
    }

    #[test]
    fn test_minimap_renderer() {
        let map = TileMap::new(8, 8, 16.0);
        let renderer = MinimapRenderer::new(2.0);
        let size = renderer.minimap_size(&map);
        assert_eq!(size.x, 20.0); // 8 * 2 + 4
        assert_eq!(size.y, 20.0);
    }

    #[test]
    fn test_tilemap_palette() {
        let mut map = TileMap::new(4, 4, 16.0);
        map.set_palette_color(1, Color::rgba(1.0, 0.0, 0.0, 1.0));
        let c = map.get_tile_color(1);
        assert!((c.r - 1.0).abs() < 0.01);
    }
}

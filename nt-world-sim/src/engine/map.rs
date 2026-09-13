use std::collections::HashMap;

use super::renderer::{Color, Vec2};

// ---------------------------------------------------------------------------
// CollisionType
// ---------------------------------------------------------------------------

/// How a tile interacts with entity movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollisionType {
    /// No collision — entities pass freely.
    None,
    /// Fully solid — blocks all movement.
    Solid,
    /// One-way platform — can jump through from below, stands on top.
    OneWay,
    /// Water — slows movement, allows swimming.
    Water,
}

impl Default for CollisionType {
    fn default() -> Self {
        Self::None
    }
}

// ---------------------------------------------------------------------------
// Tile
// ---------------------------------------------------------------------------

/// A single tile on the map.
#[derive(Debug, Clone)]
pub struct Tile {
    /// Tile ID in the tileset.
    pub id: u32,
    /// Per-tile properties (legacy key-value store).
    pub properties: HashMap<String, TileProperty>,
    /// How this tile collides with entities.
    pub collision: CollisionType,
    /// Damage dealt per step when standing on this tile.
    pub damage: i32,
    /// Movement speed multiplier (1.0 = normal, 0.5 = slow, 2.0 = fast).
    pub speed: f32,
}

impl Tile {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            properties: HashMap::new(),
            collision: CollisionType::None,
            damage: 0,
            speed: 1.0,
        }
    }

    pub fn with_collision(mut self, collision: CollisionType) -> Self {
        self.collision = collision;
        self
    }

    pub fn with_damage(mut self, damage: i32) -> Self {
        self.damage = damage;
        self
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn with_property(mut self, key: &str, value: TileProperty) -> Self {
        self.properties.insert(key.to_string(), value);
        self
    }

    pub fn get_property(&self, key: &str) -> Option<&TileProperty> {
        self.properties.get(key)
    }

    /// Whether this tile blocks horizontal/vertical movement.
    pub fn is_solid(&self) -> bool {
        self.collision == CollisionType::Solid
    }

    /// Whether this tile is a one-way platform.
    pub fn is_one_way(&self) -> bool {
        self.collision == CollisionType::OneWay
    }

    /// Whether this tile is water.
    pub fn is_water(&self) -> bool {
        self.collision == CollisionType::Water
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Tile property value (legacy key-value).
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
    /// 2D tile data: `tiles[y][x]`.
    pub tiles: Vec<Vec<Tile>>,
    pub width: usize,
    pub height: usize,
    /// Whether this layer is visible.
    pub visible: bool,
    /// Opacity (0.0..1.0).
    pub opacity: f32,
    /// Parallax scroll factor (x, y). 1.0 = normal scroll, <1 = slower.
    pub parallax: (f32, f32),
    /// Draw order (lower = earlier).
    pub z_index: i32,
    /// Whether collision is checked on this layer.
    pub collision_enabled: bool,
}

impl MapLayer {
    pub fn new(name: &str, width: usize, height: usize) -> Self {
        Self {
            name: name.to_string(),
            tiles: vec![vec![Tile::default(); width]; height],
            width,
            height,
            visible: true,
            opacity: 1.0,
            parallax: (1.0, 1.0),
            z_index: 0,
            collision_enabled: true,
        }
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        self.tiles.get(y)?.get(x)
    }

    pub fn get_tile_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        self.tiles.get_mut(y)?.get_mut(x)
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        if x < self.width && y < self.height {
            self.tiles[y][x] = tile;
        }
    }

    pub fn set_tile_id(&mut self, x: usize, y: usize, tile_id: u32) {
        if x < self.width && y < self.height {
            self.tiles[y][x].id = tile_id;
        }
    }

    pub fn get_tile_id(&self, x: usize, y: usize) -> Option<u32> {
        self.get_tile(x, y).map(|t| t.id)
    }

    pub fn fill(&mut self, tile: Tile) {
        for row in &mut self.tiles {
            for t in row.iter_mut() {
                *t = tile.clone();
            }
        }
    }

    pub fn fill_id(&mut self, tile_id: u32) {
        for row in &mut self.tiles {
            for t in row.iter_mut() {
                t.id = tile_id;
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.iter().all(|row| row.iter().all(|t| t.id == 0))
    }

    /// Get collision type at (x, y).
    pub fn collision_at(&self, x: usize, y: usize) -> CollisionType {
        self.get_tile(x, y).map_or(CollisionType::None, |t| t.collision)
    }

    /// Get speed multiplier at (x, y).
    pub fn speed_at(&self, x: usize, y: usize) -> f32 {
        self.get_tile(x, y).map_or(1.0, |t| t.speed)
    }

    /// Get damage at (x, y).
    pub fn damage_at(&self, x: usize, y: usize) -> i32 {
        self.get_tile(x, y).map_or(0, |t| t.damage)
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

    pub fn get_layer_by_index(&self, index: usize) -> Option<&MapLayer> {
        self.layers.get(index)
    }

    pub fn get_layer_by_index_mut(&mut self, index: usize) -> Option<&mut MapLayer> {
        self.layers.get_mut(index)
    }

    /// Get the tile at (x, y) on a specific layer by index.
    pub fn tile_at(&self, layer: usize, x: usize, y: usize) -> Option<&Tile> {
        self.layers.get(layer)?.get_tile(x, y)
    }

    /// Get the tile ID at (x, y) on a specific layer by index.
    pub fn tile_id_at(&self, layer: usize, x: usize, y: usize) -> Option<u32> {
        self.layers.get(layer)?.get_tile_id(x, y)
    }

    /// Set the tile at (x, y) on a specific layer by index.
    pub fn set_tile_at(&mut self, layer: usize, x: usize, y: usize, tile: Tile) {
        if let Some(l) = self.layers.get_mut(layer) {
            l.set_tile(x, y, tile);
        }
    }

    /// Set the tile ID at (x, y) on a specific layer by index.
    pub fn set_tile_id_at(&mut self, layer: usize, x: usize, y: usize, tile_id: u32) {
        if let Some(l) = self.layers.get_mut(layer) {
            l.set_tile_id(x, y, tile_id);
        }
    }

    /// Check if a position is walkable across all collision-enabled layers.
    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        for layer in &self.layers {
            if !layer.collision_enabled || !layer.visible {
                continue;
            }
            if let Some(tile) = layer.get_tile(x, y) {
                match tile.collision {
                    CollisionType::Solid => return false,
                    CollisionType::Water | CollisionType::OneWay => {} // passable
                    CollisionType::None => {}
                }
            }
        }
        true
    }

    /// Check if a world position is walkable.
    pub fn is_walkable_world(&self, world_x: f32, world_y: f32) -> bool {
        let (tx, ty) = self.world_to_tile(world_x, world_y);
        self.is_walkable(tx, ty)
    }

    /// Get the speed multiplier at a world position (minimum across all layers).
    pub fn get_speed_multiplier(&self, world_x: f32, world_y: f32) -> f32 {
        let (tx, ty) = self.world_to_tile(world_x, world_y);
        let mut min_speed = 1.0f32;
        for layer in &self.layers {
            if let Some(speed) = layer.get_tile(tx, ty).map(|t| t.speed) {
                min_speed = min_speed.min(speed);
            }
        }
        min_speed
    }

    /// Get the damage at a world position (maximum across all layers).
    pub fn get_damage(&self, world_x: f32, world_y: f32) -> i32 {
        let (tx, ty) = self.world_to_tile(world_x, world_y);
        let mut max_damage = 0i32;
        for layer in &self.layers {
            if let Some(damage) = layer.get_tile(tx, ty).map(|t| t.damage) {
                max_damage = max_damage.max(damage);
            }
        }
        max_damage
    }

    /// Get the collision type at a world position (first solid wins).
    pub fn get_collision(&self, world_x: f32, world_y: f32) -> CollisionType {
        let (tx, ty) = self.world_to_tile(world_x, world_y);
        for layer in &self.layers {
            if !layer.collision_enabled {
                continue;
            }
            let ct = layer.collision_at(tx, ty);
            if ct != CollisionType::None {
                return ct;
            }
        }
        CollisionType::None
    }

    /// World size in units.
    pub fn world_size(&self) -> Vec2 {
        Vec2::new(
            self.width as f32 * self.tile_size,
            self.height as f32 * self.tile_size,
        )
    }

    /// Convert world position to tile coordinates.
    pub fn world_to_tile(&self, world_x: f32, world_y: f32) -> (usize, usize) {
        let x = (world_x / self.tile_size).floor() as usize;
        let y = (world_y / self.tile_size).floor() as usize;
        (x.min(self.width.saturating_sub(1)), y.min(self.height.saturating_sub(1)))
    }

    /// Convert tile coordinates to world position (top-left corner).
    pub fn tile_to_world(&self, x: usize, y: usize) -> Vec2 {
        Vec2::new(x as f32 * self.tile_size, y as f32 * self.tile_size)
    }

    /// Convert tile coordinates to world center position.
    pub fn tile_to_world_center(&self, x: usize, y: usize) -> Vec2 {
        Vec2::new(
            x as f32 * self.tile_size + self.tile_size * 0.5,
            y as f32 * self.tile_size + self.tile_size * 0.5,
        )
    }

    pub fn set_palette_color(&mut self, tile_id: u32, color: Color) {
        self.palette.insert(tile_id, color);
    }

    pub fn get_tile_color(&self, tile_id: u32) -> Color {
        self.palette.get(&tile_id).copied().unwrap_or(Color::rgba(0.3, 0.3, 0.3, 1.0))
    }

    /// Sort layers by z_index for rendering.
    pub fn sort_layers(&mut self) {
        self.layers.sort_by_key(|l| l.z_index);
    }
}

impl Default for TileMap {
    fn default() -> Self {
        Self::new(32, 32, 16.0)
    }
}

// ---------------------------------------------------------------------------
// AutoTile (16-variant 4-bit bitmask)
// ---------------------------------------------------------------------------

/// Bitmask flags for auto-tile neighbor detection.
/// Cardinal directions: N=1, E=2, S=4, W=8 → 16 variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AutoTileFlags(pub u8);

impl AutoTileFlags {
    pub const EMPTY: u8 = 0;
    pub const N: u8 = 1 << 0;
    pub const E: u8 = 1 << 1;
    pub const S: u8 = 1 << 2;
    pub const W: u8 = 1 << 3;
    // Diagonal flags (for edge smoothing)
    pub const NE: u8 = 1 << 4;
    pub const SE: u8 = 1 << 5;
    pub const SW: u8 = 1 << 6;
    pub const NW: u8 = 1 << 7;

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

    /// Cardinal-only bitmask (N=1, E=2, S=4, W=8) → 16 variants.
    pub fn cardinal_bitmask(self) -> u32 {
        let mut mask = 0u32;
        if self.has(Self::N) { mask |= 1; }
        if self.has(Self::E) { mask |= 2; }
        if self.has(Self::S) { mask |= 4; }
        if self.has(Self::W) { mask |= 8; }
        mask
    }

    /// Map 4-bit cardinal bitmask to tile variant index (0..15).
    pub fn to_tile_index(self) -> u32 {
        self.cardinal_bitmask()
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

    /// Check if a tile at (x, y) matches the active terrain type.
    fn is_same_terrain(&self, layer: &MapLayer, x: usize, y: usize) -> bool {
        layer.get_tile_id(x, y) == Some(self.active_tile_id)
            || layer.get_tile(x, y).map_or(false, |t| t.id >= self.base_tile_id && t.id < self.base_tile_id + 16)
    }

    /// Compute auto-tile flags for a position on a layer.
    pub fn compute_flags(&self, layer: &MapLayer, x: usize, y: usize) -> AutoTileFlags {
        let mut flags = AutoTileFlags::new();

        // Cardinal neighbors
        if y > 0 && self.is_same_terrain(layer, x, y - 1) {
            flags = flags.with(AutoTileFlags::N);
        }
        if x + 1 < layer.width && self.is_same_terrain(layer, x + 1, y) {
            flags = flags.with(AutoTileFlags::E);
        }
        if y + 1 < layer.height && self.is_same_terrain(layer, x, y + 1) {
            flags = flags.with(AutoTileFlags::S);
        }
        if x > 0 && self.is_same_terrain(layer, x - 1, y) {
            flags = flags.with(AutoTileFlags::W);
        }

        // Diagonal neighbors (for edge smoothing — fill corners only when
        // both adjacent cardinals are present)
        if flags.has(AutoTileFlags::N) && flags.has(AutoTileFlags::E) {
            if x + 1 < layer.width && y > 0 && self.is_same_terrain(layer, x + 1, y - 1) {
                flags = flags.with(AutoTileFlags::NE);
            }
        }
        if flags.has(AutoTileFlags::E) && flags.has(AutoTileFlags::S) {
            if x + 1 < layer.width && y + 1 < layer.height && self.is_same_terrain(layer, x + 1, y + 1) {
                flags = flags.with(AutoTileFlags::SE);
            }
        }
        if flags.has(AutoTileFlags::S) && flags.has(AutoTileFlags::W) {
            if x > 0 && y + 1 < layer.height && self.is_same_terrain(layer, x - 1, y + 1) {
                flags = flags.with(AutoTileFlags::SW);
            }
        }
        if flags.has(AutoTileFlags::W) && flags.has(AutoTileFlags::N) {
            if x > 0 && y > 0 && self.is_same_terrain(layer, x - 1, y - 1) {
                flags = flags.with(AutoTileFlags::NW);
            }
        }

        flags
    }

    /// Apply auto-tiling to an entire layer.
    pub fn apply_to_layer(&self, layer: &mut MapLayer) {
        // Collect all changes first to avoid borrow issues
        let changes: Vec<(usize, usize, u32)> = {
            let mut c = Vec::new();
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let tile_id = layer.get_tile_id(x, y);
                    if tile_id == Some(self.active_tile_id) {
                        let flags = self.compute_flags(layer, x, y);
                        let tile_index = flags.to_tile_index();
                        c.push((x, y, self.base_tile_id + tile_index));
                    }
                }
            }
            c
        };
        for (x, y, new_id) in changes {
            layer.set_tile_id(x, y, new_id);
        }
    }

    /// Apply auto-tiling to a single position and its 8 neighbors.
    pub fn apply_at(&self, layer: &mut MapLayer, x: usize, y: usize) {
        let positions: Vec<(usize, usize)> = {
            let mut p = Vec::new();
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && ny >= 0 && (nx as usize) < layer.width && (ny as usize) < layer.height {
                        p.push((nx as usize, ny as usize));
                    }
                }
            }
            p
        };
        for (px, py) in positions {
            let tile_id = layer.get_tile_id(px, py);
            if tile_id == Some(self.active_tile_id) {
                let flags = self.compute_flags(layer, px, py);
                let tile_index = flags.to_tile_index();
                layer.set_tile_id(px, py, self.base_tile_id + tile_index);
            }
        }
    }

    /// Smooth transitions between two terrain types at their boundary.
    /// Sets boundary tiles of terrain_a to a transition variant if they
    /// neighbor terrain_b.
    pub fn smooth_boundary(&self, layer: &mut MapLayer, terrain_b_id: u32) {
        let changes: Vec<(usize, usize, u32)> = {
            let mut c = Vec::new();
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let tid = layer.get_tile_id(x, y);
                    if tid == Some(self.active_tile_id) || (tid.is_some() && tid.unwrap() >= self.base_tile_id && tid.unwrap() < self.base_tile_id + 16) {
                        // Check if any cardinal neighbor is terrain_b
                        let mut neighbors_b = 0u8;
                        if y > 0 && layer.get_tile_id(x, y - 1) == Some(terrain_b_id) {
                            neighbors_b |= AutoTileFlags::N;
                        }
                        if x + 1 < layer.width && layer.get_tile_id(x + 1, y) == Some(terrain_b_id) {
                            neighbors_b |= AutoTileFlags::E;
                        }
                        if y + 1 < layer.height && layer.get_tile_id(x, y + 1) == Some(terrain_b_id) {
                            neighbors_b |= AutoTileFlags::S;
                        }
                        if x > 0 && layer.get_tile_id(x - 1, y) == Some(terrain_b_id) {
                            neighbors_b |= AutoTileFlags::W;
                        }
                        if neighbors_b != 0 {
                            // Mark as boundary — use the edge variant pointing toward terrain_b
                            let boundary_id = self.base_tile_id + 16; // transition tile slot
                            c.push((x, y, boundary_id));
                        }
                    }
                }
            }
            c
        };
        for (x, y, new_id) in changes {
            layer.set_tile_id(x, y, new_id);
        }
    }
}

// ---------------------------------------------------------------------------
// FogOfWar
// ---------------------------------------------------------------------------

/// State of a fog tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FogState {
    /// Completely hidden (black overlay).
    Hidden,
    /// Previously seen but not currently visible (dim overlay).
    Explored,
    /// Currently visible (no overlay).
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

    /// Dark overlay color for rendering.
    pub fn overlay_color(&self) -> Color {
        match self {
            FogState::Hidden => Color::rgba(0.0, 0.0, 0.0, 1.0),
            FogState::Explored => Color::rgba(0.0, 0.0, 0.0, 0.5),
            FogState::Visible => Color::rgba(0.0, 0.0, 0.0, 0.0),
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

    /// Reveal tiles in a circle around (tx, ty) in tile coordinates.
    pub fn reveal_circle(&mut self, tx: i32, ty: i32, radius: i32) {
        // First pass: mark explored in the larger radius
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = tx + dx;
                let ny = ty + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < self.width && (ny as usize) < self.height {
                    if dx * dx + dy * dy <= radius * radius {
                        let x = nx as usize;
                        let y = ny as usize;
                        if self.states[y][x] != FogState::Visible {
                            self.states[y][x] = FogState::Explored;
                        }
                    }
                }
            }
        }
        // Second pass: mark visible in the inner radius
        let inner = (radius as f32 * 0.7) as i32;
        for dy in -inner..=inner {
            for dx in -inner..=inner {
                let nx = tx + dx;
                let ny = ty + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < self.width && (ny as usize) < self.height {
                    if dx * dx + dy * dy <= inner * inner {
                        self.states[ny as usize][nx as usize] = FogState::Visible;
                    }
                }
            }
        }
    }

    /// Legacy: reveal around a tile position using the stored radius.
    pub fn reveal_around(&mut self, tx: usize, ty: usize) {
        self.reveal_circle(tx as i32, ty as i32, self.visibility_radius);
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

    /// Get the fog state at (x, y).
    pub fn state_at(&self, x: usize, y: usize) -> FogState {
        self.states.get(y).and_then(|row| row.get(x)).copied().unwrap_or(FogState::Hidden)
    }

    /// Reset all fog to hidden.
    pub fn reset(&mut self) {
        for row in &mut self.states {
            for state in row.iter_mut() {
                *state = FogState::Hidden;
            }
        }
    }

    /// Get overlay color for rendering at (x, y).
    pub fn render_fog(&self, x: usize, y: usize) -> Color {
        self.state_at(x, y).overlay_color()
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
    /// Viewport indicator color.
    pub viewport_color: Color,
    /// Player marker color.
    pub player_color: Color,
    /// NPC marker color.
    pub npc_color: Color,
    /// Marker size in minimap pixels.
    pub marker_size: f32,
    /// Cached pre-rendered minimap image data (RGBA pixels).
    cached_map: Option<Vec<u8>>,
    cached_width: usize,
    cached_height: usize,
}

impl MinimapRenderer {
    pub fn new(pixel_size: f32) -> Self {
        Self {
            pixel_size,
            padding: 2.0,
            bg_color: Color::rgba(0.0, 0.0, 0.0, 0.7),
            viewport_color: Color::rgba(1.0, 1.0, 1.0, 0.8),
            player_color: Color::rgba(0.0, 1.0, 0.0, 1.0),
            npc_color: Color::rgba(1.0, 1.0, 0.0, 0.8),
            marker_size: 2.0,
            cached_map: None,
            cached_width: 0,
            cached_height: 0,
        }
    }

    /// Pre-render the full map as a cached RGBA image.
    pub fn cache_map(&mut self, tilemap: &TileMap, fog: Option<&FogOfWar>) {
        let w = (tilemap.width as f32 * self.pixel_size + self.padding * 2.0) as usize;
        let h = (tilemap.height as f32 * self.pixel_size + self.padding * 2.0) as usize;
        let mut pixels = vec![0u8; w * h * 4];

        for y in 0..tilemap.height {
            for x in 0..tilemap.width {
                // Determine color
                let color = if let Some(fog) = fog {
                    match fog.state_at(x, y) {
                        FogState::Hidden => continue,
                        FogState::Explored => Color::rgba(0.15, 0.15, 0.15, 1.0),
                        FogState::Visible => tilemap.layers.iter()
                            .filter(|l| l.visible)
                            .find_map(|l| l.get_tile_id(x, y).map(|id| tilemap.get_tile_color(id)))
                            .unwrap_or(Color::rgba(0.1, 0.1, 0.1, 1.0)),
                    }
                } else {
                    tilemap.layers.iter()
                        .filter(|l| l.visible)
                        .find_map(|l| l.get_tile_id(x, y).map(|id| tilemap.get_tile_color(id)))
                        .unwrap_or(Color::rgba(0.1, 0.1, 0.1, 1.0))
                };

                // Fill pixel block
                let px = (self.padding + x as f32 * self.pixel_size) as usize;
                let py = (self.padding + y as f32 * self.pixel_size) as usize;
                let ps = self.pixel_size as usize;
                for dy in 0..ps {
                    for dx in 0..ps {
                        let idx = ((py + dy) * w + (px + dx)) * 4;
                        if idx + 3 < pixels.len() {
                            pixels[idx] = (color.r * 255.0) as u8;
                            pixels[idx + 1] = (color.g * 255.0) as u8;
                            pixels[idx + 2] = (color.b * 255.0) as u8;
                            pixels[idx + 3] = (color.a * 255.0) as u8;
                        }
                    }
                }
            }
        }

        self.cached_map = Some(pixels);
        self.cached_width = w;
        self.cached_height = h;
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

        // Background tiles
        for y in 0..tilemap.height {
            for x in 0..tilemap.width {
                let sx = self.padding + x as f32 * self.pixel_size;
                let sy = self.padding + y as f32 * self.pixel_size;

                if let Some(fog) = fog {
                    match fog.state_at(x, y) {
                        FogState::Hidden => continue,
                        FogState::Explored => {
                            pixels.push((sx, sy, Color::rgba(0.15, 0.15, 0.15, 1.0)));
                            continue;
                        }
                        FogState::Visible => {}
                    }
                }

                let color = tilemap.layers.iter()
                    .filter(|l| l.visible)
                    .find_map(|l| l.get_tile_id(x, y).map(|id| tilemap.get_tile_color(id)))
                    .unwrap_or(Color::rgba(0.1, 0.1, 0.1, 1.0));

                pixels.push((sx, sy, color));
            }
        }

        // Camera viewport indicator
        let cam_tile_x = self.padding + (camera_pos.x / tilemap.tile_size) * self.pixel_size;
        let cam_tile_y = self.padding + (camera_pos.y / tilemap.tile_size) * self.pixel_size;
        let vp_w = (viewport_size.x / tilemap.tile_size) * self.pixel_size;
        let vp_h = (viewport_size.y / tilemap.tile_size) * self.pixel_size;

        // Draw viewport rectangle corners
        let vc = self.viewport_color;
        let corner_len = 3.0;
        // Top-left
        for i in 0..corner_len as usize {
            pixels.push((cam_tile_x + i as f32, cam_tile_y, vc));
            pixels.push((cam_tile_x, cam_tile_y + i as f32, vc));
        }
        // Top-right
        for i in 0..corner_len as usize {
            pixels.push((cam_tile_x + vp_w - i as f32, cam_tile_y, vc));
            pixels.push((cam_tile_x + vp_w, cam_tile_y + i as f32, vc));
        }
        // Bottom-left
        for i in 0..corner_len as usize {
            pixels.push((cam_tile_x + i as f32, cam_tile_y + vp_h, vc));
            pixels.push((cam_tile_x, cam_tile_y + vp_h - i as f32, vc));
        }
        // Bottom-right
        for i in 0..corner_len as usize {
            pixels.push((cam_tile_x + vp_w - i as f32, cam_tile_y + vp_h, vc));
            pixels.push((cam_tile_x + vp_w, cam_tile_y + vp_h - i as f32, vc));
        }

        pixels
    }

    /// Draw a player position marker on the minimap.
    pub fn draw_player(&self, world_pos: Vec2, tile_size: f32) -> (f32, f32, Color) {
        let sx = self.padding + (world_pos.x / tile_size) * self.pixel_size;
        let sy = self.padding + (world_pos.y / tile_size) * self.pixel_size;
        (sx, sy, self.player_color)
    }

    /// Draw an NPC position marker on the minimap.
    pub fn draw_npc(&self, world_pos: Vec2, tile_size: f32) -> (f32, f32, Color) {
        let sx = self.padding + (world_pos.x / tile_size) * self.pixel_size;
        let sy = self.padding + (world_pos.y / tile_size) * self.pixel_size;
        (sx, sy, self.npc_color)
    }

    /// Draw multiple NPC markers.
    pub fn draw_npcs(&self, positions: &[Vec2], tile_size: f32) -> Vec<(f32, f32, Color)> {
        positions.iter().map(|p| self.draw_npc(*p, tile_size)).collect()
    }

    /// Convert a minimap screen position back to world coordinates.
    /// Returns None if the click is outside the minimap area.
    pub fn minimap_click_to_world(
        &self,
        minimap_origin: Vec2,
        click_screen: Vec2,
        tile_size: f32,
    ) -> Option<Vec2> {
        let local_x = click_screen.x - minimap_origin.x - self.padding;
        let local_y = click_screen.y - minimap_origin.y - self.padding;
        if local_x < 0.0 || local_y < 0.0 {
            return None;
        }
        let world_x = (local_x / self.pixel_size) * tile_size;
        let world_y = (local_y / self.pixel_size) * tile_size;
        Some(Vec2::new(world_x, world_y))
    }

    /// World size of the minimap in screen units.
    pub fn minimap_size(&self, tilemap: &TileMap) -> Vec2 {
        Vec2::new(
            tilemap.width as f32 * self.pixel_size + self.padding * 2.0,
            tilemap.height as f32 * self.pixel_size + self.padding * 2.0,
        )
    }

    /// Get cached RGBA pixel data if available.
    pub fn cached_pixels(&self) -> Option<(&[u8], usize, usize)> {
        self.cached_map.as_deref().map(|p| (p, self.cached_width, self.cached_height))
    }

    /// Invalidate the cached map (call when map changes).
    pub fn invalidate_cache(&mut self) {
        self.cached_map = None;
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
        let t = Tile::new(5)
            .with_collision(CollisionType::Solid)
            .with_damage(10)
            .with_speed(0.5);
        assert_eq!(t.id, 5);
        assert_eq!(t.collision, CollisionType::Solid);
        assert_eq!(t.damage, 10);
        assert!((t.speed - 0.5).abs() < 0.01);
        assert!(t.is_solid());
    }

    #[test]
    fn test_tile_property() {
        let t = Tile::new(1)
            .with_property("key", TileProperty::Float(0.5))
            .with_property("water", TileProperty::Bool(true));
        assert!(matches!(t.get_property("water"), Some(TileProperty::Bool(true))));
    }

    #[test]
    fn test_tile_defaults() {
        let t = Tile::default();
        assert_eq!(t.id, 0);
        assert_eq!(t.collision, CollisionType::None);
        assert_eq!(t.damage, 0);
        assert!((t.speed - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_map_layer() {
        let mut layer = MapLayer::new("ground", 4, 4);
        let tile = Tile::new(42).with_collision(CollisionType::Solid);
        layer.set_tile(1, 2, tile);
        assert_eq!(layer.get_tile_id(1, 2), Some(42));
        assert_eq!(layer.get_tile_id(0, 0), Some(0));
        assert!(!layer.is_empty());
        assert_eq!(layer.collision_at(1, 2), CollisionType::Solid);
        assert_eq!(layer.collision_at(0, 0), CollisionType::None);
    }

    #[test]
    fn test_map_layer_speed_damage() {
        let mut layer = MapLayer::new("hazards", 4, 4);
        let tile = Tile::new(1).with_speed(0.3).with_damage(5);
        layer.set_tile(2, 2, tile);
        assert!((layer.speed_at(2, 2) - 0.3).abs() < 0.01);
        assert_eq!(layer.damage_at(2, 2), 5);
        assert!((layer.speed_at(0, 0) - 1.0).abs() < 0.01);
        assert_eq!(layer.damage_at(0, 0), 0);
    }

    #[test]
    fn test_tilemap_basics() {
        let mut map = TileMap::new(8, 8, 16.0);
        map.add_layer(MapLayer::new("ground", 8, 8));
        assert_eq!(map.layers.len(), 1);
        let tile = Tile::new(99);
        map.set_tile_at(0, 3, 4, tile);
        assert_eq!(map.tile_id_at(0, 3, 4), Some(99));
    }

    #[test]
    fn test_tilemap_walkability() {
        let mut map = TileMap::new(4, 4, 16.0);
        let mut layer = MapLayer::new("collision", 4, 4);
        layer.set_tile(2, 2, Tile::new(1).with_collision(CollisionType::Solid));
        map.add_layer(layer);
        assert!(map.is_walkable(0, 0));
        assert!(!map.is_walkable(2, 2));
    }

    #[test]
    fn test_tilemap_speed_damage_world() {
        let mut map = TileMap::new(4, 4, 16.0);
        let mut layer = MapLayer::new("gameplay", 4, 4);
        layer.set_tile(1, 1, Tile::new(1).with_speed(0.2).with_damage(10));
        map.add_layer(layer);
        let wx = 1.0 * 16.0 + 8.0;
        let wy = 1.0 * 16.0 + 8.0;
        assert!((map.get_speed_multiplier(wx, wy) - 0.2).abs() < 0.01);
        assert_eq!(map.get_damage(wx, wy), 10);
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
        let (tx, ty) = map.world_to_tile(50.0, 80.0);
        let back = map.tile_to_world(tx, ty);
        assert!((back.x - 48.0).abs() < 0.01);
        assert!((back.y - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_autotile_flags() {
        let f = AutoTileFlags(AutoTileFlags::N | AutoTileFlags::E);
        assert!(f.has(AutoTileFlags::N));
        assert!(f.has(AutoTileFlags::E));
        assert!(!f.has(AutoTileFlags::S));
        assert_eq!(f.to_tile_index(), 3); // N=1 + E=2 = 3
    }

    #[test]
    fn test_autotile_16_variants() {
        // All 16 cardinal combinations
        let cases: Vec<(u8, u32)> = vec![
            (0, 0),       // isolated
            (1, 1),       // N only
            (2, 2),       // E only
            (4, 4),       // S only
            (8, 8),       // W only
            (3, 3),       // N+E
            (6, 6),       // E+S
            (12, 12),     // S+W
            (9, 9),       // N+W
            (5, 5),       // N+S
            (10, 10),     // E+W
            (7, 7),       // N+E+S
            (11, 11),     // N+E+W
            (13, 13),     // N+S+W
            (14, 14),     // E+S+W
            (15, 15),     // all
        ];
        for (bits, expected) in cases {
            let f = AutoTileFlags(bits);
            assert_eq!(f.to_tile_index(), expected, "bitmask {bits} should map to {expected}");
        }
    }

    #[test]
    fn test_autotile_apply() {
        let mut layer = MapLayer::new("terrain", 5, 5);
        // Fill a 3x3 block in the center with the active tile
        for y in 1..4 {
            for x in 1..4 {
                layer.set_tile_id(x, y, 100); // active_tile_id
            }
        }
        let system = AutoTileSystem::new(100, 200);
        system.apply_to_layer(&mut layer);
        // Center tile should have all 4 neighbors → index 15
        assert_eq!(layer.get_tile_id(2, 2), Some(215));
        // Corner tile should have 2 neighbors
        assert_eq!(layer.get_tile_id(1, 1), Some(206)); // E+S → 6
    }

    #[test]
    fn test_fog_of_war() {
        let mut fog = FogOfWar::new(16, 16);
        fog.visibility_radius = 2;
        fog.reveal_around(5, 5);
        assert!(fog.is_visible(5, 5));
        assert!(fog.is_visible(4, 5));
        assert!(!fog.is_visible(0, 0));
        fog.fade_visible();
        assert!(!fog.is_visible(5, 5));
        assert!(fog.is_explored(5, 5));
    }

    #[test]
    fn test_fog_reveal_circle() {
        let mut fog = FogOfWar::new(16, 16);
        fog.reveal_circle(8, 8, 3);
        assert!(fog.is_visible(8, 8));
        assert!(fog.is_visible(7, 8));
        assert!(!fog.is_visible(0, 0));
    }

    #[test]
    fn test_fog_render_color() {
        let fog = FogOfWar::new(4, 4);
        let c = fog.render_fog(0, 0);
        assert!((c.a - 1.0).abs() < 0.01); // Hidden → full alpha
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
        assert_eq!(size.x, 20.0);
        assert_eq!(size.y, 20.0);
    }

    #[test]
    fn test_minimap_player_marker() {
        let renderer = MinimapRenderer::new(2.0);
        let (sx, sy, color) = renderer.draw_player(Vec2::new(32.0, 48.0), 16.0);
        assert!((sx - 6.0).abs() < 0.01); // 32/16 * 2 + 2 = 6
        assert!((sy - 8.0).abs() < 0.01); // 48/16 * 2 + 2 = 8
        assert_eq!(color, renderer.player_color);
    }

    #[test]
    fn test_minimap_click_to_world() {
        let renderer = MinimapRenderer::new(2.0);
        let origin = Vec2::new(100.0, 50.0);
        // Click at minimap tile (3, 5) → world (3*16, 5*16) = (48, 80)
        let click = Vec2::new(100.0 + 2.0 + 3.0 * 2.0, 50.0 + 2.0 + 5.0 * 2.0);
        let world = renderer.minimap_click_to_world(origin, click, 16.0).unwrap();
        assert!((world.x - 48.0).abs() < 0.1);
        assert!((world.y - 80.0).abs() < 0.1);
    }

    #[test]
    fn test_minimap_click_outside() {
        let renderer = MinimapRenderer::new(2.0);
        let origin = Vec2::new(100.0, 50.0);
        let click = Vec2::new(50.0, 50.0); // outside
        assert!(renderer.minimap_click_to_world(origin, click, 16.0).is_none());
    }

    #[test]
    fn test_tilemap_palette() {
        let mut map = TileMap::new(4, 4, 16.0);
        map.set_palette_color(1, Color::rgba(1.0, 0.0, 0.0, 1.0));
        let c = map.get_tile_color(1);
        assert!((c.r - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_collision_types() {
        assert_eq!(CollisionType::default(), CollisionType::None);
        let t = Tile::new(1);
        assert!(!t.is_solid());
        assert!(!t.is_one_way());
        assert!(!t.is_water());

        let solid = Tile::new(2).with_collision(CollisionType::Solid);
        assert!(solid.is_solid());

        let water = Tile::new(3).with_collision(CollisionType::Water);
        assert!(water.is_water());
    }
}

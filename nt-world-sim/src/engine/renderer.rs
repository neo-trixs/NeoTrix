use std::collections::HashMap;
use crate::core::world::Component;

// Re-export math types from core — single fact source.
pub use crate::core::{Vec2, Rect, Color, Transform};

// ---------------------------------------------------------------------------
// Sprite — renderable unit with atlas, flip, rotation, alpha
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub texture_id: String,
    pub frame: u32,
    pub flip_x: bool,
    pub flip_y: bool,
    pub color: Color,
    pub alpha: f32,
    pub rotation: f32,
    pub z_index: i32,
}

impl Sprite {
    pub fn new(texture_id: &str) -> Self {
        Self {
            x: 0.0, y: 0.0, width: 16.0, height: 16.0,
            texture_id: texture_id.to_string(),
            frame: 0, flip_x: false, flip_y: false,
            color: Color::white(), alpha: 1.0, rotation: 0.0, z_index: 0,
        }
    }

    pub fn from_rect(texture_id: &str, x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, width: w, height: h, texture_id: texture_id.to_string(), ..Self::new(texture_id) }
    }

    pub fn colored(color: Color, w: f32, h: f32) -> Self {
        Self { color, width: w, height: h, texture_id: String::new(), ..Self::new("") }
    }

    pub fn with_position(mut self, x: f32, y: f32) -> Self { self.x = x; self.y = y; self }
    pub fn with_frame(mut self, frame: u32) -> Self { self.frame = frame; self }
    pub fn with_flip(mut self, flip_x: bool, flip_y: bool) -> Self { self.flip_x = flip_x; self.flip_y = flip_y; self }
    pub fn with_alpha(mut self, alpha: f32) -> Self { self.alpha = alpha; self }
    pub fn with_rotation(mut self, rotation: f32) -> Self { self.rotation = rotation; self }
    pub fn with_z_index(mut self, z: i32) -> Self { self.z_index = z; self }

    pub fn world_rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }
}

// ---------------------------------------------------------------------------
// TextureAtlas — maps texture IDs to regions within an atlas texture
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AtlasRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct TextureAtlas {
    pub texture_path: String,
    pub regions: HashMap<String, AtlasRegion>,
    pub tile_size: f32,
    pub columns: u32,
    pub total_frames: u32,
}

impl TextureAtlas {
    pub fn new(texture_path: &str, tile_size: f32, columns: u32, total_frames: u32) -> Self {
        Self {
            texture_path: texture_path.to_string(),
            regions: HashMap::new(),
            tile_size, columns, total_frames,
        }
    }

    pub fn add_region(&mut self, name: &str, region: AtlasRegion) {
        self.regions.insert(name.to_string(), region);
    }

    pub fn auto_grid(&mut self) {
        let rows = (self.total_frames + self.columns - 1) / self.columns;
        for i in 0..self.total_frames {
            let col = i % self.columns;
            let row = i / self.columns;
            let name = format!("frame_{}", i);
            self.regions.insert(name, AtlasRegion {
                x: col as f32 * self.tile_size,
                y: row as f32 * self.tile_size,
                width: self.tile_size,
                height: self.tile_size,
            });
        }
    }

    pub fn get_region(&self, name: &str) -> Option<&AtlasRegion> {
        self.regions.get(name)
    }

    pub fn get_frame_region(&self, frame: u32) -> Option<&AtlasRegion> {
        self.regions.get(&format!("frame_{}", frame))
    }

    pub fn sprite_rect(&self, frame: u32) -> Rect {
        self.get_frame_region(frame)
            .map(|r| Rect::new(r.x, r.y, r.width, r.height))
            .unwrap_or(Rect::new(0.0, 0.0, self.tile_size, self.tile_size))
    }
}

impl Default for TextureAtlas {
    fn default() -> Self { Self::new("", 16.0, 0, 0) }
}

// ---------------------------------------------------------------------------
// TileDef — tile palette entry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TileDef {
    pub id: u32,
    pub texture: Option<String>,
    pub color: Color,
    pub walkable: bool,
}

impl TileDef {
    pub fn new(id: u32, color: Color, walkable: bool) -> Self {
        Self { id, texture: None, color, walkable }
    }
}

// ---------------------------------------------------------------------------
// TileMap — simple single-layer tile map
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TileMap {
    pub tiles: Vec<Vec<u32>>,
    pub tile_size: Vec2,
    pub palette: HashMap<u32, TileDef>,
}

impl TileMap {
    pub fn new(width: usize, height: usize, tile_size: Vec2) -> Self {
        Self { tiles: vec![vec![0; width]; height], tile_size, palette: HashMap::new() }
    }
    pub fn get_tile(&self, x: usize, y: usize) -> Option<u32> { self.tiles.get(y)?.get(x).copied() }
    pub fn set_tile(&mut self, x: usize, y: usize, tile_id: u32) {
        if let Some(row) = self.tiles.get_mut(y) {
            if let Some(tile) = row.get_mut(x) { *tile = tile_id; }
        }
    }
    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        if let Some(tile_id) = self.get_tile(x, y) {
            if let Some(td) = self.palette.get(&tile_id) { return td.walkable; }
        }
        false
    }
    pub fn width(&self) -> usize { self.tiles.first().map_or(0, |r| r.len()) }
    pub fn height(&self) -> usize { self.tiles.len() }
    pub fn world_size(&self) -> Vec2 {
        Vec2::new(self.width() as f32 * self.tile_size.x, self.height() as f32 * self.tile_size.y)
    }
}

// ---------------------------------------------------------------------------
// Camera — simple 2D camera
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec2,
    pub zoom: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

impl Camera {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self { position: Vec2::zero(), zoom: 1.0, viewport_width, viewport_height }
    }
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        Vec2::new(
            (world_pos.x - self.position.x) * self.zoom + self.viewport_width / 2.0,
            (world_pos.y - self.position.y) * self.zoom + self.viewport_height / 2.0,
        )
    }
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        Vec2::new(
            (screen_pos.x - self.viewport_width / 2.0) / self.zoom + self.position.x,
            (screen_pos.y - self.viewport_height / 2.0) / self.zoom + self.position.y,
        )
    }
    pub fn visible_world_rect(&self) -> Rect {
        let hw = self.viewport_width / (2.0 * self.zoom);
        let hh = self.viewport_height / (2.0 * self.zoom);
        Rect::new(self.position.x - hw, self.position.y - hh, hw * 2.0, hh * 2.0)
    }
}

// ---------------------------------------------------------------------------
// Draw commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum DrawCommand {
    Clear { color: Color },
    DrawRect { rect: Rect, color: Color },
    DrawCircle { center: Vec2, radius: f32, color: Color },
    DrawLine { start: Vec2, end: Vec2, color: Color, width: f32 },
    DrawText { text: String, position: Vec2, color: Color, size: f32 },
    DrawSprite { texture: String, dest: Rect, src_rect: Option<Rect>, color: Color, alpha: f32, flip_x: bool, flip_y: bool, rotation: f32, z_index: i32 },
    DrawTilemap { tile_colors: Vec<(Rect, Color)>, z_index: i32 },
    DrawQuad { dest: Rect, color: Color, rotation: f32, z_index: i32 },
    DrawParticles { particles: Vec<ParticleDrawVertex> },
    Present,
}

#[derive(Debug, Clone, Copy)]
pub struct ParticleDrawVertex {
    pub position: Vec2,
    pub color: Color,
    pub size: f32,
}

// ---------------------------------------------------------------------------
// Renderer trait
// ---------------------------------------------------------------------------

pub trait Renderer {
    fn clear(&mut self, color: Color);
    fn draw_sprite(&mut self, sprite: &Sprite, transform: &Transform);
    fn draw_tilemap(&mut self, tilemap: &TileMap, camera: &Camera);
    fn draw_text(&mut self, text: &str, position: Vec2, color: Color, size: f32);
    fn draw_rect(&mut self, rect: &Rect, color: Color);
    fn draw_circle(&mut self, center: Vec2, radius: f32, color: Color);
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: Color, width: f32);
    fn present(&mut self);
    fn viewport_size(&self) -> Vec2;
}

// ---------------------------------------------------------------------------
// CanvasRenderer — records DrawCommands for a backend
// ---------------------------------------------------------------------------

pub struct CanvasRenderer {
    width: f32,
    height: f32,
    commands: Vec<DrawCommand>,
    frame_count: u64,
}

impl CanvasRenderer {
    pub fn new(width: f32, height: f32) -> Self { Self { width, height, commands: Vec::new(), frame_count: 0 } }
    pub fn drain_commands(&mut self) -> Vec<DrawCommand> { std::mem::take(&mut self.commands) }
    pub fn commands(&self) -> &[DrawCommand] { &self.commands }
    pub fn command_count(&self) -> usize { self.commands.len() }
    pub fn frame_count(&self) -> u64 { self.frame_count }
}

impl Renderer for CanvasRenderer {
    fn clear(&mut self, color: Color) { self.commands.push(DrawCommand::Clear { color }); }
    fn draw_sprite(&mut self, sprite: &Sprite, transform: &Transform) {
        let dest = Rect::new(
            transform.position.x + sprite.x,
            transform.position.y + sprite.y,
            sprite.width * transform.scale.x,
            sprite.height * transform.scale.y,
        );
        self.commands.push(DrawCommand::DrawSprite {
            texture: sprite.texture_id.clone(), dest, src_rect: None,
            color: sprite.color, alpha: sprite.alpha, flip_x: sprite.flip_x, flip_y: sprite.flip_y,
            rotation: sprite.rotation, z_index: sprite.z_index,
        });
    }
    fn draw_tilemap(&mut self, tilemap: &TileMap, camera: &Camera) {
        let mut tile_colors = Vec::new();
        for (y, row) in tilemap.tiles.iter().enumerate() {
            for (x, &tile_id) in row.iter().enumerate() {
                if let Some(td) = tilemap.palette.get(&tile_id) {
                    let wp = Vec2::new(x as f32 * tilemap.tile_size.x, y as f32 * tilemap.tile_size.y);
                    let sp = camera.world_to_screen(wp);
                    tile_colors.push((Rect::new(sp.x, sp.y, tilemap.tile_size.x * camera.zoom, tilemap.tile_size.y * camera.zoom), td.color));
                }
            }
        }
        self.commands.push(DrawCommand::DrawTilemap { tile_colors, z_index: 0 });
    }
    fn draw_text(&mut self, text: &str, position: Vec2, color: Color, size: f32) {
        self.commands.push(DrawCommand::DrawText { text: text.to_string(), position, color, size });
    }
    fn draw_rect(&mut self, rect: &Rect, color: Color) { self.commands.push(DrawCommand::DrawRect { rect: *rect, color }); }
    fn draw_circle(&mut self, center: Vec2, radius: f32, color: Color) { self.commands.push(DrawCommand::DrawCircle { center, radius, color }); }
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: Color, width: f32) { self.commands.push(DrawCommand::DrawLine { start, end, color, width }); }
    fn present(&mut self) { self.commands.push(DrawCommand::Present); self.frame_count += 1; }
    fn viewport_size(&self) -> Vec2 { Vec2::new(self.width, self.height) }
}

// ---------------------------------------------------------------------------
// SpriteBatch — groups sprites by texture for batched draw calls
// ---------------------------------------------------------------------------

pub struct SpriteInstance {
    pub dest: Rect,
    pub src_rect: Option<Rect>,
    pub color: Color,
    pub alpha: f32,
    pub flip_x: bool,
    pub flip_y: bool,
    pub rotation: f32,
    pub z_index: i32,
}

pub struct SpriteBatch {
    batches: HashMap<String, Vec<SpriteInstance>>,
    draw_order: Vec<String>,
    atlas: Option<TextureAtlas>,
}

impl SpriteBatch {
    pub fn new() -> Self {
        Self { batches: HashMap::new(), draw_order: Vec::new(), atlas: None }
    }

    pub fn with_atlas(atlas: TextureAtlas) -> Self {
        Self { atlas: Some(atlas), ..Self::new() }
    }

    pub fn set_atlas(&mut self, atlas: TextureAtlas) { self.atlas = Some(atlas); }

    pub fn atlas(&self) -> Option<&TextureAtlas> { self.atlas.as_ref() }

    pub fn begin(&mut self) {
        self.batches.clear();
        self.draw_order.clear();
    }

    /// Add a sprite by texture name. If an atlas is set, uses its region as src_rect.
    pub fn add_sprite(&mut self, texture: &str, dest: Rect, color: Color, z_index: i32) {
        let src = self.atlas.as_ref().and_then(|a| a.get_region(texture)).map(|r| {
            Rect::new(r.x, r.y, r.width, r.height)
        });
        let key = self.atlas.as_ref().map(|a| a.texture_path.clone()).unwrap_or_else(|| texture.to_string());
        self.push_instance(&key, SpriteInstance {
            dest, src_rect: src, color, alpha: 1.0,
            flip_x: false, flip_y: false, rotation: 0.0, z_index,
        });
    }

    /// Add a sprite from the atlas by frame index.
    pub fn add_sprite_frame(&mut self, frame: u32, dest: Rect, color: Color, alpha: f32, z_index: i32) {
        if let Some(atlas) = &self.atlas {
            let src = atlas.get_frame_region(frame).map(|r| Rect::new(r.x, r.y, r.width, r.height));
            self.push_instance(&atlas.texture_path, SpriteInstance {
                dest, src_rect: src, color, alpha,
                flip_x: false, flip_y: false, rotation: 0.0, z_index,
            });
        }
    }

    /// Add a full Sprite struct.
    pub fn add_sprite_struct(&mut self, sprite: &Sprite) {
        let src = self.atlas.as_ref().and_then(|a| a.get_frame_region(sprite.frame)).map(|r| {
            Rect::new(r.x, r.y, r.width, r.height)
        });
        let key = if !sprite.texture_id.is_empty() {
            self.atlas.as_ref().map(|a| a.texture_path.clone()).unwrap_or_else(|| sprite.texture_id.clone())
        } else {
            String::from("__solid__")
        };
        self.push_instance(&key, SpriteInstance {
            dest: Rect::new(sprite.x, sprite.y, sprite.width, sprite.height),
            src_rect: src,
            color: sprite.color,
            alpha: sprite.alpha,
            flip_x: sprite.flip_x,
            flip_y: sprite.flip_y,
            rotation: sprite.rotation,
            z_index: sprite.z_index,
        });
    }

    pub fn add_quad(&mut self, dest: Rect, color: Color, z_index: i32) {
        self.push_instance("__solid__", SpriteInstance {
            dest, src_rect: None, color, alpha: 1.0,
            flip_x: false, flip_y: false, rotation: 0.0, z_index,
        });
    }

    pub fn add_quad_alpha(&mut self, dest: Rect, color: Color, alpha: f32, z_index: i32) {
        self.push_instance("__solid__", SpriteInstance {
            dest, src_rect: None, color, alpha,
            flip_x: false, flip_y: false, rotation: 0.0, z_index,
        });
    }

    fn push_instance(&mut self, key: &str, instance: SpriteInstance) {
        let k = key.to_string();
        if !self.batches.contains_key(&k) {
            self.draw_order.push(k.clone());
            self.batches.insert(k.clone(), Vec::new());
        }
        self.batches.get_mut(&k).unwrap().push(instance);
    }

    pub fn end(&self) -> Vec<DrawCommand> {
        let mut commands = Vec::new();
        for tex in &self.draw_order {
            if let Some(sprites) = self.batches.get(tex) {
                for s in sprites {
                    commands.push(DrawCommand::DrawSprite {
                        texture: tex.clone(), dest: s.dest, src_rect: s.src_rect,
                        color: s.color, alpha: s.alpha, flip_x: s.flip_x, flip_y: s.flip_y,
                        rotation: s.rotation, z_index: s.z_index,
                    });
                }
            }
        }
        commands
    }

    pub fn total_sprites(&self) -> usize { self.batches.values().map(|v| v.len()).sum() }
    pub fn batch_count(&self) -> usize { self.batches.len() }
}

impl Default for SpriteBatch { fn default() -> Self { Self::new() } }

// ---------------------------------------------------------------------------
// TilemapRenderer — multi-layer, z-ordered, animated tile rendering
// ---------------------------------------------------------------------------

/// Tile animation definition: cycles through tile IDs over time.
#[derive(Debug, Clone)]
pub struct TileAnimation {
    pub frames: Vec<u32>,
    pub speed: f32,
    pub timer: f32,
    pub looping: bool,
}

impl TileAnimation {
    pub fn new(frames: Vec<u32>, speed: f32) -> Self {
        Self { frames, speed, timer: 0.0, looping: true }
    }

    pub fn current_tile(&self) -> u32 {
        let idx = (self.timer * self.speed) as usize % self.frames.len();
        self.frames[idx]
    }

    pub fn update(&mut self, dt: f32) {
        self.timer += dt;
        if self.looping {
            let total = self.frames.len() as f32 / self.speed;
            if self.timer >= total { self.timer -= total; }
        }
    }

    pub fn is_done(&self) -> bool {
        !self.looping && self.timer * self.speed >= self.frames.len() as f32
    }
}

/// Layer descriptor for multi-layer tilemap rendering.
#[derive(Debug, Clone)]
pub struct TileLayer {
    pub name: String,
    pub tiles: Vec<Vec<u32>>,
    pub z_index: i32,
    pub opacity: f32,
    pub visible: bool,
}

impl TileLayer {
    pub fn new(name: &str, width: usize, height: usize, z_index: i32) -> Self {
        Self {
            name: name.to_string(),
            tiles: vec![vec![0; width]; height],
            z_index, opacity: 1.0, visible: true,
        }
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<u32> {
        self.tiles.get(y)?.get(x).copied()
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile_id: u32) {
        if let Some(row) = self.tiles.get_mut(y) {
            if let Some(t) = row.get_mut(x) { *t = tile_id; }
        }
    }

    pub fn width(&self) -> usize { self.tiles.first().map_or(0, |r| r.len()) }
    pub fn height(&self) -> usize { self.tiles.len() }
}

pub struct TilemapRenderer {
    pub chunk_size: usize,
    visible_chunks: Vec<(usize, usize)>,
    pub animations: HashMap<(usize, usize, usize), TileAnimation>,
}

impl TilemapRenderer {
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size, visible_chunks: Vec::new(), animations: HashMap::new() }
    }

    /// Register an animation for layer_index, tile_x, tile_y.
    pub fn set_animation(&mut self, layer: usize, x: usize, y: usize, anim: TileAnimation) {
        self.animations.insert((layer, x, y), anim);
    }

    pub fn update_animations(&mut self, dt: f32) {
        for anim in self.animations.values_mut() {
            anim.update(dt);
        }
    }

    /// Compute visible tile ranges (chunk-culled) for a single layer.
    pub fn compute_visible_chunks(&mut self, layer_width: usize, layer_height: usize, tile_size: Vec2, camera: &Camera) {
        self.visible_chunks.clear();
        let cam_rect = camera.visible_world_rect();
        let ts = tile_size;
        let start_x = ((cam_rect.x / ts.x).floor() as usize).max(0);
        let start_y = ((cam_rect.y / ts.y).floor() as usize).max(0);
        let end_x = ((cam_rect.right() / ts.x).ceil() as usize).min(layer_width);
        let end_y = ((cam_rect.bottom() / ts.y).ceil() as usize).min(layer_height);

        for cy in (start_y..end_y).step_by(self.chunk_size) {
            for cx in (start_x..end_x).step_by(self.chunk_size) {
                self.visible_chunks.push((cx, cy));
            }
        }
    }

    /// Render a single layer back-to-front within the viewport.
    pub fn render_layer(&self, layer: &TileLayer, palette: &HashMap<u32, TileDef>, tile_size: Vec2, camera: &Camera) -> Vec<DrawCommand> {
        let mut commands = Vec::new();
        if !layer.visible { return commands; }

        for &(cx, cy) in &self.visible_chunks {
            let end_x = (cx + self.chunk_size).min(layer.width());
            let end_y = (cy + self.chunk_size).min(layer.height());
            for y in cy..end_y {
                for x in cx..end_x {
                    if let Some(tile_id) = layer.get_tile(x, y) {
                        if tile_id == 0 { continue; }
                        if let Some(td) = palette.get(&tile_id) {
                            let wp = Vec2::new(x as f32 * tile_size.x, y as f32 * tile_size.y);
                            let sp = camera.world_to_screen(wp);
                            let alpha = if layer.opacity < 1.0 { layer.opacity } else { 1.0 };
                            let color = if alpha < 1.0 { td.color.with_alpha(alpha) } else { td.color };
                            commands.push(DrawCommand::DrawRect {
                                rect: Rect::new(sp.x, sp.y, tile_size.x * camera.zoom, tile_size.y * camera.zoom),
                                color,
                            });
                        }
                    }
                }
            }
        }
        commands
    }

    /// Render all layers sorted by z_index.
    pub fn render_all_layers(&self, layers: &[TileLayer], palette: &HashMap<u32, TileDef>, tile_size: Vec2, camera: &Camera) -> Vec<DrawCommand> {
        let mut sorted: Vec<&TileLayer> = layers.iter().filter(|l| l.visible).collect();
        sorted.sort_by_key(|l| l.z_index);

        let mut all_cmds = Vec::new();
        for layer in sorted {
            all_cmds.extend(self.render_layer(layer, palette, tile_size, camera));
        }
        all_cmds
    }

    /// Legacy: render simple TileMap.
    pub fn render_tilemap(&self, tilemap: &TileMap, camera: &Camera) -> Vec<DrawCommand> {
        let mut commands = Vec::new();
        let ts = tilemap.tile_size;
        for &(cx, cy) in &self.visible_chunks {
            let end_x = (cx + self.chunk_size).min(tilemap.width());
            let end_y = (cy + self.chunk_size).min(tilemap.height());
            for y in cy..end_y {
                for x in cx..end_x {
                    if let Some(tile_id) = tilemap.get_tile(x, y) {
                        if let Some(td) = tilemap.palette.get(&tile_id) {
                            let wp = Vec2::new(x as f32 * ts.x, y as f32 * ts.y);
                            let sp = camera.world_to_screen(wp);
                            commands.push(DrawCommand::DrawRect {
                                rect: Rect::new(sp.x, sp.y, ts.x * camera.zoom, ts.y * camera.zoom),
                                color: td.color,
                            });
                        }
                    }
                }
            }
        }
        commands
    }
}

impl Default for TilemapRenderer { fn default() -> Self { Self::new(16) } }

// ---------------------------------------------------------------------------
// ParticleRenderer — position + size + color particle system
// ---------------------------------------------------------------------------

pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub color: Color,
    pub size: f32,
    pub life: f32,
    pub max_life: f32,
}

pub struct ParticleSystem {
    pub particles: Vec<Particle>,
    pub gravity: Vec2,
    pub max_particles: usize,
}

impl ParticleSystem {
    pub fn new(max_particles: usize) -> Self {
        Self { particles: Vec::with_capacity(max_particles), gravity: Vec2::new(0.0, 200.0), max_particles }
    }

    pub fn emit(&mut self, position: Vec2, velocity: Vec2, color: Color, size: f32, life: f32) {
        if self.particles.len() < self.max_particles {
            self.particles.push(Particle { position, velocity, color, size, life, max_life: life });
        }
    }

    pub fn emit_burst(&mut self, position: Vec2, count: u32, speed: f32, color: Color, size: f32, life: f32) {
        for i in 0..count {
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
            let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);
            self.emit(position, vel, color, size, life);
        }
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.velocity.x += self.gravity.x * dt;
            p.velocity.y += self.gravity.y * dt;
            p.position.x += p.velocity.x * dt;
            p.position.y += p.velocity.y * dt;
            p.life -= dt;
            p.size *= 1.0 - dt;
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    pub fn render(&self, camera: &Camera) -> Vec<ParticleDrawVertex> {
        self.particles
            .iter()
            .map(|p| {
                let sp = camera.world_to_screen(p.position);
                let t = (p.life / p.max_life).clamp(0.0, 1.0);
                ParticleDrawVertex {
                    position: sp,
                    color: p.color.with_alpha(t),
                    size: p.size * camera.zoom,
                }
            })
            .collect()
    }

    pub fn active_count(&self) -> usize { self.particles.len() }
}

impl Default for ParticleSystem { fn default() -> Self { Self::new(1024) } }

// ---------------------------------------------------------------------------
// DebugRenderer — grid, collision boxes, text overlay, FPS counter
// ---------------------------------------------------------------------------

pub struct DebugRenderer {
    pub show_grid: bool,
    pub grid_color: Color,
    pub collision_color: Color,
    pub text_overlays: Vec<(String, Vec2, Color, f32)>,
    frame_times: Vec<f32>,
    frame_idx: usize,
}

impl DebugRenderer {
    pub fn new() -> Self {
        Self {
            show_grid: false,
            grid_color: Color::rgba(0.3, 0.3, 0.3, 0.5),
            collision_color: Color::rgba(0.0, 1.0, 0.0, 0.6),
            text_overlays: Vec::new(),
            frame_times: Vec::with_capacity(60),
            frame_idx: 0,
        }
    }

    pub fn record_frame_time(&mut self, dt: f32) {
        if self.frame_times.len() < 60 {
            self.frame_times.push(dt);
        } else {
            self.frame_times[self.frame_idx % 60] = dt;
        }
        self.frame_idx += 1;
    }

    pub fn avg_fps(&self) -> f32 {
        if self.frame_times.is_empty() { return 0.0; }
        let avg = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        if avg > 0.0 { 1.0 / avg } else { 0.0 }
    }

    pub fn add_text_overlay(&mut self, text: &str, position: Vec2, color: Color, size: f32) {
        self.text_overlays.push((text.to_string(), position, color, size));
    }

    pub fn render_grid(&self, camera: &Camera, cell_size: f32) -> Vec<DrawCommand> {
        if !self.show_grid { return Vec::new(); }
        let mut cmds = Vec::new();
        let rect = camera.visible_world_rect();
        let start_x = (rect.x / cell_size).floor() * cell_size;
        let start_y = (rect.y / cell_size).floor() * cell_size;
        let mut x = start_x;
        while x <= rect.right() {
            let sp0 = camera.world_to_screen(Vec2::new(x, rect.y));
            let sp1 = camera.world_to_screen(Vec2::new(x, rect.bottom()));
            cmds.push(DrawCommand::DrawLine { start: sp0, end: sp1, color: self.grid_color, width: 1.0 });
            x += cell_size;
        }
        let mut y = start_y;
        while y <= rect.bottom() {
            let sp0 = camera.world_to_screen(Vec2::new(rect.x, y));
            let sp1 = camera.world_to_screen(Vec2::new(rect.right(), y));
            cmds.push(DrawCommand::DrawLine { start: sp0, end: sp1, color: self.grid_color, width: 1.0 });
            y += cell_size;
        }
        cmds
    }

    pub fn render_collision_box(&self, camera: &Camera, rect: &Rect) -> Vec<DrawCommand> {
        let sp = camera.world_to_screen(Vec2::new(rect.x, rect.y));
        let screen_rect = Rect::new(sp.x, sp.y, rect.width * camera.zoom, rect.height * camera.zoom);
        vec![DrawCommand::DrawRect { rect: screen_rect, color: self.collision_color }]
    }

    pub fn render_text_overlays(&self) -> Vec<DrawCommand> {
        self.text_overlays
            .iter()
            .map(|(text, pos, color, size)| DrawCommand::DrawText {
                text: text.clone(), position: *pos, color: *color, size: *size,
            })
            .collect()
    }

    pub fn clear_overlays(&mut self) { self.text_overlays.clear(); }
}

impl Default for DebugRenderer { fn default() -> Self { Self::new() } }

// ---------------------------------------------------------------------------
// Screen effects — fade, flash, shake
// ---------------------------------------------------------------------------

pub struct ScreenEffects {
    pub fade_color: Color,
    pub fade_alpha: f32,
    pub fade_speed: f32,
    pub flash_color: Color,
    pub flash_alpha: f32,
    pub flash_speed: f32,
    pub shake_intensity: f32,
    pub shake_decay: f32,
    pub shake_offset: Vec2,
}

impl ScreenEffects {
    pub fn new() -> Self {
        Self {
            fade_color: Color::black(), fade_alpha: 0.0, fade_speed: 2.0,
            flash_color: Color::white(), flash_alpha: 0.0, flash_speed: 4.0,
            shake_intensity: 0.0, shake_decay: 8.0, shake_offset: Vec2::zero(),
        }
    }

    pub fn start_fade_in(&mut self) { self.fade_alpha = 1.0; }
    pub fn start_fade_out(&mut self) { self.fade_alpha = 0.0; }
    pub fn start_flash(&mut self, color: Color) { self.flash_color = color; self.flash_alpha = 1.0; }
    pub fn start_shake(&mut self, intensity: f32) { self.shake_intensity = intensity; }

    pub fn update(&mut self, dt: f32) {
        if self.fade_alpha > 0.0 { self.fade_alpha = (self.fade_alpha - dt * self.fade_speed).max(0.0); }
        if self.flash_alpha > 0.0 { self.flash_alpha = (self.flash_alpha - dt * self.flash_speed).max(0.0); }
        if self.shake_intensity > 0.1 {
            self.shake_offset = Vec2::new(
                rand_f32_range(-1.0, 1.0) * self.shake_intensity,
                rand_f32_range(-1.0, 1.0) * self.shake_intensity,
            );
            self.shake_intensity *= (-self.shake_decay * dt).exp();
        } else {
            self.shake_intensity = 0.0;
            self.shake_offset = Vec2::zero();
        }
    }

    pub fn is_fading(&self) -> bool { self.fade_alpha > 0.0 }
    pub fn is_flashing(&self) -> bool { self.flash_alpha > 0.0 }
    pub fn is_shaking(&self) -> bool { self.shake_intensity > 0.1 }

    pub fn render(&self) -> Vec<DrawCommand> {
        let mut cmds = Vec::new();
        if self.fade_alpha > 0.01 {
            cmds.push(DrawCommand::DrawRect {
                rect: Rect::new(0.0, 0.0, f32::MAX, f32::MAX),
                color: self.fade_color.with_alpha(self.fade_alpha),
            });
        }
        if self.flash_alpha > 0.01 {
            cmds.push(DrawCommand::DrawRect {
                rect: Rect::new(0.0, 0.0, f32::MAX, f32::MAX),
                color: self.flash_color.with_alpha(self.flash_alpha),
            });
        }
        cmds
    }
}

impl Default for ScreenEffects { fn default() -> Self { Self::new() } }

// ---------------------------------------------------------------------------
// Combined Renderer — holds all sub-renderers
// ---------------------------------------------------------------------------

pub struct GameRenderer {
    pub sprite_batch: SpriteBatch,
    pub tilemap_renderer: TilemapRenderer,
    pub particle_system: ParticleSystem,
    pub debug_renderer: DebugRenderer,
    pub effects: ScreenEffects,
    pub camera: Camera,
    pub frame_timer: FrameTimer,
    pub draw_call_batcher: DrawCallBatcher,
    pub metrics: PerformanceMetrics,
}

impl GameRenderer {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            sprite_batch: SpriteBatch::new(),
            tilemap_renderer: TilemapRenderer::new(16),
            particle_system: ParticleSystem::new(2048),
            debug_renderer: DebugRenderer::new(),
            effects: ScreenEffects::new(),
            camera: Camera::new(width, height),
            frame_timer: FrameTimer::new(),
            draw_call_batcher: DrawCallBatcher::new(256),
            metrics: PerformanceMetrics::new(),
        }
    }

    pub fn begin_frame(&mut self) { self.sprite_batch.begin(); }

    pub fn submit_tilemap(&mut self, tilemap: &TileMap) {
        self.tilemap_renderer.compute_visible_chunks(tilemap.width(), tilemap.height(), tilemap.tile_size, &self.camera);
        let cmds = self.tilemap_renderer.render_tilemap(tilemap, &self.camera);
        for cmd in cmds {
            if let DrawCommand::DrawRect { rect, color } = cmd {
                self.sprite_batch.add_quad(rect, color, 0);
            }
        }
    }

    pub fn submit_sprite(&mut self, texture: &str, dest: Rect, color: Color, z_index: i32) {
        self.sprite_batch.add_sprite(texture, dest, color, z_index);
    }

    pub fn end_frame(&mut self, dt: f32) -> Vec<DrawCommand> {
        self.frame_timer.tick();
        self.particle_system.update(dt);
        self.effects.update(dt);
        self.debug_renderer.record_frame_time(dt);
        self.tilemap_renderer.update_animations(dt);

        let mut cmds = Vec::new();

        if self.debug_renderer.show_grid {
            cmds.extend(self.debug_renderer.render_grid(&self.camera, 32.0));
        }

        cmds.extend(self.sprite_batch.end());

        let pv = self.particle_system.render(&self.camera);
        if !pv.is_empty() {
            cmds.push(DrawCommand::DrawParticles { particles: pv });
        }

        cmds.extend(self.debug_renderer.render_text_overlays());
        cmds.extend(self.effects.render());

        self.metrics.fps = self.frame_timer.fps();
        self.metrics.entities_rendered = self.sprite_batch.total_sprites() as u64;
        self.metrics.draw_calls = self.draw_call_batcher.draw_calls;
        self.metrics.avg_frame_time_ms = if self.frame_timer.fps > 0.0 {
            1000.0 / self.frame_timer.fps
        } else {
            0.0
        };

        cmds
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

impl Component for Transform {}
impl Component for Sprite {}
impl Component for TileMap {}

fn rand_f32_range(min: f32, max: f32) -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().subsec_nanos();
    let t = (nanos % 10000) as f32 / 10000.0;
    min + (max - min) * t
}

// ---------------------------------------------------------------------------
// FrameTimer
// ---------------------------------------------------------------------------

pub struct FrameTimer {
    pub frame_count: u64,
    pub last_report: std::time::Instant,
    pub fps: f64,
}

impl FrameTimer {
    pub fn new() -> Self {
        Self { frame_count: 0, last_report: std::time::Instant::now(), fps: 0.0 }
    }

    pub fn tick(&mut self) {
        self.frame_count += 1;
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_report).as_secs_f64();
        if elapsed >= 1.0 {
            self.fps = self.frame_count as f64 / elapsed;
            self.frame_count = 0;
            self.last_report = now;
        }
    }

    pub fn fps(&self) -> f64 { self.fps }
}

impl Default for FrameTimer { fn default() -> Self { Self::new() } }

// ---------------------------------------------------------------------------
// DrawCallBatcher
// ---------------------------------------------------------------------------

pub struct DrawCallBatcher {
    pub max_batch_size: usize,
    pub current_batch: Vec<DrawCommand>,
    pub draw_calls: u64,
}

impl DrawCallBatcher {
    pub fn new(max_batch_size: usize) -> Self {
        Self { max_batch_size, current_batch: Vec::new(), draw_calls: 0 }
    }

    pub fn add(&mut self, cmd: DrawCommand) -> bool {
        self.current_batch.push(cmd);
        if self.current_batch.len() >= self.max_batch_size { self.flush(); true } else { false }
    }

    pub fn flush(&mut self) {
        if !self.current_batch.is_empty() {
            self.draw_calls += 1;
            self.current_batch.clear();
        }
    }

    pub fn pending(&self) -> usize { self.current_batch.len() }

    pub fn reset_stats(&mut self) { self.draw_calls = 0; self.current_batch.clear(); }
}

impl Default for DrawCallBatcher { fn default() -> Self { Self::new(256) } }

// ---------------------------------------------------------------------------
// PerformanceMetrics
// ---------------------------------------------------------------------------

pub struct PerformanceMetrics {
    pub fps: f64,
    pub entities_rendered: u64,
    pub draw_calls: u64,
    pub avg_frame_time_ms: f64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self { fps: 0.0, entities_rendered: 0, draw_calls: 0, avg_frame_time_ms: 0.0 }
    }
}

impl Default for PerformanceMetrics { fn default() -> Self { Self::new() } }

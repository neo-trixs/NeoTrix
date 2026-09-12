use std::collections::HashMap;
use crate::core::world::Component;

/// 颜色
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    pub fn rgb(r: f32, g: f32, b: f32) -> Self { Self { r, g, b, a: 1.0 } }
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self { Self { r, g, b, a } }
    pub fn white() -> Self { Self::rgb(1.0, 1.0, 1.0) }
    pub fn black() -> Self { Self::rgb(0.0, 0.0, 0.0) }
    pub fn red() -> Self { Self::rgb(1.0, 0.0, 0.0) }
    pub fn green() -> Self { Self::rgb(0.0, 1.0, 0.0) }
    pub fn blue() -> Self { Self::rgb(0.0, 0.0, 1.0) }
    pub fn yellow() -> Self { Self::rgb(1.0, 1.0, 0.0) }
    pub fn clear() -> Self { Self::rgba(0.0, 0.0, 0.0, 0.0) }

    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    pub fn with_alpha(&self, a: f32) -> Self { Self { a, ..*self } }
}

/// 2D 向量
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self { Self { x, y } }
    pub fn zero() -> Self { Self { x: 0.0, y: 0.0 } }
    pub fn one() -> Self { Self { x: 1.0, y: 1.0 } }
    pub fn length(&self) -> f32 { (self.x * self.x + self.y * self.y).sqrt() }
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len < 1e-8 { Self::zero() } else { Self::new(self.x / len, self.y / len) }
    }
    pub fn dot(&self, other: &Self) -> f32 { self.x * other.x + self.y * other.y }
    pub fn distance_to(&self, other: &Self) -> f32 { (*self - *other).length() }
    pub fn min(&self, other: &Self) -> Self { Self::new(self.x.min(other.x), self.y.min(other.y)) }
    pub fn max(&self, other: &Self) -> Self { Self::new(self.x.max(other.x), self.y.max(other.y)) }
    pub fn clamp(&self, min: &Self, max: &Self) -> Self {
        Self::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y))
    }
}

impl std::ops::Add for Vec2 { type Output = Self; fn add(self, rhs: Self) -> Self { Self::new(self.x + rhs.x, self.y + rhs.y) } }
impl std::ops::Sub for Vec2 { type Output = Self; fn sub(self, rhs: Self) -> Self { Self::new(self.x - rhs.x, self.y - rhs.y) } }
impl std::ops::Neg for Vec2 { type Output = Self; fn neg(self) -> Self { Self::new(-self.x, -self.y) } }
impl std::ops::Mul<f32> for Vec2 { type Output = Self; fn mul(self, s: f32) -> Self { Self::new(self.x * s, self.y * s) } }

/// 矩形
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self { Self { x, y, width, height } }
    pub fn contains(&self, p: &Vec2) -> bool {
        p.x >= self.x && p.x <= self.x + self.width && p.y >= self.y && p.y <= self.y + self.height
    }
    pub fn intersects(&self, o: &Rect) -> bool {
        self.x < o.x + o.width && self.x + self.width > o.x && self.y < o.y + o.height && self.y + self.height > o.y
    }
    pub fn left(&self) -> f32 { self.x }
    pub fn right(&self) -> f32 { self.x + self.width }
    pub fn top(&self) -> f32 { self.y }
    pub fn bottom(&self) -> f32 { self.y + self.height }
    pub fn center(&self) -> Vec2 { Vec2::new(self.x + self.width * 0.5, self.y + self.height * 0.5) }
    pub fn size(&self) -> Vec2 { Vec2::new(self.width, self.height) }
}

/// 变换
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform {
    pub fn new(position: Vec2, rotation: f32, scale: Vec2) -> Self { Self { position, rotation, scale } }
    pub fn default() -> Self { Self { position: Vec2::zero(), rotation: 0.0, scale: Vec2::one() } }
}

/// 精灵
#[derive(Debug, Clone)]
pub struct Sprite {
    pub texture: Option<String>,
    pub rect: Rect,
    pub color: Color,
    pub z_index: i32,
}

impl Sprite {
    pub fn new(texture: &str) -> Self {
        Self { texture: Some(texture.to_string()), rect: Rect::new(0.0, 0.0, 16.0, 16.0), color: Color::white(), z_index: 0 }
    }
    pub fn colored(color: Color) -> Self {
        Self { texture: None, rect: Rect::new(0.0, 0.0, 16.0, 16.0), color, z_index: 0 }
    }
}

/// 瓦片定义
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

/// 瓦片图
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

/// 相机
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
    DrawSprite { texture: String, dest: Rect, color: Color, z_index: i32 },
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
            transform.position.x + sprite.rect.x,
            transform.position.y + sprite.rect.y,
            sprite.rect.width * transform.scale.x,
            sprite.rect.height * transform.scale.y,
        );
        self.commands.push(DrawCommand::DrawSprite {
            texture: sprite.texture.clone().unwrap_or_default(),
            dest, color: sprite.color, z_index: sprite.z_index,
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

pub struct SpriteBatch {
    batches: HashMap<String, Vec<SpriteInstance>>,
    draw_order: Vec<String>,
}

pub struct SpriteInstance {
    pub dest: Rect,
    pub color: Color,
    pub z_index: i32,
}

impl SpriteBatch {
    pub fn new() -> Self { Self { batches: HashMap::new(), draw_order: Vec::new() } }

    pub fn begin(&mut self) {
        self.batches.clear();
        self.draw_order.clear();
    }

    pub fn add_sprite(&mut self, texture: &str, dest: Rect, color: Color, z_index: i32) {
        let key = texture.to_string();
        if !self.batches.contains_key(&key) {
            self.draw_order.push(key.clone());
            self.batches.insert(key.clone(), Vec::new());
        }
        self.batches.get_mut(&key).unwrap().push(SpriteInstance { dest, color, z_index });
    }

    pub fn add_quad(&mut self, dest: Rect, color: Color, z_index: i32) {
        self.add_sprite("__solid__", dest, color, z_index);
    }

    pub fn end(&self) -> Vec<DrawCommand> {
        let mut commands = Vec::new();
        for tex in &self.draw_order {
            if let Some(sprites) = self.batches.get(tex) {
                for s in sprites {
                    commands.push(DrawCommand::DrawSprite {
                        texture: tex.clone(), dest: s.dest, color: s.color, z_index: s.z_index,
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
// TilemapRenderer — chunk-culled tile rendering
// ---------------------------------------------------------------------------

pub struct TilemapRenderer {
    pub chunk_size: usize,
    visible_chunks: Vec<(usize, usize)>,
}

impl TilemapRenderer {
    pub fn new(chunk_size: usize) -> Self { Self { chunk_size, visible_chunks: Vec::new() } }

    pub fn compute_visible_chunks(&mut self, tilemap: &TileMap, camera: &Camera) {
        self.visible_chunks.clear();
        let cam_rect = camera.visible_world_rect();
        let ts = tilemap.tile_size;
        let start_x = ((cam_rect.x / ts.x).floor() as usize).max(0);
        let start_y = ((cam_rect.y / ts.y).floor() as usize).max(0);
        let end_x = ((cam_rect.right() / ts.x).ceil() as usize).min(tilemap.width());
        let end_y = ((cam_rect.bottom() / ts.y).ceil() as usize).min(tilemap.height());

        for cy in (start_y..end_y).step_by(self.chunk_size) {
            for cx in (start_x..end_x).step_by(self.chunk_size) {
                self.visible_chunks.push((cx, cy));
            }
        }
    }

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
        vec![
            DrawCommand::DrawRect { rect: screen_rect, color: self.collision_color },
        ]
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
// Screen effects — fade, flash, shake (pure data, no rendering dependency)
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
// Combined Renderer — holds all sub-renderers for ergonomic frame rendering
// ---------------------------------------------------------------------------

pub struct GameRenderer {
    pub sprite_batch: SpriteBatch,
    pub tilemap_renderer: TilemapRenderer,
    pub particle_system: ParticleSystem,
    pub debug_renderer: DebugRenderer,
    pub effects: ScreenEffects,
    pub camera: Camera,
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
        }
    }

    pub fn begin_frame(&mut self) {
        self.sprite_batch.begin();
    }

    pub fn submit_tilemap(&mut self, tilemap: &TileMap) {
        self.tilemap_renderer.compute_visible_chunks(tilemap, &self.camera);
        let cmds = self.tilemap_renderer.render_tilemap(tilemap, &self.camera);
        // In a real renderer these would go to a render queue; here we
        // re-inject them as raw draw rects into the sprite batch.
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
        self.particle_system.update(dt);
        self.effects.update(dt);
        self.debug_renderer.record_frame_time(dt);

        let mut cmds = Vec::new();

        // Grid debug
        if self.debug_renderer.show_grid {
            cmds.extend(self.debug_renderer.render_grid(&self.camera, 32.0));
        }

        // Sprite batch
        cmds.extend(self.sprite_batch.end());

        // Particles
        let pv = self.particle_system.render(&self.camera);
        if !pv.is_empty() {
            cmds.push(DrawCommand::DrawParticles { particles: pv });
        }

        // Collision boxes from debug
        cmds.extend(self.debug_renderer.render_text_overlays());

        // Screen effects on top
        cmds.extend(self.effects.render());

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

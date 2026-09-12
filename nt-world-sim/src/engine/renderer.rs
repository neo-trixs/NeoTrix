use std::collections::HashMap;

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

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn white() -> Self { Self::rgb(1.0, 1.0, 1.0) }
    pub fn black() -> Self { Self::rgb(0.0, 0.0, 0.0) }
    pub fn red() -> Self { Self::rgb(1.0, 0.0, 0.0) }
    pub fn green() -> Self { Self::rgb(0.0, 1.0, 0.0) }
    pub fn blue() -> Self { Self::rgb(0.0, 0.0, 1.0) }
    pub fn yellow() -> Self { Self::rgb(1.0, 1.0, 0.0) }
    pub fn clear() -> Self { Self::rgba(0.0, 0.0, 0.0, 0.0) }
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

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len == 0.0 { Self::zero() } else { Self::new(self.x / len, self.y / len) }
    }

    pub fn dot(&self, other: &Self) -> f32 { self.x * other.x + self.y * other.y }
    pub fn distance_to(&self, other: &Self) -> f32 { (*self - *other).length() }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { Self::new(self.x + rhs.x, self.y + rhs.y) }
}
impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self { Self::new(self.x - rhs.x, self.y - rhs.y) }
}
impl std::ops::Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self { Self::new(-self.x, -self.y) }
}
impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self { Self::new(self.x * scalar, self.y * scalar) }
}

/// 矩形
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, point: &Vec2) -> bool {
        point.x >= self.x && point.x <= self.x + self.width
            && point.y >= self.y && point.y <= self.y + self.height
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width && self.x + self.width > other.x
            && self.y < other.y + other.height && self.y + self.height > other.y
    }
}

/// 变换
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform {
    pub fn new(position: Vec2, rotation: f32, scale: Vec2) -> Self {
        Self { position, rotation, scale }
    }

    pub fn default() -> Self {
        Self { position: Vec2::zero(), rotation: 0.0, scale: Vec2::one() }
    }
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

    pub fn get_tile(&self, x: usize, y: usize) -> Option<u32> {
        self.tiles.get(y)?.get(x).copied()
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile_id: u32) {
        if let Some(row) = self.tiles.get_mut(y) {
            if let Some(tile) = row.get_mut(x) { *tile = tile_id; }
        }
    }

    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        if let Some(tile_id) = self.get_tile(x, y) {
            if let Some(tile_def) = self.palette.get(&tile_id) { return tile_def.walkable; }
        }
        false
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
}

// ---------------------------------------------------------------------------
// Draw commands — recorded by CanvasRenderer, consumed by a backend
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
    Present,
}

/// 渲染器 trait
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

/// Canvas 渲染器实现 — records draw commands for later consumption
pub struct CanvasRenderer {
    width: f32,
    height: f32,
    commands: Vec<DrawCommand>,
    frame_count: u64,
}

impl CanvasRenderer {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height, commands: Vec::new(), frame_count: 0 }
    }

    /// Drain all recorded draw commands (consumes them)
    pub fn drain_commands(&mut self) -> Vec<DrawCommand> {
        std::mem::take(&mut self.commands)
    }

    /// Peek at commands without draining
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    /// Number of commands recorded this frame
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    /// Current frame number
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

impl Renderer for CanvasRenderer {
    fn clear(&mut self, color: Color) {
        self.commands.push(DrawCommand::Clear { color });
    }

    fn draw_sprite(&mut self, sprite: &Sprite, transform: &Transform) {
        let dest = Rect::new(
            transform.position.x + sprite.rect.x,
            transform.position.y + sprite.rect.y,
            sprite.rect.width * transform.scale.x,
            sprite.rect.height * transform.scale.y,
        );
        let texture = sprite.texture.clone().unwrap_or_default();
        self.commands.push(DrawCommand::DrawSprite {
            texture,
            dest,
            color: sprite.color,
            z_index: sprite.z_index,
        });
    }

    fn draw_tilemap(&mut self, tilemap: &TileMap, camera: &Camera) {
        let mut tile_colors = Vec::new();
        for (y, row) in tilemap.tiles.iter().enumerate() {
            for (x, &tile_id) in row.iter().enumerate() {
                if let Some(tile_def) = tilemap.palette.get(&tile_id) {
                    let world_pos = Vec2::new(
                        x as f32 * tilemap.tile_size.x,
                        y as f32 * tilemap.tile_size.y,
                    );
                    let screen_pos = camera.world_to_screen(world_pos);
                    let rect = Rect::new(
                        screen_pos.x,
                        screen_pos.y,
                        tilemap.tile_size.x * camera.zoom,
                        tilemap.tile_size.y * camera.zoom,
                    );
                    tile_colors.push((rect, tile_def.color));
                }
            }
        }
        self.commands.push(DrawCommand::DrawTilemap { tile_colors, z_index: 0 });
    }

    fn draw_text(&mut self, text: &str, position: Vec2, color: Color, size: f32) {
        self.commands.push(DrawCommand::DrawText {
            text: text.to_string(),
            position,
            color,
            size,
        });
    }

    fn draw_rect(&mut self, rect: &Rect, color: Color) {
        self.commands.push(DrawCommand::DrawRect { rect: *rect, color });
    }

    fn draw_circle(&mut self, center: Vec2, radius: f32, color: Color) {
        self.commands.push(DrawCommand::DrawCircle { center, radius, color });
    }

    fn draw_line(&mut self, start: Vec2, end: Vec2, color: Color, width: f32) {
        self.commands.push(DrawCommand::DrawLine { start, end, color, width });
    }

    fn present(&mut self) {
        self.commands.push(DrawCommand::Present);
        self.frame_count += 1;
    }

    fn viewport_size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }
}

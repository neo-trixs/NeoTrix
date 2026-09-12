use std::ops::{Add, Sub, Neg, Mul};

/// 2D vector — single fact source for all Vec2 usage across engine/game/ui.
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
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self { x: self.x + (other.x - self.x) * t, y: self.y + (other.y - self.y) * t }
    }
    pub fn distance_to(&self, other: &Self) -> f32 { (*self - *other).length() }
    pub fn min(&self, other: &Self) -> Self { Self::new(self.x.min(other.x), self.y.min(other.y)) }
    pub fn max(&self, other: &Self) -> Self { Self::new(self.x.max(other.x), self.y.max(other.y)) }
    pub fn clamp(&self, min: &Self, max: &Self) -> Self {
        Self::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y))
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { Self::new(self.x + rhs.x, self.y + rhs.y) }
}
impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self { Self::new(self.x - rhs.x, self.y - rhs.y) }
}
impl Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self { Self::new(-self.x, -self.y) }
}
impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, s: f32) -> Self { Self::new(self.x * s, self.y * s) }
}

/// Axis-aligned rectangle — single fact source for all Rect usage.
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

/// RGBA color — single fact source for all Color usage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self { Self { r, g, b, a } }
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

/// 2D transform — single fact source for all Transform usage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform {
    pub fn new(position: Vec2, rotation: f32, scale: Vec2) -> Self { Self { position, rotation, scale } }
}

impl Default for Transform {
    fn default() -> Self {
        Self { position: Vec2::zero(), rotation: 0.0, scale: Vec2::one() }
    }
}

use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Neg, Sub};

/// 2D Vector
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    pub fn one() -> Self {
        Self { x: 1.0, y: 1.0 }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len < 1e-10 {
            *self
        } else {
            *self / len
        }
    }
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
    pub fn distance_to(&self, other: &Self) -> f32 {
        (*self - *other).length()
    }
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        *self + (*other - *self) * t
    }
    pub fn clamp_length(&self, max: f32) -> Self {
        let len = self.length();
        if len > max {
            *self / len * max
        } else {
            *self
        }
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Neg for Vec2 {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

/// 3D Vector
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len < 1e-10 {
            *self
        } else {
            *self / len
        }
    }
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
    pub fn distance_to(&self, other: &Self) -> f32 {
        (*self - *other).length()
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

/// AABB for spatial queries
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AABB {
    pub min: Vec2,
    pub max: Vec2,
}

impl AABB {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }
}

/// Spatial hash grid for efficient neighbor queries
pub struct SpatialGrid {
    cell_size: f32,
    cells: std::collections::HashMap<(i32, i32), Vec<(String, Vec2)>>,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: std::collections::HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn insert(&mut self, id: &str, pos: Vec2) {
        let cx = (pos.x / self.cell_size).floor() as i32;
        let cy = (pos.y / self.cell_size).floor() as i32;
        self.cells
            .entry((cx, cy))
            .or_default()
            .push((id.to_string(), pos));
    }

    pub fn query_radius(&self, pos: Vec2, radius: f32) -> Vec<&str> {
        let min_cx = ((pos.x - radius) / self.cell_size).floor() as i32;
        let max_cx = ((pos.x + radius) / self.cell_size).floor() as i32;
        let min_cy = ((pos.y - radius) / self.cell_size).floor() as i32;
        let max_cy = ((pos.y + radius) / self.cell_size).floor() as i32;

        let r2 = radius * radius;
        let mut result = Vec::new();
        for cx in min_cx..=max_cx {
            for cy in min_cy..=max_cy {
                if let Some(cell) = self.cells.get(&(cx, cy)) {
                    for (id, cell_pos) in cell {
                        let dx = pos.x - cell_pos.x;
                        let dy = pos.y - cell_pos.y;
                        if dx * dx + dy * dy <= r2 {
                            result.push(id.as_str());
                        }
                    }
                }
            }
        }
        result
    }
}

/// Simple hash-based random for reproducible simulations
pub struct SimulationRng {
    state: u64,
}

impl SimulationRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_f32(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 11) as f32 / (1u64 << 53) as f32
    }

    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }

    pub fn range_usize(&mut self, min: usize, max: usize) -> usize {
        min + (self.next_f32() * (max - min) as f32) as usize
    }

    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            None
        } else {
            Some(&items[self.range_usize(0, items.len())])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec2_basic_ops() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(3.0, 4.0);
        assert_eq!(a + b, Vec2::new(4.0, 6.0));
        assert_eq!(b - a, Vec2::new(2.0, 2.0));
        assert_eq!(a * 2.0, Vec2::new(2.0, 4.0));
        assert_eq!(b / 2.0, Vec2::new(1.5, 2.0));
        assert_eq!(-a, Vec2::new(-1.0, -2.0));
    }

    #[test]
    fn vec2_length_normalize() {
        let v = Vec2::new(3.0, 4.0);
        assert!((v.length() - 5.0).abs() < 1e-6);
        let n = v.normalize();
        assert!((n.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn vec2_dot_distance() {
        let a = Vec2::new(1.0, 0.0);
        let b = Vec2::new(0.0, 1.0);
        assert!((a.dot(&b)).abs() < 1e-6);
        assert!((a.distance_to(&b) - std::f32::consts::SQRT_2).abs() < 1e-6);
    }

    #[test]
    fn vec3_cross() {
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);
        let z = x.cross(&y);
        assert_eq!(z, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn aabb_contains_intersects() {
        let a = AABB::new(Vec2::new(0.0, 0.0), Vec2::new(2.0, 2.0));
        assert!(a.contains(Vec2::new(1.0, 1.0)));
        assert!(!a.contains(Vec2::new(3.0, 3.0)));

        let b = AABB::new(Vec2::new(1.0, 1.0), Vec2::new(3.0, 3.0));
        assert!(a.intersects(&b));

        let c = AABB::new(Vec2::new(5.0, 5.0), Vec2::new(6.0, 6.0));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn spatial_grid_insert_query() {
        let mut grid = SpatialGrid::new(10.0);
        grid.insert("a", Vec2::new(5.0, 5.0));
        grid.insert("b", Vec2::new(15.0, 15.0));

        let near = grid.query_radius(Vec2::new(6.0, 6.0), 5.0);
        assert!(near.contains(&"a"));
        assert!(!near.contains(&"b"));
    }

    #[test]
    fn rng_reproducible() {
        let mut r1 = SimulationRng::new(42);
        let mut r2 = SimulationRng::new(42);
        for _ in 0..100 {
            assert_eq!(r1.next_f32(), r2.next_f32());
        }
    }
}

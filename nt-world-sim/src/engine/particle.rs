use crate::engine::renderer::{Color, Vec2, Camera, ParticleDrawVertex, DrawCommand, Rect};

#[derive(Debug, Clone)]
pub struct ParticleAdvanced {
    pub position: Vec2,
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub size: f32,
    pub color: Color,
    pub alpha: f32,
    pub rotation: f32,
    pub rotation_speed: f32,
    pub gravity: f32,
}

impl ParticleAdvanced {
    pub fn alive(&self) -> bool { self.lifetime > 0.0 }

    pub fn life_ratio(&self) -> f32 {
        if self.max_lifetime > 0.0 { (self.lifetime / self.max_lifetime).clamp(0.0, 1.0) } else { 0.0 }
    }
}

pub struct ParticleEmitter {
    pub position: Vec2,
    pub rate: f32,
    pub burst_count: u32,
    pub lifetime: (f32, f32),
    pub speed: (f32, f32),
    pub size: (f32, f32),
    pub color: Color,
    pub gravity: f32,
    pub spread: f32,
    pub particles: Vec<ParticleAdvanced>,
    pub timer: f32,
}

impl ParticleEmitter {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            rate: 10.0,
            burst_count: 0,
            lifetime: (0.5, 1.5),
            speed: (20.0, 50.0),
            size: (2.0, 6.0),
            color: Color { r: 1.0, g: 0.8, b: 0.2, a: 1.0 },
            gravity: 0.0,
            spread: std::f32::consts::PI * 2.0,
            particles: Vec::new(),
            timer: 0.0,
        }
    }

    pub fn burst(&mut self, count: u32) {
        for _ in 0..count { self.emit_particle(); }
    }

    fn emit_particle(&mut self) {
        let angle = pseudo_random_f32() * self.spread - self.spread / 2.0;
        let speed = lerp(self.speed.0, self.speed.1, pseudo_random_f32());
        let lifetime = lerp(self.lifetime.0, self.lifetime.1, pseudo_random_f32());
        let size = lerp(self.size.0, self.size.1, pseudo_random_f32());

        self.particles.push(ParticleAdvanced {
            position: self.position,
            velocity: Vec2 { x: angle.cos() * speed, y: angle.sin() * speed },
            lifetime, max_lifetime: lifetime, size,
            color: self.color, alpha: 1.0, rotation: 0.0,
            rotation_speed: pseudo_random_f32() * 6.0 - 3.0,
            gravity: self.gravity,
        });
    }

    pub fn update(&mut self, dt: f32) {
        self.timer += dt;
        while self.timer >= 1.0 / self.rate {
            self.timer -= 1.0 / self.rate;
            self.emit_particle();
        }

        self.particles.retain_mut(|p| {
            p.position.x += p.velocity.x * dt;
            p.position.y += p.velocity.y * dt;
            p.velocity.y += p.gravity * dt;
            p.lifetime -= dt;
            p.alpha = (p.lifetime / p.max_lifetime).max(0.0);
            p.rotation += p.rotation_speed * dt;
            p.size *= 0.99;
            p.lifetime > 0.0
        });
    }

    pub fn draw(&self) -> Vec<(Vec2, f32, Color)> {
        self.particles
            .iter()
            .map(|p| (p.position, p.size, Color { a: p.alpha, ..p.color }))
            .collect()
    }

    pub fn active_count(&self) -> usize { self.particles.len() }
    pub fn clear(&mut self) { self.particles.clear(); self.timer = 0.0; }
    pub fn set_position(&mut self, pos: Vec2) { self.position = pos; }
    pub fn set_rate(&mut self, rate: f32) { self.rate = rate.max(0.1); }
    pub fn set_color(&mut self, color: Color) { self.color = color; }
    pub fn set_gravity(&mut self, gravity: f32) { self.gravity = gravity; }
    pub fn set_spread(&mut self, spread: f32) { self.spread = spread; }
    pub fn set_lifetime_range(&mut self, min: f32, max: f32) { self.lifetime = (min.max(0.01), max.max(min)); }
    pub fn set_speed_range(&mut self, min: f32, max: f32) { self.speed = (min.max(0.0), max.max(min)); }
    pub fn set_size_range(&mut self, min: f32, max: f32) { self.size = (min.max(0.1), max.max(min)); }
}

impl Default for ParticleEmitter {
    fn default() -> Self { Self::new(Vec2::zero()) }
}

// ---------------------------------------------------------------------------
// ParticlePool — pre-allocated particle storage, zero-alloc at runtime
// ---------------------------------------------------------------------------

pub struct ParticlePool {
    slots: Vec<PoolSlot>,
    capacity: usize,
    free_head: Option<usize>,
}

struct PoolSlot {
    particle: ParticleAdvanced,
    alive: bool,
    next_free: Option<usize>,
}

impl ParticlePool {
    pub fn new(capacity: usize) -> Self {
        let mut slots: Vec<PoolSlot> = Vec::with_capacity(capacity);
        for i in 0..capacity {
            let next_free = if i + 1 < capacity { Some(i + 1) } else { None };
            slots.push(PoolSlot {
                particle: ParticleAdvanced {
                    position: Vec2::zero(), velocity: Vec2::zero(),
                    lifetime: 0.0, max_lifetime: 1.0, size: 1.0,
                    color: Color::white(), alpha: 1.0,
                    rotation: 0.0, rotation_speed: 0.0, gravity: 0.0,
                },
                alive: false,
                next_free,
            });
        }
        Self { slots, capacity, free_head: Some(0) }
    }

    pub fn spawn(&mut self, particle: ParticleAdvanced) -> Option<usize> {
        let idx = self.free_head?;
        let slot = &mut self.slots[idx];
        self.free_head = slot.next_free;
        slot.particle = particle;
        slot.alive = true;
        Some(idx)
    }

    pub fn despawn(&mut self, idx: usize) {
        if idx < self.capacity && self.slots[idx].alive {
            self.slots[idx].alive = false;
            self.slots[idx].next_free = self.free_head;
            self.free_head = Some(idx);
        }
    }

    pub fn update(&mut self, dt: f32) {
        for slot in &mut self.slots {
            if !slot.alive { continue; }
            let p = &mut slot.particle;
            p.position.x += p.velocity.x * dt;
            p.position.y += p.velocity.y * dt;
            p.velocity.y += p.gravity * dt;
            p.lifetime -= dt;
            p.alpha = (p.lifetime / p.max_lifetime).clamp(0.0, 1.0);
            p.rotation += p.rotation_speed * dt;
            p.size *= 1.0 - dt * 0.5;
            if p.lifetime <= 0.0 {
                slot.alive = false;
                slot.next_free = self.free_head;
                self.free_head = Some(slot as *const PoolSlot as *mut PoolSlot as usize);
            }
        }
        // Fix free list — recompute properly
        self.rebuild_free_list();
    }

    fn rebuild_free_list(&mut self) {
        self.free_head = None;
        for i in (0..self.capacity).rev() {
            if !self.slots[i].alive {
                self.slots[i].next_free = self.free_head;
                self.free_head = Some(i);
            }
        }
    }

    pub fn render(&self, camera: &Camera) -> Vec<ParticleDrawVertex> {
        self.slots
            .iter()
            .filter(|s| s.alive)
            .map(|s| {
                let p = &s.particle;
                let sp = camera.world_to_screen(p.position);
                ParticleDrawVertex {
                    position: sp,
                    color: p.color.with_alpha(p.alpha),
                    size: p.size * camera.zoom,
                }
            })
            .collect()
    }

    pub fn active_count(&self) -> usize {
        self.slots.iter().filter(|s| s.alive).count()
    }

    pub fn free_count(&self) -> usize {
        self.capacity - self.active_count()
    }

    pub fn capacity(&self) -> usize { self.capacity }

    pub fn clear(&mut self) {
        for slot in &mut self.slots { slot.alive = false; }
        self.rebuild_free_list();
    }

    /// Emit a burst of particles into the pool.
    pub fn burst(&mut self, position: Vec2, count: u32, speed: f32, color: Color, size: f32, life: f32, gravity: f32) {
        for i in 0..count {
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU + pseudo_random_f32() * 0.3;
            let spd = speed * (0.8 + pseudo_random_f32() * 0.4);
            let sz = size * (0.8 + pseudo_random_f32() * 0.4);
            let lt = life * (0.8 + pseudo_random_f32() * 0.4);
            self.spawn(ParticleAdvanced {
                position,
                velocity: Vec2 { x: angle.cos() * spd, y: angle.sin() * spd },
                lifetime: lt, max_lifetime: lt, size: sz,
                color, alpha: 1.0, rotation: 0.0,
                rotation_speed: pseudo_random_f32() * 4.0 - 2.0,
                gravity,
            });
        }
    }
}

impl Default for ParticlePool {
    fn default() -> Self { Self::new(2048) }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }

fn pseudo_random_f32() -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 10000) as f32 / 10000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emitter_default() {
        let em = ParticleEmitter::new(Vec2::new(100.0, 200.0));
        assert_eq!(em.position.x, 100.0);
        assert_eq!(em.particles.len(), 0);
        assert!(em.rate > 0.0);
    }

    #[test]
    fn test_burst_emits_particles() {
        let mut em = ParticleEmitter::new(Vec2::zero());
        em.burst(10);
        assert_eq!(em.particles.len(), 10);
    }

    #[test]
    fn test_update_removes_dead_particles() {
        let mut em = ParticleEmitter::new(Vec2::zero());
        em.lifetime = (0.01, 0.01);
        em.burst(5);
        em.update(1.0);
        assert_eq!(em.particles.len(), 0);
    }

    #[test]
    fn test_rate_emission() {
        let mut em = ParticleEmitter::new(Vec2::zero());
        em.rate = 100.0;
        em.timer = 0.0;
        em.update(0.1);
        assert!(em.particles.len() >= 5);
    }

    #[test]
    fn test_clear() {
        let mut em = ParticleEmitter::new(Vec2::zero());
        em.burst(20);
        em.clear();
        assert_eq!(em.active_count(), 0);
    }

    #[test]
    fn test_pool_spawn_despawn() {
        let mut pool = ParticlePool::new(8);
        assert_eq!(pool.capacity(), 8);
        assert_eq!(pool.free_count(), 8);

        let idx = pool.spawn(ParticleAdvanced {
            position: Vec2::new(10.0, 20.0),
            velocity: Vec2::zero(),
            lifetime: 1.0, max_lifetime: 1.0, size: 4.0,
            color: Color::red(), alpha: 1.0, rotation: 0.0,
            rotation_speed: 0.0, gravity: 0.0,
        });
        assert!(idx.is_some());
        assert_eq!(pool.active_count(), 1);
        assert_eq!(pool.free_count(), 7);

        pool.despawn(idx.unwrap());
        assert_eq!(pool.active_count(), 0);
        assert_eq!(pool.free_count(), 8);
    }

    #[test]
    fn test_pool_overflow() {
        let mut pool = ParticlePool::new(2);
        pool.spawn(ParticleAdvanced {
            position: Vec2::zero(), velocity: Vec2::zero(),
            lifetime: 1.0, max_lifetime: 1.0, size: 1.0,
            color: Color::white(), alpha: 1.0, rotation: 0.0,
            rotation_speed: 0.0, gravity: 0.0,
        });
        pool.spawn(ParticleAdvanced {
            position: Vec2::zero(), velocity: Vec2::zero(),
            lifetime: 1.0, max_lifetime: 1.0, size: 1.0,
            color: Color::white(), alpha: 1.0, rotation: 0.0,
            rotation_speed: 0.0, gravity: 0.0,
        });
        // Pool full
        assert!(pool.spawn(ParticleAdvanced {
            position: Vec2::zero(), velocity: Vec2::zero(),
            lifetime: 1.0, max_lifetime: 1.0, size: 1.0,
            color: Color::white(), alpha: 1.0, rotation: 0.0,
            rotation_speed: 0.0, gravity: 0.0,
        }).is_none());
    }

    #[test]
    fn test_pool_burst() {
        let mut pool = ParticlePool::new(32);
        pool.burst(Vec2::zero(), 16, 50.0, Color::yellow(), 3.0, 1.0, 0.0);
        assert_eq!(pool.active_count(), 16);
    }

    #[test]
    fn test_pool_update_removes_dead() {
        let mut pool = ParticlePool::new(16);
        pool.burst(Vec2::zero(), 8, 10.0, Color::white(), 2.0, 0.01, 0.0);
        pool.update(1.0);
        assert_eq!(pool.active_count(), 0);
        assert_eq!(pool.free_count(), 16);
    }

    #[test]
    fn test_pool_clear() {
        let mut pool = ParticlePool::new(16);
        pool.burst(Vec2::zero(), 10, 10.0, Color::white(), 2.0, 5.0, 0.0);
        pool.clear();
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn test_draw_output() {
        let mut em = ParticleEmitter::new(Vec2::zero());
        em.burst(3);
        let drawn = em.draw();
        assert_eq!(drawn.len(), 3);
        assert!(drawn[0].1 > 0.0);
    }
}

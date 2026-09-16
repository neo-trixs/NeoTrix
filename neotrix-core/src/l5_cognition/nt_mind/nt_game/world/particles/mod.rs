#[derive(Debug, Clone)]
pub struct Particle {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub life: f64,
    pub max_life: f64,
    pub size: f64,
    pub color: [f32; 4],
    pub alive: bool,
}

#[derive(Debug, Clone)]
pub struct ParticleEmitter {
    pub x: f64,
    pub y: f64,
    pub rate: f64,
    pub speed: f64,
    pub spread: f64,
    pub lifetime: f64,
    pub size: f64,
    pub color: [f32; 4],
    pub gravity: f64,
    pub timer: f64,
}

impl ParticleEmitter {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            rate: 10.0,
            speed: 50.0,
            spread: 3.14,
            lifetime: 1.0,
            size: 4.0,
            color: [1.0, 0.8, 0.2, 1.0],
            gravity: 100.0,
            timer: 0.0,
        }
    }

    pub fn emit(&mut self, particles: &mut Vec<Particle>, count: u32) {
        for _ in 0..count {
            let angle = (rand_f64() - 0.5) * self.spread;
            let speed = self.speed * (0.8 + rand_f64() * 0.4);
            particles.push(Particle {
                x: self.x,
                y: self.y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life: self.lifetime,
                max_life: self.lifetime,
                size: self.size * (0.8 + rand_f64() * 0.4),
                color: self.color,
                alive: true,
            });
        }
    }
}

pub struct ParticleSystem {
    pub particles: Vec<Particle>,
    pub max_particles: usize,
}

impl ParticleSystem {
    pub fn new(max: usize) -> Self {
        Self {
            particles: Vec::with_capacity(max),
            max_particles: max,
        }
    }

    pub fn update(&mut self, dt: f64) {
        for p in &mut self.particles {
            if p.alive {
                p.x += p.vx * dt;
                p.y += p.vy * dt;
                p.vy += 100.0 * dt;
                p.life -= dt;
                if p.life <= 0.0 {
                    p.alive = false;
                }
            }
        }
        self.particles.retain(|p| p.alive);
    }

    pub fn active_count(&self) -> usize {
        self.particles.iter().filter(|p| p.alive).count()
    }

    pub fn clear(&mut self) {
        self.particles.clear();
    }
}

fn rand_f64() -> f64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut h);
    h.finish() as f64 / u64::MAX as f64
}

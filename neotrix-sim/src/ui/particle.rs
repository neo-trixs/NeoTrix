pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub color: u32,
    pub size: f32,
}

pub struct Emitter {
    pub x: f32,
    pub y: f32,
    pub rate: f32,
    pub active: bool,
}

pub struct ParticleSystem {
    pub particles: Vec<Particle>,
    pub emitters: Vec<Emitter>,
    pub max_particles: usize,
}

impl ParticleSystem {
    pub fn new() -> Self {
        ParticleSystem {
            particles: Vec::new(),
            emitters: Vec::new(),
            max_particles: 1000,
        }
    }

    pub fn spawn(
        &mut self,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        life: f32,
        color: u32,
        size: f32,
    ) {
        if self.particles.len() < self.max_particles {
            self.particles.push(Particle {
                x,
                y,
                vx,
                vy,
                life,
                max_life: life,
                color,
                size,
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    pub fn count(&self) -> usize {
        self.particles.len()
    }
}

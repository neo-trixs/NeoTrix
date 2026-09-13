use crate::engine::renderer::{Color, Vec2, DrawCommand, Camera};
use crate::engine::particle::ParticlePool;

// ---------------------------------------------------------------------------
// FloatingNumber — damage / heal / crit text that rises and fades
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatingKind { Damage, Heal, Crit, HealCrit, Miss, Exp }

#[derive(Debug, Clone)]
pub struct FloatingNumber {
    pub position: Vec2,
    pub text: String,
    pub kind: FloatingKind,
    pub velocity: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: Color,
    pub alpha: f32,
    pub scale: f32,
}

impl FloatingNumber {
    pub fn damage(pos: Vec2, amount: u32) -> Self {
        Self::new(pos, &format!("-{}", amount), FloatingKind::Damage,
            Color::rgba(1.0, 0.3, 0.3, 1.0), 18.0)
    }

    pub fn heal(pos: Vec2, amount: u32) -> Self {
        Self::new(pos, &format!("+{}", amount), FloatingKind::Heal,
            Color::rgba(0.2, 0.9, 0.3, 1.0), 16.0)
    }

    pub fn crit(pos: Vec2, amount: u32) -> Self {
        Self::new(pos, &format!("-{}!", amount), FloatingKind::Crit,
            Color::rgba(1.0, 0.9, 0.1, 1.0), 24.0)
    }

    pub fn heal_crit(pos: Vec2, amount: u32) -> Self {
        Self::new(pos, &format!("+{}!", amount), FloatingKind::HealCrit,
            Color::rgba(0.3, 1.0, 0.5, 1.0), 22.0)
    }

    pub fn miss(pos: Vec2) -> Self {
        Self::new(pos, "MISS", FloatingKind::Miss,
            Color::rgba(0.6, 0.6, 0.6, 1.0), 14.0)
    }

    pub fn exp(pos: Vec2, amount: u32) -> Self {
        Self::new(pos, &format!("+{} EXP", amount), FloatingKind::Exp,
            Color::rgba(0.8, 0.8, 0.2, 1.0), 12.0)
    }

    fn new(pos: Vec2, text: &str, kind: FloatingKind, color: Color, size: f32) -> Self {
        let drift = pseudo_random_f32_range(-15.0, 15.0);
        Self {
            position: Vec2::new(pos.x + drift, pos.y - 10.0),
            text: text.to_string(),
            kind,
            velocity: Vec2::new(drift * 0.3, -60.0 - pseudo_random_f32() * 30.0),
            life: 1.2, max_life: 1.2, size, color, alpha: 1.0, scale: 1.0,
        }
    }

    pub fn alive(&self) -> bool { self.life > 0.0 }

    pub fn update(&mut self, dt: f32) {
        self.life -= dt;
        self.position.x += self.velocity.x * dt;
        self.position.y += self.velocity.y * dt;
        self.velocity.y += 40.0 * dt; // slight gravity
        let t = (self.life / self.max_life).clamp(0.0, 1.0);
        self.alpha = t;
        self.scale = if t > 0.8 { 1.0 + (1.0 - t) * 5.0 * 0.3 } else { 1.0 };
        // Pop-in at start
        if self.life > self.max_life * 0.9 {
            let entry = (self.max_life - self.life) / (self.max_life * 0.1);
            self.scale = entry.min(1.0) * 1.2;
        }
    }

    pub fn render(&self, camera: &Camera) -> Option<DrawCommand> {
        if !self.alive() || self.alpha < 0.01 { return None; }
        let sp = camera.world_to_screen(self.position);
        let size = self.size * self.scale * camera.zoom;
        let color = self.color.with_alpha(self.alpha);
        Some(DrawCommand::DrawText {
            text: self.text.clone(), position: sp, color, size,
        })
    }
}

// ---------------------------------------------------------------------------
// FloatingNumberManager — pool of floating numbers
// ---------------------------------------------------------------------------

pub struct FloatingNumberManager {
    numbers: Vec<FloatingNumber>,
    max_numbers: usize,
}

impl FloatingNumberManager {
    pub fn new(max_numbers: usize) -> Self {
        Self { numbers: Vec::with_capacity(max_numbers), max_numbers }
    }

    pub fn spawn(&mut self, num: FloatingNumber) {
        if self.numbers.len() < self.max_numbers {
            self.numbers.push(num);
        }
    }

    pub fn spawn_damage(&mut self, pos: Vec2, amount: u32) {
        self.spawn(FloatingNumber::damage(pos, amount));
    }

    pub fn spawn_heal(&mut self, pos: Vec2, amount: u32) {
        self.spawn(FloatingNumber::heal(pos, amount));
    }

    pub fn spawn_crit(&mut self, pos: Vec2, amount: u32) {
        self.spawn(FloatingNumber::crit(pos, amount));
    }

    pub fn spawn_heal_crit(&mut self, pos: Vec2, amount: u32) {
        self.spawn(FloatingNumber::heal_crit(pos, amount));
    }

    pub fn spawn_miss(&mut self, pos: Vec2) {
        self.spawn(FloatingNumber::miss(pos));
    }

    pub fn spawn_exp(&mut self, pos: Vec2, amount: u32) {
        self.spawn(FloatingNumber::exp(pos, amount));
    }

    pub fn update(&mut self, dt: f32) {
        for n in &mut self.numbers { n.update(dt); }
        self.numbers.retain(|n| n.alive());
    }

    pub fn render(&self, camera: &Camera) -> Vec<DrawCommand> {
        self.numbers.iter().filter_map(|n| n.render(camera)).collect()
    }

    pub fn active_count(&self) -> usize { self.numbers.len() }
    pub fn clear(&mut self) { self.numbers.clear(); }
}

impl Default for FloatingNumberManager {
    fn default() -> Self { Self::new(64) }
}

// ---------------------------------------------------------------------------
// SkillEffect — particle burst configuration for skill VFX
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillEffectKind {
    Fireball,
    IceShard,
    Lightning,
    Heal,
    Slash,
    Explosion,
    HealRing,
    Poison,
    DarkPulse,
}

#[derive(Debug, Clone)]
pub struct SkillEffect {
    pub kind: SkillEffectKind,
    pub position: Vec2,
    pub timer: f32,
    pub duration: f32,
    pub particles_emitted: bool,
    pub active: bool,
}

impl SkillEffect {
    pub fn new(kind: SkillEffectKind, position: Vec2) -> Self {
        Self { kind, position, timer: 0.0, duration: 0.6, particles_emitted: false, active: true }
    }

    pub fn with_duration(mut self, dur: f32) -> Self { self.duration = dur; self }

    pub fn update(&mut self, dt: f32) {
        self.timer += dt;
        if self.timer >= self.duration { self.active = false; }
    }

    pub fn alive(&self) -> bool { self.active }

    /// Emit particles into the pool. Returns true if particles were emitted.
    pub fn emit_particles(&mut self, pool: &mut ParticlePool) -> bool {
        if self.particles_emitted { return false; }
        self.particles_emitted = true;

        match self.kind {
            SkillEffectKind::Fireball => {
                pool.burst(self.position, 24, 120.0, Color::rgba(1.0, 0.5, 0.0, 1.0), 4.0, 0.8, 100.0);
                pool.burst(self.position, 12, 60.0, Color::rgba(1.0, 0.9, 0.2, 1.0), 3.0, 0.5, 80.0);
            }
            SkillEffectKind::IceShard => {
                pool.burst(self.position, 18, 100.0, Color::rgba(0.4, 0.7, 1.0, 1.0), 3.0, 1.0, 0.0);
                pool.burst(self.position, 8, 40.0, Color::rgba(0.8, 0.95, 1.0, 1.0), 2.0, 0.8, 0.0);
            }
            SkillEffectKind::Lightning => {
                pool.burst(self.position, 30, 200.0, Color::rgba(0.6, 0.8, 1.0, 1.0), 2.0, 0.4, 0.0);
                pool.burst(self.position, 15, 80.0, Color::rgba(1.0, 1.0, 0.8, 1.0), 3.0, 0.3, 0.0);
            }
            SkillEffectKind::Heal => {
                pool.burst(self.position, 16, 30.0, Color::rgba(0.2, 0.9, 0.3, 1.0), 3.0, 1.2, -60.0);
            }
            SkillEffectKind::Slash => {
                for i in 0..20 {
                    let angle = (i as f32 / 20.0) * std::f32::consts::PI - std::f32::consts::FRAC_PI_2;
                    let vel = Vec2::new(angle.cos() * 150.0, angle.sin() * 80.0);
                    use crate::engine::particle::ParticleAdvanced;
                    pool.spawn(ParticleAdvanced {
                        position: self.position,
                        velocity: vel,
                        lifetime: 0.3, max_lifetime: 0.3, size: 3.0,
                        color: Color::rgba(0.9, 0.9, 0.95, 1.0), alpha: 1.0,
                        rotation: angle, rotation_speed: 10.0, gravity: 0.0,
                    });
                }
            }
            SkillEffectKind::Explosion => {
                pool.burst(self.position, 40, 180.0, Color::rgba(1.0, 0.4, 0.0, 1.0), 5.0, 1.0, 150.0);
                pool.burst(self.position, 20, 100.0, Color::rgba(1.0, 0.8, 0.1, 1.0), 4.0, 0.6, 100.0);
                pool.burst(self.position, 10, 50.0, Color::rgba(0.5, 0.5, 0.5, 1.0), 6.0, 1.2, -20.0);
            }
            SkillEffectKind::HealRing => {
                for i in 0..24 {
                    let angle = (i as f32 / 24.0) * std::f32::consts::TAU;
                    use crate::engine::particle::ParticleAdvanced;
                    pool.spawn(ParticleAdvanced {
                        position: Vec2::new(self.position.x + angle.cos() * 20.0, self.position.y + angle.sin() * 20.0),
                        velocity: Vec2::new(angle.cos() * 10.0, angle.sin() * 10.0 - 30.0),
                        lifetime: 1.0, max_lifetime: 1.0, size: 3.0,
                        color: Color::rgba(0.3, 1.0, 0.4, 1.0), alpha: 1.0,
                        rotation: 0.0, rotation_speed: 2.0, gravity: -40.0,
                    });
                }
            }
            SkillEffectKind::Poison => {
                pool.burst(self.position, 14, 40.0, Color::rgba(0.4, 0.8, 0.1, 1.0), 3.0, 1.5, -30.0);
                pool.burst(self.position, 8, 20.0, Color::rgba(0.2, 0.6, 0.05, 1.0), 4.0, 2.0, -20.0);
            }
            SkillEffectKind::DarkPulse => {
                pool.burst(self.position, 20, 80.0, Color::rgba(0.3, 0.0, 0.5, 1.0), 4.0, 0.8, 0.0);
                pool.burst(self.position, 12, 50.0, Color::rgba(0.6, 0.0, 0.8, 1.0), 3.0, 0.6, 0.0);
            }
        }
        true
    }
}

// ---------------------------------------------------------------------------
// SkillEffectManager — manages active skill effects
// ---------------------------------------------------------------------------

pub struct SkillEffectManager {
    effects: Vec<SkillEffect>,
    max_effects: usize,
}

impl SkillEffectManager {
    pub fn new(max_effects: usize) -> Self {
        Self { effects: Vec::with_capacity(max_effects), max_effects }
    }

    pub fn spawn(&mut self, effect: SkillEffect) {
        if self.effects.len() < self.max_effects {
            self.effects.push(effect);
        }
    }

    pub fn spawn_at(&mut self, kind: SkillEffectKind, pos: Vec2) {
        self.spawn(SkillEffect::new(kind, pos));
    }

    pub fn update(&mut self, dt: f32, pool: &mut ParticlePool) {
        for effect in &mut self.effects {
            effect.update(dt);
            if effect.alive() {
                effect.emit_particles(pool);
            }
        }
        self.effects.retain(|e| e.alive());
    }

    pub fn active_count(&self) -> usize { self.effects.len() }
    pub fn clear(&mut self) { self.effects.clear(); }
}

impl Default for SkillEffectManager {
    fn default() -> Self { Self::new(32) }
}

// ---------------------------------------------------------------------------
// ScreenShake — standalone screen shake controller
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ScreenShake {
    pub intensity: f32,
    pub decay: f32,
    pub offset: Vec2,
    pub duration: f32,
    pub timer: f32,
}

impl ScreenShake {
    pub fn new() -> Self {
        Self { intensity: 0.0, decay: 8.0, offset: Vec2::zero(), duration: 0.0, timer: 0.0 }
    }

    pub fn trigger(&mut self, intensity: f32) {
        self.intensity = intensity;
        self.duration = intensity / 50.0; // rough seconds
        self.timer = 0.0;
    }

    pub fn trigger_timed(&mut self, intensity: f32, duration: f32) {
        self.intensity = intensity;
        self.duration = duration;
        self.timer = 0.0;
    }

    pub fn update(&mut self, dt: f32) {
        if self.intensity > 0.1 {
            self.timer += dt;
            self.offset = Vec2::new(
                pseudo_random_f32_range(-1.0, 1.0) * self.intensity,
                pseudo_random_f32_range(-1.0, 1.0) * self.intensity,
            );
            self.intensity *= (-self.decay * dt).exp();
        } else {
            self.intensity = 0.0;
            self.offset = Vec2::zero();
        }
    }

    pub fn is_shaking(&self) -> bool { self.intensity > 0.1 }
}

impl Default for ScreenShake {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// EffectsRenderer — orchestrates all game effects
// ---------------------------------------------------------------------------

pub struct EffectsRenderer {
    pub floating_numbers: FloatingNumberManager,
    pub skill_effects: SkillEffectManager,
    pub screen_shake: ScreenShake,
}

impl EffectsRenderer {
    pub fn new() -> Self {
        Self {
            floating_numbers: FloatingNumberManager::new(64),
            skill_effects: SkillEffectManager::new(32),
            screen_shake: ScreenShake::new(),
        }
    }

    pub fn update(&mut self, dt: f32, particle_pool: &mut crate::engine::particle::ParticlePool) {
        self.floating_numbers.update(dt);
        self.skill_effects.update(dt, particle_pool);
        self.screen_shake.update(dt);
    }

    pub fn render(&self, camera: &Camera) -> Vec<DrawCommand> {
        let mut cmds = Vec::new();
        cmds.extend(self.floating_numbers.render(camera));
        cmds
    }

    pub fn clear(&mut self) {
        self.floating_numbers.clear();
        self.skill_effects.clear();
    }
}

impl Default for EffectsRenderer {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pseudo_random_f32_range(min: f32, max: f32) -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let t = (nanos % 10000) as f32 / 10000.0;
    min + (max - min) * t
}

fn pseudo_random_f32() -> f32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 10000) as f32 / 10000.0
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_number() {
        let mut num = FloatingNumber::damage(Vec2::new(100.0, 100.0), 50);
        assert!(num.alive());
        assert_eq!(num.text, "-50");
        assert_eq!(num.kind, FloatingKind::Damage);
        num.update(1.5);
        assert!(!num.alive());
    }

    #[test]
    fn test_heal_number() {
        let num = FloatingNumber::heal(Vec2::zero(), 30);
        assert_eq!(num.text, "+30");
        assert_eq!(num.kind, FloatingKind::Heal);
    }

    #[test]
    fn test_crit_number() {
        let num = FloatingNumber::crit(Vec2::zero(), 999);
        assert_eq!(num.text, "-999!");
        assert_eq!(num.size, 24.0);
    }

    #[test]
    fn test_miss_number() {
        let num = FloatingNumber::miss(Vec2::zero());
        assert_eq!(num.text, "MISS");
    }

    #[test]
    fn test_exp_number() {
        let num = FloatingNumber::exp(Vec2::zero(), 100);
        assert_eq!(num.text, "+100 EXP");
    }

    #[test]
    fn test_floating_manager_spawn_limit() {
        let mut mgr = FloatingNumberManager::new(4);
        for _ in 0..10 {
            mgr.spawn(FloatingNumber::damage(Vec2::zero(), 10));
        }
        assert_eq!(mgr.active_count(), 4);
    }

    #[test]
    fn test_floating_manager_update_removes_dead() {
        let mut mgr = FloatingNumberManager::new(8);
        mgr.spawn_damage(Vec2::zero(), 10);
        mgr.update(2.0);
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn test_skill_effect_lifecycle() {
        let mut effect = SkillEffect::new(SkillEffectKind::Fireball, Vec2::zero());
        assert!(effect.alive());
        assert!(!effect.particles_emitted);
        let mut pool = crate::engine::particle::ParticlePool::new(256);
        effect.emit_particles(&mut pool);
        assert!(effect.particles_emitted);
        assert!(pool.active_count() > 0);
        effect.update(1.0);
        assert!(!effect.alive());
    }

    #[test]
    fn test_screen_shake() {
        let mut shake = ScreenShake::new();
        shake.trigger(20.0);
        assert!(shake.is_shaking());
        shake.update(0.1);
        assert!(shake.intensity < 20.0);
    }

    #[test]
    fn test_screen_shake_decays() {
        let mut shake = ScreenShake::new();
        shake.trigger(10.0);
        for _ in 0..60 { shake.update(0.016); }
        assert!(!shake.is_shaking());
    }

    #[test]
    fn test_effects_renderer() {
        let mut effects = EffectsRenderer::new();
        effects.floating_numbers.spawn_damage(Vec2::new(10.0, 20.0), 42);
        effects.skill_effects.spawn_at(SkillEffectKind::Heal, Vec2::zero());
        let mut pool = crate::engine::particle::ParticlePool::new(256);
        effects.update(0.016, &mut pool);
        assert!(effects.floating_numbers.active_count() > 0);
    }
}

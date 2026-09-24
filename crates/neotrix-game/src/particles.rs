// 粒子系统 — 吸收 Godot GPUParticles2D + GPP 对象池.
//
// 公理：固定槽位预分配 + 空闲栈（free-list），运行时零分配；
// 池满丢弃新粒子（满屏时不可见，标准策略）；槽位复用全量替换初始化。

use macroquad::prelude::{Color, YELLOW, RED, ORANGE, SKYBLUE, draw_text, draw_rectangle};
use macroquad::rand::gen_range;

const POOL_SIZE: usize = 200;

#[derive(Debug, Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: Color,
    pub text: Option<String>,
}

impl Particle {
    fn dead() -> Self {
        Self {
            x: 0.0, y: 0.0, vx: 0.0, vy: 0.0,
            life: 0.0, max_life: 1.0, size: 0.0,
            color: Color::new(0.0, 0.0, 0.0, 0.0),
            text: None,
        }
    }

    fn alive(&self) -> bool {
        self.life > 0.0
    }
}

pub struct ParticleSystem {
    slots: Vec<Particle>,
    free: Vec<usize>,
    alive: usize,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            slots: vec![Particle::dead(); POOL_SIZE],
            free: (0..POOL_SIZE).rev().collect(),
            alive: 0,
        }
    }

    /// 当前存活粒子数
    pub fn count(&self) -> usize { self.alive }

    /// 容量（池大小固定）
    pub fn capacity(&self) -> usize { self.slots.len() }

    /// spawn 原语（整结构替换 = 全量初始化，无随机，供单测与调用方）
    /// 池满返回 false（调用方丢弃）
    pub fn spawn_raw(&mut self, p: Particle) -> bool {
        match self.free.pop() {
            Some(i) => {
                self.slots[i] = p;
                self.alive += 1;
                true
            }
            None => false,
        }
    }

    pub fn spawn_damage_number(&mut self, x: f32, y: f32, damage: f64, critical: bool) {
        let text = if critical { format!("暴击 {:.0}!", damage) } else { format!("-{:.0}", damage) };
        self.spawn_raw(Particle {
            x: x + gen_range(-10.0, 10.0),
            y: y - 20.0,
            vx: gen_range(-20.0, 20.0),
            vy: -80.0,
            life: 1.2, max_life: 1.2,
            size: if critical { 22.0 } else { 16.0 },
            color: if critical { YELLOW } else { RED },
            text: Some(text),
        });
    }

    pub fn spawn_hit_effect(&mut self, x: f32, y: f32) {
        for _ in 0..8 {
            let angle = gen_range(0.0, std::f32::consts::TAU);
            let speed = gen_range(50.0, 150.0);
            if !self.spawn_raw(Particle {
                x, y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life: 0.5, max_life: 0.5,
                size: gen_range(2.0, 5.0),
                color: ORANGE,
                text: None,
            }) {
                break;
            }
        }
    }

    pub fn spawn_level_up(&mut self, x: f32, y: f32) {
        for _ in 0..12 {
            let angle = gen_range(0.0, std::f32::consts::TAU);
            let speed = gen_range(30.0, 100.0);
            if !self.spawn_raw(Particle {
                x, y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed - 50.0,
                life: 1.5, max_life: 1.5,
                size: gen_range(3.0, 6.0),
                color: YELLOW,
                text: None,
            }) {
                break;
            }
        }
        self.spawn_raw(Particle {
            x, y: y - 30.0,
            vx: 0.0, vy: -40.0,
            life: 2.0, max_life: 2.0,
            size: 24.0,
            color: YELLOW,
            text: Some("LEVEL UP!".to_string()),
        });
    }

    pub fn spawn_interact_hint(&mut self, x: f32, y: f32) {
        self.spawn_raw(Particle {
            x, y: y - 40.0,
            vx: 0.0, vy: -15.0,
            life: 1.5, max_life: 1.5,
            size: 14.0,
            color: SKYBLUE,
            text: Some("[E] 互动".to_string()),
        });
    }

    pub fn update(&mut self, dt: f32) {
        for i in 0..self.slots.len() {
            let died = {
                let p = &mut self.slots[i];
                if !p.alive() {
                    continue;
                }
                p.x += p.vx * dt;
                p.y += p.vy * dt;
                p.vy += 100.0 * dt;
                p.life -= dt;
                !p.alive()
            };
            if died {
                self.free.push(i);
                self.alive -= 1;
            }
        }
    }

    pub fn render(&self) {
        for p in &self.slots {
            if !p.alive() {
                continue;
            }
            let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
            if let Some(ref text) = p.text {
                let c = Color::new(p.color.r, p.color.g, p.color.b, alpha);
                draw_text(text, p.x, p.y, p.size, c);
            } else {
                let c = Color::new(p.color.r, p.color.g, p.color.b, alpha);
                draw_rectangle(p.x - p.size/2.0, p.y - p.size/2.0, p.size, p.size, c);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn live(x: f32, life: f32) -> Particle {
        Particle {
            x, y: 0.0, vx: 10.0, vy: 0.0,
            life, max_life: life.max(0.1),
            size: 3.0, color: RED, text: None,
        }
    }

    #[test]
    fn pool_caps_and_recycles() {
        let mut ps = ParticleSystem::new();
        assert_eq!(ps.count(), 0);
        for i in 0..POOL_SIZE {
            assert!(ps.spawn_raw(live(i as f32, 5.0)), "slot {}", i);
        }
        assert_eq!(ps.count(), POOL_SIZE);
        assert!(!ps.spawn_raw(live(0.0, 5.0))); // 满池丢弃
        ps.update(10.0); // 全灭
        assert_eq!(ps.count(), 0);
        assert!(ps.spawn_raw(live(1.0, 5.0))); // 槽位回收
        assert_eq!(ps.count(), 1);
    }

    #[test]
    fn no_alloc_after_warm() {
        let mut ps = ParticleSystem::new();
        let (c0, f0) = (ps.slots.capacity(), ps.free.capacity());
        for _ in 0..5 {
            for i in 0..POOL_SIZE {
                let _ = ps.spawn_raw(live(i as f32, 0.05));
            }
            ps.update(1.0);
        }
        assert_eq!(ps.slots.capacity(), c0);
        assert_eq!(ps.free.capacity(), f0); // 空闲栈不伸缩
        assert_eq!(ps.count(), 0);
    }

    #[test]
    fn update_integrates_and_kills() {
        let mut ps = ParticleSystem::new();
        ps.spawn_raw(live(0.0, 1.0));
        ps.update(0.5);
        assert_eq!(ps.count(), 1);
        let x = ps.slots.iter().find(|p| p.alive()).map_or(-1.0, |p| p.x);
        assert!((x - 5.0).abs() < 1e-4);
        ps.update(1.0);
        assert_eq!(ps.count(), 0);
    }
}

//! neotrix-game 引擎演示 —— 游戏体在 `games/neotrix-guixu`，本 demo 只用 engine lib：
//! ECS 生成/移动/绘制 + 点击粒子 + 死区相机跟随 + 输入。无游戏内容。

use macroquad::prelude::*;
use neotrix_game::components::{Position, Rgba, Sprite, Velocity};
use neotrix_game::ecs::SimpleEcs;
use neotrix_game::nt_behavior::{Behavior, BehaviorRegistry};
use neotrix_game::input::InputState;
use neotrix_game::nt_camera::{follow, Deadzone};
use neotrix_game::nt_object as obj;
use neotrix_game::nt_timer::Scheduler;
use neotrix_game::particles::ParticleSystem;

fn window_conf() -> Conf {
    Conf {
        window_title: "NeoTrix Engine Demo".into(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        ..Default::default()
    }
}

/// 往返运动 Behavior（引擎自举示例：逻辑块可挂载/开关/复用，不再手写进主循环）。
struct PingPong {
    min_x: f32,
    max_x: f32,
}

impl Behavior for PingPong {
    fn name(&self) -> &'static str {
        "pingpong"
    }
    fn on_update(&mut self, id: u64, world: &mut SimpleEcs, dt: f32) {
        let flip = match world.get::<Position>(id) {
            Some(p) => p.x > self.max_x || p.x < self.min_x,
            None => false,
        };
        if flip {
            if let Some(v) = world.get_mut::<Velocity>(id) {
                v.vx = -v.vx;
            }
        }
        let vx = world.get::<Velocity>(id).map_or(0.0, |v| v.vx);
        if let Some(p) = world.get_mut::<Position>(id) {
            p.x += vx * dt;
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut ecs = SimpleEcs::new();
    let mut behaviors = BehaviorRegistry::new();
    let mut input = InputState::new();
    let mut fx = ParticleSystem::new();
    let dz = Deadzone::platformer();
    let (mut cx, mut cy) = (640.0, 360.0);

    // 测试移动体：挂载 PingPong Behavior（相机死区跟随它）
    let mover = ecs.spawn();
    ecs.insert(mover, Position { x: 640.0, y: 360.0 });
    ecs.insert(mover, Velocity { vx: 120.0, vy: 0.0 });
    ecs.insert(
        mover,
        Sprite { w: 24.0, h: 24.0, color: Rgba::from(YELLOW) },
    );
    behaviors.attach(
        mover,
        &mut ecs,
        Box::new(PingPong { min_x: 400.0, max_x: 880.0 }),
    );
    // 三引擎吸收接线：KAPLAY 对象标签 + 定时调度（每秒一次心跳粒子）
    obj::tag(&mut ecs, mover, "player");
    obj::tag(&mut ecs, mover, "mover");
    let mut sched = Scheduler::new();
    let beat = sched.every(1.0);

    loop {
        let dt = get_frame_time().clamp(0.0, 0.05);
        input.update();

        // Behavior 驱动（含 PingPong 往返）
        behaviors.tick(&mut ecs, dt);
        // 调度器心跳：每秒在屏幕中心冒一簇粒子（证明调度器已进主循环）
        for fire in sched.update(dt) {
            if fire.id == beat {
                let _ = ecs.get::<Position>(mover);
                fx.spawn_hit_effect(640.0, 360.0);
            }
        }
        // 点击生粒子（打击感通道冒烟）
        if let Some((mx, my)) = input.clicked {
            fx.spawn_hit_effect(mx, my);
        }
        fx.update(dt);

        // 死区相机跟随
        if let Some(p) = ecs.get::<Position>(mover) {
            (cx, cy) = follow((cx, cy), (p.x, p.y), dt, &dz);
        }

        clear_background(Color::new(0.02, 0.03, 0.05, 1.0));
        // 世界层（相机偏移演示：直接平移绘制）
        for (_, p, s) in ecs.query2::<Position, Sprite>() {
            draw_rectangle(p.x - cx - s.w / 2.0, p.y - cy - s.h / 2.0, s.w, s.h, s.color.into());
        }
        fx.render();
        // HUD（屏幕层，不跟相机）
        draw_text(
            &format!(
                "engine demo | fps {} | particles {} | tagged player:{} t={:.1}",
                get_fps(),
                fx.count(),
                obj::get_by_tag(&ecs, "player").len(),
                sched.now(),
            ),
            12.0,
            24.0,
            20.0,
            WHITE,
        );
        draw_text("click: particles | engine lib only, game lives in neotrix-swords", 12.0, 48.0, 16.0, GRAY);
        next_frame().await;
    }
}

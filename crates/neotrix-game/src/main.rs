//! neotrix-game 引擎演示 — Slice-4b 后 game bin 仅留引擎 demo.
//!
//! 游戏体已迁 `games/neotrix-cards`（双跑道 B）。本 demo 只用 engine lib：
//! ECS 生成/移动/绘制 + 点击粒子 + 死区相机跟随 + 输入。无游戏内容。

use macroquad::prelude::*;
use neotrix_game::components::{Position, Sprite, Velocity};
use neotrix_game::ecs::SimpleEcs;
use neotrix_game::input::InputState;
use neotrix_game::nt_camera::{follow, Deadzone};
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

#[macroquad::main(window_conf)]
async fn main() {
    let mut ecs = SimpleEcs::new();
    let mut input = InputState::new();
    let mut fx = ParticleSystem::new();
    let dz = Deadzone::platformer();
    let (mut cx, mut cy) = (640.0, 360.0);

    // 测试移动体：水平往返（相机死区跟随它）
    let mover = ecs.spawn();
    ecs.insert(mover, Position { x: 640.0, y: 360.0 });
    ecs.insert(mover, Velocity { vx: 120.0, vy: 0.0 });
    ecs.insert(
        mover,
        Sprite { w: 24.0, h: 24.0, color: YELLOW },
    );

    loop {
        let dt = get_frame_time().clamp(0.0, 0.05);
        input.update();

        // 往返运动（碰边翻转速度，ECS 自举）
        let flip = match ecs.get::<Position>(mover) {
            Some(p) => p.x > 880.0 || p.x < 400.0,
            None => false,
        };
        if flip {
            if let Some(v) = ecs.get_mut::<Velocity>(mover) {
                v.vx = -v.vx;
            }
        }
        let vx = ecs.get::<Velocity>(mover).map_or(0.0, |v| v.vx);
        if let Some(p) = ecs.get_mut::<Position>(mover) {
            p.x += vx * dt;
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
            draw_rectangle(p.x - cx - s.w / 2.0, p.y - cy - s.h / 2.0, s.w, s.h, s.color);
        }
        fx.render();
        // HUD（屏幕层，不跟相机）
        draw_text(
            &format!("engine demo | fps {} | particles {}", get_fps(), fx.count()),
            12.0,
            24.0,
            20.0,
            WHITE,
        );
        draw_text("click: particles | engine lib only, game lives in neotrix-swords", 12.0, 48.0, 16.0, GRAY);
        next_frame().await;
    }
}

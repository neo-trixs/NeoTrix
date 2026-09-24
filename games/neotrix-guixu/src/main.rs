#![forbid(unsafe_code)]
/// 歸墟 — 南宋武侠动作游戏（吸收 swordgame.ai）
///
/// 流程：标题（任意键）→ 战斗（波次）→ 暂停（Esc）→ 劍折（R/空格重开）。
/// 操作：A/D 或 ←/→ 移动 · J 攻击 · K 跳/二段跳 · L 轻功冲刺（耗氣 12）
///       Q 飛笠（耗氣 20，3s cd）· Shift 疾跑 · Esc 暂停。
/// 引擎件：ECS + 输入表（移动） + 粒子 + juice + 死区前视相机 + nt_move 跳跃 kit。

use macroquad::prelude::*;
use neotrix_game::components::{Health, Position, Sprite, Velocity};
use neotrix_game::ecs::SimpleEcs;
use neotrix_game::input::InputState;
use neotrix_game::nt_camera::{follow_lookahead, Deadzone};
use neotrix_game::nt_juice::Juice;
use neotrix_game::nt_move::{JumpKind, MoveState, MoveTune};
use neotrix_game::nt_platform::{move_on_platforms, overlap};
use neotrix_game::particles::ParticleSystem;
use neotrix_game::audio::{AudioSystem, SoundEffect};
use neotrix_abilities::nt_tuning::TierTable;
use macroquad::rand::gen_range;
use std::collections::HashMap;

mod combo;
mod qi;
mod waves;
mod story;
mod arts;
mod mastery;
mod damage;
mod lore;
mod atlas;

use combo::ComboTracker;
use qi::Qi;
use mastery::{WeaponMastery, AttrTrack};
use damage::{resolve, roll_crit, BACKSTAB_MULT, JUGGLE_MULT};
use waves::{composition, scale_of, FoeSpec, ELITE};
use arts::{match_art, Stroke, ArtGate, MoveDef, WEAPONS};
use story::{Story, Effect};
use atlas::AtlasFont;

const GRAV: f32 = 1500.0;
const MOVE_SPEED: f32 = 260.0;
const SPRINT_MULT: f32 = 1.6;
const MAX_FALL: f32 = 900.0;
const ATK_DMG: f32 = 18.0;
/// 基础暴击率（PoE 式，耗尽前 announce？不，此处静默纳入管线）
const CRIT_CH: f32 = 0.1;
const HAT_CD: f32 = 3.0;
const HAT_COST: f32 = 20.0;
const DASH_COST: f32 = 12.0;
const QI_PER_HIT: f32 = 8.0;
const MAX_FOES: usize = 8;

/// 平台（左上系 x,y,w,h）
fn platforms() -> Vec<(f32, f32, f32, f32)> {
    vec![
        (0.0, 640.0, 1280.0, 80.0), // 地面
        (180.0, 480.0, 220.0, 24.0),
        (520.0, 360.0, 200.0, 24.0),
        (860.0, 470.0, 220.0, 24.0),
    ]
}

/// AABB 平台物理与重叠判定已上收引擎（`neotrix_game::nt_platform`），此处不重复实现。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunState {
    Title,
    Play,
    Pause,
    Story,
    Over,
}

struct FoeMeta {
    dmg: f32,
    speed: f32,
    touch_cd: f32,
    elite: bool,
    score: u32,
    kbx: f32,
    armor: f32,
    facing: f32,
    airborne: bool,
}

struct Hat {
    x: f32,
    y: f32,
    vx: f32,
    life: f32,
    hit: Vec<u64>,
}

struct Player {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    w: f32,
    h: f32,
    hp: f32,
    max_hp: f32,
    facing: f32,
    atk_cd: f32,
    hat_cd: f32,
    hurt_cd: f32,
    grounded_flag: bool,
    mv: MoveState,
}

struct Game {
    state: RunState,
    player: Player,
    ecs: SimpleEcs,
    foes: Vec<u64>,
    meta: HashMap<u64, FoeMeta>,
    hats: Vec<Hat>,
    wave: u32,
    score: u32,
    kills: u32,
    time: f32,
    combo: ComboTracker,
    qi: Qi,
    special_flash: (String, f32),
    banner: (String, f32),
    story: Option<Story>,
    elite_bank: u32,
    spawn_queue: Vec<FoeSpec>,
    spawn_t: f32,
    tune: MoveTune,
    tiers: TierTable,
    input: InputState,
    fx: ParticleSystem,
    juice: Juice,
    audio: AudioSystem,
    cam: (f32, f32),
    dz: Deadzone,
    prev_k: bool,
    prev_l: bool,
    prev_q: bool,
    prev_esc: bool,
    prev_any: bool,
    weapon: usize,
    dirbuf: Vec<(Stroke, f32)>,
    prev_dirs: (bool, bool, bool, bool),
    wmas: [WeaponMastery; 2],
    grit: AttrTrack,
    arm: AttrTrack,
    auto_t: f32,
    atlas: Option<AtlasFont>,
}

impl Game {
    fn new() -> Self {
        Self {
            state: RunState::Title,
            player: Player {
                x: 640.0,
                y: 300.0,
                vx: 0.0,
                vy: 0.0,
                w: 28.0,
                h: 44.0,
                hp: 100.0,
                max_hp: 100.0,
                facing: 1.0,
                atk_cd: 0.0,
                hat_cd: 0.0,
                hurt_cd: 0.0,
                grounded_flag: false,
                mv: MoveState::new(),
            },
            ecs: SimpleEcs::new(),
            foes: Vec::new(),
            meta: HashMap::new(),
            hats: Vec::new(),
            wave: 0,
            score: 0,
            kills: 0,
            time: 0.0,
            combo: ComboTracker::new(),
            qi: Qi::new(100.0),
            special_flash: (String::new(), 0.0),
            banner: (String::new(), 0.0),
            story: None,
            elite_bank: 0,
            spawn_queue: Vec::new(),
            spawn_t: 0.0,
            tune: MoveTune::platformer(),
            tiers: TierTable::default_juice(),
            input: InputState::new(),
            fx: ParticleSystem::new(),
            juice: Juice::new(),
            audio: AudioSystem::new(),
            cam: (640.0, 360.0),
            dz: Deadzone::platformer(),
            prev_k: false,
            prev_l: false,
            prev_q: false,
            prev_esc: false,
            prev_any: false,
            weapon: 0,
            dirbuf: Vec::new(),
            prev_dirs: (false, false, false, false),
            wmas: [WeaponMastery::new(), WeaponMastery::new()],
            grit: AttrTrack::new(40),
            arm: AttrTrack::new(40),
            auto_t: 0.0,
            atlas: None,
        }
    }

    fn new_run(&mut self) {
        let fresh = Self::new();
        self.player = fresh.player;
        self.ecs = SimpleEcs::new();
        self.foes.clear();
        self.meta.clear();
        self.hats.clear();
        self.wave = 0;
        self.score = 0;
        self.kills = 0;
        self.time = 0.0;
        self.combo = ComboTracker::new();
        self.qi = Qi::new(100.0);
        self.dirbuf.clear();
        self.wmas = [WeaponMastery::new(), WeaponMastery::new()];
        self.grit = AttrTrack::new(40);
        self.arm = AttrTrack::new(40);
        self.auto_t = 0.0;
        self.special_flash = (String::new(), 0.0);
        self.banner = (String::new(), 0.0);
        self.story = None;
        self.elite_bank = 0;
        self.spawn_queue.clear();
        self.state = RunState::Play;
        self.next_wave();
    }

    fn next_wave(&mut self) {
        self.wave += 1;
        let mut comp = composition(self.wave);
        // 改线兑现：剧情存入的精锐插到队首（ModifyEvent 精神）
        for _ in 0..self.elite_bank {
            comp.insert(0, ELITE);
        }
        self.elite_bank = 0;
        self.spawn_queue = comp;
        self.spawn_t = 0.0;
        let sect = lore::chapter_sect(self.wave);
        self.banner = (
            format!("第 {} 波 · {}（{}）", self.wave, lore::chapter_name(self.wave), sect.creed),
            2.5,
        );
        self.audio.play(SoundEffect::Confirm);
        // 章节剧情触发（jynew 触发器三槽精神：波次即场景，进入即检）
        if let Some(ev) = Self::story_for_wave(self.wave) {
            self.story = Some(ev);
            self.state = RunState::Story;
            self.drain_story();
        }
    }

    /// 波次剧情表（v1 两幕；后续波次按需加行）
    fn story_for_wave(wave: u32) -> Option<Story> {
        use story::{ChoiceOpt, Cmd};
        match wave {
            1 => Some(Story::new(vec![
                Cmd::Do(vec![Effect::GiveQi(10.0)]),
                Cmd::Say { who: "说书人".into(), text: "话说南宋绍兴年间，烽火初歇，江湖又起。".into() },
                Cmd::Say { who: "说书人".into(), text: "一剑在手，且从第一波宵小杀起。".into() },
                Cmd::Say { who: "少侠".into(), text: "正合我意。".into() },
                Cmd::End,
            ])),
            4 => Some(Story::new(vec![
                Cmd::Say { who: "夜行人".into(), text: "前方破庙可避风，且歇一夜？".into() },
                Cmd::Choice {
                    prompt: "夜宿破庙，如何？".into(),
                    options: vec![
                        ChoiceOpt {
                            text: "打坐调息（回60命+20氣）".into(),
                            goto: 2,
                            effects: vec![Effect::Heal(60.0), Effect::GiveQi(20.0)],
                        },
                        ChoiceOpt {
                            text: "秉烛夜练（下波+1精锐，得30氣10分）".into(),
                            goto: 3,
                            effects: vec![
                                Effect::AddElite(1),
                                Effect::GiveQi(30.0),
                                Effect::Score(10),
                            ],
                        },
                    ],
                },
                Cmd::Say { who: "你".into(), text: "气息绵长，明日再战。".into() },
                Cmd::Say { who: "你".into(), text: "剑气又利三分，明日见分晓。".into() },
                Cmd::End,
            ])),
            7 => Some(Story::new(vec![
                Cmd::Say { who: "刀客".into(), text: "站住！天王盖地虎！".into() },
                Cmd::Choice {
                    prompt: "盘道（切口对答）".into(),
                    options: vec![
                        ChoiceOpt {
                            text: lore::slang_reply("天王盖地虎")
                                .unwrap_or("宝塔镇河妖")
                                .into(),
                            goto: 2,
                            effects: vec![Effect::GiveQi(30.0), Effect::Score(10)],
                        },
                        ChoiceOpt {
                            text: "拔刀便砍".into(),
                            goto: 3,
                            effects: vec![Effect::AddElite(1)],
                        },
                    ],
                },
                Cmd::Say { who: "刀客".into(), text: "好汉子，请！".into() },
                Cmd::Say { who: "刀客".into(), text: "嘿，点子扎手，并肩子上！".into() },
                Cmd::End,
            ])),
            _ => None,
        }
    }

    fn spawn_foe(&mut self, spec: FoeSpec, side: f32) {
        let s = scale_of(self.wave);
        // 门派章加成（lore 门派轮转，jynew 场景差异精神）
        let sect = lore::chapter_sect(self.wave).mults;
        let id = self.ecs.spawn();
        let x = if side > 0.0 { 1220.0 } else { 60.0 };
        self.ecs.insert(id, Position { x, y: 80.0 });
        self.ecs.insert(id, Velocity { vx: 0.0, vy: 0.0 });
        let (w, h) = if spec.elite { (36.0, 46.0) } else { (30.0, 36.0) };
        self.ecs.insert(
            id,
            Sprite { w, h, color: if spec.elite { ORANGE } else { RED } },
        );
        self.ecs.insert(
            id,
            Health { hp: spec.hp * s * sect.0, max_hp: spec.hp * s * sect.0 },
        );
        self.meta.insert(
            id,
            FoeMeta {
                dmg: spec.dmg * s * sect.1,
                speed: spec.speed * sect.2,
                touch_cd: 0.0,
                elite: spec.elite,
                score: spec.score,
                kbx: 0.0,
                armor: spec.armor + ((self.wave / 2).min(4) as f32),
                facing: -side,
                airborne: true,
            },
        );
        self.foes.push(id);
    }

    /// 玩家移动段（借用域独立：仅 player 可变，input/tune/fx 为不交叠只读/字段借用）
    fn update_player(&mut self, dt: f32, plats: &[(f32, f32, f32, f32)]) {
        let p = &mut self.player;
        // ── 移动（引擎输入表：方向可重绑） ──
        let mut dir: f32 = 0.0;
        if self.input.pressed("left") {
            dir -= 1.0;
        }
        if self.input.pressed("right") {
            dir += 1.0;
        }
        // ── 方向笔划缓冲（流星语法；左镜像右，0.6s 过期；字段借用与 p 不交叠） ──
        {
            let held = (
                self.input.pressed("up"),
                self.input.pressed("down"),
                self.input.pressed("left"),
                self.input.pressed("right"),
            );
            let (pu, pd, pl, pr) = self.prev_dirs;
            let (hu, hd, hl, hr) = held;
            let now = self.time;
            if hu && !pu {
                self.dirbuf.push((Stroke::Up, now));
            }
            if hd && !pd {
                self.dirbuf.push((Stroke::Down, now));
            }
            if (hl && !pl) || (hr && !pr) {
                self.dirbuf.push((Stroke::Right, now));
            }
            self.prev_dirs = held;
            while self.dirbuf.len() > 4 {
                self.dirbuf.remove(0);
            }
            self.dirbuf.retain(|(_, t)| now - t <= 0.6);
        }
        let sprint = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        let speed = MOVE_SPEED * if sprint { SPRINT_MULT } else { 1.0 };
        if dir != 0.0 {
            p.facing = dir.signum();
        }

        // ── 跳跃 kit（土狼/缓冲/二段/可变跳高；S+K 跳穿下降） ──
        let k_down = is_key_down(KeyCode::K);
        if k_down && !self.prev_k {
            if self.input.pressed("down") && p.grounded_flag {
                p.mv.drop();
            } else {
                p.mv.want_jump(&self.tune);
            }
        }
        if !k_down && self.prev_k {
            p.vy = MoveState::cut_jump(p.vy, &self.tune);
        }
        self.prev_k = k_down;

        // ── 冲刺 L（耗氣） ──
        let l_down = is_key_down(KeyCode::L);
        if l_down && !self.prev_l && self.qi.spend(DASH_COST) {
            if p.mv.dash(&self.tune).is_none() {
                self.qi.gain(DASH_COST); // cd 中则退款
            }
        }
        self.prev_l = l_down;

        // ── 速度合成 ──
        if p.mv.dashing() {
            p.vx = p.facing * self.tune.dash_speed;
            p.vy = 0.0;
        } else {
            p.vx = dir * speed;
        }
        p.mv.tick(dt);
        let was_ground = p.grounded_flag;
        let grounded = move_on_platforms(&mut p.x, &mut p.y, p.vx, &mut p.vy, p.w, p.h,
            plats, dt, GRAV, MAX_FALL, 1280.0, p.mv.dropping());
        if grounded {
            p.mv.land(&self.tune);
        } else if was_ground {
            p.mv.walk_off(&self.tune);
        }
        p.grounded_flag = grounded;
        if let Some(kind) = p.mv.consume_jump(&self.tune, grounded) {
            p.vy = -self.tune.jump_speed;
            if kind == JumpKind::Air {
                self.fx.spawn_hit_effect(p.x, p.y + 20.0);
            }
        }
    }

    fn update_play(&mut self, dt: f32, plats: &[(f32, f32, f32, f32)]) {
        self.update_player(dt, plats);

        // ── 攻击 J ──
        self.player.atk_cd = (self.player.atk_cd - dt).max(0.0);
        self.player.hat_cd = (self.player.hat_cd - dt).max(0.0);
        self.player.hurt_cd = (self.player.hurt_cd - dt).max(0.0);
        // ── 自动战斗（军师指挥优先，否则最强可负担；出招耗 atk 计时） ──
        self.auto_t -= dt;
        if self.auto_t <= 0.0 {
            let w = WEAPONS[self.weapon.min(WEAPONS.len() - 1)];
            // 寻敌：射程内最低血量（集火战术）
            let range = 130.0 * w.reach;
            let mut target_hp = f32::MAX;
            let mut engaged = false;
            for i in 0..self.foes.len() {
                let id = self.foes[i];
                if let (Some(p), Some(h)) = (self.ecs.get::<Position>(id), self.ecs.get::<Health>(id)) {
                    let d = (p.x - self.player.x).abs() + (p.y - self.player.y).abs();
                    if d < range && h.hp < target_hp {
                        target_hp = h.hp;
                        engaged = true;
                    }
                }
            }
            if engaged {
                self.auto_t = w.swing_cd;
                self.player.atk_cd = w.swing_cd;
                let mut seq: Vec<Stroke> = self.dirbuf.iter().map(|(s, _)| *s).collect();
                seq.push(Stroke::Atk);
                let air = !self.player.grounded_flag;
                let wlvl = self.wmas[self.weapon.min(1)].level;
                let open: Vec<&str> =
                    arts::unlocked(&w, wlvl).iter().map(|m| m.name).collect();
                let commanded = match_art(&w, &seq, air)
                    .filter(|m| open.contains(&m.name) && self.qi.afford(m.qi));
                let pick = commanded
                    .or_else(|| arts::choose_art(&w, wlvl, self.qi.cur(), air));
                if let Some(m) = pick {
                    self.qi.spend(m.qi);
                    self.play_art(m);
                    let wi = self.weapon.min(1);
                    if let Some(nl) = self.wmas[wi].add_use() {
                        self.special_flash = (format!("精進 Lv{}", nl), 1.5);
                        self.audio.play(SoundEffect::LevelUp);
                    }
                }
            } else {
                self.auto_t = 0.15;
            }
            self.dirbuf.clear();
        }

        // ── 飛笠 Q ──
        let q_down = is_key_down(KeyCode::Q);
        if q_down && !self.prev_q && self.player.hat_cd <= 0.0 && self.qi.spend(HAT_COST) {
            self.player.hat_cd = HAT_CD;
            let (hx, hy, hf) = (self.player.x, self.player.y, self.player.facing);
            self.hats.push(Hat {
                x: hx,
                y: hy - 10.0,
                vx: hf * 520.0,
                life: 1.2,
                hit: Vec::new(),
            });
            self.audio.play(SoundEffect::Select);
        }
        self.prev_q = q_down;

        self.update_foes(dt, plats);
        self.update_hats(dt);
        self.update_waves(dt);

        // combo 超时 / 特效计时
        self.combo.timeout(self.time);
        if self.special_flash.1 > 0.0 {
            self.special_flash.1 -= dt;
        }
        if self.banner.1 > 0.0 {
            self.banner.1 -= dt;
        }
        self.fx.update(dt);

        // 死亡
        if self.player.hp <= 0.0 {
            self.state = RunState::Over;
            self.audio.play(SoundEffect::Dialogue);
        }
    }

    /// 剧情副作用 drains（jynew Do 即时语义；终幕即回战斗）
    fn drain_story(&mut self) {
        let fx: Vec<Effect> = match &mut self.story {
            Some(st) => std::mem::take(&mut st.outbox),
            None => return,
        };
        for e in fx {
            match e {
                Effect::GiveQi(n) => self.qi.gain(n),
                Effect::Heal(n) => {
                    self.player.hp = (self.player.hp + n).min(self.player.max_hp);
                }
                Effect::AddElite(n) => {
                    self.elite_bank += n;
                }
                Effect::Score(n) => {
                    self.score += n;
                }
            }
        }
        if self.story.as_ref().map_or(false, |s| s.done) {
            self.story = None;
            if self.state == RunState::Story {
                self.state = RunState::Play;
            }
        }
    }

    /// 出招执行：兵器距离×招式倍率＋击退（流星核心）；有名招闪名
    fn play_art(&mut self, art: &MoveDef) {
        let w = WEAPONS[self.weapon.min(WEAPONS.len() - 1)];
        let (px, py, facing) = (self.player.x, self.player.y, self.player.facing);
        let r = w.reach;
        let (ax0, ax1) = if facing > 0.0 {
            (px + 10.0 * r, px + 80.0 * r)
        } else {
            (px - 80.0 * r, px - 10.0 * r)
        };
        let dmg = ATK_DMG * art.dmg_mult;
        let arm_mult = 1.0 + 0.05 * self.arm.level as f32;
        let base_dmg = dmg * arm_mult;
        let mut dead: Vec<u64> = Vec::new();
        let mut hits = 0u32;
        for i in 0..self.foes.len() {
            let id = self.foes[i];
            let (ex, ey, ew, eh) = match (self.ecs.get::<Position>(id), self.ecs.get::<Sprite>(id)) {
                (Some(p), Some(s)) => (p.x, p.y, s.w, s.h),
                _ => continue,
            };
            if ex > ax0.min(ax1) - ew / 2.0
                && ex < ax0.max(ax1) + ew / 2.0
                && (ey - py).abs() < 30.0 + eh / 2.0
            {
                // 分阶管线：基础×招式×臂力 → 情境（背刺/挑空）→ 暴击 → 护甲穿透 → 方差
                let (armor, backstab, juggle) = match self.meta.get(&id) {
                    Some(m) => (m.armor, (px - ex) * m.facing < 0.0, m.airborne),
                    None => (0.0, false, false),
                };
                let mut premit = base_dmg;
                if backstab {
                    premit *= BACKSTAB_MULT;
                }
                if juggle {
                    premit *= JUGGLE_MULT;
                }
                let crit = roll_crit(gen_range(0.0, 1.0), CRIT_CH);
                let r = resolve(premit, crit, armor, 0.0, gen_range(0.0, 1.0));
                let ups = self.arm.add(1);
                if ups > 0 {
                    self.special_flash = (format!("臂力 Lv{}", self.arm.level), 1.2);
                }
                hits += 1;
                if art.knockback > 0.0 {
                    if let Some(m) = self.meta.get_mut(&id) {
                        m.kbx = facing * art.knockback;
                    }
                    if art.knockback >= 150.0 {
                        if let Some(v) = self.ecs.get_mut::<Velocity>(id) {
                            v.vy = -220.0;
                        }
                    }
                }
                let killed = self.damage_foe(id, r.damage, ex, ey, r.crit);
                if killed {
                    dead.push(id);
                }
            }
        }
        if hits > 0 {
            self.combo.hit(self.time);
            self.qi.gain(QI_PER_HIT * hits as f32);
            self.audio.play(SoundEffect::Hit);
        }
        // 有名招闪名（素招静默，连击数自会说话）
        if art.seq.len() > 1 || art.gate == ArtGate::Air {
            self.special_flash = (art.name.to_string(), 1.5);
        }
        for id in dead {
            self.kill_foe(id);
        }
    }

    /// 结算单体伤害，返回是否击杀（含 juice 分级 + 暴击数字）
    fn damage_foe(&mut self, id: u64, dmg: f32, ex: f32, ey: f32, crit: bool) -> bool {
        if let Some(h) = self.ecs.get_mut::<Health>(id) {
            h.hp = (h.hp - dmg).max(0.0);
        }
        let killed = self.ecs.get::<Health>(id).map_or(false, |h| h.hp <= 0.0);
        let (_, params) = self.tiers.tier_for(dmg, false, killed);
        self.juice.trigger(&params);
        self.fx.spawn_damage_number(ex, ey - 20.0, dmg as f64, crit);
        self.fx.spawn_hit_effect(ex, ey);
        killed
    }

    fn kill_foe(&mut self, id: u64) {
        if let Some(m) = self.meta.get(&id) {
            self.score += m.score;
        }
        self.kills += 1;
        self.ecs.despawn(id);
        self.meta.remove(&id);
        self.foes.retain(|f| *f != id);
    }

    fn update_foes(&mut self, dt: f32, plats: &[(f32, f32, f32, f32)]) {
        let (px, py, pw, ph) = (self.player.x, self.player.y, self.player.w, self.player.h);
        // 追击 + 重力 + 平台
        for i in 0..self.foes.len() {
            let id = self.foes[i];
            let speed = self.meta.get(&id).map_or(90.0, |m| m.speed);
            let (ex, _) = match self.ecs.get::<Position>(id) {
                Some(p) => (p.x, p.y),
                None => continue,
            };
            let dir = if (px - ex).abs() > 8.0 { (px - ex).signum() } else { 0.0 };
            // borrow 分段：先读后写
            let (w, h) = match self.ecs.get::<Sprite>(id) {
                Some(s) => (s.w, s.h),
                None => continue,
            };
            // 击退通道（流星核心）：冲量衰减后叠入寻路速度
            let kb = self.meta.get(&id).map_or(0.0, |m| m.kbx);
            let evx = dir * speed + kb;
            if let Some(v) = self.ecs.get_mut::<Velocity>(id) {
                v.vx = evx;
            }
            // 手动积分（foe 无 nt_move 状态，轻量）
            let (mut fx_, mut fy_, mut fvy_) = (0.0, 0.0, 0.0);
            if let Some(p) = self.ecs.get::<Position>(id) {
                fx_ = p.x;
                fy_ = p.y;
            }
            if let Some(v) = self.ecs.get::<Velocity>(id) {
                fvy_ = v.vy;
            }
            let grounded = move_on_platforms(&mut fx_, &mut fy_, evx, &mut fvy_, w, h, plats, dt, GRAV, MAX_FALL, 1280.0, false);
            if let Some(p) = self.ecs.get_mut::<Position>(id) {
                p.x = fx_;
                p.y = fy_;
            }
            if let Some(v) = self.ecs.get_mut::<Velocity>(id) {
                v.vy = fvy_;
            }
            if let Some(m) = self.meta.get_mut(&id) {
                m.touch_cd = (m.touch_cd - dt).max(0.0);
                m.kbx = arts::kb_decay(m.kbx, dt);
                if dir != 0.0 {
                    m.facing = dir;
                }
                m.airborne = !grounded;
            }
        }
        // 怪间分离（n≤8，O(n²) 可接受；索引直读，零分配）
        for i in 0..self.foes.len() {
            for j in (i + 1)..self.foes.len() {
                let (a, b) = (self.foes[i], self.foes[j]);
                let (pa, pb) = match (self.ecs.get::<Position>(a), self.ecs.get::<Position>(b)) {
                    (Some(x), Some(y)) => ((x.x, x.y), (y.x, y.y)),
                    _ => continue,
                };
                let dx = pa.0 - pb.0;
                if dx.abs() < 30.0 && (pa.1 - pb.1).abs() < 30.0 {
                    let push = if dx >= 0.0 { 26.0 * dt } else { -26.0 * dt };
                    if let Some(p) = self.ecs.get_mut::<Position>(a) {
                        p.x += push;
                    }
                    if let Some(p) = self.ecs.get_mut::<Position>(b) {
                        p.x -= push;
                    }
                }
            }
        }
        // 接触伤害
        if self.player.hurt_cd <= 0.0 {
            for i in 0..self.foes.len() {
                let id = self.foes[i];
                let hurt = match (self.ecs.get::<Position>(id), self.ecs.get::<Sprite>(id)) {
                    (Some(p), Some(s)) => overlap(px, py, pw, ph, p.x, p.y, s.w, s.h),
                    _ => false,
                };
                if hurt {
                    let dmg = self.meta.get(&id).map_or(8.0, |m| m.dmg);
                    if self.meta.get(&id).map_or(false, |m| m.touch_cd <= 0.0) {
                        self.player.hp = (self.player.hp - dmg).max(0.0);
                        self.player.hurt_cd = 0.6;
                        // 体魄修炼：挨打记（用进废退）
                        let ups = self.grit.add(dmg.max(0.0) as u32);
                        if ups > 0 {
                            self.player.max_hp += 6.0 * ups as f32;
                            self.player.hp =
                                (self.player.hp + 6.0 * ups as f32).min(self.player.max_hp);
                            self.special_flash = ("體魄提升".into(), 1.2);
                        }
                        if let Some(m) = self.meta.get_mut(&id) {
                            m.touch_cd = 0.8;
                        }
                        let (_, params) = self.tiers.tier_for(dmg, false, false);
                        self.juice.trigger(&params);
                        self.audio.play(SoundEffect::Hurt);
                        self.combo.reset();
                        break;
                    }
                }
            }
        }
    }

    fn update_hats(&mut self, dt: f32) {
        // 两阶段：先收集命中（不可变借用），再结算（可变借用）——借用分离
        let mut impacts: Vec<(u64, f32, f32)> = Vec::new();
        let mut dead_hats: Vec<usize> = Vec::new();
        for (hi, hat) in self.hats.iter_mut().enumerate() {
            hat.life -= dt;
            hat.x += hat.vx * dt;
            if hat.life <= 0.0 || hat.x < 0.0 || hat.x > 1280.0 {
                dead_hats.push(hi);
                continue;
            }
            for i in 0..self.foes.len() {
                let id = self.foes[i];
                if hat.hit.contains(&id) {
                    continue;
                }
                let hit = match (self.ecs.get::<Position>(id), self.ecs.get::<Sprite>(id)) {
                    (Some(p), Some(s)) => overlap(hat.x, hat.y, 26.0, 14.0, p.x, p.y, s.w, s.h),
                    _ => false,
                };
                if hit {
                    hat.hit.push(id);
                    if let Some(p) = self.ecs.get::<Position>(id) {
                        impacts.push((id, p.x, p.y));
                    }
                }
            }
        }
        for hi in dead_hats.into_iter().rev() {
            self.hats.remove(hi);
        }
        let mut dead_foes = Vec::new();
        for (id, ex, ey) in impacts {
            if self.damage_foe(id, 30.0, ex, ey, false) {
                dead_foes.push(id);
            }
            self.combo.hit(self.time);
            self.qi.gain(QI_PER_HIT);
        }
        for id in dead_foes {
            self.kill_foe(id);
        }
    }

    fn update_waves(&mut self, dt: f32) {
        // 补怪（上限内滴灌）
        if !self.spawn_queue.is_empty() && self.foes.len() < MAX_FOES {
            self.spawn_t -= dt;
            if self.spawn_t <= 0.0 {
                self.spawn_t = 0.5;
                let spec = self.spawn_queue.remove(0);
                let side = if self.kills % 2 == 0 { 1.0 } else { -1.0 };
                self.spawn_foe(spec, side);
            }
        }
        // 清波 → 下一波
        if self.spawn_queue.is_empty() && self.foes.is_empty() && self.state == RunState::Play {
            self.next_wave();
        }
    }

    fn cx_text(&self, s: &str, y: f32, size: f32, color: Color) {
        let w = match &self.atlas {
            Some(a) => a.measure(s, size),
            None => measure_text(s, None, size as u16, 1.0).width,
        };
        self.gtext(s, 640.0 - w / 2.0, y, size, color);
    }

    /// 文本统一出口：烘焙图集优先（零光栅化），缺失回 macroquad。
    fn gtext(&self, s: &str, x: f32, y: f32, size: f32, color: Color) {
        if let Some(a) = &self.atlas {
            a.draw(s, x, y, size, color);
        } else {
            draw_text(s, x, y, size, color);
        }
    }

    fn bar(&self, x: f32, y: f32, w: f32, h: f32, pct: f32, fg: Color, label: &str) {
        draw_rectangle(x, y, w, h, Color::new(0.1, 0.1, 0.12, 0.9));
        draw_rectangle(x, y, w * pct.clamp(0.0, 1.0), h, fg);
        draw_rectangle_lines(x, y, w, h, 1.0, GRAY);
        self.gtext(label, x + 4.0, y + h - 5.0, 14.0, WHITE);
    }

    fn render(&self, plats: &[(f32, f32, f32, f32)]) {
        if self.state == RunState::Title {
            clear_background(Color::new(0.02, 0.02, 0.04, 1.0));
            draw_circle(640.0, 300.0, 150.0, Color::new(0.85, 0.82, 0.7, 0.10));
            self.cx_text("歸墟", 220.0, 110.0, YELLOW);
            self.cx_text("G U I X U", 270.0, 26.0, GRAY);
            self.cx_text("南宋 · 紹興年間", 310.0, 22.0, Color::new(0.8, 0.7, 0.5, 1.0));
            self.cx_text("WASD 走位 · 自动战斗 · K 跳/二段跳 · L 輕功 · Q 飛笠", 400.0, 20.0, WHITE);
            self.cx_text("Shift 疾跑 · Esc 暫停 · 方向鍵亦可", 430.0, 20.0, WHITE);
            self.cx_text("按任意鍵開始", 520.0, 26.0, YELLOW);
            return;
        }
        // ── 世界（相机居中） ──
        let (ox, oy) = (self.cam.0 - 640.0, self.cam.1 - 360.0);
        clear_background(Color::new(0.03, 0.04, 0.08, 1.0));
        draw_circle(900.0 - ox * 0.3, 140.0, 90.0, Color::new(0.85, 0.82, 0.7, 0.25));
        draw_triangle(
            Vec2::new(100.0 - ox * 0.5, 640.0),
            Vec2::new(400.0 - ox * 0.5, 380.0),
            Vec2::new(700.0 - ox * 0.5, 640.0),
            Color::new(0.08, 0.10, 0.16, 1.0),
        );
        draw_triangle(
            Vec2::new(700.0 - ox * 0.5, 640.0),
            Vec2::new(1000.0 - ox * 0.5, 420.0),
            Vec2::new(1300.0 - ox * 0.5, 640.0),
            Color::new(0.06, 0.08, 0.13, 1.0),
        );
        for &(px, py, pw, ph) in plats {
            draw_rectangle(px - ox, py - oy, pw, ph, Color::new(0.16, 0.18, 0.14, 1.0));
            draw_rectangle(px - ox, py - oy, pw, 3.0, Color::new(0.35, 0.38, 0.3, 1.0));
        }
        // 怪
        for id in &self.foes {
            if let (Some(p), Some(s)) = (self.ecs.get::<Position>(*id), self.ecs.get::<Sprite>(*id)) {
                let x = p.x - ox;
                let y = p.y - oy;
                draw_rectangle(x - s.w / 2.0, y - s.h / 2.0, s.w, s.h, s.color);
                if self.meta.get(id).map_or(false, |m| m.elite) {
                    draw_rectangle_lines(x - s.w / 2.0, y - s.h / 2.0, s.w, s.h, 2.0, YELLOW);
                }
                if let Some(h) = self.ecs.get::<Health>(*id) {
                    let pct = (h.hp / h.max_hp).clamp(0.0, 1.0);
                    draw_rectangle(x - 15.0, y - s.h / 2.0 - 8.0, 30.0, 4.0, DARKGRAY);
                    draw_rectangle(x - 15.0, y - s.h / 2.0 - 8.0, 30.0 * pct, 4.0, RED);
                }
            }
        }
        // 飛笠
        for hat in &self.hats {
            draw_rectangle(hat.x - ox - 13.0, hat.y - oy - 7.0, 26.0, 14.0, YELLOW);
        }
        // 玩家（白衣 + 剑线）
        {
            let p = &self.player;
            let x = p.x - ox;
            let y = p.y - oy;
            draw_rectangle(x - p.w / 2.0, y - p.h / 2.0, p.w, p.h, WHITE);
            draw_line(
                x + p.facing * 10.0,
                y - 6.0,
                x + p.facing * 44.0,
                y - 6.0,
                3.0,
                YELLOW,
            );
        }
        self.fx.render();
        // ── HUD ──
        self.bar(12.0, 12.0, 220.0, 20.0, self.player.hp / self.player.max_hp, RED, "命");
        self.bar(12.0, 38.0, 220.0, 16.0, self.qi.pct(), SKYBLUE,
            &format!("氣 {:.0}", self.qi.cur()));
        self.gtext(&format!("分 {}", self.score), 1150.0, 30.0, 22.0, YELLOW);
        self.gtext(&format!("第 {} 波", self.wave), 1150.0, 56.0, 18.0, WHITE);
        if self.combo.count() >= 2 {
            self.cx_text(&format!("{}連擊", self.combo.count()), 120.0, 34.0, ORANGE);
        }
        if self.special_flash.1 > 0.0 {
            self.cx_text(&self.special_flash.0, 170.0, 30.0, YELLOW);
        }
        if self.banner.1 > 0.0 {
            self.cx_text(&self.banner.0, 300.0, 44.0, YELLOW);
        }
        self.gtext(
            "自动战斗 K跳 L輕功 Q飛笠 Shift疾跑 Esc暫停 1劍2刀",
            12.0,
            706.0,
            16.0,
            GRAY,
        );
        self.gtext(
            &format!(
                "{}Lv{} 體魄Lv{} 臂力Lv{}",
                WEAPONS[self.weapon.min(WEAPONS.len() - 1)].name,
                self.wmas[self.weapon.min(1)].level,
                self.grit.level,
                self.arm.level
            ),
            12.0,
            70.0,
            18.0,
            YELLOW,
        );
        if self.state == RunState::Pause {
            draw_rectangle(0.0, 0.0, 1280.0, 720.0, Color::new(0.0, 0.0, 0.0, 0.6));
            self.cx_text("暫停", 330.0, 60.0, WHITE);
            self.cx_text("按 Esc 繼續", 380.0, 22.0, GRAY);
            // 兵器卡：当前兵器实测＋纲领（lore 数据消费位）
            let wname = WEAPONS[self.weapon.min(WEAPONS.len() - 1)].name;
            if let Some(wl) = lore::weapon_lore(wname) {
                self.cx_text(&format!("佩{} · {}", wl.name, wl.spec), 430.0, 18.0, WHITE);
                self.cx_text(wl.doctrine, 460.0, 18.0, YELLOW);
            }
        }
        if self.state == RunState::Over {
            draw_rectangle(0.0, 0.0, 1280.0, 720.0, Color::new(0.0, 0.0, 0.0, 0.7));
            self.cx_text("劍折", 330.0, 80.0, RED);
            self.cx_text(
                &format!("第 {} 波 · 分 {} · 斬 {} · 連擊 {} · 存活 {:.0}s",
                    self.wave, self.score, self.kills, self.combo.max(), self.time),
                390.0,
                22.0,
                WHITE,
            );
            self.cx_text("按 R 或空格鍵 再戰一次", 440.0, 24.0, YELLOW);
        }
        if self.state == RunState::Story {
            if let Some(st) = &self.story {
                draw_rectangle(0.0, 0.0, 1280.0, 720.0, Color::new(0.0, 0.0, 0.0, 0.55));
                draw_rectangle(340.0, 200.0, 600.0, 300.0, Color::new(0.05, 0.05, 0.08, 0.95));
                draw_rectangle_lines(340.0, 200.0, 600.0, 300.0, 1.5, YELLOW);
                if let Some((who, text)) = &st.line {
                    self.gtext(who, 370.0, 250.0, 22.0, YELLOW);
                    self.gtext(text, 370.0, 290.0, 20.0, WHITE);
                    self.gtext("J / 空格 继续", 370.0, 470.0, 16.0, GRAY);
                }
                if let Some((prompt, opts)) = &st.choices {
                    self.gtext(prompt, 370.0, 250.0, 22.0, YELLOW);
                    for (i, o) in opts.iter().enumerate() {
                        self.gtext(&format!("{}. {}", i + 1, o), 370.0, 290.0 + i as f32 * 34.0, 20.0, WHITE);
                    }
                }
            }
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "歸墟".into(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    println!("歸墟 · 南宋绍兴年间");
    let mut g = Game::new();
    g.atlas = AtlasFont::load();
    g.input = InputState::new();
    let plats = platforms();

    // 像素字体（cards 侧合并版，2529 字）
    match load_ttf_font_from_bytes(include_bytes!("../assets/fonts/pixel-game.ttf")) {
        Ok(font) => {
            let chars: Vec<char> = include_str!("../assets/fonts/charset.txt").chars().collect();
            font.populate_font_cache(&chars, 14);
            font.populate_font_cache(&chars, 20);
            font.populate_font_cache(&chars, 26);
            font.populate_font_cache(&chars, 40);
            set_default_font(font);
        }
        Err(e) => println!("字体加载失败: {}", e),
    }

    // 音频：合成 SFX 加载（失败回退静默，不断链）
    g.audio.init().await;

    // SHOT 审计钩子（title/play）
    if let Ok(scene) = std::env::var("NEOTRIX_GUIXU_SHOT") {
        let out = std::env::var("NEOTRIX_GUIXU_SHOT_OUT")
            .unwrap_or_else(|_| format!("/tmp/swords_{}.png", scene));
        g.new_run();
        if scene == "title" {
            g.state = RunState::Title;
        }
        for _ in 0..40 {
            let dt = 1.0 / 60.0;
            if g.state == RunState::Play {
                g.time += dt;
                g.update_play(dt, &plats);
            }
            g.render(&plats);
            next_frame().await;
        }
        use macroquad::texture::render_target;
        let (sw, sh) = (screen_width(), screen_height());
        let rt = render_target(sw as u32, sh as u32);
        let fbo = Camera2D { render_target: Some(rt.clone()), ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, sw, sh)) };
        set_camera(&fbo);
        g.render(&plats);
        set_default_camera();
        rt.texture.get_texture_data().export_png(&out);
        println!("SHOT saved -> {}", out);
        return;
    }

    loop {
        let raw = get_frame_time().clamp(0.0, 0.05);
        g.input.update();
        // juice 顿帧阀（输入先采样）
        let dt = g.juice.tick(raw);

        // Esc 暂停（边沿）
        let esc = is_key_down(KeyCode::Escape);
        if esc && !g.prev_esc {
            if g.state == RunState::Play {
                g.state = RunState::Pause;
            } else if g.state == RunState::Pause {
                g.state = RunState::Play;
            }
        }
        g.prev_esc = esc;

        match g.state {
            RunState::Title => {
                // 任意键/点击开始
                let any = is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::Enter)
                    || g.input.clicked.is_some()
                    || is_mouse_button_pressed(MouseButton::Left);
                if any && !g.prev_any {
                    g.new_run();
                }
                g.prev_any = any;
            }
            RunState::Play => {
                g.time += dt;
                // 兵器切换（1 劍 2 刀；剧情态独占数字键，无冲突）
                if is_key_pressed(KeyCode::Key1) && g.weapon != 0 {
                    g.weapon = 0;
                    g.special_flash = ("劍·青萍".into(), 1.2);
                    g.audio.play(SoundEffect::Select);
                }
                if is_key_pressed(KeyCode::Key2) && g.weapon != 1 {
                    g.weapon = 1;
                    g.special_flash = ("刀·胡家".into(), 1.2);
                    g.audio.play(SoundEffect::Select);
                }
                g.update_play(dt, &plats);
            }
            RunState::Pause => {}
            RunState::Story => {
                // J/空格/点击确认；1/2/3 选择
                let confirm = is_key_pressed(KeyCode::J)
                    || is_key_pressed(KeyCode::Space)
                    || g.input.clicked.is_some()
                    || is_mouse_button_pressed(MouseButton::Left);
                if confirm {
                    if let Some(st) = &mut g.story {
                        st.advance();
                    }
                    g.drain_story();
                }
                for (key, idx) in [
                    (KeyCode::Key1, 0),
                    (KeyCode::Key2, 1),
                    (KeyCode::Key3, 2),
                ] {
                    if is_key_pressed(key) {
                        if let Some(st) = &mut g.story {
                            st.choose(idx);
                        }
                        g.drain_story();
                    }
                }
            }
            RunState::Over => {
                if is_key_pressed(KeyCode::R) || is_key_pressed(KeyCode::Space) {
                    g.new_run();
                }
            }
        }

        // 相机死区前视跟随
        let (px, py, vx) = (g.player.x, g.player.y, g.player.vx);
        g.cam = follow_lookahead(g.cam, (px, py), (vx, 0.0), raw, &g.dz, 120.0);
        g.render(&plats);
        next_frame().await;
    }
}

#[cfg(test)]
mod atlas_guard {
    /// 回归哨兵：旧路径诊断（macroquad atlas 在 Retina 2× 下会到 32768）+
    /// 新不变量（baked atlas 覆盖全部游戏字符）。背景见 RENDER-MIN §1。
    #[test]
    fn probe_atlas_pressure() {
        let data: &[u8] = include_bytes!("../assets/fonts/pixel-game.ttf");
        let font = fontdue::Font::from_bytes(data, fontdue::FontSettings::default()).unwrap();
        // 先验尺
        let (m, _) = font.rasterize('A', 40.0);
        assert!((5..=80).contains(&m.width) && (5..=80).contains(&m.height), "ruler broken: {m:?}");
        let chars: Vec<char> = include_str!("../assets/fonts/charset.txt")
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        assert_eq!(chars.len(), 2593);
        let dims = |size: f32| -> Vec<(usize, usize)> {
            chars
                .iter()
                .map(|&c| {
                    let (mm, _) = font.rasterize(c, size);
                    (mm.width.max(1), mm.height.max(1))
                })
                .collect()
        };
        // 打包机复刻（atlas.rs cache_sprite 行打包 + 溢出重放）
        fn simulate(groups: &[Vec<(usize, usize)>]) -> (u32, u32) {
            const GAP: u32 = 2;
            let (mut w, mut h) = (512u32, 512u32);
            let (mut cx, mut cy, mut mlh) = (0u32, 0u32, 0u32);
            let mut placed: Vec<(u32, u32)> = Vec::new();
            let mut doublings = 0u32;
            for g in groups {
                for &(sw, sh) in g {
                    let (sw, sh) = (sw as u32, sh as u32);
                    let (x, y);
                    if cx + sw < w {
                        if sh > mlh {
                            mlh = sh;
                        }
                        x = cx + GAP;
                        cx += sw + GAP * 2;
                    } else {
                        cy += mlh + GAP * 2;
                        cx = sw + GAP * 2;
                        mlh = sh;
                        x = GAP;
                    }
                    y = cy;
                    if y + sh > h || x + sw > w {
                        w *= 2;
                        h *= 2;
                        doublings += 1;
                        if w > 65536 {
                            return (w, doublings);
                        }
                        cx = 0;
                        cy = 0;
                        mlh = 0;
                        // 重放：已放 + 当前
                        let mut replay = std::mem::take(&mut placed);
                        replay.push((sw, sh));
                        for &(rw, rh) in &replay {
                            let (rx, ry);
                            if cx + rw < w {
                                if rh > mlh {
                                    mlh = rh;
                                }
                                rx = cx + GAP;
                                cx += rw + GAP * 2;
                            } else {
                                cy += mlh + GAP * 2;
                                cx = rw + GAP * 2;
                                mlh = rh;
                                rx = GAP;
                            }
                            ry = cy;
                            placed.push((rw, rh));
                            let _ = (rx, ry);
                        }
                    } else {
                        placed.push((sw, sh));
                    }
                }
            }
            (w, doublings)
        }
        let startup = [dims(14.0), dims(20.0), dims(26.0), dims(40.0)];
        let (w0, d0) = simulate(&startup);
        println!("startup atlas={w0} doublings={d0}");
        // 真实缓存集：populate{14,20,26,40} + draw×dpi2{28,32,36,40,44}
        let mut with_lazy = startup.to_vec();
        with_lazy.push(dims(28.0));
        with_lazy.push(dims(32.0));
        with_lazy.push(dims(36.0));
        with_lazy.push(dims(44.0));
        let (w1, d1) = simulate(&with_lazy);
        // 旧路径诊断数（实锤 32768 → graphics.rs:335 溢出；游戏正文已走 baked atlas）。
        println!("legacy macroquad path would reach atlas={w1} doublings={d1}");
        // 新不变量：烘焙图集覆盖全部游戏字符 → macroquad 兜底路恒空。
        let map = super::atlas::AtlasFont::parse_metrics(include_str!(
            "../assets/fonts/glyph_metrics.ron"
        ));
        let missing: Vec<char> =
            chars.iter().copied().filter(|c| !map.contains_key(c)).collect();
        assert!(
            missing.is_empty(),
            "chars outside baked atlas: {:?}",
            &missing[..missing.len().min(20)]
        );
    }
}

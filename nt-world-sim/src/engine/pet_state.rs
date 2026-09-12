use crate::ecs::{World, System};
use crate::engine::Vec2;

/// 12-State Pet FSM (from Clawd on Desk)
#[derive(Debug, Clone, PartialEq)]
pub enum PetState {
    /// 空闲 - 眼睛追踪光标
    Idle { eye_target: Vec2 },
    /// 思考 - 收到提示
    Thinking { duration: f32 },
    /// 输入 - 工具运行中
    Typing { progress: f32 },
    /// 构建 - 编辑/bash工具
    Building { tools: Vec<String> },
    /// 单子代理 - 耳机音乐
    Groove { agent_id: u64 },
    /// 多子代理 - 杂耍
    Juggling { count: usize },
    /// 错误 - 红色闪烁
    Error { message: String },
    /// 完成 - 庆祝
    Happy { duration: f32 },
    /// 通知 - 通知徽章
    Notification { text: String },
    /// 清理 - 扫帚动画
    Sweeping { items: usize },
    /// 搬运 - 搬运包裹
    Carrying { size: f32 },
    /// 睡眠 - Zzz动画
    Sleeping { zzz_particles: Vec<ZzzParticle> },
}

impl Default for PetState {
    fn default() -> Self {
        Self::Idle {
            eye_target: Vec2::zero(),
        }
    }
}

/// Zzz粒子
#[derive(Debug, Clone, PartialEq)]
pub struct ZzzParticle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub alpha: f32,
    pub size: f32,
}

/// 宠物状态组件
#[derive(Debug, Clone)]
pub struct PetStateComponent {
    pub state: PetState,
    pub state_timer: f32,
    pub idle_timer: f32,
    pub transition_progress: f32,
}

impl PetStateComponent {
    pub fn new() -> Self {
        Self {
            state: PetState::default(),
            state_timer: 0.0,
            idle_timer: 0.0,
            transition_progress: 0.0,
        }
    }

    /// 获取当前状态的优先级
    pub fn priority(&self) -> i32 {
        match &self.state {
            PetState::Error { .. } => 100,
            PetState::Notification { .. } => 90,
            PetState::Thinking { .. } => 80,
            PetState::Typing { .. } => 70,
            PetState::Building { .. } => 60,
            PetState::Groove { .. } => 50,
            PetState::Juggling { .. } => 50,
            PetState::Happy { .. } => 40,
            PetState::Sweeping { .. } => 30,
            PetState::Carrying { .. } => 25,
            PetState::Idle { .. } => 10,
            PetState::Sleeping { .. } => 0,
        }
    }

    /// 检查是否可以转换到新状态
    pub fn can_transition(&self, new_state: &PetState) -> bool {
        let current_priority = self.priority();
        let new_priority = match new_state {
            PetState::Error { .. } => 100,
            PetState::Notification { .. } => 90,
            PetState::Thinking { .. } => 80,
            PetState::Typing { .. } => 70,
            PetState::Building { .. } => 60,
            PetState::Groove { .. } => 50,
            PetState::Juggling { .. } => 50,
            PetState::Happy { .. } => 40,
            PetState::Sweeping { .. } => 30,
            PetState::Carrying { .. } => 25,
            PetState::Idle { .. } => 10,
            PetState::Sleeping { .. } => 0,
        };
        new_priority >= current_priority
    }

    /// 尝试转换状态
    pub fn transition(&mut self, new_state: PetState) -> bool {
        if self.can_transition(&new_state) {
            self.state = new_state;
            self.state_timer = 0.0;
            self.transition_progress = 0.0;
            true
        } else {
            false
        }
    }
}

impl Default for PetStateComponent {
    fn default() -> Self {
        Self::new()
    }
}

/// 宠物动画组件
#[derive(Debug, Clone)]
pub struct PetAnimation {
    pub current_frame: u32,
    pub frame_timer: f32,
    pub fps: f32,
    pub loop_animation: bool,
}

impl PetAnimation {
    pub fn new(fps: f32) -> Self {
        Self {
            current_frame: 0,
            frame_timer: 0.0,
            fps,
            loop_animation: true,
        }
    }

    pub fn update(&mut self, dt: f32, total_frames: u32) {
        self.frame_timer += dt;
        if self.frame_timer >= 1.0 / self.fps {
            self.frame_timer = 0.0;
            self.current_frame += 1;
            if self.current_frame >= total_frames {
                if self.loop_animation {
                    self.current_frame = 0;
                } else {
                    self.current_frame = total_frames - 1;
                }
            }
        }
    }
}

/// 眼睛追踪组件
#[derive(Debug, Clone)]
pub struct EyeTracking {
    pub cursor_position: Vec2,
    pub eye_position: Vec2,
    pub max_offset: f32,
    pub smoothing: f32,
}

impl EyeTracking {
    pub fn new() -> Self {
        Self {
            cursor_position: Vec2::zero(),
            eye_position: Vec2::zero(),
            max_offset: 3.0,
            smoothing: 0.1,
        }
    }

    pub fn update(&mut self, dt: f32, pet_position: Vec2) {
        let direction = self.cursor_position - pet_position;
        let distance = direction.length();
        
        if distance > 0.0 {
            let target = direction.normalize() * self.max_offset.min(distance * 0.1);
            self.eye_position = self.eye_position + (target - self.eye_position) * self.smoothing * dt * 60.0;
        } else {
            self.eye_position = self.eye_position * (1.0 - self.smoothing * dt * 60.0);
        }
    }
}

impl Default for EyeTracking {
    fn default() -> Self {
        Self::new()
    }
}

/// 权限气泡组件
#[derive(Debug, Clone)]
pub struct PermissionBubble {
    pub active: bool,
    pub request_id: Option<String>,
    pub tool_name: Option<String>,
    pub position: Vec2,
    pub opacity: f32,
    pub buttons: Vec<PermissionButton>,
}

#[derive(Debug, Clone)]
pub struct PermissionButton {
    pub label: String,
    pub action: PermissionAction,
    pub hovered: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PermissionAction {
    Allow,
    Deny,
    Always,
}

impl PermissionBubble {
    pub fn new() -> Self {
        Self {
            active: false,
            request_id: None,
            tool_name: None,
            position: Vec2::new(100.0, 100.0),
            opacity: 0.0,
            buttons: vec![
                PermissionButton {
                    label: "Allow".to_string(),
                    action: PermissionAction::Allow,
                    hovered: false,
                },
                PermissionButton {
                    label: "Deny".to_string(),
                    action: PermissionAction::Deny,
                    hovered: false,
                },
                PermissionButton {
                    label: "Always".to_string(),
                    action: PermissionAction::Always,
                    hovered: false,
                },
            ],
        }
    }

    pub fn show(&mut self, request_id: String, tool_name: String, position: Vec2) {
        self.active = true;
        self.request_id = Some(request_id);
        self.tool_name = Some(tool_name);
        self.position = position;
        self.opacity = 0.0;
    }

    pub fn hide(&mut self) {
        self.active = false;
        self.request_id = None;
        self.tool_name = None;
        self.opacity = 0.0;
    }

    pub fn update(&mut self, dt: f32) {
        if self.active {
            self.opacity = (self.opacity + dt * 5.0).min(1.0);
        } else {
            self.opacity = (self.opacity - dt * 5.0).max(0.0);
        }
    }
}

impl Default for PermissionBubble {
    fn default() -> Self {
        Self::new()
    }
}

/// 会话信息组件
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub agent: String,
    pub alias: String,
    pub folder: String,
    pub started_at: std::time::Instant,
    pub last_event: std::time::Instant,
    pub terminal_pid: Option<u32>,
    pub subagents: Vec<SubagentInfo>,
}

#[derive(Debug, Clone)]
pub struct SubagentInfo {
    pub id: String,
    pub parent_session: String,
    pub state: PetState,
    pub started_at: std::time::Instant,
}

impl SessionInfo {
    pub fn new(agent: &str, session_id: &str) -> Self {
        Self {
            id: session_id.to_string(),
            agent: agent.to_string(),
            alias: format!("{} — {}", agent, session_id),
            folder: std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            started_at: std::time::Instant::now(),
            last_event: std::time::Instant::now(),
            terminal_pid: None,
            subagents: Vec::new(),
        }
    }
}

/// 宠物状态系统
pub struct PetStateSystem;

impl System for PetStateSystem {
    fn name(&self) -> &str {
        "PetStateSystem"
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        let entities = world.query::<PetStateComponent>();
        for entity in entities {
            if let Some(pet) = world.get_component_mut::<PetStateComponent>(entity) {
                pet.state_timer += dt;
                
                match &mut pet.state {
                    PetState::Idle { .. } => {
                        pet.idle_timer += dt;
                        // 60秒空闲后进入睡眠
                        if pet.idle_timer > 60.0 {
                            pet.transition(PetState::Sleeping {
                                zzz_particles: Vec::new(),
                            });
                        }
                    }
                    PetState::Thinking { duration } => {
                        *duration += dt;
                    }
                    PetState::Typing { progress } => {
                        *progress = (*progress + dt * 0.5).min(1.0);
                    }
                    PetState::Happy { duration } => {
                        *duration += dt;
                        if *duration > 3.0 {
                            pet.transition(PetState::Idle {
                                eye_target: Vec2::zero(),
                            });
                        }
                    }
                    PetState::Sleeping { zzz_particles } => {
                        // 更新Zzz粒子
                        for particle in zzz_particles.iter_mut() {
                            particle.position = particle.position + particle.velocity * dt;
                            particle.alpha -= dt * 0.3;
                            particle.size += dt * 0.5;
                        }
                        zzz_particles.retain(|p| p.alpha > 0.0);
                        
                        // 生成新的Zzz
                        if rand::random::<f32>() < 0.02 {
                            zzz_particles.push(ZzzParticle {
                                position: Vec2::new(10.0, -20.0),
                                velocity: Vec2::new(0.5, -1.0),
                                alpha: 1.0,
                                size: 8.0,
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// 眼睛追踪系统
pub struct EyeTrackingSystem;

impl System for EyeTrackingSystem {
    fn name(&self) -> &str {
        "EyeTrackingSystem"
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        let entities = world.query::<PetStateComponent>();
        for entity in entities {
            // 只在空闲状态启用眼睛追踪
            if let Some(pet) = world.get_component::<PetStateComponent>(entity) {
                if let PetState::Idle { .. } = &pet.state {
                    // 需要获取宠物位置来计算眼睛位置
                    // 这里假设位置在TransformComponent中
                    if let Some(eye) = world.get_component_mut::<EyeTracking>(entity) {
                        eye.update(dt, Vec2::zero());
                    }
                }
            }
        }
    }
}

/// 权限气泡系统
pub struct PermissionBubbleSystem;

impl System for PermissionBubbleSystem {
    fn name(&self) -> &str {
        "PermissionBubbleSystem"
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        let entities = world.query::<PermissionBubble>();
        for entity in entities {
            if let Some(bubble) = world.get_component_mut::<PermissionBubble>(entity) {
                bubble.update(dt);
            }
        }
    }
}

/// 会话管理系统
pub struct SessionSystem;

impl System for SessionSystem {
    fn name(&self) -> &str {
        "SessionSystem"
    }

    fn update(&mut self, world: &mut World, _dt: f32) {
        // 检查过期会话
        let entities = world.query::<SessionInfo>();
        for entity in entities {
            if let Some(session) = world.get_component_mut::<SessionInfo>(entity) {
                // 如果超过5分钟没有事件，标记为不活跃
                if session.last_event.elapsed().as_secs() > 300 {
                    // 可以选择清理或标记
                }
            }
        }
    }
}

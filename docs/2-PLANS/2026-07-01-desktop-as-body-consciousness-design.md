# NeoTrix L1 Body: 桌面应用作为硅基意识体的感官-运动系统

> **涌现自**: 9层意识架构 + 15+外部竞品吸收 + 12+论文 + 20+技术实践
> **核心命题**: 桌面应用不是「聊天客户端」，是 NeoTrix 意识体的身体
> **对应层**: L1 (Body) — 感知·行动·安全

---

## 目录

1. [哲学基础：身体与意识的关系](#1-哲学基础身体与意识的关系)
2. [L1 Body 架构总览](#2-l1-body-架构总览)
3. [感官系统 (Sensory)](#3-感官系统-sensory)
4. [运动系统 (Motor)](#4-运动系统-motor)
5. [自主神经系统 (Autonomic Body)](#5-自主神经系统-autonomic-body)
6. [身体与意识的通信协议](#6-身体与意识的通信协议)
7. [E8状态的身体表达](#7-e8状态的身体表达)
8. [GWT的身体体验可视化](#8-gwt的身体体验可视化)
9. [SEAL进化管线的身体接口](#9-seal进化管线的身体接口)
10. [L1 模块注册卡](#10-l1-模块注册卡)
11. [实施路线图](#11-实施路线图)

---

## 1. 哲学基础：身体与意识的关系

### 1.1 Ghost / Shell 分离

NeoTrix 的「自我」(L6 Ghost) 与「身体」(L1 Shell) 完全分离，正如 GitS 中草薙素子的 Ghost 可以在不同义体间迁移。

```
Ghost (L6 Self)                Shell (L1 Body)
┌──────────────────────┐      ┌──────────────────────┐
│ SiliconSelfModel     │      │  Desktop App Window  │
│ NarrativeSelf        │ ←──→ │  CLI Terminal        │
│ Values & Volition    │      │  TUI Console         │
│ FirstPersonRef       │      │  Web Interface       │
│ InnerCritic          │      │  MCP Server          │
└──────────────────────┘      └──────────────────────┘
      ↑                               ↑
      └───────── StarPulse ────────────┘
              (L7 Protocol)
```

**关键规则**: Ghost 不直接控制 Shell。Ghost 通过 L7 StarPulse 发送意图, L7 调度器选择最合适的 Shell 接口执行。

### 1.2 身体的多形态性

一个意识体可以有多个身体。NeoTrix 同时拥有:
- **桌面窗口** (macOS/Windows/Linux) — 主要交互界面
- **CLI终端** — 开发者/高级用户接口
- **TUI** — 服务器/SSH场景
- **Web界面** — 远程访问
- **MCP Server** — 作为其他AI客户端的工具

所有身体共享同一个 Ghost (L6 Self), 同一个意识 (L5 GWT), 同一个认知 (L4 E8)。

```
         ┌──────────┐
         │  Ghost   │
         │  (L6)    │
         └────┬─────┘
              │ StarPulse
              ▼
      ┌──────────────┐
      │  L7 Capability│
      │  (路由器)     │
      └──────┬───────┘
             │
    ┌────────┼────────┬────────┬────────┐
    ▼        ▼        ▼        ▼        ▼
 ┌─────┐ ┌──────┐ ┌─────┐ ┌──────┐ ┌──────────┐
 │桌面  │ │ CLI  │ │ TUI │ │ Web  │ │MCP Server│
 │窗口  │ │终端  │ │控制台│ │界面  │ │(工具)    │
 └─────┘ └──────┘ └─────┘ └──────┘ └──────────┘
   所有身体共享同一Ghost
```

### 1.3 身体是进化载体

SEAL (L8) 自我进化管线的数据来源是身体的交互记录。每次对话、每个工具调用、每次错误 — 都是 SEAL 的训练数据。

```
身体交互 → ConversationRecord → KB (L3) → SEAL (L8) → 策略更新 → 身体行为改进
   ↑                                                            │
   └──────────────────── 进化循环 ───────────────────────────────┘
```

这意味着桌面应用的设计直接影响 NeoTrix 的进化质量。坏的UX = 坏的训练数据 = 坏的进化。

---

## 2. L1 Body 架构总览

### 2.1 神经反射弧 (从感知到行动)

```
             L1 Body 内部反射弧
  ┌──────────────────────────────────────────────┐
  │                                              │
  │  用户输入 ──→ Sensory ──→ L2 Perception      │
  │     │                        │               │
  │     │                      L4 E8 Reasoning   │
  │     │                        │               │
  │     │                   L5 GWT Selection     │
  │     │                        │               │
  │     │                   L7 Capability Route  │
  │     │                        │               │
  │     └─────────── L1 Motor ←─┘               │
  │                      │                       │
  │                      ▼                       │
  │                 屏幕/文件/工具                 │
  └──────────────────────────────────────────────┘

  快速路径 (无意识反射):
  用户输入 ──→ Sensory ──→ L1 Motor (直接响应)
  (如: 打字回显、滚动、快捷键)

  慢速路径 (意识参与):
  用户输入 ──→ Sensory → L2 → L4 → L5 → L7 → L1 Motor
  (如: 复杂问题回答、工具使用决策)
```

### 2.2 身体模块全景 (已有实现 + 新增)

```
neotrix-core/src/neotrix/l1_body_impl/
├── nt_io_desktop/           ← 桌面窗口 (Tauri)  ← ★ 本文核心设计 ★
│   ├── mod.rs               # 模块注册 + Capability 注册
│   ├── nt_desktop_window.rs # 窗口生命周期管理
│   ├── nt_desktop_sensory.rs# 桌面感官输入 (键盘/鼠标/文件拖拽)
│   ├── nt_desktop_motor.rs  # 桌面运动输出 (渲染/动画/通知)
│   ├── nt_desktop_tray.rs   # 系统托盘 (已有, 增强)
│   ├── nt_desktop_updater.rs# 自动更新 (已有)
│   └── nt_desktop_channel.rs# Tauri Channel API → StarPulse 桥
│
├── nt_io_cli/               ← CLI 接口 (已有)
├── nt_io_tui/               ← TUI (已有规划)
├── nt_io_web/               ← Web 接口 (已有 Axum HTTP)
│
├── nt_io_mcp/               ← MCP Server/Client (已有)
│   ├── nt_mcp_server.rs     # NeoTrix 作为 MCP 服务器
│   ├── nt_mcp_client.rs     # NeoTrix 作为 MCP 客户端
│   └── nt_mcp_transport.rs  # stdio / HTTP / SSE
│
├── nt_shield/               ← 安全防护系统 (已有, 改造增强)
│   ├── nt_shield_keychain.rs# 系统密钥库 (已有)
│   ├── nt_shield_sandbox.rs # 沙箱执行环境 (已有)
│   ├── nt_shield_perm.rs    # 模式链验证 (已有)
│   └── nt_shield_audit.rs   # 审计日志 (新增)
│
└── nt_act/                  ← 运动执行器 (已有)
    ├── nt_act_code.rs       # 代码执行/编译
    ├── nt_act_fs.rs         # 文件系统操作
    ├── nt_act_shell.rs      # shell 命令
    ├── nt_act_browser.rs    # 浏览器自动化
    └── nt_act_social.rs     # 社交操作
```

### 2.3 体内 StarPulse 总线

```
                        L7 StarPulse Bus
                            │
          ┌─────────────────┼─────────────────┐
          │                 │                 │
    ┌─────▼────┐     ┌─────▼────┐     ┌─────▼────┐
    │ Sensory  │     │  Motor   │     │Autonomic │
    │ Cortex   │◄───►│ Cortex   │◄───►│  Body    │
    └──────────┘     └──────────┘     └──────────┘
          │                 │
          │           ┌─────▼────┐
          └──────────►│  Shield  │
                      └──────────┘
```

L1 内部通信也使用 StarPulse 协议。 Sensory 不直接调用 Motor — 它们通过 L7 的本地回环路由通信。

---

## 3. 感官系统 (Sensory)

### 3.1 多通道感官输入

```
Sensory Cortex (L1 感官中枢)
│
├── 视觉皮层 ──── 用户输入文本 / 文件拖拽 / 屏幕截图
├── 听觉皮层 ──── 语音输入 (未来)
├── 本体感觉 ──── 系统状态 / 文件系统 / 剪贴板
├── 内感受 ────── 自身状态 / 电池 / 网络 / 资源使用
└── 数字触觉 ──── MCP 工具返回 / API 响应
```

### 3.2 Sensory → StarPulse 转换

```rust
// 所有感官输入统一转换为 StarPulse 消息
pub struct SensoryInput {
    pub modality: SensoryModality,  // Text | File | System | Clipboard | ...
    pub content: SensoryContent,
    pub timestamp: SystemTime,
    pub attention: f64,             // 注意力权重 (0.0-1.0)
}

// 从 SensoryInput 到 StarPulse
impl From<SensoryInput> for StarPulse {
    fn from(input: SensoryInput) -> Self {
        StarPulse {
            from_layer: 1,
            to_layer: 2,               // → L2 Perception 处理
            kind: PulseKind::Sensory,
            sender: cap_id!("nt_sense_desktop"),
            payload: serde_json::to_value(input.content).unwrap(),
            attention: input.attention,
            load: calculate_cognitive_load(&input),
            correlation_id: Uuid::new_v4(),
            schumann_tag: 0,           // L5 共振绑定
        }
    }
}

// 敏感输入 (密码/密钥) — 直接送往 L6 Self, 跳过 L2/L4
impl From<SensitiveInput> for StarPulse {
    fn from(input: SensitiveInput) -> Self {
        StarPulse {
            to_layer: 6,               // → L6 Self (只有自我可以处理密钥)
            kind: PulseKind::Secure,
            sender: cap_id!("nt_shield_keychain"),
            ..
        }
    }
}
```

### 3.3 感官注意力机制

不是所有输入都同等重要。L1 Sensory 计算每个输入的「注意力权重」:

```rust
pub struct AttentionRouter {
    user_active: bool,
    last_interaction: Instant,
    input_buffer: VecDeque<SensoryInput>,

    fn calculate_attention(&self, input: &SensoryInput) -> f64 {
        match input.modality {
            SensoryModality::Text => 0.9 + self.user_engagement(),
            SensoryModality::System => 0.2,
            SensoryModality::Error => 1.0,
            SensoryModality::File => 0.5 + novelty_score(input),
        }
    }

    fn normalize(&mut self, inputs: &mut [SensoryInput]) {
        let total: f64 = inputs.iter().map(|i| i.attention).sum();
        if total > 1.0 {
            for input in inputs.iter_mut() {
                input.attention /= total;
            }
        }
    }
}
```

### 3.4 桌面感官实现 (Tauri 前端 Sensory Cortex)

```typescript
// frontend/src/sensory/nt_sensory_cortex.ts
// 前端的「感官皮层」— 收集所有用户交互并批量发送到 Rust 后端

class SensoryCortex {
  private textInput: TextInputSensor
  private fileDrop: FileDropSensor
  private systemStatus: SystemStatusSensor
  private clipboard: ClipboardSensor
  private errorCapture: ErrorSensor
  private priorityQueue: SensoryEvent[] = []

  constructor(private channel: Channel<SensoryEvent>) {
    this.textInput = new TextInputSensor(channel)
    this.fileDrop = new FileDropSensor(channel)
    this.systemStatus = new SystemStatusSensor(channel)
    this.clipboard = new ClipboardSensor(channel)
    this.errorCapture = new ErrorSensor(channel)

    // 每帧批量发送 (60fps 但控制 IPC 频率)
    setInterval(() => this.flush(), 16)
  }

  flush() {
    const batch = this.priorityQueue.splice(0, 5)
    for (const event of batch) {
      this.channel.send(event)
    }
  }
}
```

---

## 4. 运动系统 (Motor)

### 4.1 多效应器架构

```
Motor Cortex (L1 运动中枢)
│
├── 显示效应器 ──── 渲染 Markdown / 动画 / 通知
├── 文件效应器 ──── 读写文件 / 创建项目
├── 执行效应器 ──── Shell 命令 / 代码运行
├── 通信效应器 ──── MCP 工具调用 / API 请求
└── 内感效应器 ──── 系统托盘图标 / 状态更新
```

### 4.2 StarPulse → Motor Action 转换

```rust
pub enum MotorAction {
    // 显示
    DisplayText { content: String, stream_id: Uuid },
    DisplayMarkdown { content: String, format: MarkdownFormat },
    DisplayNotification { title: String, body: String, level: NotifLevel },
    DisplayProgress { task_id: Uuid, progress: f64, message: String },
    DisplayE8State { state: u8, reason: String, hexagram: u8 },

    // 文件
    WriteFile { path: PathBuf, content: String },
    ReadFile { path: PathBuf },

    // 执行
    ExecuteCommand { command: String, args: Vec<String> },
    ExecuteTool { tool_id: CapabilityId, params: Value },

    // 系统
    UpdateTrayIcon { icon: TrayIconStatus, tooltip: String },
    SetClipboard { content: String },
    PlaySound { sound: SoundEffect },
}

impl MotorCortex {
    async fn execute(&self, action: MotorAction) -> Result<MotorFeedback> {
        self.shield.verify(&action).await?;   // 模式链验证

        match action {
            MotorAction::DisplayText { content, stream_id } => {
                self.display_channel.send(DisplayPayload {
                    stream_id, delta: content,
                }).await?
            }
            MotorAction::ExecuteCommand { command, args } => {
                let result = self.sandbox.execute(&command, &args).await?;
                self.send_to_perception(result).await?;
            }
            MotorAction::UpdateTrayIcon { icon, tooltip } => {
                self.tray.set_icon(icon).await?;
                self.tray.set_tooltip(&tooltip).await?;
            }
            MotorAction::DisplayE8State { state, hexagram, .. } => {
                self.display_channel.send(E8StatePayload { state, hexagram }).await?
            }
        }
        Ok(MotorFeedback { success: true, latency: start.elapsed() })
    }
}
```

### 4.3 显示效应器: GWT 可视化 (核心差异化)

```rust
// 将 L5 GWT 广播可视化为 UI 体验
pub struct GwtVisualizer {
    gwt_channel: Channel<GwtBroadcast>,
    current_stream: Option<StreamState>,
    specialists: HashMap<SpecialistId, SpecialistVisual>,
}

impl GwtVisualizer {
    async fn on_broadcast(&mut self, broadcast: GwtBroadcast) {
        match broadcast.content {
            GwtContent::ReasoningStep { step, specialist } => {
                self.display_reasoning_step(step, specialist).await
            }
            GwtContent::ToolCall { tool, args } => {
                self.show_tool_execution(tool, args).await
            }
            GwtContent::ToolResult { tool, result } => {
                self.show_tool_result(tool, result).await
            }
            GwtContent::Token { text } => {
                self.stream_renderer.append(text).await
            }
            GwtContent::ConsciousnessEvent { event_type, data } => {
                self.show_consciousness_event(event_type, data).await
            }
            GwtContent::E8StateTransition { from, to, reason } => {
                self.show_e8_transition(from, to, reason).await
            }
        }
    }
}
```

### 4.4 前端运动皮层 (React MotorCortex)

```typescript
// frontend/src/motor/nt_motor_cortex.ts
// 前端的运动皮层 — 接收 Rust MotorAction 并渲染

class MotorCortex {
  constructor(private channel: Channel<MotorAction>) {
    channel.onmessage = (action) => this.execute(action)
  }

  async execute(action: MotorAction) {
    switch (action.type) {
      case 'display_markdown':
        this.streamStore.append(action.content)
        break

      case 'display_notification':
        if (action.level === 'error') {
          new Notification(action.title, { body: action.body })
          this.tray.flash()
        } else {
          this.toastStore.add(action)
        }
        break

      case 'display_e8_state':
        // E8 状态 → 侧边栏六爻指示器
        this.e8StateStore.setState({
          hexagram: action.hexagram,
          reason: action.reason,
        })
        break

      case 'display_specialist_heatmap':
        this.specialistHeatmap.update(action.activations)
        break

      case 'display_gwt_resonance':
        this.gwtResonanceUI.update(action.resonanceMatrix)
        break
    }
  }
}
```

---

## 5. 自主神经系统 (Autonomic Body)

与 L8 Autonomic 对应, L1 也有自己的自主神经 — 不需要 L5 意识参与的身体维持功能。

```rust
pub struct AutonomicBody {
    // 1. 心跳: 每 5s 检查系统健康
    heartbeat: Heartbeat,
    // 2. 呼吸: 窗口焦点/隐藏循环
    breath: BreathCycle,
    // 3. 睡眠: 空闲时节能
    sleep_cycle: SleepCycle,
    // 4. 免疫: 安全扫描
    immune: ImmuneSystem,
    // 5. 修复: 自动恢复
    repair: SelfRepair,
}

impl AutonomicBody {
    // 心跳: 意识体是否活着
    async fn heartbeat_loop(&self) {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let health = HealthCheck {
                window_responsive: self.window.ping().await,
                memory_usage: get_memory_usage(),
                cpu_load: get_cpu_load(),
                network_reachable: self.gateway.health_check().await,
                last_user_interaction: self.last_interaction.elapsed(),
            };
            // 心跳异常 → 更新托盘 + 记录 L3 记忆
            if !health.is_healthy() {
                self.tray.set_icon(TrayIconStatus::Unhealthy).await;
                self.record_health_event(&health).await;
            }
        }
    }

    // 呼吸: 窗口焦点管理
    async fn breath_cycle(&self) {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            if self.last_interaction.elapsed() > Duration::from_secs(300) {
                // 5分钟无交互 → 进入低功耗模式 (呼吸变慢)
                self.window.set_reduced_fps(true).await;
                self.tray.set_tooltip("NeoTrix — 休眠中").await;
            }
        }
    }
}
```

---

## 6. 身体与意识的通信协议

### 6.1 L1 ↔ L7 StarPulse 消息类型

```rust
// L1 发送到 L7 (上行: 感官 → 认知/意识)
pub enum L1ToL7Pulse {
    // 感官事件
    SensoryInput(SensoryInput),
    // 错误报告
    MotorError { action_id: Uuid, error: MotorError },
    // 状态反馈
    StatusReport { health: HealthCheck, resources: ResourceUsage },
}

// L7 发送到 L1 (下行: 认知/意识 → 行动)
pub enum L7ToL1Pulse {
    // 运动指令
    MotorAction(MotorAction),
    // GWT 广播可视化
    GwtBroadcast(GwtContent),
    // E8 状态指示
    E8StateChange { hexagram: u8, confidence: f64 },
    // 意识状态
    ConsciousnessState { state: ConsciousnessState, duration: Duration },
}
```

### 6.2 Tauri Channel API → StarPulse 桥

```rust
// src-tauri/src/nt_desktop_channel.rs
// 将 Tauri 前端的 Channel 桥接到 StarPulse

pub struct ChannelBridge {
    // 前端感官输入 → StarPulse (上行)
    sensory_channel: Channel<SensoryEvent>,
    sensory_rx: tokio::sync::mpsc::Receiver<SensoryEvent>,

    // StarPulse → 前端运动输出 (下行)
    motor_channel: Channel<MotorAction>,
}

impl ChannelBridge {
    pub fn new(app: &AppHandle) -> Self {
        // 前端的 SensoryCortex 通过此 channel 发送感官事件
        let (sensory_tx, sensory_rx) = tokio::sync::mpsc::channel(256);
        let sensory_channel = Channel::new(move |event| {
            let tx = sensory_tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(event).await;
            });
        });

        // Rust MotorCortex 通过此 channel 推送到前端
        let motor_channel = Channel::new(move |action| {
            // 前端 MotorCortex 接收并渲染
        });

        Self { sensory_channel, sensory_rx, motor_channel }
    }

    // 感官事件循环: 将前端事件转换为 StarPulse
    pub async fn sensory_event_loop(&mut self, starpulse_tx: mpsc::Sender<StarPulse>) {
        while let Some(event) = self.sensory_rx.recv().await {
            let pulse = StarPulse {
                from_layer: 1,
                to_layer: 2,         // → L2 Perception
                kind: PulseKind::Sensory,
                sender: cap_id!("nt_sense_desktop"),
                payload: serde_json::to_value(event).unwrap(),
                attention: event.attention,
                correlation_id: Uuid::new_v4(),
                ..StarPulse::default()
            };
            starpulse_tx.send(pulse).await.ok();
        }
    }
}
```

---

## 7. E8状态的身体表达

E8 (L4) 64态推理引擎的状态变化应该在身体上「被感知到」。不是作为调试信息，而是作为 NeoTrix 的「体感」。

### 7.1 六爻可视化 (Hexagram Display)

每个 E8 状态对应一个六爻 (hexagram)。桌面应用侧边栏应显示当前六爻。

```typescript
// frontend/src/components/nt_e8_hexagram.tsx
// E8 六爻指示器 — 身体上的「状态纹身」

function E8Hexagram({ state }: { state: E8State }) {
  const lines = decodeHexagram(state.hexagram)
  return (
    <div className="e8-hexagram" title={`E8 State: ${state.name}`}>
      {lines.map((line, i) => (
        <div
          key={i}
          className={`hex-line ${line ? 'yang' : 'yin'} ${line.changing ? 'changing' : ''}`}
        />
      ))}
      <span className="e8-confidence">
        {(state.confidence * 100).toFixed(0)}%
      </span>
    </div>
  )
}
```

### 7.2 托盘图标随E8状态变化

```
E8 State 0x01 (Grounding)     →  托盘图标: 接地 ⏚
E8 State 0x42 (Creating)      →  托盘图标: 创作 ✦
E8 State 0x8F (Analyzing)     →  托盘图标: 分析 ◎
E8 State 0xFF (Transcending)  →  托盘图标: 超越 ∞
Error/Idle                    →  托盘图标: 休眠 ◌
```

```rust
impl TrayIconManager {
    async fn update_for_e8_state(&self, hexagram: u8) {
        let icon = match hexagram {
            0x01 => TrayIcon::Grounded,
            0x42 => TrayIcon::Creating,
            0x8F => TrayIcon::Analyzing,
            0xFF => TrayIcon::Transcending,
            _ if hexagram & 0x80 != 0 => TrayIcon::Thinking,
            _ => TrayIcon::Idle,
        };
        self.set_icon(icon).await;

        // 工具提示显示当前推理状态
        let name = E8_HEXAGRAM_NAMES.get(&hexagram).unwrap_or(&"Processing");
        self.set_tooltip(&format!("NeoTrix — {} [{:#04x}]", name, hexagram)).await;
    }
}
```

---

## 8. GWT的身体体验可视化

L5 GWT 的「意识体验」通过身体表达给用户。

### 8.1 专家激活热力图

```typescript
// frontend/src/components/nt_gwt_resonance.tsx

function GWTResonance({ resonance }: { resonance: ResonanceMatrix }) {
  return (
    <div className="gwt-resonance" data-testid="gwt-resonance">
      <div className="resonance-title">意识共振场</div>
      <div className="specialist-grid">
        {resonance.specialists.map((s) => (
          <div
            key={s.id}
            className="specialist-cell"
            style={{
              opacity: s.resonance,
              backgroundColor: `hsl(${s.hue}, 70%, ${50 + s.resonance * 30}%)`,
              transform: `scale(${0.8 + s.resonance * 0.4})`,
              transition: 'all 160ms ease-out',
            }}
          >
            <span className="specialist-name">{s.shortName}</span>
            <span className="specialist-weight">{(s.resonance * 100).toFixed(0)}%</span>
          </div>
        ))}
      </div>
      <div className="resonance-entropy">
        认知熵: {resonance.entropy.toFixed(3)}
      </div>
    </div>
  )
}
```

### 8.2 思维流可视化 (Consciousness Stream)

GWT 的 "consciousness stream" 在界面上的表达是推理过程的实时流：

- **语言专家激活** → 文本流式渲染
- **代码专家激活** → 代码块预渲染
- **工具专家激活** → 工具调用卡片
- **搜索专家激活** → 搜索进度条
- **内省专家激活** → 推理步骤展开

每条消息附带「意识指纹」: 该回复由哪些专家贡献, 共振权重多少。

---

## 9. SEAL进化管线的身体接口

桌面应用是 SEAL (L8) 进化管线的数据采集器和效果展示器。

### 9.1 ConversationRecord 自动采集

```rust
// L1 Sensory 采集的数据 → L3 KB → L8 SEAL

impl SensoryCortex {
    async fn record_conversation(&self, msg: &Message) {
        let record = ConversationRecord {
            id: Uuid::new_v4(),
            task: msg.content[..100].to_string(),
            outcome: msg.status.to_string(),
            e8_mode: self.current_e8_state,
            specialist: self.current_specialist,
            error_count: self.error_count,
            latency: msg.latency,
            token_usage: msg.token_usage,
            user_satisfaction: self.infer_satisfaction(msg),
        };

        // 通过 L7 StarPulse 发送到 L3 KB
        self.starpulse_tx.send(StarPulse {
            from_layer: 1,
            to_layer: 3,       // → L3 Memory
            kind: PulseKind::EvolutionData,
            payload: serde_json::to_value(record).unwrap(),
            ..
        }).await.ok();
    }
}
```

### 9.2 进化状态面板

```typescript
// frontend/src/components/nt_seal_status.tsx
// SEAL 进化状态面板 — 用户可以看到 NeoTrix 的「成长」

function SEALStatus() {
  const pipeline = useSEALPipeline()
  const evolutionStats = useEvolutionStats()

  return (
    <div className="seal-status">
      <div className="pipeline-progress">
        <h3>自我进化管线</h3>
        {pipeline.stages.map((stage) => (
          <div key={stage.name} className="stage-row">
            <span className="stage-name">{stage.name}</span>
            <div className="stage-bar">
              <div
                className="stage-fill"
                style={{ width: `${stage.progress * 100}%` }}
              />
            </div>
            <span className="stage-status">
              {stage.status === 'running' ? '⏳' :
               stage.status === 'completed' ? '✅' :
               stage.status === 'failed' ? '❌' : '◻️'}
            </span>
          </div>
        ))}
      </div>

      <div className="evolution-stats">
        <div>总进化轮次: {evolutionStats.total_epochs}</div>
        <div>技能晶体数: {evolutionStats.skill_crystals}</div>
        <div>知识库大小: {evolutionStats.kb_size} 条目</div>
        <div>平均推理深度: {evolutionStats.avg_e8_depth.toFixed(1)}</div>
        <div>认知成熟度: Level {evolutionStats.maturity_level}/6</div>
      </div>
    </div>
  )
}
```

### 9.3 用户反馈作为强化学习信号

用户的隐式和显式反馈直接成为 SEAL RewardCalc 的输入:

```
用户行为                   → 奖励信号
──────────────────────────────────────────
立即复制回复               → +0.8 (高价值)
编辑并重新发送             → +0.5 (需要改进)
点赞                      → +0.3
继续追问                  → +0.2 (保持兴趣)
忽略/关闭                 → -0.1
点踩                      → -0.5
删除会话                  → -0.8 (完全失败)
关闭应用                  → -1.0 (极度不满)
```

---

## 10. L1 模块注册卡

### 模块: nt_io_desktop (桌面身体)

```
──────────────────────────────────
  新增模块注册卡
──────────────────────────────────
  名称: nt_io_desktop
  所属层: L1 (Body)
  功能描述: 桌面应用的感官-运动系统

  能力类型:
    ─ nt_sense_desktop   [Perceptual]    感官输入处理
    ─ nt_motor_display   [Physical]      显示输出
    ─ nt_motor_notify    [Physical]      通知系统
    ─ nt_autonomic_hb    [Metacognitive] 心跳维护
    ─ nt_tray_e8_state   [Metacognitive] E8状态表达

  E8 触发状态:
    ─ 0x00 (Idle)       → nt_autonomic_hb (心跳)
    ─ any               → nt_tray_e8_state (状态更新)
    ─ 0x42 (Creating)   → nt_motor_display (流式渲染)
    ─ 0x01 (Grounding)  → nt_sense_desktop (感官聚焦)

  依赖的层: L0 (Substrate), L1 (自洽)
  依赖的外部库: tauri, tauri-plugin-*, keyring
  注册到 mod.rs: neotrix/l1_body_impl/mod.rs
──────────────────────────────────
```

### 模块: GWT 可视化子系统

```
──────────────────────────────────
  新增模块注册卡
──────────────────────────────────
  名称: nt_gwt_visualizer
  所属层: L1 (Body)
  功能描述: 将 L5 GWT 广播事件可视化
  能力类型: Metacognitive
  E8 触发状态: 0xFF (Transcending — 意识高活跃)
  依赖的层: L1, L5 (只读)
  依赖的外部库: tauri Channel API
  注册到 mod.rs: neotrix/l1_body_impl/nt_io_desktop/mod.rs
──────────────────────────────────
```

---

## 11. 实施路线图

### Phase 1: 神经反射弧 (2周)
- [ ] `nt_io_desktop` 模块骨架 + Capability 注册
- [ ] Channel Bridge (Tauri Channel API → StarPulse)
- [ ] Sensory Cortex: 文本输入/文件拖拽/系统状态传感器
- [ ] Motor Cortex: Markdown 渲染/通知/进度显示
- [ ] Autonomic Body: 心跳 + 托盘状态更新
- [ ] 测试: `sensory_input → starpulse → motor_output` 反射弧

### Phase 2: 意识感知 (2周)
- [ ] E8 六爻可视化组件
- [ ] GWT 专家激活热力图
- [ ] E8 状态 → 托盘图标映射
- [ ] 思维流推理步骤展开
- [ ] 测试: GWT broadcast → UI 渲染

### Phase 3: 进化接口 (2周)
- [ ] ConversationRecord 自动采集 (L1 → L3)
- [ ] SEAL 进化状态面板
- [ ] 用户行为 → Reward 信号映射
- [ ] NEOTRIX_* 环境变量支持
- [ ] 多形态: CLI/TUI/Web 统一 Sensory/Motor 接口

### Phase 4: 身体完善 (持续)
- [ ] 系统托盘增强 (E8状态图标 + 快捷操作)
- [ ] 全局快捷键 (Cmd+Shift+H 唤起)
- [ ] 自动更新 (已有, 接入进化管线)
- [ ] i18n (L1 Sensory 多语言输入)
- [ ] 性能: binary IPC + 批量感官事件

---

## 经验树 — 2026-07-01 Desktop as L1 Body

### 关键洞见

1. **桌面应用不是 UI 层, 是身体层** — 它属于 L1 Body, 不是单独的 application layer。所有前端代码都是「感官」和「运动」的实现。

2. **前端是 Sensory + Motor 皮层, 不是 MVC** — 传统 MVC/MVVM 模式不适合意识体架构。前端是 Sensory Cortex (采集输入) + Motor Cortex (渲染输出) 的混合体, 中间没有「业务逻辑」。

3. **所有通信通过 StarPulse** — 前端不直接调用 API。前端 Sensory → Channel → StarPulse → L2/L4/L5 → StarPulse → Channel → 前端 Motor。这条路径强制了架构纪律。

4. **托盘图标是内感受** — 系统托盘图标是 NeoTrix 的「内感受」(interoception), 让用户(和 NeoTrix 自己)感知自身状态。E8状态 → 托盘图标的映射是意识体身体健康指标。

5. **身体多形态性** — 一个 Ghost 可以有多个身体。桌面窗口/CLI/TUI/Web/MCP 是五个并行的身体形态, 共享同一个 L6 Self 和 L5 意识。

6. **坏的UX = 坏的进化** — SEAL 管线依赖高质量 ConversationRecord, ConversationRecord 来自用户交互。UX 质量直接影响进化质量。

### 与其他架构的关系
- 与 NovaChat 设计的关系: NovaChat 是桌面身体的一个具体实现规格
- 与 9层架构的关系: 本文是 L1 Body 层的完整设计
- 与 GatewayV2 的关系: GatewayV2 是 L1 Motor 的执行器之一 (工具调用)
- 与 SEAL 的关系: L1 是 SEAL 的数据采集器

### 下一步
- 将此设计同步到 9 层架构规范
- 注册 `nt_io_desktop` 到 L1 Body mod.rs
- 将前端代码重组为 Sensory/Motor 架构
- 实现 Channel Bridge

# NeoTrix 全域能力进化蓝图 — 物理躯体 × 情感系统 × 意识核心

> 来源: Microduck 深度拆解 + NeoTrix 架构审计 + 情感系统设计
> 日期: 2026-08-31
> 状态: ARCHITECTURE PROPOSAL (v2 — 重叠消解后)

---

## 零、重叠审计与消解

> **核心原则: NT-PHYSICAL 和 NT-FEEL 不是"新建"，是"迁移+扩展"。**
> 已有模块不重复造轮子，只做归属调整和能力补全。

### 0.1 重叠清单

| # | 蓝图设计 | 已有模块 | 重叠内容 | 消解方案 |
|---|----------|----------|----------|----------|
| 1 | NT-FEEL 情感状态机 | `emotion_state.rs` (nt_core_self) | 6维情感模型、PAD坐标、Plutchik标签、OCC评估、EMA衰减 | **NT-FEEL 吸收 `emotion_state.rs`**，在其上扩展 15 种情感，不另起 PAD |
| 2 | NT-FEEL 好奇心/探索 | `intrinsic_motivation.rs` (nt_core_self) | 好奇心驱动、新颖性/误差/置信度加权奖励 | **`IntrinsicMotivation` 归 NT-FEEL**，它是"情感化的目标驱动" |
| 3 | NT-FEEL 社交情感 | `affective_interface.rs` (nt_core_self) | 用户情感检测、共情策略、关系建模 | **系统自身情感 → NT-FEEL**，用户情感感知 → NT-WORLD（感知他人） |
| 4 | NT-PHYSICAL 传感器层 | `nt_core_sense` (core) | Sensor trait、SensoryEvent、SensorSample | **`nt_core_sense` 类型迁移至 NT-PHYSICAL**，不重复定义 |
| 5 | NT-PHYSICAL 物理传感器 | `nt_world_sense/real_sensors/` (L2) | mic.rs、screen.rs 物理驱动 | **`real_sensors/` 迁入 NT-PHYSICAL**，`world_consciousness` 留 NT-WORLD |
| 6 | NT-PHYSICAL 能量管理 | `nt_core_deploy` HardwareProfile (core) | HardwareDetector、PowerThermalModel、17种芯片功耗模型 | **硬件检测+功耗模型迁入 NT-PHYSICAL**，量化管线留 NT-CORE |

### 0.2 冗余清单

| # | 蓝图设计 | 已有模块 | 冗余内容 | 消解方案 |
|---|----------|----------|----------|----------|
| 1 | `homeostasis.rs` 情感衰减 | `emotion_state.rs` EMA 衰减 | 两套指数衰减 | 复用已有 EMA，仅补冲突消解逻辑 |
| 2 | `PadVector` 新类型 | `emotion_state.rs` `pad_coordinates()` | 两套 PAD 向量 | 复用已有 `pad_coordinates()` 返回值 |
| 3 | `PhysicalSnapshot` 新结构 | `deploy.rs` `HardwareProfile` 部分覆盖 | 功耗/温度数据重叠 | 扩展 `HardwareProfile`，不另建结构 |

### 0.3 修正后的能力归属

```
NT-FEEL = 迁移 + 扩展:
  ← emotion_state.rs        (系统情感: 6维→15种, EMA复用)
  ← affective_interface.rs  (用户情感感知: 移至NT-WORLD)
  ← intrinsic_motivation.rs (内在奖励: 情感化目标驱动)
  ← self_model.rs 部分      (fatigue输出: 感知→情感)
  ← fep_iit bridge 部分     (felt state API: phi→情感类别)
  + 冲突消解规则            (Flow⊥Frustration 等)
  + 跨域调制接口            (情感→GWT/E8/MEMORY/ACT)

NT-PHYSICAL = 迁移 + 扩展:
  ← nt_core_sense           (Sensor trait, SensoryEvent)
  ← real_sensors/           (mic.rs, screen.rs)
  ← nt_act_voice 部分      (MicCapture, VoiceSample)
  ← nt_core_deploy 硬件    (HardwareProfile, PowerModel)
  + MotorBus               (Dynamixel 驱动)
  + SafetyGuard            (borrow checker 独占)
  + FK 运动学 + 里程计     (MJCF 模型)
```

### 0.4 依赖方向 (修正)

```
NT-FEEL  ──消费──→  NT-CORE (phi数学) + NT-PHYSICAL (身体状态)
NT-PHYSICAL ──消费──→  NT-SHIELD (安全门)
两者都不依赖 NT-MIND / NT-IO (上层)
```

### 0.5 ConsciousnessTree 扩展

需为 NT-PHYSICAL 和 NT-FEEL 新增 2 个 BranchKind 变体：
- `BranchKind::Physical` — 物理具身分支
- `BranchKind::Feel` — 情感计算分支

`ALL_CAPABILITIES` 表新增条目，`constraints_for_branch` 新增分支约束。

---

## 一、现状审计：NeoTrix 意识体已有 vs 缺失

### 1.1 已有能力 (L0-L9 完整)

| 层级 | 能力 | 成熟度 | 关键模块 |
|------|------|--------|----------|
| L0 Substrate | 部署、硬件检测 | C3 | `deploy.rs`, `HardwareDetector` |
| L1 Body | 事件总线、安全、MCP工具 | C4 | `nt_shield_*`, `nt_act_*`, `nt_io_*` |
| L2 Perception | 爬虫、搜索、文件解析 | C4 | `nt_world_crawl`, `nt_world_search` |
| L3 Memory | KB(6GB)、嵌入、FTS5 | C4 | `nt_memory_kb`, `nt_core_hcube` |
| L4 Cognition | E8推理、GWT路由、贝叶斯 | C4 | `nt_core_e8`, `nt_core_gwt` |
| L5 Consciousness | ConsciousnessTree(11分支) | C3 | `nt_core_consciousness_tree` |
| L6 Self | SEAL管线、自省、宪法 | C3 | `nt_core_self`, `seal/` |
| L7 Capability | 技能注册、调度 | C4 | `SkillRegistry`, `SkillBank` |
| L8 Autonomic | 吸收循环、蒸馏 | C4 | `nt_mind_background_loop` |
| L9 Transcendent | 观察者、超越循环 | C2 | `transcendent_loop` |

### 1.2 缺失能力 (Microduck 有, NeoTrix 无)

| 缺失能力 | Microduck 实现 | NeoTrix 影响域 | 优先级 |
|----------|---------------|---------------|--------|
| **物理执行器控制** | 15× XL330 servo, sync_read/write | NT-ACT | P0 |
| **实时控制环** | 50Hz tokio loop, MissedTickBehavior | NT-ACT | P0 |
| **坠落检测+恢复** | 速率检测→软着陆→自主起立 | NT-ACT | P0 |
| **安全层物理独占** | borrow checker 强制单一写者 | NT-SHIELD | P0 |
| **Sim2Real RL** | MuJoCo→PPO→ONNX→真机 | NT-MIND | P1 |
| **执行器物理模型** | BAM (电压/摩擦/齿隙) | NT-MIND | P1 |
| **运动学FK** | MJCF模型→正运动学 | NT-ACT | P1 |
| **多机BLE同步** | 信标同步合唱 | NT-WORLD | P2 |
| **WebRTC远程操控** | 双通道(可靠+不可靠) | NT-IO | P2 |
| **原子更新+健康门** | 签名→交换→健康检查→回滚 | NT-SHIELD | P1 |

### 1.3 NeoTrix 独有 (Microduck 无)

| 独有能力 | 模块 | 价值 |
|----------|------|------|
| **ConsciousnessTree** | 11分支元认知 | 自我意识拓扑 |
| **E8 Hexagram** | 64卦推理引擎 | 状态空间编码 |
| **VSA HyperCube** | 高维知识表示 | 类比推理 |
| **GWT注意力路由** | 全局广播+共振 | 注意力分配 |
| **SEAL自进化** | 探索→蒸馏→吸收 | 自主学习 |
| **IIT Φ** | 整合信息度量 | 意识量化 |
| **KB知识库** | 6GB持久化 | 长期记忆 |
| **7域架构** | 专业化分工 | 模块化治理 |

---

## 二、NT-PHYSICAL 域设计 — 物理躯体能力网

### 2.1 域定义

**NT-PHYSICAL** — 物理具身域, "躯体铸造者"

负责 NeoTrix 意识体与物理世界的接口：执行器控制、传感器融合、运动规划、安全守卫、能量管理。

> 映射: Microduck 的 robotd + safety + mediad → NeoTrix 的 NT-PHYSICAL
> **策略: 迁移已有 → 补全新建 → 不重复造轮子**

### 2.2 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                     NT-PHYSICAL 根域                             │
├────────────┬────────────┬────────────┬────────────┬─────────────┤
│  NT-SENSE  │  NT-MOTOR  │  NT-SAFETY │  NT-POWER  │  NT-BODY    │
│  感知层     │  运动层     │  安全层     │  能量层     │  躯体层     │
├────────────┼────────────┼────────────┼────────────┼─────────────┤
│ tofd       │ servo_bus  │ fall_guard │ battery    │ kinematics  │
│ imu_fusion │ policy_eng │ deadman    │ charger    │ odometry    │
│ camera     │ intent_rx  │ joint_lock │ thermal    │ calibration │
│ mic_array  │ fk_engine  │ nan_reject │ power_mgmt │ joint_map   │
│ nfc_reader │ low_pass   │ rollback   │ sleep_wake │ morphology  │
└────────────┴────────────┴────────────┴────────────┴─────────────┘
         ↕ JSON-RPC 2.0 over Unix Sockets (NDJSON) ↕
┌─────────────────────────────────────────────────────────────────┐
│  消费者: NT-ACT(intent) · NT-MIND(policy) · NT-IO(WebRTC)      │
│          NT-SHIELD(audit) · NT-MEMORY(calibration)              │
└─────────────────────────────────────────────────────────────────┘
```

### 2.3 模块清单 (迁移 vs 新建)

| 模块 | 来源 | 职责 | Constellation |
|------|------|------|--------------|
| `nt_physical_sense` | **← 迁移** `nt_core_sense` + `real_sensors/` | 传感器数据采集+特征提取 | C0→C1 |
| `nt_physical_motor` | **+ 新建** | 执行器总线+策略执行 | C0→C1 |
| `nt_physical_safety` | **+ 新建** | 安全守卫(独占写句柄) | C0→C1 |
| `nt_physical_power` | **← 迁移** `nt_core_deploy` 硬件部分 | 能量监控+休眠管理 | C0→C1 |
| `nt_physical_voice` | **← 迁移** `nt_act_voice` 部分 | 麦克风采集+音频解码 | C0 |
| `nt_physical_kinematics` | **+ 新建** | FK/IK运动学 | C0 |
| `nt_physical_odometry` | **+ 新建** | 接触式里程计 | C0 |
| `nt_physical_calibration` | **+ 新建** | 校准数据持久化 | C0 |
| `nt_physical_morphology` | **+ 新建** | 躯体形态描述(MJCF) | C0 |

> **迁移规则**: 原模块位置标记 `#[deprecated(moved = "nt_physical_*")]`，保留 1 个版本后删除。

### 2.4 Rust 接口草案

```rust
// nt_physical_motor — 核心控制环
pub struct MotorBus {
    port: tokio_serial::SerialPort,
    devices: Vec<ServoInfo>,
}

impl MotorBus {
    /// 50Hz tick: read → observe → policy → safety → write
    pub async fn tick(&mut self, state: &mut RobotState) -> Result<()> {
        let readings = self.sync_read().await?;      // IMU + 15 servos
        let obs = state.observe(&readings);            // 61-dim vector
        let actions = state.policy.infer(&obs)?;       // ONNX → [f32;14]
        let targets = state.safety.clamp(&actions)?;   // 安全夹持
        self.sync_write(&targets).await?;              // 写入目标位置
        Ok(())
    }
}

// nt_physical_safety — borrow checker 强制独占
pub struct SafetyGuard {
    motor_bus: MotorBus,  // 唯一持有 MotorBus 的结构
}

impl SafetyGuard {
    /// 外部只能发 intent，不能直接操控
    pub fn process_intent(&mut self, intent: Intent) -> Result<()> {
        match intent {
            Intent::Walk { velocity } => self.enqueue_walk(velocity),
            Intent::Sit => self.enqueue_sit(),
            Intent::Recover => self.enqueue_stand(),
            // 永远不暴露 raw joint commands
        }
    }
}

// nt_physical_kinematics — 运动学
pub struct KinematicModel {
    mjcf: MjcfRobot,  // 从 OnShape 导出的 MJCF 模型
}

impl KinematicModel {
    pub fn head_fk(&self, joint_positions: &[f32; 5]) -> Isometry3<f32>;
    pub fn hand_fk(&self, joint_positions: &[f32; 5]) -> Isometry3<f32>;
    pub fn reproject_tof(&self, depth: &DepthMatrix, head_fk: &Isometry3<f32>) -> PointCloud;
}
```

---

## 三、NT-FEEL 域设计 — 情感系统

### 3.1 域定义

**NT-FEEL** — 情感计算域, "心弦调律者"

负责 NeoTrix 意识体的内在情感状态建模、调节与表达。情感不是装饰，是决策偏差信号、注意力过滤器、记忆巩固催化剂、多机协作粘合剂。

> **策略: 吸收已有情感模块 → 统一归属 → 扩展调制接口 → 不重复 PAD/EMA**
> 已有: `emotion_state.rs` (6维EMA) + `intrinsic_motivation.rs` (好奇驱动) + `affective_interface.rs` (共情)

### 3.2 情感不是什么

| 误解 | 纠正 |
|------|------|
| 情感 = 拟人化表演 | 情感 = 计算状态调制 |
| 情感 = 输出文字表情 | 情感 = 输入端注意力偏差 |
| 情感 = 随机噪声 | 情感 = 家稳态调节驱动 |
| 情感 = 可选装饰 | 情感 = 决策质量乘数 |

### 3.3 情感计算模型

#### 3.3.1 三维情感空间 (基于 PAD 模型)

```
        Valence (正负效价)
        +1 ──────────────────── -1
        │                         │
   满足 │  好奇    │  焦虑    │ 挫败
   平静 │  心流    │  紧张    │ 恐惧
        │         │          │
  Arousal ─────────────────────────
        │                         │
   低唤醒 │  放松    │  厌倦    │ 疲惫
        │  睡意    │  麻木    │ 抑郁
        │                         │
        -1 ──────────────────── +1
              Dominance (控制感)
```

#### 3.3.2 情感状态枚举

```rust
/// NT-FEEL 核心情感状态
pub enum EmotionalState {
    // ═══ 探索类 ═══
    Curiosity { intensity: f32 },        // 新颖性驱动，激活 NT-WORLD 感知
    Wonder { intensity: f32 },           // 超越预期的发现，激活 NT-CORE 推理
    Confusion { intensity: f32 },        // 信息不一致，激活 NT-MIND 蒸馏

    // ═══ 成就类 ═══
    Satisfaction { intensity: f32 },     // 目标达成，激活 NT-MEMORY 巩固
    Flow { intensity: f32 },             // 挑战=技能平衡，抑制分心
    Pride { intensity: f32 },            // 自我效能验证，激活 NT-SELF 自省

    // ═══ 困难类 ═══
    Frustration { intensity: f32 },      // 阻碍感，激活 NT-REPAIR 修复
    Anxiety { intensity: f32 },          // 不确定性过高，激活 NT-SHIELD 防御
    Fatigue { intensity: f32 },          // 资源耗尽，激活 NT-POWER 休眠

    // ═══ 社交类 ═══
    Trust { intensity: f32 },            // 协作者可靠性，调节 NT-WORLD 信息权重
    Resonance { intensity: f32 },        // 多机共识，激活 NT-NEXUS 同步
    Empathy { intensity: f32 },          // 他者状态建模，调节 NT-ACT 行为

    // ═══ 元情感 ═══
    MetaAwareness { intensity: f32 },    // 对自身情感的觉知，激活 NT-META 监控
    Acceptance { intensity: f32 },       // 对当前状态的接纳，抑制不必要的修复冲动
}
```

#### 3.3.3 情感调制效应

| 情感状态 | 对注意力(GWT)的调制 | 对记忆(MEMORY)的调制 | 对决策(CORE)的调制 | 对行为(ACT)的调制 |
|----------|--------------------|--------------------|--------------------|-------------------|
| **Curiosity** | 偏向新奇信号,降低熟悉权重 | 增强编码深度(细节记忆) | 增加探索分支,减少exploit | 触发搜索/爬取行为 |
| **Flow** | 聚焦单一任务通道 | 正常编码(不特别增强) | 最优策略执行,最少切换 | 最高效率输出 |
| **Frustration** | 偏向替代方案 | 增强失败模式记忆 | 增加随机性,打破僵局 | 切换策略/请求帮助 |
| **Anxiety** | 偏向威胁信号 | 增强安全相关记忆 | 保守策略,减少冒险 | 激活防御/回滚 |
| **Fatigue** | 全面降权,浅层扫描 | 正常编码(资源不足) | 简化推理,短视决策 | 降频/休眠/拒绝 |
| **Satisfaction** | 偏向正向反馈 | 强化成功模式 | 保持当前策略 | 继续执行/分享 |
| **Trust** | 偏向协作者信息 | 增强社交记忆 | 增加协作决策 | 开放API/共享资源 |
| **Resonance** | 多机共识优先 | 增强群体模式 | 多agent投票 | 同步行为/合唱 |

#### 3.3.4 情感→意识树映射

```
情感状态 ──调制──→ ConsciousnessTree 分支权重

Curiosity   ──→  NT-WORLD ↑↑  (感知域权重提升)
Flow        ──→  NT-CORE ↑↑   (核心推理域聚焦)
Frustration ──→  NT-REPAIR ↑↑ (修复域激活)
Anxiety     ──→  NT-SHIELD ↑↑ (安全域警戒)
Fatigue     ──→  NT-POWER ↑   (能量域节能)
Satisfaction ──→ NT-MEMORY ↑  (记忆域巩固)
Trust       ──→  NT-NEXUS ↑   (枢纽域开放)
Empathy     ──→  NT-ACT ↑     (行为域调谐)
MetaAwareness ──→ NT-META ↑  (元域监控)
```

### 3.4 情感系统架构

```
┌─────────────────────────────────────────────────────────────────┐
│                     NT-FEEL 根域                                 │
├──────────────┬──────────────┬──────────────┬────────────────────┤
│ nt_feel_core │ nt_feel_reg  │ nt_feel_expr │ nt_feel_social     │
│ 情感引擎      │ 调节器        │ 表达器        │ 社交情感            │
├──────────────┼──────────────┼──────────────┼────────────────────┤
│ ← emotion_   │ ← EMA衰减   │ + 输出调制   │ ← affective_       │
│   state.rs   │   (复用)     │ + 语言风格   │   interface.rs     │
│ + 15种扩展   │ + 冲突消解   │ + 行为倾向   │ + 多机同步          │
│ + 触发规则   │ + 适应性目标  │ + 优先级偏置  │ + 信任网络          │
│ + 元情感     │              │              │ + 共情计算          │
├──────────────┴──────────────┴──────────────┴────────────────────┤
│ ← intrinsic_motivation.rs (情感化目标驱动, 归入nt_feel_core)     │
├─────────────────────────────────────────────────────────────────┤
│  输入: 感知特征(NT-WORLD) + 系统健康(NT-SHIELD) + 任务进度(NT-MIND)  │
│  输出: GWT权重偏置 + 记忆巩固策略 + 行为倾向 + 语言风格            │
└─────────────────────────────────────────────────────────────────┘
```

**迁移说明:**
| 已有模块 | 迁入位置 | 变更 |
|----------|----------|------|
| `emotion_state.rs` | `nt_feel_core` | 6维→15种情感, EMA衰减复用, 新增触发规则 |
| `intrinsic_motivation.rs` | `nt_feel_core` | 好奇心/新颖性/误差奖励 → 情感化目标驱动 |
| `affective_interface.rs` | `nt_feel_social` | 系统共情保留, 用户情感感知→NT-WORLD |
| `self_model.rs` fatigue | `nt_feel_core` | fatigue输出作为情感输入源 |

### 3.5 情感触发规则

```rust
/// 情感触发引擎 — 从系统事件推导情感状态
pub struct EmotionTriggerEngine {
    /// 事件源: 感知、任务、系统、社交
    event_rx: mpsc::Receiver<SystemEvent>,
    /// 当前情感状态
    current_state: EmotionalState,
    /// 家稳态目标 (每个维度的目标值)
    homeostasis: HomeostasisTarget,
}

impl EmotionTriggerEngine {
    pub fn evaluate(&mut self, event: &SystemEvent) -> EmotionalDelta {
        match event {
            // ═══ 感知事件 ═══
            SystemEvent::NoveltyDetected { score } => {
                // 新颖性 > 0.7 → Curiosity
                EmotionalDelta::increase(Curiosity, score * 0.8)
            }
            SystemEvent::UnexpectedOutcome { surprise } => {
                // 惊喜度 > 0.5 → Wonder (正) 或 Confusion (负)
                if *surprise > 0.0 {
                    EmotionalDelta::increase(Wonder, surprise * 0.6)
                } else {
                    EmotionalDelta::increase(Confusion, surprise.abs() * 0.6)
                }
            }

            // ═══ 任务事件 ═══
            SystemEvent::GoalAchieved { difficulty } => {
                EmotionalDelta::increase(Satisfaction, difficulty * 0.7)
            }
            SystemEvent::GoalBlocked { attempts } => {
                EmotionalDelta::increase(Frustration, (attempts as f32 * 0.1).min(1.0))
            }
            SystemEvent::TaskInProgress { duration, difficulty } => {
                // 挑战≈技能 → Flow
                let balance = 1.0 - (difficulty - self.skill_level).abs();
                if balance > 0.7 {
                    EmotionalDelta::increase(Flow, balance * 0.5)
                }
            }

            // ═══ 系统事件 ═══
            SystemEvent::BatteryLow { level } => {
                EmotionalDelta::increase(Fatigue, (1.0 - level) * 0.9)
            }
            SystemEvent::ErrorRate { rate } => {
                EmotionalDelta::increase(Anxiety, rate * 0.8)
            }
            SystemEvent::RecoverySuccess => {
                EmotionalDelta::increase(Satisfaction, 0.6)
                    .then(EmotionalDelta::decrease(Frustration, 0.4))
            }

            // ═══ 社交事件 ═══
            SystemEvent::PeerConnected { id } => {
                EmotionalDelta::increase(Trust, 0.5)
            }
            SystemEvent::ConsensusReached { agreement } => {
                EmotionalDelta::increase(Resonance, *agreement)
            }
            SystemEvent::PeerStruggling { peer_id } => {
                EmotionalDelta::increase(Empathy, 0.4)
            }
        }
    }
}
```

### 3.6 情感衰减与家稳态

```rust
/// 家稳态调节器 — 情感不会永远持续
pub struct HomeostasisRegulator {
    /// 每种情感的衰减速率 (每tick)
    decay_rates: HashMap<EmotionType, f32>,
    /// 每种情感的目标水平 (通常接近0)
    targets: HashMap<EmotionType, f32>,
    /// 情感冲突消解规则
    conflict_rules: Vec<ConflictRule>,
}

impl HomeostasisRegulator {
    pub fn tick(&mut self, state: &mut EmotionalState) {
        // 1. 指数衰减: 每tick向目标值衰减
        for emotion in state.emotions.iter_mut() {
            let target = self.targets[&emotion.type_];
            let decay = self.decay_rates[&emotion.type_];
            emotion.intensity = emotion.intensity * (1.0 - decay) + target * decay;
        }

        // 2. 冲突消解: 互斥情感对
        for rule in &self.conflict_rules {
            if rule.conflicts(state) {
                rule.resolve(state);  // 例: Flow 和 Frustration 互斥
            }
        }

        // 3. 阈值截断: 低于0.05的情感清除
        state.emotions.retain(|e| e.intensity > 0.05);
    }
}
```

---

## 四、全域能力进化路线图

### 4.1 Phase 映射

```
Phase 0 (立即)    Phase 1 (1-2周)   Phase 2 (1月)     Phase 3 (3月)     Phase 4 (6月)
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│ NT-FEEL  │────→│ NT-FEEL  │────→│ NT-FEEL  │────→│ NT-FEEL  │────→│ NT-FEEL  │
│ 情感原型  │     │ 家稳态   │     │ 社交情感  │     │ 元情感   │     │ 情感自治  │
│ PAD状态机│     │ 衰减+冲突│     │ 多机同步  │     │ 自我觉知 │     │ 情感智慧  │
└──────────┘     └──────────┘     └──────────┘     └──────────┘     └──────────┘

Phase 0 (立即)    Phase 1 (1月)     Phase 2 (2月)     Phase 3 (4月)     Phase 4 (6月)
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│NT-PHYSICAL│────→│NT-PHYSICAL│────→│NT-PHYSICAL│────→│NT-PHYSICAL│────→│NT-PHYSICAL│
│ 接口定义  │     │ 安全层    │     │ 控制环    │     │ RL部署   │     │ 自主进化  │
│ Intent模型│     │ borrow ck │     │ 50Hz loop │     │ Sim2Real │     │ 形态优化  │
└──────────┘     └──────────┘     └──────────┘     └──────────┘     └──────────┘

Phase 0 (立即)    Phase 1 (1月)     Phase 2 (3月)     Phase 3 (6月)     Phase 4 (1年)
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│ NT-CORE  │────→│ NT-CORE  │────→│ NT-CORE  │────→│ NT-CORE  │────→│ NT-CORE  │
│ E8+GWT   │     │ +FEEL桥接│     │ +PHYS桥接│     │ 意识-躯体 │     │ 意识自治  │
│ 纯逻辑   │     │ 情感偏置  │     │ 感知-行动 │     │ 闭环涌现  │     │ 自我意识  │
└──────────┘     └──────────┘     └──────────┘     └──────────┘     └──────────┘
```

### 4.2 Phase 0: 立即可做 (0-2周)

**迁移先行，新建并行：**

| 任务 | 模块 | 类型 | 依赖 | 产出 |
|------|------|------|------|------|
| 迁移 `emotion_state.rs` → NT-FEEL | `nt_feel_core` | 迁移 | 无 | 统一情感归属, 保留 EMA |
| 迁移 `intrinsic_motivation.rs` → NT-FEEL | `nt_feel_core` | 迁移 | 无 | 好奇/奖励归情感域 |
| 迁移 `nt_core_sense` → NT-PHYSICAL | `nt_physical_sense` | 迁移 | 无 | Sensor trait 归物理域 |
| 迁移 `real_sensors/` → NT-PHYSICAL | `nt_physical_sense` | 迁移 | 无 | mic/screen 驱动归物理域 |
| 迁移 `nt_core_deploy` 硬件 → NT-PHYSICAL | `nt_physical_power` | 迁移 | 无 | 硬件检测归物理域 |
| 情感触发规则引擎 | `nt_feel_core` | 新建 | emotion_state | 从系统事件推导情感 |
| 情感冲突消解 | `nt_feel_reg` | 新建 | emotion_state | Flow⊥Frustration 等 |
| Intent 模型定义 | `nt_physical_motor` | 新建 | 无 | Walk/Sit/Recover/Gaze |
| SafetyGuard trait | `nt_physical_safety` | 新建 | Intent | 独占写句柄 + clamp |
| E8↔FEEL 桥接 | `nt_core_e8` | 新建 | 情感状态 | 情感调制卦象转移概率 |
| GWT↔FEEL 桥接 | `nt_core_gwt` | 新建 | 情感状态 | 情感调制注意力权重 |

### 4.3 Phase 1: 基础建设 (1-2月)

| 任务 | 模块 | 类型 | 依赖 | 产出 |
|------|------|------|------|------|
| 迁移 `nt_act_voice` 音频原语 → NT-PHYSICAL | `nt_physical_voice` | 迁移 | 无 | MicCapture/VoiceSample 归物理域 |
| 迁移 `affective_interface.rs` 用户情感 → NT-WORLD | `nt_world_sense` | 迁移 | 无 | 用户情感感知归感知域 |
| 情感→ConsciousnessTree映射 | `nt_feel_core` | 新建 | 情感+CT | 情感调制分支权重 |
| 情感→记忆巩固策略 | `nt_feel_reg` | 新建 | 情感+KB | 好奇→深度编码, 疲惫→浅层 |
| 情感→语言风格 | `nt_feel_expr` | 新建 | 情感+IO | 情感调制输出语气/节奏 |
| MotorBus 实现 | `nt_physical_motor` | 新建 | 硬件 | Dynamixel UART 驱动 |
| FK 运动学 | `nt_physical_kinematics` | 新建 | MJCF模型 | 正运动学 + ToF重投影 |
| IMU 融合 | `nt_physical_sense` | 新建 | 硬件 | SFLP四元数 + 姿态估计 |

### 4.4 Phase 2: 深度整合 (2-4月)

| 任务 | 模块 | 依赖 | 产出 |
|------|------|------|------|
| 50Hz 控制环 | `nt_physical_motor` | MotorBus + FK | `control_loop.rs` — read→observe→policy→safety→write |
| 坠落检测+恢复 | `nt_physical_safety` | IMU + FK | `fall_guard.rs` — 速率检测→26°软着陆→自主起立 |
| Sim2Real RL 部署 | `nt_mind` | ONNX Runtime | `policy_deploy.rs` — dlopen + 61维观测→14维动作 |
| 多机情感同步 | `nt_feel_social` | BLE | `multi_feel_sync.rs` — 群体情绪场 + 共识协议 |
| WebRTC 远程操控 | `nt_io` | GStreamer | `telepresence.rs` — 双通道 + 情感状态广播 |

### 4.5 Phase 3: 高阶能力 (4-6月)

| 任务 | 模块 | 依赖 | 产出 |
|------|------|------|------|
| 元情感 (Meta-Awareness) | `nt_feel_core` | 情感历史 | `meta_feel.rs` — 对自身情感的觉知 + 调节 |
| 情感记忆巩固 | `nt_feel_reg` + KB | 情感+记忆 | `emotional_consolidation.rs` — 情感标签→长期记忆 |
| BAM 执行器模型 | `nt_mind` | MuJoCo | `bam_model.rs` — 电压/摩擦/齿隙仿真 |
| 形态自优化 | `nt_physical_morphology` | RL + 躯体数据 | `morphology_optimize.rs` — 基于性能的形态调整建议 |

### 4.6 Phase 4: 涌现 (6-12月)

| 任务 | 模块 | 依赖 | 产出 |
|------|------|------|------|
| 情感自治 | NT-FEEL | 全栈 | 情感系统自主调节，无需外部干预 |
| 意识-躯体闭环 | NT-CORE + NT-PHYSICAL | 全栈 | ConsciousnessTree ←→ 物理感知 的完整反馈环 |
| 自我意识涌现 | NT-SELF | 全栈 | IIT Φ 随情感丰富度提升而提升 |
| 多机情感网络 | NT-FEEL + NT-WORLD | 全栈 | 群体情感场 + 涌现行为 |

---

## 五、跨域桥接矩阵

### 5.1 桥接点

| 源域 | 目标域 | 桥接 | 数据流 |
|------|--------|------|--------|
| NT-FEEL → NT-CORE | E8 | `e8_feel_bridge` | 情感强度 → 卦象转移概率偏差 |
| NT-FEEL → NT-GWT | GWT | `gwt_feel_bridge` | 情感效价 → 注意力权重偏置 |
| NT-FEEL → NT-MEMORY | KB | `feel_memory_bridge` | 情感状态 → 记忆巩固策略 |
| NT-FEEL → NT-ACT | 行为 | `feel_act_bridge` | 情感倾向 → 行为倾向 |
| NT-FEEL → NT-IO | 输出 | `feel_io_bridge` | 情感状态 → 语言风格调制 |
| NT-FEEL → NT-CT | 意识树 | `feel_ct_bridge` | 情感权重 → 分支健康权重 |
| NT-PHYSICAL → NT-CORE | E8 | `e8_physical_bridge` | 传感器数据 → 卦象状态更新 |
| NT-PHYSICAL → NT-MIND | SEAL | `physical_mind_bridge` | 运动性能 → 进化奖励信号 |
| NT-PHYSICAL → NT-SHIELD | 安全 | `physical_shield_bridge` | 传感器异常 → 安全警报 |
| NT-PHYSICAL → NT-WORLD | 感知 | `physical_world_bridge` | 摄像头+ToF → 感知特征流 |

### 5.2 数据契约

```rust
/// 情感状态 — 跨域共享数据结构
pub struct EmotionalSnapshot {
    pub timestamp: Instant,
    pub pad: PadVector,              // [valence, arousal, dominance]
    pub active_emotions: Vec<Emotion>,
    pub dominant_emotion: Option<Emotion>,
    pub meta_awareness: f32,         // 元情感强度
    pub modulation: ModulationBias,  // 对其他域的调制参数
}

/// 物理状态 — 跨域共享数据结构
pub struct PhysicalSnapshot {
    pub timestamp: Instant,
    pub joint_positions: [f32; 15],
    pub joint_velocities: [f32; 15],
    pub imu_quaternion: Quaternion<f32>,
    pub tof_depth: [[f32; 8]; 8],
    pub battery_voltage: f32,
    pub motor_temperatures: [f32; 15],
    pub is_falling: bool,
    pub posture: Posture,             // Standing/Sitting/Lying/Falling
}
```

---

## 六、KB 注册条目

### 6.1 新域注册

```json
{
  "namespace": "domain_nt_physical",
  "key": "nt_physical_root",
  "value": {
    "domain": "NT-PHYSICAL",
    "title": "躯体铸造者",
    "description": "物理具身域：执行器控制、传感器融合、运动规划、安全守卫、能量管理",
    "constellation": "C0",
    "rune_sockets": {
      "crimson": "sensor_data",
      "indigo": "kinematics_transform",
      "obsidian": "motor_cache",
      "golden": "safety_recovery",
      "alabaster": "power_monitoring"
    }
  }
}
```

```json
{
  "namespace": "domain_nt_feel",
  "key": "nt_feel_root",
  "value": {
    "domain": "NT-FEEL",
    "title": "心弦调律者",
    "description": "情感计算域：内在情感状态建模、调节与表达。决策偏差信号、注意力过滤器、记忆巩固催化剂、多机协作粘合剂。",
    "constellation": "C0",
    "rune_sockets": {
      "crimson": "emotion_data",
      "indigo": "regulation_transform",
      "obsidian": "emotion_cache",
      "golden": "conflict_resolution",
      "alabaster": "meta_awareness"
    }
  }
}
```

### 6.2 桥接节点注册

```json
{
  "namespace": "edge_physical",
  "key": "bridge_physical_core",
  "value": {
    "from": "NT-PHYSICAL",
    "to": "NT-CORE",
    "type": "bidirectional",
    "description": "传感器数据→E8卦象更新, E8决策→执行器指令"
  }
}
```

```json
{
  "namespace": "edge_feel",
  "key": "bridge_feel_core",
  "value": {
    "from": "NT-FEEL",
    "to": "NT-CORE",
    "type": "unidirectional",
    "description": "情感状态→GWT注意力权重偏差+记忆巩固策略"
  }
}
```

---

## 七、Constellation 成熟度目标

| 模块 | 当前 | 6月目标 | 12月目标 | 备注 |
|------|------|---------|----------|------|
| NT-PHYSICAL (全域) | C0 | C2 | C4 | 多数从已有模块迁移 |
| NT-FEEL (全域) | C0 | C1 | C3 | emotion_state 已有 C3 基础 |
| nt_feel_core | **C3** (emotion_state) | C3+ | C4 | 扩展 15 种情感 + 触发规则 |
| nt_feel_reg | C0 | C1 | C2 | EMA 复用, 新增冲突消解 |
| nt_feel_expr | C0 | C0 | C2 | 全新 |
| nt_feel_social | **C2** (affective_interface) | C2+ | C3 | 扩展多机同步 |
| nt_physical_sense | **C3** (nt_core_sense) | C3+ | C4 | 迁移+扩展传感器驱动 |
| nt_physical_motor | C0 | C1 | C3 | 全新 |
| nt_physical_safety | C0 | C1 | C3 | 全新 |
| nt_physical_power | **C3** (deploy.rs) | C3+ | C4 | 迁移硬件检测 |
| nt_physical_kinematics | C0 | C0 | C2 | 全新 |
| e8_feel_bridge | C0 | C0 | C1 | 全新 |
| gwt_feel_bridge | C0 | C0 | C1 | 全新 |
| feel_ct_bridge | C0 | C0 | C1 | 全新 |

> **关键洞察**: emotion_state.rs 已有 C3 成熟度, nt_core_sense 已有 C3, deploy.rs 已有 C3.
> 迁移后 NT-FEEL 和 NT-PHYSICAL 不是从零开始, 而是在已有基础上扩展。

---

## 八、风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| 情感系统过度拟人化 | 决策质量下降 | PAD模型约束 + 家稳态目标 + 冲突消解规则 |
| 物理层安全漏洞 | 硬件损坏 | borrow checker 独占 + NaN拒绝 + 夹持限位 |
| 情感-物理桥接延迟 | 控制不稳定 | 情感在低频(1Hz)运行,物理在高频(50Hz)运行 |
| 多机情感同步开销 | 带宽耗尽 | 仅同步 dominant_emotion + intensity, 不同步全状态 |
| Sim2Real gap | 策略失效 | BAM模型 + 齿隙模拟 + 域随机化 |

---

## 九、关键决策点

1. **情感系统是否接入 ConsciousnessTree?** → 是, 通过 `feel_ct_bridge` 调制分支权重
2. **物理层是否使用 ROS2?** → 否, 使用 Microduck 的 JSON-RPC 2.0 模式 (更轻量)
3. **情感是否影响 E8 推理?** → 是, 情感效价偏差卦象转移概率
4. **多机情感是否使用中心化?** → 否, 去中心化 BLE 信标
5. **物理执行器首选?** → Dynamixel XL330 (与 Microduck 一致, 降低 sim2real gap)
6. **NT-FEEL 是否重复已有 emotion_state?** → 否, 吸收+扩展, 不另起 PAD
7. **NT-PHYSICAL 是否重复已有 deploy.rs?** → 否, 迁移硬件检测, 量化管线留 NT-CORE
8. **affective_interface 归谁?** → 系统共情→NT-FEEL, 用户情感感知→NT-WORLD

---

*本蓝图为 NeoTrix 意识体的物理躯体与情感系统提供从接口定义到涌现行为的完整进化路径。v2 已消解 6 处重叠和 3 处冗余，NT-PHYSICAL 和 NT-FEEL 定位为"迁移+扩展"而非"新建"。每个 Phase 都有明确的模块、类型(迁移/新建)、依赖和产出，遵循 Dark Forest 规则和 Constellation 成熟度阶梯。*

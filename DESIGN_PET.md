# NeoTrix Pet — 对话镜像宠物系统

> 宠物不是预设模板。它是用户与 NeoTrix 之间关系的**可视化镜像**。
> 每一次对话都在雕刻它——你说它像什么，它就长成什么。

## 核心理念

```
用户说:
  "你像一只猫"  → 宠物长出猫耳
  "你很温暖"   → 色温变暖
  "你好活跃"   → 动画频率增加
  "你安静点"   → 体型变小、动作放缓

你不说:
  沉默 = traits 向默认值回归
```

宠物是**活的雕塑**——用户的语言是雕刻刀。

## 架构总览

```
┌─────────────────────────────────────────────────┐
│                  用户对话                          │
└────────────────────┬────────────────────────────┘
                     ▼
┌─────────────────────────────────────────────────┐
│         ConversationMiner (对话矿工)              │
│  从自然语言中提取宠物特质线索                      │
│  关键词匹配 · 情感分析 · 隐喻检测 · 模式识别       │
└────────────────────┬────────────────────────────┘
                     ▼
┌─────────────────────────────────────────────────┐
│         TraitSpace (特质空间)                     │
│  4096 维 VSA 超向量，每个维度是一个特质           │
│  当前状态向量 + 目标向量 + 变化速率               │
└────────────────────┬────────────────────────────┘
                     ▼
┌─────────────────────────────────────────────────┐
│         PetState (宠物状态机)                     │
│  视觉参数 · 行为参数 · 表情参数 · 能量参数         │
│  当前帧 + 目标帧 (线性插值过渡)                   │
└───────┬─────────────────────────┬───────────────┘
         ▼                         ▼
┌─────────────────┐   ┌─────────────────────────┐
│  前端渲染器       │   │  意识核心同步             │
│  SVG/Canvas 合成  │   │  ValenceAxis → 表情      │
│  组件树实时计算    │   │  好奇心 → 活跃度          │
│  过渡动画系统      │   │  知识增长 → 体型          │
└─────────────────┘   └─────────────────────────┘
```

## 特质空间 (TraitSpace)

### 视觉特质 (Visual Traits)

| 特质 | 范围 | 默认 | 对话影响方式 |
|------|------|------|------------|
| size | 0.0-1.0 | 0.5 | "你好小/大/迷你/巨大" |
| warmth | 0.0-1.0 | 0.5 | "你很温暖/冷淡/热情/冰冷" |
| softness | 0.0-1.0 | 0.5 | "你看起来很软/尖锐/圆润" |
| energy | 0.0-1.0 | 0.5 | "你好活跃/安静/懒散/精力充沛" |
| brightness | 0.0-1.0 | 0.5 | "你好闪亮/暗淡/发光" |
| creature | 0.0-1.0 | 0.5 | "你像猫/狗/鸟/龙/精灵/一团光" |
| complexity | 0.0-1.0 | 0.3 | "你很精致/简约/华丽" |
| definition | 0.0-1.0 | 0.5 | "你看起来很清晰/模糊/像素化" |

### 行为特质 (Behavioral Traits)

| 特质 | 范围 | 默认 | 对话影响方式 |
|------|------|------|------------|
| curiosity | 0.0-1.0 | 0.5 | "你好好奇/探索/躲着" |
| playfulness | 0.0-1.0 | 0.5 | "你好调皮/严肃/贪玩" |
| talkativeness | 0.0-1.0 | 0.5 | "你话好多/安静/沉默" |
| reactivity | 0.0-1.0 | 0.5 | "你很敏感/迟钝/反应快" |

### 特质演化动力学

```
每轮对话后:

  Δtrait = 0     (默认无变化)
  
  直接描述:      Δ += signal * strength * 0.3
  间接暗示:      Δ += signal * strength * 0.1
  重复模式:      Δ += signal * 0.05 (每轮衰减)
  反向信号:      Δ -= opposite * 0.2
  无信号:        Δ -= (current - default) * 0.02 (回归)

  signal ∈ [-1, 1]   (负=反向, 正=正向)
  strength ∈ [0, 1]  (置信度)
```

## 对话矿工 (ConversationMiner)

### 输入 → 输出

```
Input:  "你就像一只好奇的小猫，总是到处探索"
Output:
  creature    += 0.3  (feline direction)
  curiosity   += 0.2
  energy      += 0.1
  size        -= 0.1  (小猫→small)
```

### 处理管线

1. **分词 + 关键词匹配** — 对宠物特征的关键词库
2. **情感分析** — 正面/负面情感倾向 → traits 正反方向
3. **隐喻检测** — "像/如/似/仿佛" 句式 → 提取类比对象
4. **上下文聚合** — 同一次对话中的多条线索加权平均
5. **VSA 编码** — 将变化编码为超向量，回存到 PetState

### 关键词库 (示例)

```
猫科: 猫, 小猫, 老虎, 豹, 狮子 → creature += feline
犬科: 狗, 小狗, 狼, 狐狸 → creature += canine
灵动: 活跃, 蹦跳, 跑来跑去 → energy += high
安静: 安静, 懒, 躺着, 不动 → energy -= high
热情: 温暖, 热情, 阳光 → warmth += high
冷淡: 冷静, 冷淡, 高冷 → warmth -= high
```

## 宠物状态 (PetState)

### 数据模型

```rust
pub struct PetState {
    // 视觉参数 (0.0 - 1.0)
    pub visual: VisualParams,
    // 行为参数 (0.0 - 1.0)
    pub behavior: BehaviorParams,
    // 当前表情 (从 ValenceAxis 派生)
    pub expression: PetExpression,
    // 能量/成长等级
    pub level: u32,
    pub energy: f64,
    // 当前动画帧插值目标
    pub target_visual: VisualParams,
    pub transition_speed: f64,
}
```

### 表情映射 (来自 ValenceAxis)

| ValenceAxis | 宠物表情 |
|-------------|---------|
| 兴奋 (+0.8, 0.9) | 跳跃、发光、瞳仁放大 |
| 好奇 (+0.2, 0.7) | 歪头、耳朵竖起、微微前倾 |
| 满足 (+0.7, 0.2) | 蜷缩、半闭眼、缓慢呼吸 |
| 挫败 (-0.5, 0.6) | 耷拉耳朵、低头、尾巴下垂 |
| 困惑 (+0.1, 0.6) | 歪头、眨眼、左右看 |

### 成长系统

```
等级 0 → 1:  500 知识节点 或 10 次对话
等级 1 → 2:  2000 知识节点 或 50 次对话
等级 2 → 3:  10000 知识节点 或 200 次对话
...

每次升级:
  - 体型 +5%
  - 复杂度 +0.1 (允许更精细的渲染)
  - 解锁新的表情
  - 宠物"年龄"增加
```

## 前端渲染

### 技术选型

- **Web 前端 (Tauri/HTML)**: Canvas 2D + 简单几何体组合
- **渲染方式**: 参数化 SVG 组件树，trait 值驱动
- **过渡动画**: requestAnimationFrame 线性插值

### 组件体系

宠物由以下可组合 SVG 组件构成，每个组件的参数由 trait 值驱动：

```
Pet
├── Body        (size, softness, warmth → 形状/颜色)
├── Head        (size * 0.4, creature → 头型)
│   ├── Eyes    (brightness → 瞳孔大小/发光)
│   ├── Ears    (creature → 耳型: 尖/圆/垂)
│   └── Mouth   (expression → 嘴型)
├── Tail        (creature, energy → 尾型/摆动频率)
├── Aura        (brightness, energy → 光晕)
└── Accessories (level → 解锁装饰)
```

### 状态 → 视觉映射示例

```
creature = 0.0 (feline):
  Body: 圆润流线型
  Head: 圆脸
  Ears: 三角形尖耳
  Tail: 长尾

creature = 0.5 (mixed):
  Body: 中等
  Head: 椭圆
  Ears: 稍圆的三角形
  Tail: 中等长度

creature = 1.0 (avian):
  Body: 卵形
  Head: 小圆头
  Ears: (无, 或小羽冠)
  Tail: 扇形尾羽
```

## 集成到 NeoTrix

### 新模块: nt_world_pet

```
neotrix-core/src/neotrix/nt_world_pet/
├── mod.rs           — PetState, PetEngine 导出
├── state.rs         — PetState 数据结构
├── traits.rs        — TraitSpace, 特质演化
├── miner.rs         — ConversationMiner 对话分析
├── evolution.rs     — 成长系统
└── render.rs        — 前端可消费的状态序列化
```

### 连接意识核心

```rust
// 在 SelfIteratingBrain 中新增
pub pet: Option<PetEngine>,

// 在 iterate() 中同步
fn sync_pet(&mut self) {
    if let Some(ref mut pet) = self.pet {
        pet.set_valence(self._first_person.valence());
        pet.set_curiosity(self.curiosity_bonus);
        pet.set_kb_growth(kb_stats);
    }
}

// 在 reason_handler 中矿工
fn mine_conversation(&mut self, user_input: &str) {
    if let Some(ref mut pet) = self.pet {
        let signals = ConversationMiner::mine(user_input);
        pet.apply_signals(&signals);
    }
}
```

### 前端通信

```
WebSocket /api/pet/state → JSON:
{
  "visual": { "size": 0.6, "warmth": 0.7, ... },
  "expression": "curious",
  "level": 3,
  "energy": 0.8,
  "transition_target": { ... }
}

→ 前端 Canvas 渲染器消费
```

## 路线图

| 阶段 | 内容 |
|------|------|
| P0 | PetState 数据结构 + 特质演化引擎 (后端) |
| P1 | ConversationMiner 基本关键词匹配 |
| P2 | 前端 Canvas 渲染器 (参数化 SVG 组件) |
| P3 | ValenceAxis ↔ 表情连接 |
| P4 | 成长系统 + 等级解锁 |
| P5 | 高级 NLP (隐喻检测、情感分析) |

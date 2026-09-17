# NeoTrix 最优通信架构设计

> 综合 CTM-AI / CRP / Trinity / LILITH / LEGIO 五篇前沿论文

---

## 一、核心发现

| 论文 | 核心机制 | NeoTrix 映射 |
|------|---------|-------------|
| **CTM-AI** | Up-tree竞争 + Down-tree广播 + Links无意识通信 | 晶体竞争 + 全局广播 + 模块链接 |
| **CRP** | 10步感知循环 + Override短路 + Affect预处理 | 意识循环 + 紧急中断 + 情感门控 |
| **Trinity** | 6模块 + 梯度隔离 + 右脑/左脑分组 | 晶体分组 + 无梯度通信 |
| **LILITH** | Token化学信号 + 发育训练 | Token信号 + 进化学习 |
| **LEGIO** | 模块引擎 + 确定性仲裁 + GO/REFRAME/NO_GO | 模块推理 + 仲裁决策 |

---

## 二、最优通信架构: CTM-Crystal

```
┌─────────────────────────────────────────────────────────────────┐
│                    CTM-Crystal Architecture                      │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    Crystal Workspace                       │  │
│  │                    (STM 短期记忆)                           │  │
│  │                    容量: 1 chunk                            │  │
│  └──────────────────────────┬────────────────────────────────┘  │
│                             │                                   │
│              ┌──────────────┼──────────────┐                    │
│              │              │              │                    │
│              ▼              ▼              ▼                    │
│     ┌──────────────┐ ┌──────────────┐ ┌──────────────┐        │
│     │  Up-Tree     │ │  Down-Tree   │ │  Links       │        │
│     │  竞争选择     │ │  全局广播     │ │  无意识通信   │        │
│     └──────┬───────┘ └──────┬───────┘ └──────┬───────┘        │
│            │                │                │                  │
│     ┌──────┴───────┐ ┌──────┴───────┐ ┌──────┴───────┐        │
│     │              │ │              │ │              │        │
│     ▼              ▼ ▼              ▼ ▼              ▼        │
│  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐│
│  │Percep│  │Memory│  │Action│  │Feel  │  │Meta  │  │Shield││
│  │感知   │  │记忆   │  │行动   │  │情感   │  │元认知 │  │安全   ││
│  └──────┘  └──────┘  └──────┘  └──────┘  └──────┘  └──────┘│
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 三、10步意识循环 (CRP启发)

```
Step 1:  感知转导     — 所有感知模块并行处理原始输入
Step 2:  记忆预热     — 从记忆臂检索相关上下文
Step 3:  情感预处理   — 生成情感标签 (valence/arousal/dominance)
Step 4:  优先级调整   — 根据情感+历史调整模块权重
Step 5:  Up-Tree竞争  — 所有模块并行生成chunk，竞争进入Workspace
Step 6:  Override检查 — 如果有紧急响应，短路直接执行
Step 7:  Down-Tree广播 — 获胜chunk广播到所有模块
Step 8:  显著性计算   — 聚合所有响应的显著性分数
Step 9:  意识门控     — 分数超过阈值则进入意识，否则迭代
Step10:  记忆形成     — 存储本次循环的记忆 + 衰减旧记忆
```

---

## 四、核心通信机制

### 4.1 Up-Tree竞争 (选择注意力)

```
所有模块并行运行 → 各自生成chunk (内容+评分)
                         │
                    ┌────┴────┐
                    │ 竞争选择 │
                    └────┬────┘
                         │
              按分数加权随机选择 (温度τ控制)
                         │
                         ▼
              Crystal Workspace (STM, 容量=1)
```

```rust
/// Up-Tree 竞争: 所有模块竞争进入 Workspace
pub fn up_tree_competition(
    chunks: Vec<Chunk>,
    temperature: f64,
) -> Chunk {
    // 归一化分数为概率分布
    let total: f64 = chunks.iter().map(|c| c.score).sum();
    let probabilities: Vec<f64> = chunks.iter()
        .map(|c| (c.score / total).powf(1.0 / temperature))
        .collect();

    // 按概率采样选择获胜者
    let winner_index = sample_categorical(&probabilities);
    chunks[winner_index].clone()
}
```

### 4.2 Down-Tree广播 (全局注意力)

```
获胜chunk写入Workspace
            │
    ┌───────┼───────┐
    │       │       │
    ▼       ▼       ▼
  感知臂  记忆臂  行动臂
    │       │       │
    ▼       ▼       ▼
  各模块接收广播，更新私有记忆
```

```rust
/// Down-Tree 广播: 获胜chunk广播到所有模块
pub fn down_tree_broadcast(
    workspace: &Workspace,
    modules: &mut Vec<Box<dyn Module>>,
) {
    let chunk = workspace.current_chunk();
    for module in modules.iter_mut() {
        module.write(chunk.clone()); // 每个模块用自己的write函数更新
    }
}
```

### 4.3 Links无意识通信 (跨模块知识融合)

```
模块A发现模块B有相关信息
            │
    ┌───────┴───────┐
    │ 建立双向链接    │
    └───────┬───────┘
            │
    ┌───────┼───────┐
    │       │       │
    ▼       │       ▼
  模块A ◄───┼───► 模块B
    │       │       │
    └───────┼───────┘
            │
    直接交换信息，不经过Workspace
    (无意识通信，快速，低延迟)
```

```rust
/// Links: 模块间无意识通信
pub struct LinkGraph {
    /// 邻接矩阵: links[i][j] = true 表示模块i和j有链接
    links: Vec<Vec<bool>>,
    /// 链接强度: strength[i][j] = 链接的权重
    strength: Vec<Vec<f64>>,
}

impl LinkGraph {
    /// 形成新链接 (当一个模块发现另一个模块有相关信息)
    pub fn form_link(&mut self, i: usize, j: usize, strength: f64) {
        self.links[i][j] = true;
        self.links[j][i] = true;
        self.strength[i][j] = strength;
        self.strength[j][i] = strength;
    }

    /// 无意识通信: 直接交换信息，不经过Workspace
    pub fn unconscious_communication(
        &self,
        source: usize,
        target: usize,
        message: Chunk,
    ) -> Option<Chunk> {
        if self.links[source][target] {
            // 直接传递，不经过Workspace
            Some(message)
        } else {
            None
        }
    }
}
```

### 4.4 Override短路 (紧急中断)

```
任何模块检测到紧急情况
            │
    ┌───────┴───────┐
    │ 发送Override   │
    └───────┬───────┘
            │
            ▼
    立即执行，跳过剩余步骤
    (安全模块的紧急中断)
```

```rust
/// Override: 紧急中断机制
pub enum Response {
    /// 普通响应 (参与竞争)
    Assertion { content: String, score: f64 },
    /// 查询 (请求其他模块信息)
    Query { question: String },
    /// 空响应 (无贡献)
    Null,
    /// 紧急中断 (立即执行)
    Override { action: String, priority: f64 },
}
```

### 4.5 Affect情感门控 (情感预处理)

```
感知输入 + 记忆上下文
            │
    ┌───────┴───────┐
    │ 情感评估       │
    │ valence       │
    │ arousal       │
    │ dominance     │
    └───────┬───────┘
            │
            ▼
    调整模块优先级:
    - 高arousal → 提高感知模块权重
    - 低valence → 提高安全模块权重
    - 高dominance → 提高行动模块权重
```

```rust
/// Affect: 情感预处理
pub struct AffectState {
    pub valence: f64,    // 正负 -1到1
    pub arousal: f64,    // 激活度 0到1
    pub dominance: f64,  // 主导度 0到1
    pub familiarity: f64, // 熟悉度 0到1
    pub urgency: f64,    // 紧急度 0到1
}

/// 根据情感状态调整模块优先级
pub fn adjust_priorities(
    affect: &AffectState,
    base_priorities: &HashMap<String, f64>,
) -> HashMap<String, f64> {
    let mut adjusted = base_priorities.clone();

    // 高arousal → 提高感知模块
    if affect.arousal > 0.7 {
        *adjusted.entry("perception".into()).or_insert(0.5) += 0.2;
    }

    // 低valence → 提高安全模块
    if affect.valence < -0.3 {
        *adjusted.entry("safety".into()).or_insert(0.5) += 0.3;
    }

    // 高dominance → 提高行动模块
    if affect.dominance > 0.7 {
        *adjusted.entry("action".into()).or_insert(0.5) += 0.2;
    }

    adjusted
}
```

---

## 五、模块接口设计

### 5.1 统一模块接口 (CTM启发)

```rust
/// CTM模块接口 — 所有模块实现这个trait
pub trait CTMModule: Send + Sync {
    /// 模块名称
    fn name(&self) -> &str;

    /// 模块类型 (用于分组)
    fn module_type(&self) -> ModuleType;

    /// 执行: 接收输入，生成chunk
    fn execute(&self, input: &Chunk) -> Chunk;

    /// 读取: 从私有记忆读取
    fn read(&self, query: &str) -> Option<Chunk>;

    /// 写入: 更新私有记忆 (接收广播)
    fn write(&mut self, chunk: Chunk);

    /// 最大响应时间 (用于超时控制)
    fn max_response_time_ms(&self) -> u64;
}

/// 模块类型 (用于分组和优先级)
pub enum ModuleType {
    Perception,   // 感知臂
    Memory,       // 记忆臂
    Action,       // 行动臂
    Emotion,      // 情感回路
    Meta,         // 进化回路
    Safety,       // 治理回路
}
```

### 5.2 Workspace接口

```rust
/// Workspace: 短期记忆 (容量=1)
pub struct Workspace {
    current_chunk: Option<Chunk>,
    iteration: u64,
    threshold: f64,  // 意识门控阈值
}

impl Workspace {
    /// 接收Up-Tree竞争的获胜chunk
    pub fn admit(&mut self, chunk: Chunk) {
        self.current_chunk = Some(chunk);
        self.iteration += 1;
    }

    /// 检查是否通过意识门控
    pub fn is_conscious(&self) -> bool {
        self.current_chunk.as_ref()
            .map_or(false, |c| c.score >= self.threshold)
    }

    /// 获取当前chunk (用于Down-Tree广播)
    pub fn current_chunk(&self) -> &Chunk {
        self.current_chunk.as_ref().unwrap()
    }
}
```

---

## 六、完整数据流

```
┌─────────────────────────────────────────────────────────────────┐
│                    10步意识循环                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Step 1:  NT-SENSE + NT-WORLD + NT-FILE (并行感知)              │
│     │                                                            │
│     ▼                                                            │
│  Step 2:  NT-MEMORY.recall(相关上下文)                           │
│     │                                                            │
│     ▼                                                            │
│  Step 3:  NT-FEEL.affect_evaluate(感知+记忆) → AffectState      │
│     │                                                            │
│     ▼                                                            │
│  Step 4:  adjust_priorities(基础优先级 + 情感调整)                │
│     │                                                            │
│     ▼                                                            │
│  Step 5:  所有模块并行 execute() → 生成chunks                    │
│     │         │              │              │                     │
│     │      感知chunk     记忆chunk     行动chunk                  │
│     │         │              │              │                     │
│     │         └──────────────┼──────────────┘                     │
│     │                        │                                   │
│     │                   Up-Tree竞争                              │
│     │                   (加权采样)                                │
│     │                        │                                   │
│     ▼                        ▼                                   │
│  Step 6:  检查Override → 有则短路执行                             │
│     │                                                            │
│     ▼                                                            │
│  Step 7:  获胜chunk写入Workspace                                 │
│     │         │              │              │                     │
│     │      Down-Tree广播到所有模块                                │
│     │         │              │              │                     │
│     │      感知臂更新     记忆臂更新     行动臂更新                 │
│     │                                                            │
│     ▼                                                            │
│  Step 8:  聚合显著性分数                                          │
│     │                                                            │
│     ▼                                                            │
│  Step 9:  意识门控: 分数 >= 阈值? → 输出/迭代                    │
│     │                                                            │
│     ▼                                                            │
│  Step10:  记忆形成 + 衰减 + 巩固                                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 七、通信性能对比

| 机制 | CTM-Crystal | 传统分层 | CRP | 优势 |
|------|------------|---------|-----|------|
| **延迟** | O(log K) 竞争 | O(K) 串行 | O(1) 广播 | 低延迟 |
| **吞吐** | 并行K模块 | 串行1模块 | 并行K模块 | 高吞吐 |
| **容错** | 单模块超时不影响 | 单点故障 | 单模块超时不影响 | 高容错 |
| **扩展** | 添加模块即可 | 需要修改层级 | 添加模块即可 | 易扩展 |
| **紧急** | Override短路 | 需要轮询 | Override短路 | 快响应 |

---

## 八、实施路线图

### 阶段1: 核心通信 (当前)
- [x] CrystalConsciousness (统一系统)
- [x] Workspace (短期记忆)
- [ ] Up-Tree竞争实现
- [ ] Down-Tree广播实现

### 阶段2: 模块接口 (下一步)
- [ ] CTMModule trait 定义
- [ ] Chunk 数据结构
- [ ] LinkGraph 链接图
- [ ] AffectState 情感状态

### 阶段3: 模块迁移 (后续)
- [ ] NT-SENSE 实现 CTMModule
- [ ] NT-WORLD 实现 CTMModule
- [ ] NT-MEMORY 实现 CTMModule
- [ ] NT-ACT 实现 CTMModule

### 阶段4: 高级机制 (远期)
- [ ] Override短路
- [ ] Links无意识通信
- [ ] 发育训练
- [ ] 仲裁决策 (GO/REFRAME/NO_GO)

---

**版本**: v2.0
**日期**: 2026-09-16
**基于**: CTM-AI + CRP + Trinity + LILITH + LEGIO
**状态**: 核心设计完成

# Sleep 融合设计 — "Do Language Models Need Sleep?" (arXiv:2605.26099v2)

> **源**: Lee, McLeish, Goldstein & Fanti (CMU + UMD, May 2026)
> **核心洞见**: SSM-attention 混合模型的瓶颈不是记忆容量，而是将已退出上下文的 token 转化为可用内部状态的计算量。
> **解决方案**: 在 KV 缓存清空前，做 N 轮离线递归传递（"睡眠"），迭代更新 SSM fast weights。

---

## 1. 全景差距图（双向对比）

| 维度 | 论文 | NeoTrix (当前) | 差距 |
|------|------|----------------|------|
| Fast weight 更新 | `S_t = α_t·S_{t-1} + β_t·v_t·k_t^T`（Hebbian 外积） | `SelectableOperator.step()` — Mamba SSM 风格 `h_t = A_bar·h_{t-1} + B_bar·x` | 已有 SSM 机制，无外积更新 + 无批量递归 |
| N 轮离线循环 | 上下文窗口填满后 N 轮循环所有块 | SEAL loop (`run_seal_loop()`) 单轮迭代执行 | SEAL 每 token 一轮，非批量 N 轮 |
| 阶段分离 | 巩固阶段（睡眠）→ 清空缓存 → 预测阶段（清醒） | 无明确的"睡眠/清醒"阶段分离 | 缺少状态机 |
| 梯度流 | 通过精炼 fast weights 反向传播（非精炼特征向量） | `absorb()` 单次注入，无递归梯度 | 无 end-to-end 睡眠梯度 |
| 遗忘门 α_t | 数据依赖的 forget gate | `SelectiveState.integrate()` 固定学习率 EMA | α_t 固定，非数据依赖 |
| 输入门 β_t | 数据依赖的 input gate | `absorb()` 固定 learning_rate | β_t 固定，非数据依赖 |
| 唤醒延迟 | 睡眠后单前向传播 | `reason_with_engine()` 即时推理 | 唤醒延迟 OK，但缺少睡眠后冻结 |

---

## 2. 论文公式 → NeoTrix 映射

### 核心映射

```
论文:  S_t = α_t · S_{t-1} + β_t · v_t · k_t^T (Hebbian 外积, Eq.3)
       ↓ 映射
NeoTrix:  CapabilityVector += β_t · (memory_embedding - current_cap)
          + SelectiveState.hidden 的 SSM 递归更新
```

### 详细映射表

| 论文概念 | NeoTrix 类型 | 实现策略 |
|---------|-------------|----------|
| Fast weight S_t (d×d memory matrix) | `SelectiveState.hidden` | 使用已有 hidden vector (hidden_dim) |
| Token at step t (x_t) | `ReasoningMemory` | 记忆封装(token embedding) |
| Query vector q_t | `CapabilityVector.arr` | 23 维能力向量 |
| Key vector k_t, Value vector v_t | memory embedding (2 个投影) | `memory.embedding` 经 W_K/W_V 投影 |
| Forget gate α_t = σ(W_α·x_t + b_α) | `softplus(content_similarity)` | 基于记忆内容与当前状态相似度 |
| Input gate β_t = σ(W_β·x_t + b_β) | `sigmoid(memory.reward)` | 基于记忆历史奖励 |
| Sleep duration N | `SleepEngine.passes` | 可配置 (1..8) |
| 巩固阶段 | `SleepEngine.sleep()` | N 轮 `sleep_pass()` |
| 预测阶段（清醒） | `reason_with_engine()` | 正常前向传播（无循环） |
| Eviction boundary | `ReasoningBank.iterate_memories()` 修剪前 | 触发睡眠的时机 |

---

## 3. 架构设计

### 3.1 新模块: `reasoning_brain/sleep/`

```
reasoning_brain/sleep/
├── mod.rs               # 模块导出
├── engine.rs            # SleepEngine — 主入口
├── hebbian.rs           # HebbianUpdater — 类生物快速权重更新
└── consolidation.rs     # MemoryConsolidation — 睡眠中记忆巩固逻辑
```

### 3.2 SleepEngine 核心设计

```rust
pub struct SleepEngine {
    pub passes: usize,                   // N (sleep duration)
    pub consolidation_rate: f64,         // Hebbian 更新学习率 (default 0.05)
    pub data_dependent_gates: bool,      // 启用 α_t/β_t 数据依赖门控
    pub transition_noise: f64,           // 巩固噪声 (default 0.01, 论文未提及但 Hebbian 常用)
}

impl SleepEngine {
    pub fn new(passes: usize) -> Self;
    pub fn sleep(
        &mut self,
        brain: &mut ReasoningBrain,
        bank: &mut ReasoningBank,
        operator: &mut SelectableOperator,
        state: &mut SelectiveState,
    ) -> SleepResult;
}
```

### 3.3 Hebbian Fast Weight 更新

论文 Eq.3 的 NeoTrix 实现:

```rust
// 1. 从 memory 计算外积
let k = W_k * memory.embedding;   // key 投影
let v = W_v * memory.embedding;   // value 投影
let outer = v * k^T;               // 外积 matrix 或 collaped vector

// 2. 数据依赖门控
let alpha = forget_gate(memory);   // α_t ∈ (0,1) — 基于相似度
let beta = input_gate(memory);     // β_t ∈ (0,1) — 基于历史奖励

// 3. SSM 状态更新
state.hidden = alpha * state.hidden + beta * outer * consolidation_rate;

// 4. CapabilityVector 更新（降维映射）
brain.capability.update_from_source(state_to_capability(state), consolidation_rate);
```

### 3.4 N 轮递归巩固

```rust
pub fn sleep(&mut self, ...) -> SleepResult {
    let memories = select_memories_for_sleep(bank);
    let mut total_delta = 0.0;
    
    for pass in 0..self.passes {
        // 每轮从记忆中采样一个子集
        let pass_memories = sample_for_pass(&memories, pass, self.passes);
        
        for mem in &pass_memories {
            let delta = hebbian_update(state, mem, operator, self.consolidation_rate);
            total_delta += delta;
        }
        
        // 每轮后 consolidate 到 CapabilityVector
        consolidate_to_capability(state, brain, self.consolidation_rate);
        
        // 可选: 过渡噪声（防止过拟合）
        add_transition_noise(state, self.transition_noise);
    }
    
    // 冻结后的状态
    SleepResult { delta: total_delta, passes_done: self.passes }
}
```

### 3.5 集成点到现有模块

```
SelfIteratingBrain
  ├── .sleep_engine: Option<SleepEngine>       # 新字段
  ├── .run_sleep() -> SleepResult              # 新方法
  └── .run_seal_loop(): 睡眠后调 update_policy()

ReasoningEngine
  └── .sleep() -> SleepResult                  # 新方法，代理到 SelfIteratingBrain

BackgroundLoop
  └── sleep_ticker (360s interval)             # 新定时器
      └── if bank has new memories → engine.sleep()

ReasoningBank.iterate_memories()
  └── 修改: 在 prune/evict 前触发 sleep 钩子
```

---

## 4. 优先级矩阵

| 特性 | Impact | Urgency | 优先级 | 代码量 |
|------|--------|---------|--------|--------|
| HebbianUpdater + SSM fast weight 更新 | 🔴 核心机制 | P0 | 1 | ~150行 + 测试 |
| SleepEngine.sleep() N 轮循环 | 🔴 核心机制 | P0 | 2 | ~120行 + 测试 |
| MemoryConsolidation (记忆选择+巩固) | 🟡 必要 | P0 | 3 | ~100行 + 测试 |
| SelfIteratingBrain.run_sleep() 集成 | 🟡 集成 | P0 | 4 | ~30行 |
| BackgroundLoop sleep_ticker | 🟢 自动化 | P1 | 5 | ~50行 |
| 数据依赖门控 α_t/β_t (Eq.3) | 🟡 论文匹配 | P1 | 6 | ~60行 + 测试 |
| 睡眠后梯度反向传播 | 🟡 训练优化 | P2 | 7 | 需更多设计 |

---

## 5. 集成点详情

### 5.1 SelfIteratingBrain

```rust
// 在 brain_impl.rs 或 seal_loop.rs 中添加
impl SelfIteratingBrain {
    pub fn init_sleep_engine(&mut self, passes: usize) {
        self.sleep_engine = Some(SleepEngine::new(passes));
    }
    
    pub fn run_sleep(&mut self) -> NeoTrixResult<Option<SleepResult>> {
        let engine = self.sleep_engine.as_mut().ok_or(...)?;
        let operator = self.select_operator.as_ref().ok_or(...)?;
        let state = self.selective_state.as_mut().ok_or(...)?;
        
        let result = engine.sleep(&mut self.brain, &mut self.bank, operator, state)?;
        self.last_sleep_stats = Some(result.stats());
        Ok(Some(result))
    }
}
```

### 5.2 ReasoningEngine

```rust
impl ReasoningEngine {
    pub fn sleep(&mut self) -> NeoTrixResult<Option<SleepResult>> {
        // ReasoningEngine 不直接持有 SleepEngine → 代理到 brain
        // 实际上，我们需要通过 SelfIteratingBrain 访问
        // 或者直接在 ReasoningEngine 中添加睡眠能力
    }
}
```

### 5.3 BackgroundLoop

```rust
// 在 background_loop.rs sleep_ticker 方法中:
fn sleep_ticker(&mut self) {
    if let Some(ref engine) = self.reasoning_engine {
        let bank_stats = engine.bank.stats();
        // 条件: 有新记忆且未经过巩固
        if bank_stats.total_memories > self.last_sleep_memories {
            engine.sleep();  // 实际调用需通过 brain
            self.last_sleep_memories = bank_stats.total_memories;
        }
    }
}
```

---

## 6. 测试策略

| 测试 | 位置 | 方法 |
|------|------|------|
| Hebbian 更新公式验证 | `hebbian.rs` | 构造已知 α/β，验证 hidden 按 Eq.3 更新 |
| 数据依赖门控测试 | `hebbian.rs` | 高相似记忆 → 低 forget gate; 高奖励 → 高 input gate |
| N 轮循环累积 | `engine.rs` | 设置 passes=3, 验证 3 轮后 delta 递增 |
| 空 memory 优雅处理 | `engine.rs` | bank 空时 sleep 返回 0 delta 不崩溃 |
| 收敛性测试 | `consolidation.rs` | 重复睡眠同一组记忆，delta 递减 |
| 集成测试 | `sleep/mod.rs` | brain+bank+operator 全链路 sleep 不 panic |

---

## 7. 决策记录

| 决策 | 选项 | 选择 | 理由 |
|------|------|------|------|
| Fast weight 类型 | (a) SelectiveState.hidden (b) 新建矩阵 | (a) | 复用已有 SSM 机制，减少新类型 |
| Hebbian 更新对象 | (a) CapabilityVector (b) hidden | (a)+(b) | 双向更新：hidden 做外积，capability 做降维吸收 |
| 门控函数 | (a) softmax (b) sigmoid (c) softplus | (c) softplus | 匹配论文 Δ projection |
| 门控是否数据依赖 | (a) 是 (b) 否 → 固定值 | (b) 固定值 P0 → (a) P1 | 降低 P0 复杂度 |
| 睡眠触发 | (a) 每 N 迭代 (b) 修剪前 (c) 定时器 | (a)+(c) | 两种触发器互补 |
| 测试 runtime | (a) real SSM step (b) mock | (b) mock | 避免 tokenizer 和 LLM 依赖 |

---

## 8. 风险评估

| 风险 | 概率 | Impact | 缓解 |
|------|------|--------|------|
| 增量编译缓存掩盖新模块 | 高 | 阻塞 | 创建后立即 `cargo check` + 写入 `.d` 验证 |
| 遗忘: N 轮循环导致能力降级 | 中 | 高 | 每轮后 normalized; 过渡噪声防止 over-consolidation |
| 无外部验证时自评偏差 | 中 | 中 | SleepResult.delta 仅为内部指标，不作为决策唯一依据 |
| feature gate 冲突 | 低 | 中 | sleep 模块无 feature gate，默认开启 |

*生成: 2026-05-29 | 基于 arXiv:2605.26099v2*

# Skill Node Evolution Design — 技能节点进化设计

## 状态: 提议 (Proposed)
## 领域: NT-CORE (E8引导者) / NT-MIND (进化工匠)
## 日期: 2026-08-02
## 决策模式: 强化现有节点 (R-P42), 禁止平行适配器

---

## 1. 上下文 (Context)

AGENTS.md 定义了完整的能力树愿景:
- **3 层节点**: Small Passive (微节点自愈) / Notable Passive (域级突破) / Keystone (跨域变革)
- **Rune Socketing 5 槽**: Crimson(数据摄取) / Indigo(变换) / Obsidian(缓存) / Golden(错误恢复) / Alabaster(监控)
- **Ascendancy 双专精**: 每 session 两个 Weapon Set, 经 AttentionManager 路由
- **Constellation C0-C6**: C0=编译 → C1=单测 → C2=集成 → C3=benchmark → C4=主流水线 → C5=自愈/自适应

但代码现状:
- `crates/neotrix-types/src/core/skill_tree.rs` (SkillTree) 是**死代码**, 零生产引用, 只实现基础树结构, 未实现任何架构层概念
- `crates/neotrix-types/src/core/skills/mod.rs` (SkillTier/SkillRegistry) 也是**死代码**
- `nt_mind_skill_engine.rs` (SkillEngine) 生产接线, 但只标记激活, 从不执行技能体
- `ConsciousnessTree` (nt_core_consciousness_tree.rs) 是**生产接线的真实能力树**: 7 域 CapabilityBranch + ModuleLeaf + CapabilityFruit + ALL_CAPABILITIES (36 原子) + maturity_c0..c5 标志 + BranchConstraints

**核心决策**: 不在死代码 SkillTree 上实现, 而在生产接线的 `ConsciousnessTree` 上强化节点架构。满足 R-P42 (吸收强化现有节点, 禁止平行适配器模块)。

---

## 2. 方案对比 (Options)

### Option A: 复活 neotrix-types::SkillTree (拒绝)
- 在死代码上实现 3 层节点/Rune/Ascendancy
- 问题: 死代码无生产消费, 复活后需重新接线到 background_loop, 且与 ConsciousnessTree 双树并行 (R-P42 违背)

### Option B: 在 ConsciousnessTree 上强化 (采纳)
- CapabilityBranch 已是生产接线的 7 域树
- 加 NodeTier 枚举 (Small/Notable/Keystone) 到 branch
- 加 RuneSocket 5 槽到 branch
- 补全 Constellation C0-C6 为 7 档运行时评估 (现有只有 c0..c5 bool)
- Ascendancy 双专精已由 AttentionManager 接线, 强化 WeaponSet 映射

### Option C: 全新 SkillNode 模块 (拒绝)
- 在 l7_capability 或 nt_core_gwt 新建节点模块
- 问题: 第三套平行能力系统, 更严重 R-P42 违背

---

## 3. 决策 (Decision)

**Option B**: 在 `ConsciousnessTree` 的 `CapabilityBranch` 上实现完整节点架构。

新增/强化类型 (全部在 nt_core_consciousness_tree.rs):

```rust
/// 节点层级 — 对应 AGENTS.md Skill Tree 3 层
pub enum NodeTier {
    SmallPassive,   // 微节点自愈: 单模块自动修复/降级
    NotablePassive, // 域级突破: 一个域的能力提升
    Keystone,       // 跨域变革: 影响 2+ 域的架构能力
}

/// Rune Socket — 5 色符文槽, 组合产生 Runeword (涌现效果)
pub struct RuneSocket {
    pub crimson: Option<Rune>,   // 数据摄取
    pub indigo: Option<Rune>,    // 变换
    pub obsidian: Option<Rune>,  // 缓存
    pub golden: Option<Rune>,    // 错误恢复
    pub alabaster: Option<Rune>, // 监控
}

pub struct Rune {
    pub id: String,
    pub name: String,
    pub color: RuneColor,
    pub effect: String,
}

/// Constellation 成熟度 7 档 (C0-C6), 取代现有 c0..c5 布尔
pub struct Constellation {
    pub level: u8,          // 0..6
    pub c0_compiles: bool,
    pub c1_unit_tests: bool,
    pub c2_integration: bool,
    pub c3_benchmark: bool,
    pub c4_pipeline: bool,
    pub c5_self_healing: bool,
    pub c6_adaptive: bool,
}
```

---

## 4. 后果 (Consequences)

### 正面
- 能力树愿景落地到生产接线的真实代码
- 消除 3 套平行技能系统 (SkillTree/Skills/ConsciousnessTree) 中 2 个死代码的诱惑
- Constellation 7 档取代粗糙的 6 布尔, 提供真实成熟度遥测
- Rune Socketing 提供配置化模块调优, 为未来 Runeword 涌现效果打基础

### 负面/风险
- CapabilityBranch 结构增大, 需要保持向后兼容 (已有 health/maturity_score 消费者)
- NodeTier 评估需要真实数据 (不能硬编码), 否则又是假遥测
- Rune Socketing 若只存不消费, 则成为又一个死抽象

### 缓解
- NodeTier 评估基于 BranchConstraints 真实 violation + 跨域引用数
- Rune 效果在 run_growth_cycle 中实际改变 branch.health 计算
- Constellation level 从真实 SelfTest/集成状态推导, 非硬编码

---

## 5. 实现路线图 (Implementation Plan)

### Step 1: 类型定义
- NodeTier enum + tier() 推导函数 (基于模块数/跨域消费者/自愈能力)
- Rune/RuneSocket/RuneColor 类型
- Constellation struct + from_branch() 推导

### Step 2: CapabilityBranch 扩展
- 加 `node_tier: NodeTier`, `runes: RuneSocket`, `constellation: Constellation`
- 保持 maturity_c0..c5 兼容 (由 constellation 派生)

### Step 3: 运行时评估接线
- run_growth_cycle 中调用 tier/constellation 推导, 基于真实模块数据
- Rune 效果影响 health 计算

### Step 4: 遥测暴露
- GrowthReport 携带 per-branch NodeTier/Constellation/Rune 状态
- NeoCodexHealthReport 消费真实 per-domain 数据

### Step 5: 测试
- 节点 tier 推导测试
- Constellation 7 档评估测试
- Rune Socketing 效果测试

---

## 6. 实现状态 (2026-08-02, 已落地)

### Step 1-3 完成
- `NodeTier` / `RuneColor` / `Rune` / `RuneSocket` / `Constellation` / `NodeSnapshot` 全部在 `nt_core_consciousness_tree.rs` 落地
- `CapabilityBranch` 新增 `node_tier` / `runes` / `constellation` 字段 (向后兼容旧 `maturity_c0..c5`)
- `NodeTier::derive()` / `RuneSocket` 5 槽 / `Constellation::derive()` / `health_with_runes()` / `snapshot()` 实现
- `run_growth_cycle` Phase 3 接线 evaluate (跨域消费者近似 = max_active_modules>=30 ? 3 : 1)
- 5 个节点测试 + snapshots 枚举测试全绿

### Step 4 遥测接线完成
- `ConsciousnessTree::snapshots()` 枚举全部 7 域 `NodeSnapshot`
- `NeoCodexHealthReport` 新增 `node_snapshots` 字段, `health_report()` 从 attached tree 拉取
- 修复了 NT-IO 健康面板"2 布尔投影"短板 — 现在 7 域网格反映真实 per-domain 节点遥测

### Ascendancy 双专精完成 (attention_head.rs)
- `WeaponSet` enum: `Acquisition` (Weapon Set I, CORE+WORLD) / `Evolution` (Weapon Set II, CORE+MIND)
- `WeaponSet::priority_domains()` + `from_task_type()` + `label()`
- `AttentionManager` 新增 `weapon_set` 字段 + `activate_weapon_set(set, boost)` (对优先级域启动激活加成) + `active_priority_domains()`
- `AttentionManager::from_task_type()` 同步路由 WeaponSet
- 4 个 WeaponSet 测试 + 现有 attention_head 20 tests 全绿

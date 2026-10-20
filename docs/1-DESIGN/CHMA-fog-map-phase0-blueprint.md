# CHMA × Fog Map — Phase 0 每细节实现蓝图

## 状态: 已批准实现 (Blueprint for Implementation)
## 领域: NT-CORE (意识树) / NT-IO (遥测) / NT-META (雾测度)
## 日期: 2026-08-05
## 前置: `docs/1-DESIGN/CHMA-fog-map-evolution.md` (v2, 外部验证)

---

## 0. 蓝图目标

把"迷雾地图"从设计文档落成可测度代码：**给每个节点加迷雾浓度(fog_level)，
聚合为全仓加权雾和(weighted_fog_sum)，接入既有成长周期与遥测，让每次进化
都有"雾退散"的量化证据。**

全部改动收敛在**一个文件**：`neotrix-core/src/core/nt_core_consciousness_tree.rs`
（+ 消费端 `nt_io_neocodex.rs` 零侵入，因 NodeSnapshot 序列化自动携带新字段）。
零新增模块，满足 R-P42（强化现有节点）。

---

## 1. 真实代码锚点（已核实）

| 锚点 | 位置 | 现状 |
|------|------|------|
| `NodeSnapshot` 结构 | `nt_core_consciousness_tree.rs:381-388` | branch/tier/constellation_level/rune_filled_slots/runeword/composite_effect |
| `CapabilityBranch` 结构 | `:134-155` | 含 node_tier/runes/constellation + maturity_c0..c5 |
| `NodeTier` 枚举 + weight() | `:164-206` | Small(1.0)/Notable(2.0)/Keystone(3.0) |
| `Constellation::derive()` | `:347-360` | 6 布尔 → level |
| `evaluate_node_tier()` | `:403-405` | 运行时评估（Phase 3 调用） |
| `snapshot()` | `:430-439` | 生成 NodeSnapshot |
| `run_growth_cycle()` Phase 3 | `:970-1017` | 984-985 行调 evaluate_node_tier/evaluate_constellation |
| `snapshots()` | `:1188-1193` | 枚举 7 域 branch → Vec<NodeSnapshot> |
| `GrowthReport` | `:1699-1707` | phase0..phase7 字段 |
| `NeoCodexHealthReport.node_snapshots` | `nt_io_neocodex.rs:2353` | 消费 snapshots()，序列化自动携带新字段 |
| 跨域消费者近似 | `:983` | `constraints.max_active_modules >= 30 → 3 else 1` |

---

## 2. 新类型：FogLevel（迷雾浓度）

### 2.1 定义

```rust
/// 迷雾浓度 — 节点未被生产验证的程度。[0,1]，0=全清晰 1=全雾。
/// 对应 CHMA 轴 3 (§8.3 外部锚点: Martin D + 可达性 + 测试覆盖)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FogLevel {
    /// 生产可达性: 是否被生产路径引用 (false=孤儿/死代码 → 浓雾)
    pub wired: bool,
    /// 消费者数量: 0 = 无消费者 (dead island 信号)
    pub consumer_count: usize,
    /// 测试覆盖: 无 SelfTest = 雾
    pub has_tests: bool,
    /// 聚合浓度 [0,1]
    pub level: f64,
}
```

### 2.2 推导（纯函数，只从真实可得指标 derive，无硬编码）

```rust
impl FogLevel {
    /// 从真实模块指标推导迷雾浓度。
    /// - 未接线 → 基础浓度 0.85 (孤儿)
    /// - 无消费者 → 浓度 +0.10
    /// - 无测试 → 浓度 +0.15
    /// - 已接线 + 有消费者 + 有测试 → 收敛到 0.05 (近乎全清晰)
    pub fn derive(wired: bool, consumer_count: usize, has_tests: bool) -> Self {
        let mut level = 0.0_f64;
        if !wired { level += 0.85; }
        if consumer_count == 0 { level += 0.10; }
        if !has_tests { level += 0.15; }
        if wired && consumer_count > 0 && has_tests { level = 0.05; }
        Self {
            wired,
            consumer_count,
            has_tests,
            level: level.clamp(0.0, 1.0),
        }
    }

    pub fn label(&self) -> &str {
        if self.level <= 0.10 {
            "Clear"
        } else if self.level <= 0.15 {
            "LightFog"
        } else if self.level <= 0.8 {
            "Fog"
        } else {
            "DenseFog"
        }
    }
}
```

### 2.3 设计决策
- **wired 判定数据源**：`CapabilityBranch` 现无 wired 字段——由 `set_branch_health_from_self_tests`
  已隐含（有 SelfTest 结果 = 被接线）；消费者数用 Phase 3 已有的
  `cross_domain_consumers` 近似（:983 已有）。**零新增数据采集管线**，全部复用既存在手数据。
- **为什么不用 Martin D**：D 需要抽象度 A 与扇入 Ca 的精确类统计，当前代码库无此采集；
  用"接线+消费者+测试"三信号是 D 的工程化代理，未来若接入依赖图可替换（蓝图保留扩展点）。

---

## 3. 侵入点清单（最小化）

### 3.1 `CapabilityBranch` 加字段
在 `:154` (constellation 字段后) 加：
```rust
/// 迷雾浓度 (CHMA Phase 0)
pub fog: FogLevel,
```
所有构造点需初始化。查构造点（`ConsciousnessTree::new()` 或 `CapabilityBranch::new()`）。

### 3.2 `NodeSnapshot` 加字段
在 `:388` (composite_effect 后) 加：
```rust
/// 迷雾浓度 [0,1] — CHMA 轴 3
pub fog_level: f64,
/// 迷雾标签: Clear/LightFog/Fog/DenseFog
pub fog_label: String,
```

### 3.3 `snapshot()` 填充
`:430-439` 中 `NodeSnapshot { ... }` 加两字段，来源 `self.fog.level` / `self.fog.label()`。

### 3.4 Phase 3 评估点接线（:984-985 附近）
```rust
branch.evaluate_node_tier(cross_domain_consumers);
branch.evaluate_constellation();
branch.evaluate_fog(cross_domain_consumers > 0, cross_domain_consumers, branch.self_test_count > 0);
```
新增 `CapabilityBranch::evaluate_fog(wired, consumers, has_tests)` 方法：
```rust
pub fn evaluate_fog(&mut self, wired: bool, consumer_count: usize, has_tests: bool) {
    self.fog = FogLevel::derive(wired, consumer_count, has_tests);
}
```

### 3.5 `GrowthReport` 加全局雾摘要
`:1707` (phase7_drift 后) 加：
```rust
/// 全仓加权雾和 (Σ branch.fog.level × NodeTier.weight) — 迷雾地图主量纲
pub weighted_fog_sum: f64,
/// 每域迷雾浓度摘要 (branch → level)
pub fog_by_branch: std::collections::BTreeMap<String, f64>,
```

### 3.6 `ConsciousnessTree::weighted_fog_sum()` 新方法
```rust
/// 全仓加权雾和: 高权 tier 的浓雾贡献更大 (Keystone 浓雾 = 大问题)
pub fn weighted_fog_sum(&self) -> f64 {
    self.branches.values()
        .map(|b| b.fog.level * b.node_tier.weight())
        .sum()
}

/// 每域迷雾摘要
pub fn fog_by_branch(&self) -> std::collections::BTreeMap<String, f64> {
    self.branches.values()
        .map(|b| (b.kind.label().to_string(), b.fog.level))
        .collect()
}
```

### 3.7 `run_growth_cycle` 返回前填充报告
```rust
report.weighted_fog_sum = self.weighted_fog_sum();
report.fog_by_branch = self.fog_by_branch();
```

---

## 4. 数据流（改动后）

```
run_growth_cycle (Phase 3)
  └─ evaluate_fog(wired, consumers, has_tests)   ← 每 branch
       └─ FogLevel::derive → branch.fog
Phase 末
  └─ weighted_fog_sum() + fog_by_branch() → GrowthReport
snapshots()
  └─ branch.snapshot() → NodeSnapshot { fog_level, fog_label }
       └─ NeoCodexHealthReport.node_snapshots (nt_io_neocodex.rs:2353, 自动携带)
```

**遥测零改动**：`NeoCodexHealthReport` 反序列化时新字段自动生效（serde），
健康面板即可显示每域迷雾。这是"雾退散可视化"的最小可行第一步。

---

## 5. 测试计划（每个细节对应断言）

### 5.1 `FogLevel::derive` 单测
| 输入 (wired, consumers, has_tests) | 期望 level | 期望 label |
|---|---|---|
| (false, 0, false) | 1.0 | DenseFog |
| (false, 1, true) | 0.85 | DenseFog |
| (true, 0, true) | 0.10 | Clear |
| (true, 1, false) | 0.15 | LightFog |
| (true, 1, true) | 0.05 | Clear |

### 5.2 `weighted_fog_sum` 单测
- 7 域全 Clear (0.05) + 各 tier 权重 → 期望值 `0.05 × Σweight`
- 手动改一个 Keystone branch fog=1.0 → sum 上升 ≥ 3.0（证明 tier 加权生效）

### 5.3 `snapshot()` 携带 fog 单测
- evaluate_fog 后 snapshot().fog_level == branch.fog.level
- snapshot().fog_label == "Clear"

### 5.4 序列化兼容单测
- NodeSnapshot JSON round-trip 含 fog_level 字段

---

## 6. 风险与缓解

| 风险 | 缓解 |
|------|------|
| CapabilityBranch 构造点遗漏导致编译错 | 编译即捕获（字段必填）；查全部构造点 |
| 误改健康面板 JSON 结构 | serde 新增字段向后兼容（旧消费者忽略未知字段）；新字段有默认值语义 |
| wired 近似不准 | 蓝图明确记录：消费者数复用现有 cross_domain_consumers 近似，未来可换精确依赖图 |
| weighted_fog_sum 初始值偏悲观 | 这是**特性而非缺陷**：真实反映当前"雾浓"状态；随时间退散可见下降曲线 |

---

## 7. 验收标准（Definition of Done）

1. `cargo check --all-targets -p neotrix` 通过
2. `cargo test -p neotrix --lib nt_core_consciousness_tree` 全绿（含新 4+ 测试）
3. `weighted_fog_sum` 首次采样记录为基线（后续每提交对比）
4. 遥测面板可读每域 fog_label（无需前端改动）

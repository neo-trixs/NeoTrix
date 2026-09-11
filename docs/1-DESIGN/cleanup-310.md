# Cleanup #310 — 跨层引用扫描

> 扫描时间: 2026-09-11
> 范围: `neotrix-core/src/` 下 L1-L6 六层
> 排除: `*test*`, `*facade*`, 注释行

## 架构约束

依赖方向: **高层 → 低层** (L6 → L5 → L4 → L3 → L2 → L1)
违反方向: **低层 → 高层** (上行依赖 = 违规)

```
L6 Meta-Cognition  ──  nt_meta + nt_repair + nt_nexus
L5 Cognition       ──  nt_core + nt_mind
L4 Emotion         ──  nt_feel
L3 Embodiment      ──  nt_physical + nt_shield + nt_feel
L2 Perception      ──  nt_world + nt_sense
L1 Action          ──  nt_act + nt_io + nt_memory
```

## 违规汇总 (低层 → 高层 = 上行)

| # | 违规方向 | 文件数 | 引用次数 | 严重度 |
|---|---------|--------|---------|--------|
| V1 | L1 → L3 | 1 | 3 | 中 |
| V2 | L1 → L5 | 1 | 2 | 高 |
| V3 | L2 → L3 | 17 | 75 | 高 (系统性) |
| V4 | L5 → L6 | 6 | 19 | 中 |
| **合计** | | **25** | **99** | |

## 合规依赖 (高层 → 低层 = 允许)

| 方向 | 文件数 | 说明 |
|------|--------|------|
| L2 → L1 | 14 | nt_world 使用 nt_memory (KB/HTTP/crawl) |
| L3 → L1 | 1 | nt_shield_traffic 使用 nt_io_provider |
| L5 → L1 | 29 | nt_mind 使用 nt_memory/nt_io/nt_act |
| L5 → L2 | 2 | nt_mind 使用 nt_world |
| L5 → L3 | 3 | nt_mind 使用 nt_shield |
| L5 → L4 | 1 | 已注释, 不计 |
| L6 → L1 | 2 | nt_meta 使用 nt_memory |
| L6 → L5 | 5 | nt_meta 实现 l5_cognition::traits |

---

## V1: L1 → L3 (1 file, 3 refs)

**违规**: 底层 L1 Action 依赖高层 L3 Embodiment

| 文件 | 引用 |
|------|------|
| `l1_action/nt_io/nt_io_provider/factory.rs:530` | `l3_embodiment::nt_shield::nt_shield::policy::PolicyDecision` |

**类型**: Provider 工厂在选择模型时查询 Shield 策略决策。

**修复建议**: 将 `PolicyDecision` 枚举下沉到 L1，或通过 trait 抽象反向依赖。

---

## V2: L1 → L5 (1 file, 2 refs)

**违规**: 底层 L1 Action 依赖高层 L5 Cognition

| 文件 | 引用 |
|------|------|
| `l1_action/nt_io/nt_io_neocodex/agent.rs:110` | `l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain` |
| `l1_action/nt_io/nt_io_neocodex/agent.rs:233` | `l5_cognition::nt_mind::nt_mind::self_iterating::SelfIteratingBrain` |

**类型**: Neocodex agent 直接持有 L5 的 SelfIteratingBrain 引用。

**修复建议**: 引入 L5 trait, L1 只依赖 trait 而非 concrete type。

---

## V3: L2 → L3 (17 files, 75 refs) — 系统性违规

**违规**: 感知层 L2 系统性依赖具身层 L3 的 EgressRule/EgressPolicy

**违规模式**: 每个 nt_world_* 数据源模块都定义自己的 egress_rule/egress_policy 函数, 直接引用 `l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule` / `EgressPolicy`。

### 受影响文件 (17)

| # | 文件 | 引用类型 |
|---|------|---------|
| 1 | `nt_world/nt_world_urlhaus.rs` | EgressRule/EgressPolicy (6 refs) |
| 2 | `nt_world/nt_world_usgs.rs` | EgressRule/EgressPolicy (4 refs) |
| 3 | `nt_world/nt_world_bgpview.rs` | EgressRule/EgressPolicy (4 refs) |
| 4 | `nt_world/nt_world_gdelt.rs` | EgressRule/EgressPolicy + INTEL_GDELT_HOST (7 refs) |
| 5 | `nt_world/nt_world_ucdp.rs` | EgressRule/EgressPolicy (4 refs) |
| 6 | `nt_world/nt_world_aoi.rs` | EgressRule/EgressPolicy (4 refs) |
| 7 | `nt_world/nt_world_gdacs.rs` | EgressRule/EgressPolicy (4 refs) |
| 8 | `nt_world/nt_world_ofac.rs` | EgressRule/EgressPolicy (4 refs) |
| 9 | `nt_world/nt_world_polymarket.rs` | EgressRule/EgressPolicy (4 refs) |
| 10 | `nt_world/nt_world_opencorporates.rs` | EgressRule/EgressPolicy (4 refs) |
| 11 | `nt_world/nt_world_adsb.rs` | EgressRule/EgressPolicy (4 refs) |
| 12 | `nt_world/nt_world_edgar.rs` | EgressRule/EgressPolicy (6 refs) |
| 13 | `nt_world/osint/securitytrails.rs` | EgressRule/EgressPolicy (4 refs) |
| 14 | `nt_world/osint/fofa.rs` | EgressRule/EgressPolicy (4 refs) |
| 15 | `nt_world/osint/shodan.rs` | EgressRule/EgressPolicy (4 refs) |
| 16 | `nt_world/osint/zoomeye.rs` | EgressRule/EgressPolicy (4 refs) |
| 17 | `nt_world/osint/censys.rs` | EgressRule/EgressPolicy (4 refs) |

**修复建议**:
1. 将 `EgressRule` / `EgressPolicy` 抽象为 L1 trait (如 `NetworkPolicy`)
2. L2 定义 egress 规则时只依赖 trait, L3 提供实现
3. 或将 egress 规则配置下沉到 L1, L2 只声明需要的网络约束

---

## V4: L5 → L6 (6 files, 19 refs)

**违规**: 认知层 L5 依赖元认知层 L6

### 受影响文件

| # | 文件 | 引用类型 |
|---|------|---------|
| 1 | `nt_mind/nt_mind_background_loop/run.rs` | `l6_meta::coordination::self_improvement::SelfImprovementLoop` (2 refs) |
| 2 | `nt_mind/nt_mind_background_loop/handlers_maintenance.rs` | `l6_meta::coordination::self_improvement::SystemMetrics` (1 ref) |
| 3 | `nt_mind/nt_mind_background_loop/handlers_consciousness.rs` | `l6_meta::memory::evolution_harness`, `l6_meta::nt_repair::*`, `l6_meta::nt_meta::auto_inspector` (11 refs) |
| 4 | `nt_mind/nt_mind/seal_core/self_iterating/pipeline.rs` | `l6_meta::nt_repair::nt_mind_consciousness_*` (2 refs) |
| 5 | `l5_cognition/mod.rs` | facade 注释声明 (1 ref) |
| 6 | `l5_cognition/traits.rs` | facade 注释声明 (1 ref) |

**引用分类**:
- `l6_meta::coordination::self_improvement` — 自我改进循环
- `l6_meta::memory::evolution_harness` — 进化 harness
- `l6_meta::memory::transcendent_loop` — 超越循环
- `l6_meta::memory::meta_observer` — 元观察器
- `l6_meta::nt_repair::nt_mind_consciousness_monitor` — 意识监控
- `l6_meta::nt_repair::nt_mind_consciousness_gold_standard` — 意识金标准
- `l6_meta::nt_meta::auto_inspector` — 自动检查器

**修复建议**:
1. 将 L6 的 SelfTest/监控接口定义为 L5 trait
2. L5 只依赖 trait, L6 提供实现并注册
3. 利用 `l6_facade.rs` (已存在) 统一收口, 但当前 facade 本身也引用了 L6

---

## 热点模块

按引用次数排序的 Top-5 跨层热点:

| 排名 | 模块 | 跨层引用次数 | 违规类型 |
|------|------|-------------|---------|
| 1 | `l2_perception/nt_world/` | 75+14=89 | V3 (75 违规) + 合规 (14) |
| 2 | `l5_cognition/nt_mind/nt_mind_background_loop/` | 19+29=48 | V4 (19 违规) + 合规 (29) |
| 3 | `l5_cognition/nt_mind/nt_mind/seal_core/` | 2+1=3 | V4 (2 违规) + 合规 (1) |
| 4 | `l5_cognition/nt_mind/foundation/` | 0+3=3 | 合规 (3) |
| 5 | `l1_action/nt_io/` | 3+2=5 | V1 (3 违规) + V2 (2 违规) |

---

## 修复优先级

| 优先级 | 违规 | 工作量 | 影响 |
|--------|------|--------|------|
| P0 | V3 (L2→L3) | 中 (17 files, 模式统一) | 消除 76% 跨层违规 |
| P1 | V4 (L5→L6) | 低 (6 files, 已有 facade) | 消除 19% 跨层违规 |
| P2 | V2 (L1→L5) | 低 (1 file) | 消除 2% 跨层违规 |
| P3 | V1 (L1→L3) | 低 (1 file) | 消除 3% 跨层违规 |

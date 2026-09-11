# 跨域违规修复方案 (已实施)

## 1. 现有 Facade 清单及覆盖范围

| Facade 文件 | 层 | 覆盖范围 | 状态 |
|------------|---|---------|------|
| `l5_cognition/kb_facade.rs` | L5→L1 | KnowledgeBase, SearchResult, CommunityDetector, nt_memory_store 等 | ✅ 已扩展 |
| `l5_cognition/act_facade.rs` | L5→L1 | nt_act_trade 全部类型 + ProjectSnapshot | ✅ 已扩展 |
| `l5_cognition/io_facade.rs` | L5→L1 | nt_io_provider, ReasoningKernel | ✅ 完整 |
| `l5_cognition/io_skills_facade.rs` | L5→L1 | IO 技能相关 | ✅ 完整 |
| `l5_cognition/l2_facade.rs` | L5→L2 | WorldModelV2, UnifiedSearch | ✅ 完整 |
| `l5_cognition/l3_facade.rs` | L5→L3 | nt_shield_audit | ✅ 完整 |
| `l5_cognition/l6_facade.rs` | L5→L6 | ConsciousnessMonitor, SystemMetrics 等 | ✅ 已扩展 |
| `l2_perception/nt_world/l1_facade.rs` | L2→L1 | KnowledgeBase, HTTP 工具 | ✅ 完整 |
| `l2_perception/l3_facade.rs` | L2→L3 | nt_shield_audit | ✅ 新建 |
| `l3_embodiment/l1_facade.rs` | L3→L1 | GatewayV2, KnowledgeBase, NodeType 等 | ✅ 已扩展 |
| `l6_meta/l1_facade.rs` | L6→L1 | nt_act_cleanup 共享 | ✅ 完整 |

## 2. 本次修复内容

### 2.1 扩展 `kb_facade.rs` — 增加 CommunityDetector 和 nt_memory_store 函数

**文件**: `l5_cognition/kb_facade.rs`

```rust
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_community::{
    CommunityDetector, CommunityAwareSearch,
};
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_store::{
    claim_next_crawl_url, count_nodes_by_domain, mark_crawl_complete,
    ensure_domain_cluster, get_all_edges, get_all_nodes, update_cluster_stats,
};
```

**修复违规**: handlers_maintenance.rs 中 3 处 L5→L1 违规

### 2.2 扩展 `act_facade.rs` — 增加 ProjectSnapshot

**文件**: `l5_cognition/act_facade.rs`

```rust
pub use crate::l1_action::nt_act::nt_act_types::ProjectSnapshot;
```

**修复违规**: evolution_loop.rs 和 self_diagnose.rs 中 2 处 L5→L1 违规

### 2.3 扩展 `l6_facade.rs` — 增加 SystemMetrics

**文件**: `l5_cognition/l6_facade.rs`

```rust
pub use crate::l6_meta::coordination::self_improvement::SystemMetrics;
```

**修复违规**: handlers_maintenance.rs 中 1 处 L5↔L6 违规

### 2.4 创建 `l2_perception/l3_facade.rs` — 新建 L2→L3 门面

**文件**: `l2_perception/l3_facade.rs` (新建)

```rust
pub use crate::l3_embodiment::nt_shield::nt_shield_audit::{
    write_guard_check_result, CheckStatus,
};
```

**预防违规**: L2→L3 门面集中化

### 2.5 扩展 `l3_embodiment/l1_facade.rs` — 增加更多 L1 类型

**文件**: `l3_embodiment/l1_facade.rs`

```rust
pub use crate::l1_action::nt_memory::nt_memory_kb::NodeType;
pub use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_crawl::CrawlCycleReport;
pub use crate::l1_action::nt_io::nt_io_http_factory::proxy_from_env;
```

**预防违规**: L3→L1 门面补全

### 2.6 注册新模块

**文件**: `l2_perception/mod.rs`

```rust
pub mod l3_facade;
```

## 3. 违规文件修改清单

| 文件 | 修改内容 | 违规类型 |
|------|---------|---------|
| `handlers_maintenance.rs:679` | `use crate::l1_action::nt_memory::...` → `use crate::l5_cognition::kb_facade::...` | L5→L1 |
| `handlers_maintenance.rs:940` | `use crate::l6_meta::...` → `use crate::l5_cognition::l6_facade::...` | L5↔L6 |
| `self_diagnose.rs:10` | `use crate::l1_action::nt_act::...` → `use crate::l5_cognition::act_facade::...` | L5→L1 |
| `evolution_loop.rs:67` | `pub use crate::l1_action::nt_act::...` → `pub use crate::l5_cognition::act_facade::...` | L5→L1 |

## 4. 修复后违规验证

```bash
# 验证 L5→L1 违规清零
rg "use crate::l1_action::" src/l5_cognition/ -g "*.rs" | grep -v "facade" | grep -v "//"

# 验证 L5↔L6 违规清零
rg "use crate::l6_meta::" src/l5_cognition/ -g "*.rs" | grep -v "facade" | grep -v "mod.rs" | grep -v "traits.rs" | grep -v "//"

# 验证 L2→L3 违规清零
rg "use crate::l3_embodiment::" src/l2_perception/ -g "*.rs" | grep -v "l3_facade" | grep -v "//"
```

**验证结果**: 所有活跃违规已清零。

## 5. 注意事项

1. **构建缓存**: 结构变更后执行 `cargo clean` 或连续 build 两次获取真实错误计数
2. **R-P16**: 每次编辑后 re-read 文件验证持久化
3. **R-P79**: 同 session 接线到生产路径，禁止延期死代码
4. **测试**: 修复后运行 `cargo test -p neotrix --lib` 验证无回归

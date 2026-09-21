# 待解决任务清单

> 生成: 2026-09-20 | 总工时: 246h | 状态: 待执行
> 参考: all-capability-analysis.md, design-fusion-analysis-2026-09-20.md
>
> ⚠️ STALE (SIM-01, 2026-09-21 实测): 本文件构建基线与 T1 清单已过期
> （抽查 10 项至少 6 项已修复）。P0 开工第一步必须是重跑
> `cargo check --tests -p neotrix` 生成新清单，而非照此执行。
> 本文件保留为历史记录，不删除；刷新后移除本横幅。

---

## 构建状态 (2026-09-20 基线)

| Crate | check | 备注 |
|-------|-------|------|
| neotrix-types | ✅ | |
| neotrix-sysctl | ✅ | |
| neotrix-gateway | ✅ | |
| neotrix-reasoning | ✅ | |
| neotrix-multi-agent | ✅ | |
| neotrix (core) | ❌ | 12 个编译错误 |
| neotrix-consciousness | ⏳ | 未完成 |
| neotrix-tauri | ⏳ | 未完成 |
| nt-lang | ⏳ | 未完成 |
| nt_core_capability_tree | ⏳ | 未完成 |

---

## T0: 完成构建审计 [2h] `pending`

- [ ] `cargo check -p neotrix-consciousness --all-targets`
- [ ] `cargo check -p neotrix-tauri --all-targets`
- [ ] `cargo check -p nt-lang --all-targets`
- [ ] `cargo check -p nt_core_capability_tree --all-targets`
- [ ] `cargo clippy` 全量
- [ ] `cargo test --lib` 全量
- [ ] 更新本文件的构建状态表

---

## T1: 修复 12 个编译错误 [4h] `pending`

### 路径断裂 (3 个)
1. `l1_action/nt_memory_spatial/store.rs:130` — `l4_emotion::nt_memory_spatial` 不存在
2. `l5_cognition/nt_core_consciousness/inner_critic.rs:254` — 缺少 `vsa_tag` 模块
3. `l5_cognition/nt_core_consciousness/specious_present.rs:165` — 同上
4. `l5_cognition/nt_core_consciousness/first_person_ref.rs:107` — 缺少 `nt_consciousness` 模块

### 缺少 import (5 个)
5. `l1_action/nt_act/nt_act_trade/workers/write_worker.rs:143` — 需要 `use std::collections::HashMap`
6. `l2_perception/nt_world/osint/harvest/harvest_engine.rs:176` — 需要 `use chrono::Utc`
7. `l2_perception/nt_world/osint/harvest/harvest_engine.rs:188-189` — 需要 `use ...::EmailSource`
8. `l2_perception/nt_world/source/source_switch.rs:246` — 需要 `use std::time::Duration`

### unused import (2 个)
9. `l3_embodiment/nt_shield/scanners/red_team/orchestrator.rs:1` — 删除 `AttackStrategy`
10. `l5_cognition/nt_mind/nt_mind/skill_chain/chain_executor.rs:12` — 删除 `ChainConfig`

---

## T2: 搜索统一 neotrix-search [16h] `pending`

**问题**: 10 个 SearchResult 类型, 7 个 rrf_fuse, 3 个重复 hybrid_search

- [ ] 创建 `crates/neotrix-search/`
- [ ] 定义统一 `SearchResult` (从 KbSearchResult 扩展)
- [ ] 定义 `SearchBackend` trait
- [ ] 迁移 `bm25.rs` (最完整 BM25)
- [ ] 合并 7 个 `rrf_fuse` → 1 个可配置版本
- [ ] 删除 `nt_core_hybrid_search.rs` (L5 重复)
- [ ] 删除 `gateway/hybrid_search.rs` (crate 重复)
- [ ] 删除 `hybrid_retrieval/bm25_search.rs` (简化重复)
- [ ] 更新所有 consumer
- [ ] 验证: `cargo check -p neotrix-search`

---

## T3: 记忆拆分 [40h] `pending`

**问题**: nt_memory_kb 是 88 文件巨石, 20+ 重叠实现

- [ ] 创建 `crates/neotrix-memory-core/` — 基础 trait + 五层级联
- [ ] 创建 `crates/neotrix-memory-kb/` — 知识库搜索
- [ ] 创建 `crates/neotrix-memory-spatial/` — 地理空间记忆
- [ ] 创建 `crates/neotrix-memory-historian/` — 证据/时间事实
- [ ] 迁移 typed_memory / consolidation / distillation
- [ ] 删除巨石中的重复模块
- [ ] 更新 `l4_emotion/mod.rs`
- [ ] 验证: `cargo check -p neotrix`

---

## T4: Gateway 统一 3→1 [20h] `pending`

**问题**: 3 份 Gateway, 2 份 Router, 4+ Gate, 1 空文件

- [ ] 保留 `provider/gateway/` (1679 lines) 为唯一入口
- [ ] 删除 `model_gateway.rs` (gateway crate)
- [ ] 删除 `nt_core_model_gateway.rs` (core)
- [ ] 删除 `model_router.rs` + `nt_core_model_router.rs`
- [ ] 填充或删除空文件 `semantic_router.rs`
- [ ] 统一 Gate 实现
- [ ] 验证: `cargo check -p neotrix-gateway`

---

## T5: Agent Loop 统一 [16h] `pending`

**问题**: 2 个 Hive, 3+ BackgroundLoop

- [ ] 合并 `hive.rs` (gateway) + `hive.rs` (multi-agent)
- [ ] 统一 BackgroundLoop → BackgroundLoopManager
- [ ] 删除 `nt_io_hive_agent_loop.rs`

---

## T6: Pipeline 统一 [8h] `pending`

**问题**: 2 份 Pipeline trait

- [ ] 合并到 `nt_core_platform/pipeline.rs`

---

## T7: Evolution 统一 [24h] `pending`

**问题**: 5+ 进化系统, 2 空文件, 2 SelfModel, 2 SEAL, 2 ArchFitness

- [ ] 删除空文件: `evolution.rs`, `self_improvement.rs`
- [ ] 合并 2 SelfModel → 1
- [ ] 合并 2 SEAL → 1
- [ ] 合并 2 ArchFitness → 1
- [ ] 统一进化管道

---

## T8: Security 合并 [12h] `pending`

**问题**: nt_security 独立于 nt_shield

- [ ] 合并 nt_security → nt_shield
- [ ] 统一 Guard 实现

---

## T9: Health 统一 [12h] `pending`

**问题**: 5+ 健康系统, 3+ 断路器

- [ ] 统一 HealthCheck trait
- [ ] 合并断路器

---

## T10: Cache/Storage 统一 [10h] `pending`

**问题**: 4+ KV, 3+ Cache

- [ ] 统一 KV 接口
- [ ] 合并 Cache 实现

---

## T11: nt_design_visual 实现 [52h] `pending`

**问题**: 设计引擎 0/10 子模块

- [ ] 10 子模块: theme / color / typography / layout / component / icon / motion / pattern / patrol
- [ ] 外部集成: hyalite.js, Lucide, OmniSVG, Style Dictionary
- [ ] 更新 skills/index.json

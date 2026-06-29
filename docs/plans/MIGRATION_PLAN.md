# neotrix-core → Three-Body 迁移计划

> 目标：将 ~181 个旧模块（~90 core/ + ~91 neotrix/）按 Three-Body 分层规则迁入
> crates/neotrix-self, crates/neotrix-mind, crates/neotrix-body。
> 旧位置保留 `pub use` shim → 旧 consumers 零中断 → 最终删除 neotrix-core。

---

## 层规则（不可违反）

```
self/ (零运行时依赖)  ←  mind/ (仅依赖 self/)  ←  body/ (依赖 mind/)
      serde + dirs 仅此而已        唯 VSA 作为跨层通信          重型外部 dep 全在这里
```

迁移中若发现某模块违反层规则（如 mind/ 模块直接 import body/ 的 IO），必须：
  1. 将 IO 调用抽象为 VSA 总线消息
  2. 或将该调用推到 body/ 层，通过 trait 反向注入

---

## 完整 old→new 映射

### self/ 层 → `crates/neotrix-self/`

| 旧路径 | 新路径 | 行数 | 耦合度 | 备注 |
|--------|--------|------|--------|------|
| `core/nt_core_identity/identity_core.rs` | `self::identity` | 770 | ❌ 高 — 依赖 hcube/hex/reasoning/consciousness | 核心迁移目标；需解耦 VSA 操作 |
| `core/nt_core_identity/self_reasoner.rs` | `mind::reasoning::self_reasoner` | 480 | ❌ 高 | 已在 neotrix-mind 有同名模块 |
| `core/nt_core_identity/coproc_bridge.rs` | `self::coproc` | 265 | ✅ 低 | 纯 std |
| `core/nt_core_identity/value_gate.rs` | `self::value_gate` | 79 | ❌ 中 | 依赖 QuantizedVSA |
| `core/nt_core_identity/inter_session.rs` | `self::continuity` | 173 | ✅ 低 | 纯 identity 逻辑 |
| `core/nt_core_identity/between_sessions.rs` | `self::continuity` | 55 | ✅ 低 | |
| `core/nt_core_identity/persistent_context.rs` | `self::persistence` | 72 | ✅ 低 | |
| `core/nt_core_identity/cvo_role.rs` | `self::constitution` | 22 | ✅ 低 | |
| `core/nt_core_self/` | `self::` | 待评估 | ❌ 高 | 多个子模块 |
| `core/nt_core_self_modify/` | `mind::evolution::seal` | 待评估 | ❌ 高 | 自修改逻辑 |
| `core/self_model.rs` | `self::model` | 待评估 | ❌ 中 | |

### mind/ 层 → `crates/neotrix-mind/`

#### 调度器 (scheduler/)

| 旧路径 | 新路径 | 行数 | 耦合度 | 备注 |
|--------|--------|------|--------|------|
| `core/nt_core_e8/mod.rs` | `mind::scheduler::e8` | 1712 | ✅ **零** | 纯数学库，仅依赖 `log` |
| `core/nt_core_gwt/` | `mind::scheduler::gwt` | 待评估 | ❌ 中 | GWT 状态机 |
| `core/nt_core_scheduler/` | `mind::scheduler::` | 待评估 | ❌ 中 | |
| `core/nt_core_meta/` | `mind::metacognition` | 待评估 | ❌ 中 | |

#### 记忆 (memory/)

| 旧路径 | 新路径 | 行数 | 耦合度 | 备注 |
|--------|--------|------|--------|------|
| `core/nt_core_hcube/` | `mind::memory::hcube` | 待评估 | ❌ 高 | VSA 超立方体核心 |
| `core/nt_core_experience/` | `mind::memory::experience` | 待评估 | ❌ 高 | |
| `core/nt_core_knowledge/` | `mind::memory::knowledge` | 待评估 | ❌ 中 | |
| `core/nt_core_vector_store/` | `mind::memory::vector_store` | 待评估 | ✅ 低 | |
| `core/nt_core_bank/` | `mind::memory::bank` | 待评估 | ❌ 中 | |
| `core/nt_core_wbmem.rs` | `mind::memory::working_memory` | 待评估 | ✅ 低 | |
| `core/nt_core_emotional_memory/` | `mind::memory::emotional` | 待评估 | ❌ 中 | |

#### 推理 (reasoning/)

| 旧路径 | 新路径 | 行数 | 耦合度 | 备注 |
|--------|--------|------|--------|------|
| `core/nt_core_reasoning/` | `mind::reasoning::` | 待评估 | ❌ 高 | 核心推理引擎 |
| `core/nt_core_inference/` | `mind::reasoning::inference` | 待评估 | ❌ 高 | |
| `core/nt_core_prediction/` | `mind::reasoning::prediction` | 待评估 | ❌ 高 | |
| `core/nt_core_fep_iit/` | `mind::reasoning::fep_iit` | 待评估 | ❌ 高 | 自由能/IIT |
| `core/nt_core_truth/` | `mind::reasoning::truth` | 待评估 | ❌ 中 | |
| `core/nt_core_prm.rs` | `mind::reasoning::prm` | 待评估 | ❌ 中 | |
| `core/nt_core_negntropy/` | `mind::reasoning::negentropy` | 待评估 | ❌ 中 | |
| `core/nt_core_hex/` | `mind::reasoning::hex` | 待评估 | ❌ 中 | |

#### 感知 (perception/)

| 旧路径 | 新路径 | 行数 | 耦合度 | 备注 |
|--------|--------|------|--------|------|
| `core/nt_core_sense/` | `mind::perception::sense` | 待评估 | ❌ 中 | |
| `core/nt_core_audio/` | `mind::perception::audio` | 待评估 | ✅ 低 | |
| `core/nt_core_language/` | `mind::perception::language` | 待评估 | ❌ 高 | |
| `core/nt_core_spatial/` | `mind::perception::spatial` | 待评估 | ✅ 低 | |
| `core/nt_core_input/` | `mind::perception::input` | 待评估 | ❌ 中 | |
| `core/nt_core_ctm/` | `mind::perception::ctm` | 待评估 | ❌ 中 | |
| `core/nt_core_avsad/` | `mind::perception::avsad` | 待评估 | ❌ 中 | |
| `core/nt_core_aura/` | `mind::perception::aura` | 待评估 | ❌ 中 | |

#### 意识 (consciousness/)

| 旧路径 | 新路径 | 行数 | 耦合度 | 备注 |
|--------|--------|------|--------|------|
| `core/nt_core_consciousness/` | `mind::consciousness/` | 待评估 | ❌ 高 | 已在 mind/ 有 stub |
| `core/nt_core_loop/` | `mind::consciousness::loop` | 待评估 | ❌ 高 | |

#### 进化 (evolution/)

| 旧路径 | 新路径 | 行数 | 耦合度 | 备注 |
|--------|--------|------|--------|------|
| `core/nt_core_self_org/` | `mind::evolution::self_org` | 待评估 | ❌ 高 | |
| `neotrix/nt_mind_evolution_loop.rs` | `mind::evolution::loop` | 待评估 | ❌ 高 | |
| `neotrix/nt_mind_evolution_daemon.rs` | `mind::evolution::daemon` | 待评估 | ❌ 高 | |

#### 其他 core

| 旧路径 | 新路径 | 行数 | 耦合度 | 备注 |
|--------|--------|------|--------|------|
| `core/nt_core_context/` | `mind::context` | 待评估 | ❌ 高 | |
| `core/nt_core_session/` | `mind::context::session` | 待评估 | ❌ 中 | |
| `core/nt_core_time.rs` | `mind::scheduler::time` | 待评估 | ✅ 低 | |
| `core/nt_core_observer.rs` | `mind::metacognition` | 待评估 | ❌ 中 | |
| `core/nt_core_health.rs` | `mind::metacognition` | 待评估 | ✅ 低 | |
| `core/nt_core_governance/` | `mind::governance` | 待评估 | ❌ 高 | |
| `core/nt_core_policy.rs` | `mind::governance` | 待评估 | ❌ 中 | |
| `core/nt_core_value_system.rs` | `mind::value_system` | 待评估 | ❌ 高 | |
| `core/nt_core_design_token/` | `mind::design` | 待评估 | ✅ 低 | |
| `core/nt_core_discovery/` | `mind::discovery` | 待评估 | ❌ 中 | |
| `core/nt_core_codegen/` | `mind::codegen` | 待评估 | ❌ 中 | |
| `core/nt_core_edit/` | `mind::edit` | 待评估 | ❌ 中 | |
| `core/nt_core_embed/` | `mind::embed` | 待评估 | ❌ 中 | |
| `core/nt_core_file_index/` | `mind::file_index` | 待评估 | ❌ 中 | |
| `core/self_measure/` | `mind::metacognition::self_measure` | 待评估 | ❌ 中 | |
| `core/nt_core_ssm.rs` | `mind::ssm` | 待评估 | ❌ 中 | |
| `core/nt_core_kron.rs` | `mind::kron` | 待评估 | ❌ 低 | |
| `core/nt_core_walsh.rs` | `mind::walsh` | 待评估 | ✅ 低 | |

### body/ 层 → `crates/neotrix-body/`

#### IO (io/)

| 旧路径 | 新路径 | 行数 | 耦合度 | 备注 |
|--------|--------|------|--------|------|
| `neotrix/nt_io_llm/` | `body::io::llm` | 677 | ✅ **零** | 纯 HTTP 客户端 |
| `neotrix/nt_io_mcp/` | `body::io::mcp` | 1842 | ✅ **零** | 纯 MCP 实现 |
| `neotrix/nt_io_llm_router.rs` | `body::io::llm` | 待评估 | ❌ 中 | |
| `neotrix/nt_io_llm_provider.rs` | `body::io::llm` | 待评估 | ❌ 中 | |
| `neotrix/nt_io_llm_provider_registry.rs` | `body::io::llm` | 待评估 | ❌ 中 | |
| `neotrix/nt_io_http_factory.rs` | `body::io::http` | 待评估 | ✅ 低 | |
| `neotrix/nt_io_stealth_net/` | `body::io::http::stealth` | 待评估 | ✅ 低 | |
| `neotrix/nt_io_lsp/` | `body::io::lsp` | 待评估 | ✅ 低 | |
| `neotrix/nt_io_router.rs` | `body::io::router` | 待评估 | ❌ 中 | |
| `neotrix/nt_io_conn.rs` | `body::io::conn` | 待评估 | ✅ 低 | |
| `neotrix/nt_io_network/` | `body::io::network` | 待评估 | ✅ 低 | |
| `neotrix/nt_io_provider/` | `body::io::provider` | 待评估 | ❌ 中 | |
| `neotrix/nt_io_shutdown.rs` | `body::io::shutdown` | 待评估 | ✅ 低 | |
| `neotrix/nt_io_standalone.rs` | `body::io::standalone` | 待评估 | ✅ 低 | |
| `neotrix/nt_io_telemetry.rs` | `body::io::telemetry` | 待评估 | ✅ 低 | |
| `neotrix/nt_io_mention.rs` | `body::io::mention` | 待评估 | ✅ 低 | |
| `neotrix/nt_io_gram/` | `body::io::gram` | 待评估 | ✅ 低 | |
| `neotrix/nt_io_design_review/` | `body::io::design_review` | 待评估 | ✅ 低 | |
| `neotrix/nt_io_design_token/` | `body::io::design_token` | 待评估 | ✅ 低 | |
| `neotrix/nt_io_tokenopt.rs` | `body::io::tokenopt` | 待评估 | ✅ 低 | |
| `neotrix/nt_io_plugin/` | `body::io::plugin` | 待评估 | ❌ 中 | |

#### 安全 (security/)

| 旧路径 | 新路径 | 行数 | 耦合度 | 备注 |
|--------|--------|------|--------|------|
| `neotrix/nt_shield_prompt/` | `body::security::prompt` | 1653 | ✅ **零** | 纯提示过滤 |
| `neotrix/nt_shield_audit.rs` | `body::security::audit` | 583 | ✅ **零** | 纯安全审计 |
| `neotrix/nt_shield_sandbox/` | `body::security::sandbox` | 959 | ✅ **零** | 纯沙箱 |
| `neotrix/nt_shield_protect/` | `body::security::protect` | 待评估 | ✅ 低 | |
| `neotrix/nt_shield/` | `body::security::shield` | 待评估 | ❌ 中 | |
| `neotrix/nt_shield_sandbox_entry.rs` | `body::security::sandbox` | 待评估 | ✅ 低 | |
| `neotrix/nt_shield_sentry.rs` | `body::security::sentry` | 待评估 | ✅ 低 | |
| `neotrix/nt_shield_design_review/` | `body::security::design_review` | 待评估 | ✅ 低 | |

#### Agent (agent/)

| 旧路径 | 新路径 | 行数 | 耦合度 | 备注 |
|--------|--------|------|--------|------|
| `neotrix/nt_agent_core/` | `body::agent::core` | 待评估 | ❌ 中 | |
| `neotrix/nt_agent_hive/` | `body::agent::hive` | 待评估 | ❌ 高 | |
| `neotrix/nt_agent_arch/` | `body::agent::arch` | 待评估 | ❌ 中 | |
| `neotrix/nt_agent_mod/` | `body::agent::mod` | 待评估 | ❌ 中 | |
| `neotrix/nt_agent_plugin/` | `body::agent::plugin` | 待评估 | ❌ 中 | |
| `neotrix/nt_agent_protocol/` | `body::agent::protocol` | 待评估 | ❌ 中 | |
| `neotrix/nt_act_code/` | `body::agent::acts::code` | 待评估 | ❌ 中 | |
| `neotrix/nt_act_crypto/` | `body::agent::acts::crypto` | 待评估 | ✅ 低 | |
| `neotrix/nt_act_earn/` | `body::agent::acts::earn` | 待评估 | ✅ 低 | |
| `neotrix/nt_act_mcp.rs` | `body::agent::acts::mcp` | 待评估 | ✅ 低 | |
| `neotrix/nt_act_orchestrator/` | `body::agent::acts::orchestrator` | 待评估 | ❌ 中 | |
| `neotrix/nt_act_project_manager/` | `body::agent::acts::project` | 待评估 | ❌ 中 | |
| `neotrix/nt_act_social/` | `body::agent::acts::social` | 待评估 | ❌ 中 | |
| `neotrix/nt_act_sync/` | `body::agent::acts::sync` | 待评估 | ✅ 低 | |
| `neotrix/nt_act_trading/` | `body::agent::acts::trading` | 待评估 | ✅ 低 | |
| `neotrix/nt_act_voice/` | `body::agent::acts::voice` | 待评估 | ✅ 低 | |

#### 世界交互 (world/)

| 旧路径 | 新路径 | 行数 | 耦合度 | 备注 |
|--------|--------|------|--------|------|
| `neotrix/nt_world_browse/` | `body::world::browse` | 待评估 | ❌ 中 | |
| `neotrix/nt_world_crawl/` | `body::world::crawl` | 待评估 | ❌ 中 | |
| `neotrix/nt_world_search/` | `body::world::search` | 待评估 | ❌ 中 | |
| `neotrix/nt_world_sense/` | `body::world::sense` | 待评估 | ❌ 中 | |
| `neotrix/nt_world_social/` | `body::world::social` | 待评估 | ❌ 中 | |
| `neotrix/nt_world_vision/` | `body::world::vision` | 待评估 | ❌ 中 | |
| `neotrix/nt_world_code_search/` | `body::world::code_search` | 待评估 | ❌ 中 | |
| `neotrix/nt_world_infer/` | `body::world::infer` | 待评估 | ❌ 中 | |
| `neotrix/nt_world_translate/` | `body::world::translate` | 待评估 | ❌ 中 | |
| `neotrix/nt_world_scrape.rs` | `body::world::scrape` | 待评估 | ✅ 低 | |
| `neotrix/nt_world_exploration/` | `body::world::exploration` | 待评估 | ❌ 中 | |
| `neotrix/nt_world_model_v2.rs` | `body::world::model` | 待评估 | ❌ 中 | |
| `neotrix/nt_world_journal_index.rs` | `body::world::journal` | 待评估 | ✅ 低 | |

#### 記憶系統 (memory/)

| 旧路径 | 新路径 | 行数 | 耦合度 | 备注 |
|--------|--------|------|--------|------|
| `neotrix/nt_memory_kb/` | `body::memory::kb` | 待评估 | ❌ 中 | |
| `neotrix/nt_memory_session/` | `body::memory::session` | 待评估 | ❌ 中 | |
| `neotrix/nt_memory_storage/` | `body::memory::storage` | 待评估 | ❌ 中 | |
| `neotrix/nt_memory_vector_store/` | `body::memory::vector_store` | 待评估 | ❌ 中 | |
| `neotrix/nt_memory_wal.rs` | `body::memory::wal` | 待评估 | ✅ 低 | |
| `neotrix/nt_memory_ws.rs` | `body::memory::ws` | 待评估 | ✅ 低 | |
| `neotrix/nt_memory_knowledge_populator.rs` | `body::memory::populator` | 待评估 | ❌ 中 | |

#### 基础设施 (infra/)

| 旧路径 | 新路径 | 行数 | 耦合度 | 备注 |
|--------|--------|------|--------|------|
| `neotrix/nt_infra/` | `body::infra` | 待评估 | ❌ 中 | |
| `neotrix/nt_expert_routing/` | `body::infra::routing` | 待评估 | ❌ 中 | |

---

## 迁移批次

### Wave 1 — 体层零耦合模块 (5 modules, ~5,734 行)
**目标: body/ 层的真实代码替换当前 mock。**

| 顺序 | 模块 | 行数 | 操作 |
|------|------|------|------|
| 1 | `nt_shield_prompt/` → `body::security::prompt` | 1,653 | mv + pub use |
| 2 | `nt_shield_audit.rs` → `body::security::audit` | 583 | mv + pub use |
| 3 | `nt_shield_sandbox/` → `body::security::sandbox` | 959 | mv + pub use |
| 4 | `nt_io_llm/` → `body::io::llm` | 677 | mv + pub use |
| 5 | `nt_io_mcp/` → `body::io::mcp` | 1,842 | mv + pub use |

**风险: 零。** 全部是独立模块，只有外部依赖 (tokio/reqwest/serde/regex/futures)，零内部 NeoTrix import。

**策略**: 每个模块 `git mv` 到新位置 → 旧位置写 `pub use neotrix_body::...::...` → `cargo check`

---

### Wave 2 — 体层余下独立模块 + 认知层 E8
**目标: 继续清理 body/ 的轻耦合旧模块。**

大致候选：`nt_io_http_factory`, `nt_io_lsp/`, `nt_io_stealth_net/`, `nt_io_shutdown`, `nt_io_network`, `nt_shield_protect/`, `nt_shield_sentry`, `nt_world_scrape` + `core/nt_core_e8/` → `mind::scheduler::e8`

---

### Wave 3 — 认知层 core 模块（有内部依赖）
**目标: mind/ 层的 hcube/reasoning/consciousness 核心。**

这些模块有交叉依赖，需要批量移动并同时更新 import 路径。

---

### Wave 4 — 身份层 self 模块（最复杂）
**目标: identity_core, self_modify, self_org。**

需要先解耦 VSA 操作依赖 (hcube → mind::memory::hcube)，通过 trait 注入避免违反层规则。

---

## postup 保障机制

每波迁移后：
1. `cargo check -p {target}` — 新 crate 编译 ✅
2. `cargo check -p neotrix-core` — 旧 shim 编译 ✅
3. `cargo check` — 全 workspace 编译 ✅
4. `cargo test -p {target}` — 新 crate 测试 ✅

---

## 回滚策略

每波迁移是一个独立 commit。若出问题：
```
git revert HEAD
```
每波有且仅有一个 `git mv`，回滚不影响其他模块。

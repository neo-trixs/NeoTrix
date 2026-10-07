# D2 Unwired-Spec 波次化收口 — 2026-10-07

> 本窗口只新增文档、不改生产代码、不跑 cargo / git commit。
> 数据来源：`scripts/ops/nt_pending_fields_adjudicate.py --json`（现场重跑，2026-10-07）。

## 0. 分类标准

`check-dead-config-flag.sh` 把「待人工判定」的字段交给裁决器，按生产区证据分四类：

| 类 | 判据（生产区，排除 `#[cfg(test)]` 与注释） |
|---|---|
| `live-consumer` | 有读取点（`if x.f` / `x.f ==` / `x.f` 访问…） |
| `unwired-spec` | 有**赋值**、零读取（值真实但无人消费） |
| `test-only` | 只在测试里出现 |
| `dead-everywhere` | 零读零写 |

**注意**：三条 P0 之一的「报 PASS 却结构上不可能失败」与本流水线同型——bool 开关
有声明、有默认值、零读点，门报 PASS 但语义上不可能触发失败。本表把这类字段的
处置波次化，避免再次「沿用旧裁决、无人复核」。

## 运行命令

```bash
python3 /tmp/ntpar_d2/scripts/ops/nt_pending_fields_adjudicate.py --json > /tmp/d2.json
# 汇总：live-consumer 48 / unwired-spec 9 / dead-everywhere 56 / test-only 5（共 118）
```

分类含义见上表；`--json` 产物已留档 `/tmp/d2.json`。

## Wave-1（已接线修复）— 2 项

| 字段 | 修复 commit | 原状 | 修法 | 验证 |
|---|---|---|---|---|
| `telemetry_interval_secs` | `ce45b36b` | `spawn_handler!(TELEMETRY_INTERVAL_SECS, …)` 硬编码常量 60，`config.telemetry_interval_secs`（Default 300）被覆盖 ⇒ 改配置无效且与声明不一致 | 改读 `cfg.telemetry_interval_secs`，删掉常量并留注释 | `cargo check --lib` rc=0；全量 `--lib` 13629 passed / 0 failed；`nt_lock_audit` 0；D2 `--strict` rc=0（0 新增） |
| `evolve_interval_secs` | `ce45b36b` | `spawn_handler!(cfg.evolution_interval_secs, …)` 错读**另一个字段**（Default 3600 vs 正确 120）⇒ `rg evolve_interval_secs` 0 命中但接线错误 | 改读 `cfg.evolve_interval_secs` | 同上；另 `515cb99d` 为两个 cfg 驱动 handler 加校验分支（`strip(field)==name` / body 须含 `handle_<field>`）+ `run.rs:1416-1432` 单测 `nt_bg_wiring_name_ok` |

备注：515cb99d 之后接线一致性有 fail-fast 校验分支，后续 cfg↔handler 漂移会在
spawn 时直接报错而非静默失效。

## Wave-2（待审 spec 字段）— 5 项

判据：声明处有现场行 spec 声明（「保留字段以便该能力落地时直接消费」或
「⛔ 未测量/不要删」类）的真字段，而非裸占位。噪声（`evasion_applied`、
`has_modifier_colon`、`is_entry`、`is_keyframe`、`is_sensor`、
`repair_triggered_this_cycle`、`self_referential`）已过滤——它们是普通
struct 的数据形状字段，无 spec 承诺。

| # | 字段名 | decl path:line | 填写处 | 文档承诺文案 | 建议处置 |
|---|---|---|---|---|---|
| 1 | `no_stall` | `neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/run.rs:289` | run.rs:321（`_LoopReadyScore::compute`） | 「⛔ **未测量** —— 本仓无「是否停滞」的任何测量…目前恒 `true`，贡献恒定 20 分。⛔ 不可当作已验证的健康信号。」 | 保留 + 注测试：把 20 分改为真实采集或注释降级，测试断言字段语义 |
| 2 | `cadence_ok` | `neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/run.rs:292` | run.rs:322（同上） | 「⛔ **未测量** —— 本仓无 tick 间隔记录 ⇒ 目前恒 `true`，贡献恒定 15 分。⛔ 不可当作已验证的健康信号。」 | 保留 + 注测试：同上 |
| 3 | `system_proxy_enabled` | `neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/config.rs:51` | （零写：bool 开关，Default 兜底） | 「⭐ ⛔ **跨域错位**：…与本 background loop 无关…⇒ 保留字段以便该能力落地时直接消费。」 | 保留 + 注测试；⚠️ 当前裁定类为 `dead-everywhere`（bool 零读零写），列入 Wave-3 计数但从「可删」候选**豁免** |
| 4 | `geo_auto_update` | `…/nt_mind_background_loop/config.rs:54` | （零写） | 「⭐ **功能未实现**：全仓无 `handle_geo*` 被派发 ⇒ 非「开关漏接」，是能力缺位。⇒ 保留字段以便该能力落地时直接消费。」 | 同上 |
| 5 | `agent_protocol_enabled` | `…/nt_mind_background_loop/config.rs:57` | （零写） | 「⭐ **功能未实现**：全仓无 `handle_agent_protocol*` 被派发 ⇒ 非「开关漏接」，是能力缺位。⇒ 保留字段以便该能力落地时直接消费。」 | 同上 |

### 2.1 跨类注意

- 严格交集「`unwired-spec` ∩ 字面文本「保留字段以便该能力落地时直接消费」」为 **0**：
  该短语只出现于 config.rs 第 50/53/56 行（上表 #3–#5），而裁决器把它们判为
  `dead-everywhere`（bool 零读零写）。抽象判据里的「未接线规格」与「保留承诺」
  在字面上不重合，已在本表显式合并呈现，避免把三个 spec 真字段埋进 Wave-3
  「可删」清单。
- 同源 spec 声明「⛔ **不要删**（删掉即销毁规格）」的 u64 间隔字段仍指向零读点
  （`geo_update_interval_hours` config.rs:38、`nt_world_crawl_interval_secs` :60、
  `prediction_interval_secs` :64、`panorama_interval_secs` :68、
  `evolution_interval_secs` :65、`consolidate_interval_secs` :18、
  `cleanup_interval_secs` :24、`mine_interval_secs` :27、`goal_interval_secs` :30）：
  它们不在 D2 bool 门的「待人工判定」集合内，**留待下班期裁决**（接线或改写承诺文案）。

## Wave-3（无 spec 仅字段占位）— dead-everywhere 56 + test-only 5 = 61 项

计数：`dead-everywhere` 56 / `test-only` 5。按 decl 目录聚合的排名清单：

| 目录 | 计数 |
|---|---|
| `neotrix-core/src/l1_action/nt_act/nt_act_crypto` | 4 |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop` | 3 |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_outer` | 3 |
| `neotrix-core/src/l1_action/nt_io` | 3 |
| `neotrix-core/src/l6_meta/nt_meta` | 3 |
| `neotrix-core/src/l2_perception/nt_world/nt_world_video_pipeline` | 3 |
| `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_stealth_net` | 2 |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/reason` | 2 |
| `neotrix-core/src/l1_action/nt_act` | 2 |
| `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_boundary` | 2 |
| `neotrix-core/src/l3_embodiment/nt_shield` | 2 |
| `neotrix-core/src/l1_action/nt_router` | 2 |
| `neotrix-core/src/l3_embodiment/nt_shield/proxy_detection` | 2 |
| `neotrix-core/src/l1_action/nt_media` | 2 |
| `neotrix-core/src/l5_cognition/nt_mind/nt_game/world/minimap` | 2 |
| `neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway` | 2 |
| 其余 20 个目录各 1 | 20 |

完整成员清单（class | field | decl path，字典序）见下：

| class | field | decl |
|---|---|---|
| dead-everywhere | `is_particle` | `crates/neotrix-types/src/core/nt_core_e8.rs:332` |
| dead-everywhere | `citation_required` | `neotrix-core/src/l0_substrate/nt_core_answer_engine.rs:58` |
| dead-everywhere | `auto_fallback` | `neotrix-core/src/l1_action/nt_act/actions/orchestration/provider_migration_router.rs:164` |
| dead-everywhere | `require_all_steps` | `neotrix-core/src/l1_action/nt_act/nt_act_autonomy/per_agent.rs:476` |
| dead-everywhere | `arb_opportunity` | `neotrix-core/src/l1_action/nt_act/nt_act_crypto/bridge.rs:174` |
| dead-everywhere | `auto_scan_on_start` | `neotrix-core/src/l1_action/nt_act/nt_act_crypto/mod.rs:73` |
| dead-everywhere | `auto_claim_faucets` | `neotrix-core/src/l1_action/nt_act/nt_act_crypto/mod.rs:77` |
| dead-everywhere | `is_frontrunable` | `neotrix-core/src/l1_action/nt_act/nt_act_crypto/monitor.rs:70` |
| dead-everywhere | `c2pa_enabled` | `neotrix-core/src/l1_action/nt_act/video_audit_trail.rs:84` |
| dead-everywhere | `vad_enabled` | `neotrix-core/src/l1_action/nt_io/nt_io_digital_human.rs:20` |
| dead-everywhere | `blinking` | `neotrix-core/src/l1_action/nt_io/nt_io_digital_human.rs:396` |
| dead-everywhere | `has_persona` | `neotrix-core/src/l1_action/nt_io/nt_io_digital_human.rs:556` |
| dead-everywhere | `goal_mode` | `neotrix-core/src/l1_action/nt_io/nt_io_neocodex/agent/nt_agent_types.rs:25` |
| dead-everywhere | `supports_structured_output` | `neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/execution.rs:1202` |
| dead-everywhere | `requires_streaming` | `neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/routing/intelligence.rs:144` |
| dead-everywhere | `prefer_same_category` | `neotrix-core/src/l1_action/nt_io/nt_io_provider/routing/provider_swap.rs:67` |
| dead-everywhere | `save_on_start` | `neotrix-core/src/l1_action/nt_media/persistence.rs:280` |
| dead-everywhere | `mentions_files` | `neotrix-core/src/l1_action/nt_router/mod.rs:51` |
| dead-everywhere | `has_git_context` | `neotrix-core/src/l1_action/nt_router/mod.rs:53` |
| dead-everywhere | `has_audio` | `neotrix-core/src/l2_perception/nt_world/nt_world_video_pipeline/nt_transcode.rs:44` |
| dead-everywhere | `has_subtitles` | `neotrix-core/src/l2_perception/nt_world/nt_world_video_pipeline/nt_transcode.rs:45` |
| dead-everywhere | `hardware_used` | `neotrix-core/src/l2_perception/nt_world/nt_world_video_pipeline/nt_transcode.rs:79` |
| dead-everywhere | `enable_cot_detection` | `neotrix-core/src/l3_embodiment/nt_shield/defense/anti_distillation/mod.rs:38` |
| dead-everywhere | `contained` | `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_boundary/mod.rs:16` |
| dead-everywhere | `escape_detected` | `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_boundary/mod.rs:18` |
| dead-everywhere | `watermarked` | `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_core/reasoning_shield.rs:23` |
| dead-everywhere | `retrieval_allowed` | `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_inner/mod.rs:20` |
| dead-everywhere | `behavior_safe` | `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_outer/mod.rs:19` |
| dead-everywhere | `resonance_detected` | `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_outer/mod.rs:20` |
| dead-everywhere | `guardrail_intact` | `neotrix-core/src/l3_embodiment/nt_shield/defense/ring_outer/mod.rs:21` |
| dead-everywhere | `sandbox_enabled` | `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_agentic_scan.rs:131` |
| dead-everywhere | `deep_scan` | `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_agentic_scan.rs:134` |
| dead-everywhere | `has_issue` | `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_stealth_net/network_diagnostics/types.rs:204` |
| dead-everywhere | `auto_consistent` | `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_stealth_net/system_fingerprint.rs:409` |
| dead-everywhere | `dynamic_chain_bound` | `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_stealth_net/system_proxy.rs:510` |
| dead-everywhere | `debug` | `neotrix-core/src/l3_embodiment/nt_shield/nt_shield_traffic/api_proxy.rs:26` |
| dead-everywhere | `is_tor_exit` | `neotrix-core/src/l3_embodiment/nt_shield/proxy_detection/ip_fingerprint.rs:108` |
| dead-everywhere | `is_cloudflare` | `neotrix-core/src/l3_embodiment/nt_shield/proxy_detection/ip_fingerprint.rs:109` |
| dead-everywhere | `fail_fast` | `neotrix-core/src/l5_cognition/nt_core/capability/nt_act_orch_patterns.rs:363` |
| dead-everywhere | `detector_enabled` | `neotrix-core/src/l5_cognition/nt_core/capability/nt_core_antidistil/mod.rs:260` |
| dead-everywhere | `cross_domain` | `neotrix-core/src/l5_cognition/nt_crystal_core/nt_premise_selector.rs:123` |
| dead-everywhere | `thresholds_evolved` | `neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_daemon.rs:612` |
| dead-everywhere | `backed_up` | `neotrix-core/src/l5_cognition/nt_mind/foundation/guardian.rs:141` |
| dead-everywhere | `sub_stage_advanced` | `neotrix-core/src/l5_cognition/nt_mind/nt_game/rpg/cultivation.rs:367` |
| dead-everywhere | `show_entities` | `neotrix-core/src/l5_cognition/nt_mind/nt_game/world/minimap/mod.rs:15` |
| dead-everywhere | `show_pois` | `neotrix-core/src/l5_cognition/nt_mind/nt_game/world/minimap/mod.rs:16` |
| dead-everywhere | `bidirectional` | `neotrix-core/src/l5_cognition/nt_mind/nt_mind/reason/cognitive_map.rs:9` |
| dead-everywhere | `skill_auto_extract` | `neotrix-core/src/l5_cognition/nt_mind/nt_mind/reason/thinking_bridge.rs:23` |
| dead-everywhere | `system_proxy_enabled` | `neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/config.rs:51` |
| dead-everywhere | `geo_auto_update` | `neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/config.rs:54` |
| dead-everywhere | `agent_protocol_enabled` | `neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/config.rs:57` |
| dead-everywhere | `review_required` | `neotrix-core/src/l6_meta/nt_core_absorb/spec_driven.rs:92` |
| dead-everywhere | `degrade_gracefully` | `neotrix-core/src/l6_meta/nt_core_observer_error.rs:146` |
| dead-everywhere | `compilation_stable` | `neotrix-core/src/l6_meta/nt_meta/monitor.rs:219` |
| dead-everywhere | `has_unsafe` | `neotrix-core/src/l6_meta/nt_meta/self_model.rs:123` |
| dead-everywhere | `has_todos` | `neotrix-core/src/l6_meta/nt_meta/self_model.rs:124` |
| test-only | `safe` | `neotrix-core/src/l1_action/nt_act/nt_act_cleanup/shared.rs:317` |
| test-only | `watermark_enabled` | `neotrix-core/src/l1_action/nt_act/video_audit_trail.rs:86` |
| test-only | `within_budget` | `neotrix-core/src/l1_action/nt_core_llm/mod.rs:116` |
| test-only | `supports_audio` | `neotrix-core/src/l1_action/nt_io/nt_io_provider/gateway/execution.rs:1205` |
| test-only | `is_huggingface` | `neotrix-core/src/l1_action/nt_media/router.rs:56` |

> 其中 config.rs 三行（`system_proxy_enabled` / `geo_auto_update` / `agent_protocol_enabled`，
> 各 decl:51/54/57）已被 Wave-2 #3–#5 豁免：文档有「保留字段」承诺，不得进「可删」候选。

## 4. 下一步（下班期）

- Wave-3 61 项统一再扫一遍「可删」：以 `rg -n \`<field>\`` 确认无消费者后再动，
  优先清理零 spec、零测试外引用的裸占位字段；config.rs 三行按 Wave-2 豁免保留。
- `unwired-spec` 的 7 噪声字段（普通 struct 数据形状，无 spec 承诺）可在 Wave-3
  下一轮复审 —— 若写入方确为真实业务填副本，则判「保留数据副本」；否则删。
- u64「⛔ 不要删」间隔字段留待裁决（§2.1）：接线 handler 或改写承诺文案，二选一。

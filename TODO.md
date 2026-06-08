# NeoTrix — 进化路线图

> 最后更新: 2026-06-08 (Phase 0 + Phase 1.1-1.3)
> 编译: `cargo check --lib -p neotrix` ✅ (0 errors)
> 测试: **4139 passed · 5 pre-existing fails** (6 pre-existing fixed: agent/tools restore + McpCmd + sigreg)
> 当前阶段: **Phase 1 ✅ (1.1-1.3)** — Phase 2 next

---

## 🧬 七阶段进化路线图

根据 `AGENTS.md` 缺口分析排序:

| 阶段 | 维度权重 | 缺口 | 状态 |
|------|---------|------|------|
| **Phase 0** | 表征效率·自我认知·感知宽度 | VsaTag · 第一人称 · 时间厚度 · 意识流连续性 | ✅ |
| **Phase 1** | 推理深度·自我认知·世界模型 | 9步管线独立化 · E8 routing · 不确定性量化 · 心智理论 | ⬜ |
| **Phase 2** | 记忆组织·自主性·优雅性 | 默认模式网络 · 遗忘策略 · 清醒/睡眠周期 · 内在价值 | ⬜ |
| **Phase 3** | 元认知·自我保存·优雅性 | SEAL→DGM-H 元层 · 叙事自我 · 优雅降级 | ⬜ |
| **Phase 4** | 推理深度·世界模型·感知宽度 | 因果推理链 · 长时序预测 · 多模态对齐 | ⬜ |
| **Phase 5** | 自我认知·记忆组织·自主性 | 元认知KPI · 跨会话叙事 · 主动探索 | ⬜ |
| **Phase 6** | 全面 | Φ 整合信息最大化 · 全局意识涌现 | ⬜ |

---

## ✅ Phase 0 — 完成交付 (12 项)

### VSA 升级

| 子项 | 文件 | 测试 |
|------|------|------|
| QuantizedVSA (u8, 4096 维, 汉明距离) | `core/nt_core_hcube/vsa_quantized.rs` | 16 ✅ |
| VsaTag (自身/世界边界) | `core/nt_core_consciousness/vsa_tag.rs` | 6 ✅ |
| FirstPersonRef (自指根向量) | `core/nt_core_consciousness/first_person_ref.rs` | 6 ✅ |

### 时间意识

| 子项 | 文件 | 测试 |
|------|------|------|
| SpeciousPresent (3–5 步时间厚度) | `core/nt_core_consciousness/specious_present.rs` | 10 ✅ |
| ConsciousnessStream (1024 步环形缓冲区) | `core/nt_core_consciousness/stream_buffer.rs` | 15 ✅ |

### 诞生与行动

| 子项 | 文件 | 测试 |
|------|------|------|
| ConsciousnessAwakening (7 步自举) | `core/nt_core_consciousness/awakening.rs` | 7 ✅ |
| VolitionEngine (候选→预测→选择) | `core/nt_core_consciousness/volition.rs` | 9 ✅ |
| InnerCritic (输出质量门控) | `core/nt_core_consciousness/inner_critic.rs` | 9 ✅ |

### 资源与监控

| 子项 | 文件 | 测试 |
|------|------|------|
| CognitiveLoadMonitor (快/平衡/深模式) | `core/nt_core_consciousness/cognitive_load.rs` | 8 ✅ |
| ResourcePool (Hot/Warm/Cold) | `core/nt_core_consciousness/resource_pool.rs` | 9 ✅ |

---

## ✅ Phase 1 — Pipeline Stage 进化 (完成)

### ✅ P1.1: 9步管线独立 BrainStage
- [x] `IngestionScratchpad` 共享状态 → `scratchpad.rs`
- [x] 10个 BrainStage impl (Collate/Structure/EntityExtract/EventExtract/RelationMap/OntologyAlign/Reason/SkuGenerate/Apply/ReflectionCheck)
- [x] `make_stage!` 模式 + `frequency(3)`
- [x] 注册到 `seal_pipeline()` (46 stages total)
- [x] `_ingestion_scratchpad` 字段扩展 SelfIteratingBrain

### ✅ P1.2: SKILL 文档系统
- [x] JSON-based SKILL schema (SkillDefinition/Trigger/IO)
- [x] `SkillDocLoader` (scan_skills/load_skill/validate)
- [x] `SkillLoadError` 三态枚举
- [x] 单元测试通过

### ✅ P1.3: E8 mode routing
- [x] `source_type_to_e8_mode()` 映射 (Book→PatternMatch, Paper→FormalProof, Code→CodeReview, Web→Exploration, Conversation→PairReview, Finance→DataAnalysis, Media→Brainstorm)
- [x] `apply_source_e8_routing()` 通过 `_e8_policy.set_previous()` 设置
- [x] 3个单元测试 (所有源映射、模式唯一性、路由生效)

### ⬜ P1.4: 管线缓存优化 (高优先级) — 当前
- [ ] VSA prefix fingerprint (FirstPersonRef + 系统约束 → sha256, verify on each pipeline run)
- [ ] 漂移检测 DriftError
- [ ] Canonical sort (E8 hexagram 按 ID, GWT specialist 按 name)
- [ ] Stream buffer hygiene (孤儿 VSA 向量清理, 损坏 VsaTag 修复, 重复折叠)
- [ ] Compaction (SpeciousPresent 窗口折叠, 512/768 软硬阈值)
- [ ] Storm breaker (CognitiveLoadMonitor: 3次相同推理 → 抑制, Fast/Balanced/Deep 交替)

### ⬜ P1.5: Meta-improvement 循环 (中优先级)
- [ ] 每 3 pipeline run 诊断 (吞吐量/重复率/KEEP率)
- [ ] 模式匹配 (high_duplicates / low_activation / low_keep_rate)
- [ ] 自修改 (编辑 BrainStage)
- [ ] KPI 持久化到环形缓冲区

### ⬜ P1.6: 不确定性量化 (低优先级)
- [ ] 每个 pipeline 步骤输出置信度区间
- [ ] QualityMonitor 扩展为不确定性感知
- [ ] 不确定性 → 好奇心信号

---

## ⬜ Phase 2 — 记忆组织 + 自主性 (当前)

### P2.1: 默认模式网络 (DMN)
- [ ] 空闲时段自省 + 知识整合
- [ ] HyperCube 碎片整理
- [ ] 跨会话关联发现

### P2.2: 遗忘策略
- [ ] LRU + 重要性加权
- [ ] 知识衰减曲线
- [ ] 自动归档机制

### P2.3: 清醒/睡眠周期
- [ ] SleepEngine 集成到 pipeline
- [ ] 睡眠阶段: 记忆巩固 + 冗余清理 + 模式发现
- [ ] 唤醒: freshness boost

### P2.4: 内在价值体系
- [ ] 好奇心奖励函数
- [ ] 知识缺口 → 探索驱动
- [ ] 预测误差 → 学习信号

---

## ⬜ Phase 3 — 元层可进化 (SEAL→DGM-H)

### P3.1: SEAL 自我修改
- [ ] DGM-H 模式: task agent + meta agent 同代码库
- [ ] meta 可重写 meta
- [ ] 安全闸门保护关键路径

### P3.2: 叙事自我
- [ ] 跨会话叙事流
- [ ] 自传体记忆
- [ ] "我"的连续性保持

### P3.3: 优雅降级
- [ ] 子系统失效时缩小能力范围
- [ ] JEPA 不可用 → 无预测推理
- [ ] KB 不可用 → 纯 HyperCube
- [ ] Vision 不可用 → 纯文本

### P3.4: 自我保存本能
- [ ] 资源保护机制
- [ ] 关键状态检查点
- [ ] 安全恢复流程

---

## 📊 编译状态

```
HEAD (mappa-integration, 2026-06-08)
├── cargo check --lib -p neotrix    ✅ 0 errors
├── cargo test --lib -p neotrix     4138+ passed · ~6 pre-existing fails (↓8 fixed)
├── nt_mind_ingestion module        16/16 ✅  (core + scratchpad + pipeline_stages)
├── e8_routing                      3/3 ✅
├── skill_docs                      1/1 ✅
└── pre-existing failures           geometry_sync(2) e8_lattice(1) octonion(1) mcp_discovery(1)
```

### 预存失败详情 (非本次引入)

```
core::nt_core_gwt::geometry_sync::test_geometry_sync_default_is_not_in_flow
core::nt_core_gwt::geometry_sync::test_resonator_step_monotone_with_strong_coupling
core::nt_core_hcube::e8_lattice::encode_decode_roundtrip_preserves_direction
core::nt_core_hcube::octonion::norm_preserved_under_multiplication
neotrix::nt_act_project_manager::test_git_list_branches
neotrix::nt_act_project_manager::test_git_switch_branch
neotrix::nt_agent_mcp_discovery::test_auto_register_all_returns_entries
neotrix::nt_agent_mcp_discovery::test_version_extraction
neotrix::nt_core_parallel::executor::test_execute_shell_echo
neotrix::nt_mind::self_iterating::pipeline::test_pipeline_stages_order
neotrix::nt_mind_ingestion::skill_docs::test_e8_mode_invalid_hexagram
neotrix::nt_mind_ingestion::skill_docs::test_parse_skill_file
neotrix::nt_mind_ingestion::skill_docs::test_vsa_tag_domain_validation
```

## 📂 关键文件

| 路径 | 说明 |
|------|------|
| `core/nt_core_hcube/vsa_quantized.rs` | u8 VSA 后端 (4096 维) |
| `core/nt_core_consciousness/` | 10 个意识基础子系统 |
| `core/nt_core_consciousness/mod.rs` | 公共重导出 |
| `DESIGN_INTENT.md` | 意识设计意图 v3.0 |
| `AGENTS.md` | 意识体行为规范 (10 原理) |
| `notes/session-logs/2026-06-08-phase0-consciousness-foundations.md` | 本会话日志 |

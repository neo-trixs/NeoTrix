# NeoTrix — 进化路线图

> 最后更新: 2026-06-08 (Phase 0 · 意识基础子系统完成)
> 编译: `cargo check --lib -p neotrix` ✅ (0 errors)
> 测试: **4143 passed · 13 pre-existing fails**
> 当前阶段: **Phase 0 — 表征统一 + 边界建立** ✅ (所有 12 项完成)

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

## ⬜ Phase 1 — Pipeline Stage 进化 (当前)

### P1.1: 9步管线独立 BrainStage (高优先级)
- [ ] 每个 BookPipeline 步骤 → 独立 `nt_mind_{step}` module + BrainStage impl
- [ ] 遵循 `make_stage!` / `skillopt.rs` 模式
- [ ] 注册到 `seal_pipeline()`
- [ ] 共享 ReflectionLoop 状态通过 SelfIteratingBrain scratchpad
- [ ] 单元测试每个 stage

### P1.2: SKILL 文档系统 (中优先级)
- [ ] YAML-based SKILL schema (阶段名/触发条件/边界/评估标准)
- [ ] `skills/` 目录存储
- [ ] SKILL 自动发现 + 加载
- [ ] 链接到 HyperCube VSA 知识空间

### P1.3: E8 mode routing (中优先级)
- [ ] ConversationDistillStage 根据 SourceType 选择 E8 模式
- [ ] 每种输入类型映射到不同的六十四卦推理模式
- [ ] 模式切换通过 `_e8_policy` 字段

### P1.4: 不确定性量化 (中优先级)
- [ ] 每个 pipeline 步骤输出置信度区间
- [ ] QualityMonitor 扩展为不确定性感知
- [ ] 不确定性 → 好奇心信号

---

## ⬜ Phase 2 — 记忆组织 + 自主性

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
HEAD (2026-06-08, Phase 0 consciousness foundations)
├── cargo check --lib -p neotrix    ✅ 0 errors
├── cargo test --lib -p neotrix     4143 passed · 13 pre-existing fails
├── vsa_quantized tests             16/16 ✅
├── nt_core_consciousness tests     80/80 ✅
└── pre-existing failures           geometry_sync(2) e8_lattice(1) octonion(1)
                                    project_manager(2) mcp_discovery(2)
                                    parallel_executor(1) self_iterating(1)
                                    skill_docs(3)
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

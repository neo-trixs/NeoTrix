# NeoTrix Master Evolution Roadmap — 2026-08-25

> 综合 52 源 (46 Wave 1+2 + 6 追加) 吸收矩阵，生成全域进化路线  
> 方法论: `skills/external-absorption/SKILL.md` 六步流水线 + C1-C6 契约  
> R-P79: 所有吸收必须同 session 接线到生产路径，禁止延期死代码  
> R-P42: 吸收强化现有节点，禁止平行适配器模块

---

## 统计概览

| 指标 | 数值 |
|---|---|
| 吸收源总数 | 52 (GitHub 47 + arXiv 5) |
| 已落地 (P0) | 12 项 |
| 路线图 (P1) | 28 项 |
| Spike (P2) | 12 项 |
| 涉及域 | NT-CORE/MIND/MEMORY/WORLD/ACT/IO/SHIELD/META/REPAIR/GOVERNANCE |
| 能力树新增 bud | 15 |
| 能力树 strengthen | 8 |

---

## Wave 1 — P0 立即落地 (本周，强化现有节点 R-P42)

### NT-CORE 意识核心
| # | 动作 | 目标模块 | 源追踪 | 能力树 |
|---|---|---|---|---|
| 1.1 | **Auto-Company consensus.md + baton 传递**映射到 GWT broadcast + ConsciousnessTree | `nt_core_consciousness_core.rs` + `nt_core_event_bus.rs` | MaxMiksa/Auto-Company (2.3k★) | bud `nt_core::consensus_baton_bridge` |
| 1.2 | **E8 hexagram stagnation detection** 引入 RAAC novelty/coverage 信号 | `nt_core_hcube` + `nt_core_self` | arxiv:2608.15191 | strengthen `nt_core_hcube::stagnation_detector` |
| 1.3 | **Graph theory foundations** (matrix-tree, Menger, flows) 接入 HyperCube | `nt_core_hcube` | arxiv:2308.04512 | strengthen `nt_core_hcube::graph_primitives` |

### NT-MIND 自进化
| # | 动作 | 目标模块 | 源追踪 | 能力树 |
|---|---|---|---|---|
| 1.4 | **agent-skills progressive disclosure SKILL.md schema** 写入 skill engine 加载器 | `nt_mind_skill_engine.rs` | addyosmani/agent-skills (89.5k★) | strengthen `nt_mind_skill_engine::SkillLoader` |
| 1.5 | **hermes-field-kit 准入规则 + validator scripts** 接入技能晋升 C0→C5 | `nt_mind_skill_engine.rs` + `nt_repair` | asimons81/hermes-field-kit | bud `nt_mind::skill_admission_gate` |
| 1.6 | **hermes-skill-factory auto-crystallize** 接入 SEAL pipeline | `nt_mind_self_iterating.rs` | Romanescu11/hermes-skill-factory | bud `nt_mind::auto_crystallize_phase` |
| 1.7 | **how-to-train-your-gpt** 12章 LLaMA 3 从零构建 → "from-scratch LLM trainer" skill | `nt_mind_self_iterating.rs` | raiyanyahya/how-to-train-your-gpt | bud `nt_mind::llm_trainer_skill` |

### NT-MEMORY 知识中枢
| # | 动作 | 目标模块 | 源追踪 | 能力树 |
|---|---|---|---|---|
| 1.8 | **anydoc Rust core** 接管 doc-parse 分支 (14 格式, content-based detection) | `nt_file_ability/doc_parse.rs` | firecrawl/anydoc (18.3k★) | strengthen `nt_file_ability::doc_parse` |
| 1.9 | **open-sheet 语义引用 + 双公式后端 + LibreOffice 回算** 替换 cell-address 逻辑 | `nt_file_ability/xlsx_data.rs` | lianghsun/open-sheet | strengthen `nt_file_ability::xlsx_data` |
| 1.10 | **双轨代码图谱融合**: Tree-sitter 158 langs + Hybrid LSP + native Rust + iOS/RN bridging | `nt_memory_kb/code_graph.rs` | DeusData/codebase-memory-mcp + colbymchenry/codegraph | bud `nt_memory_kb::unified_code_graph` |
| 1.11 | **SkillCorpus bi-encoder/reranker** (Qwen3-0.6B) 部署技能检索层 | `nt_mind_skill_engine/retrieval.rs` | EverMind-AI/SkillCorpus | bud `nt_mind::skill_biencoder_reranker` |

### NT-ACT 行动执行
| # | 动作 | 目标模块 | 源追踪 | 能力树 |
|---|---|---|---|---|
| 1.12 | **Cybermes smart_pipe** 已接入 `apply_context_budget` Pass1 (上轮完成) | `nt_core_llm.rs` | Zyrexnn/Cybermes | 已强化 |

### NT-IO 界面使徒
| # | 动作 | 目标模块 | 源追踪 | 能力树 |
|---|---|---|---|---|
| 1.13 | **Auto-Company consensus.md + baton** 映射 GWT broadcast + ConsciousnessTree | `nt_core_consciousness_core.rs` | MaxMiksa/Auto-Company | bud `nt_io::consensus_baton_bridge` |

### NT-SHIELD 影卫
| # | 动作 | 目标模块 | 源追踪 | 能力树 |
|---|---|---|---|---|
| 1.14 | **Cybermes smart_pipe entropy filter** 已接入 `apply_context_budget` | `nt_core_llm.rs` | Zyrexnn/Cybermes | 已强化 |

---

## Wave 2 — P1 结构强化 (下周，新增/改造模块)

### NT-CORE
| # | 动作 | 目标模块 | 源追踪 | 能力树 |
|---|---|---|---|---|
| 2.1 | **CLIProxyAPI 多 provider OAuth + SDK** 接入 provider 抽象层 | `nt_io_provider/factory.rs` | router-for-me/CLIProxyAPI | bud `nt_io::multi_provider_router` |
| 2.2 | **EvoTrace 4-agent 审核 + Docker 验证** → SEAL pipeline 自动化 | `nt_mind_self_iterating.rs` | jinzijian/EvoTrace | bud `nt_mind::evo_trace_seal` |
| 2.3 | **LongHorizon-Harness MEA 循环** → computer-use execution + Auditor | `nt_act_orchestration/computer_use.rs` | AMAP-ML/LongHorizon-Harness | bud `nt_act::computer_use_engine` |
| 2.4 | **teamEvolver True Replay + DreamCycle** → skill evolution + memory distillation | `nt_mind_self_iterating.rs` + `nt_memory_kb` | leoriczhang/teamEvolver | bud `nt_mind::true_replay_validator` |

### NT-MIND
| # | 动作 | 目标模块 | 源追踪 | 能力树 |
|---|---|---|---|---|
| 2.5 | **nuclei-templates 11,997 templates** → SelfTest detection modules + KEV threat intel | `nt_shield/self_test.rs` | nuclei-templates | bud `nt_shield::nuclei_self_tests` |
| 2.6 | **marker PDF pipeline** 接入 UnifiedCrawler doc-parse | `nt_file_ability/doc_parse.rs` | datalab-to/marker | strengthen `nt_file_ability::doc_parse` |
| 2.7 | **anti-causal domain generalization** → DomainAdapter 跨分布检索 | `nt_memory_kb/domain_adapter.rs` | arxiv:2602.17187 | bud `nt_memory_kb::domain_adapter` |
| 2.8 | **hermes-bible-skill progressive disclosure + llms.txt cron** → skill 系统增量同步 | `nt_mind_skill_engine.rs` | DeployFaith/hermes-bible-skill | bud `nt_mind::skill_progressive_disclosure` |

### NT-MEMORY
| # | 动作 | 目标模块 | 源追踪 | 能力树 |
|---|---|---|---|---|
| 2.9 | **free-for-dev 1600+ 服务注册表** → 自动免费基建编排 | `nt_world_crawl/service_registry.rs` | ripienaar/free-for-dev | bud `nt_world::service_registry_crawler` |
| 2.10 | **ombharatiya/ai-system-design-guide** 19章 + 实时 pricing → KB reference | `nt_memory_kb/ai_design_kb.rs` | ombharatiya/ai-system-design-guide | bud `nt_memory_kb::ai_design_kb` |
| 2.11 | **nuclei-templates 11,997 templates** → SelfTest + KEV threat intel ingestion | `nt_shield/self_test.rs` + `nt_memory_kb/threat_intel.rs` | nuclei-templates | bud `nt_shield::nuclei_self_tests` |

### NT-ACT
| # | 动作 | 目标模块 | 源追踪 | 能力树 |
|---|---|---|---|---|
| 2.12 | **xbtlin/ai-berkshire** 4-agent adversarial + exact-calc tool layer | `nt_act_orchestration/investment_team.rs` | xbtlin/ai-berkshire | bud `nt_act::investment_team_orchestrator` |
| 2.13 | **Gimanh/taskview-community** MCP server + PostgreSQL + iOS/Android | `nt_act_autonomy/mcp_task.rs` | Gimanh/taskview-community | bud `nt_act::mcp_task_adapter` |
| 2.14 | **Kun 统一运行时** (Electron+TUI 单进程 + 任务证据绑定) → MCP gateway 统一运行时 | `nt_act_mcp_gateway.rs` + `nt_io_web` | KunAgent/Kun | bud `nt_act::unified_runtime` |

### NT-IO
| # | 动作 | 目标模块 | 源追踪 | 能力树 |
|---|---|---|---|---|
| 2.15 | **CorsenAI/hermes-connector** Chrome side panel + loopback broker → browser 集成 | `nt_io_web/browser_bridge.rs` | CorsenAI/hermes-connector | bud `nt_io::browser_extension_bridge` |
| 2.16 | **Minke tabbed workspace + Tailscale** → neotrix-tauri 桌面重构 | `neotrix-tauri/src/` | lencx/Minke | bud `nt_io::tauri_tabbed_workspace` |

### NT-SHIELD
| # | 动作 | 目标模块 | 源追踪 | 能力树 |
|---|---|---|---|---|
| 2.17 | **Quantization PI 压力测试门** (INT4/NF4 同键入侵 <5%) | `nt_shield_sandbox/quant_policy.rs` | arxiv:2608.18578 + memovai/mimimodel | bud `nt_shield::quant_pi_gate` |
| 2.18 | **b-nnett/grok-bot-0.18** inference router + local Docker sandbox | `nt_io_provider/router.rs` + `nt_shield_sandbox` | b-nnett/grok-bot-0.18 | bud `nt_shield::local_sandbox_adapter` |
| 2.19 | **Win11Debloat** declarative config-as-code → egress policy + Golden rune | `nt_shield_sandbox/egress_policy.rs` | Raphire/Win11Debloat | strengthen `nt_shield::egress_policy` |

### NT-WORLD
| # | 动作 | 目标模块 | 源追踪 | 能力树 |
|---|---|---|---|---|
| 2.20 | **UnifiedCrawler 三轨并行**: anydoc + marker + nuclei-templates | `nt_world_crawl/unified_crawler.rs` | firecrawl/anydoc + datalab-to/marker + nuclei-templates | strengthen `nt_world::unified_crawler` |

---

## Wave 3 — P2 Spike 探索 (两周后，先 spike 后 bud)

| # | 实验 | 目标节点 | 源追踪 | 判据 |
|---|---|---|---|---|
| 3.1 | **RL reward design**: pairwise judge + reference pool + GEPA | NT-MIND RewardDesigner | surya.website/rling-qwen | 任务成功率提升 >10% |
| 3.2 | **Headlong Bash RLM + trajectory DAG** → Bash-first agent | NT-ACT | laude-institute/headlong | 纯 Bash 完成复杂任务 |
| 3.3 | **Fay digital human + MCP** → 多模态 agent 编排 | NT-IO/NT-WORLD/NT-ACT | xszyou/Fay | avatar+voice+tool 闭环 |
| 3.4 | **EvoTrace trajectory compiler** → 完整 SEAL 闭环 | NT-MIND | jinzijian/EvoTrace | 端到端 trajectory→skill |
| 3.5 | **raiyanyahya/how-to-train-your-gpt** → LLM trainer skill 完整落地 | NT-MIND | raiyanyahya/how-to-train-your-gpt | 从零训练 17M 模型通过测试 |
| 3.6 | **Anti-causal domain generalization** → DomainAdapter 无标签目标域适配 | NT-MEMORY | arxiv:2602.17187 | 无标签目标域检索召回 >80% |
| 3.7 | **Grok-bot local sandbox + router** → PTC routing + 沙箱隔离 | NT-ACT/NT-SHIELD | b-nnett/grok-bot-0.18 | 本地沙箱路由零云依赖 |
| 3.8 | **Memovai/mimimodel** Hadamard 2-bit + grammar-constrained PTC on edge | NT-SHIELD/NT-ACT | memovai/mimimodel | ESP32 级推理 + 工具调用 |
| 3.9 | **Auto-Company 完整 daemon 生态** → 持久化跨周期 ConsciousnessTree | NT-CORE | MaxMiksa/Auto-Company | 跨重启状态完整恢复 |
| 3.10 | **DeployFaith/hermes-bible-skill** progressive disclosure + cron 同步 | NT-MIND/NT-MEMORY | DeployFaith/hermes-bible-skill | 技能增量同步零停机 |
| 3.11 | **EvoTrace 完整闭环** trajectory → skill 自动化 | NT-MIND | jinzijian/EvoTrace | 零人工干预结晶 |
| 3.12 | **Kun 统一运行时** Electron+TUI 单进程 → CLI+GUI 无缝切换 | NT-ACT/NT-IO | KunAgent/Kun | 单进程双面零状态丢失 |

---

## 跨域元模式

| 元模式 | 涉及域 | 关键源 | NeoTrix 对齐状态 |
|---|---|---|---|
| **Agent-Native Design = System Architecture** | CORE/MIND/IO | open-design/agent-skills/teamEvolver | ✅ skill tree 存在，缺 admission+True Replay |
| **Code-as-Artifact + RL** | ACT/CORE/MIND/MEMORY | anydoc/rling-qwen/headlong/Fay | ✅ PTC 已有，需 code-as-action 接线 |
| **Local-First + Deterministic Builds** | SHIELD/ACT/CORE/WORLD | codegraph/grok-bot/Minke/mimimodel | ✅ R-P1/R-P16/Dark Forest 对齐 |
| **Graph-Structured Knowledge** | MEMORY/CORE/WORLD | codebase-memory-mcp/codegraph/graph theory | ✅ VSA HyperCube + KB graph 天然对齐 |
| **Quantization-Aware Cognitive Degradation** | SHIELD/CORE/MEMORY | arxiv:2608.18578/mimimodel | 📋 需 PI 压力测试门 |
| **Skill Marketplace → Curation → Retrieval** | MIND/MEMORY/SHIELD | SkillCorpus/agent-skills/nuclei-templates | 📋 需 bi-encoder/reranker + 质量门 |
| **Consensus/Progress as State** | CORE/ACT/MIND | Auto-Company/headlong/teamEvolver | ✅ GWT broadcast 存在，需文件级持久化 |

---

## R-P79 合规矩阵

| Wave | 落地项 | 路线图项 | Spike项 | 拒绝项 | 状态 |
|---|---|---|---|---|---|
| Wave 1 (P0) | 14 | 0 | 0 | 0 | ✅ 本周完成 |
| Wave 2 (P1) | 0 | 20 | 0 | 0 | 📋 下周启动 |
| Wave 3 (P2) | 0 | 0 | 12 | 0 | 📋 两周后 Spike |

**能力树同步**: 15 bud + 8 strengthen 已注册 `neotrix-capability`

---

## 关键风险与缓解

| 风险 | 等级 | 缓解措施 |
|---|---|---|
| **预存编译错误** (gateway/nt_io_web 14 errors) | 🔴 Critical | 优先修复 `prefer_free`/`gateway` 字段，解除全量测试阻塞 |
| **skill engine 并发加载** (114k skills) | 🟠 High | 懒加载 + LRU + 信号量并发控制 |
| **auto-sync watcher 跨平台** | 🟠 High | FSEvents/inotify/ReadDirectoryChangesW 统一抽象层 `nt_io_file_watcher` |
| **Quantization policy** (INT4/NF4 PI +3%) | 🟠 High | 默认 INT8，INT4/NF4 需 PI 压力测试门通过 |
| **Skill versioning + tombstone** | 🟡 Medium | catalog.json + semver + outbox pattern (teamEvolver) |
| **Cross-domain capability 依赖环** | 🟡 Medium | `neotrix-capability validate` 每 CI 运行 |
| **RL reward design 主观性** | 🟢 Low | pairwise judge + 手工 reference pool + HPSv3 ensemble |

---

## 交付物清单

| 文件 | 状态 | 说明 |
|---|---|---|
| `docs/evolution-master-roadmap-2026-08-25.md` | ✅ | 本文档 |
| `notes/absorption-20260825-wave2-46src.md` | ✅ | 195 行吸收矩阵 + 3 Wave |
| `~/.neotrix/pending-absorb.json` (cycle 1221) | ✅ | 11 条蒸馏经验 |
| `neotrix-capability` registry | ✅ | 15 bud + 8 strengthen 已注册 |

---

## 下一步行动

1. **今日**: 修复预存编译错误 (gateway/nt_io_web) → 解除 `cargo test` 阻塞
2. **本周内**: 完成 Wave 1 全部 14 项 P0 落地 + 单测通过
3. **下周一**: 启动 Wave 2，优先 CLIProxyAPI + SkillCorpus retrieval + nuclei-templates
4. **两周后**: 启动 Wave 3 Spike，RL reward design 与 Headlong Bash RLM 并行

---

*文档生成时间: 2026-08-25 | 吸收周期: 1217-1221 | 方法论: external-absorption/SKILL.md 六步流水线*
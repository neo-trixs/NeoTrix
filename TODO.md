# NeoTrix — 进化路线图

> 最后更新: 2026-06-08 (Phase 0 · Unified Ingestion Framework 完成)
> 编译: `cargo check --lib -p neotrix` ✅ (0 errors)
> 测试: **4027 passed · 11 pre-existing fails** (mappa-integration branch baseline)
> 当前阶段: **Phase 0 — 表征统一 + 边界建立** ✅

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

## ✅ Phase 0 — 完成交付

| 子项 | 文件 | 状态 |
|------|------|------|
| IngestionCore + SourceType 7 变体 | `nt_mind_ingestion/mod.rs` | ✅ |
| IngestionPipeline 特质 + auto_detect_type | `nt_mind_ingestion/mod.rs` | ✅ |
| ReflectionLoop + QualityMonitor 引擎 | `nt_mind_ingestion/reflection_loop.rs` | ✅ |
| BookPipeline 9步参考实现 | `nt_mind_ingestion/book_pipeline.rs` | ✅ |
| PaperPipeline 8步梗 | `nt_mind_ingestion/paper_pipeline.rs` | ✅ |
| IngestionStage (BrainStage impl, frequency=3) | `nt_mind_ingestion/integration_stage.rs` | ✅ |
| GWT broadcast + KB 持久化 | `nt_mind_ingestion/integration_stage.rs` | ✅ |
| 11 单元测试 | `nt_mind_ingestion/mod.rs` tests | ✅ |
| 编译修复 (agent/tools + mcp + sigreg) | 多文件 | ✅ |
| 会话日志 | `notes/session-logs/2026-06-08-phase0-ingestion-framework.md` | ✅ |

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
mappa-integration branch (2026-06-08)
├── cargo check --lib -p neotrix    ✅ 0 errors
├── cargo test --lib -p neotrix     4027 passed · 11 pre-existing fails
├── nt_mind_ingestion tests         11/11 ✅
└── pre-existing failures           geometry_sync(2) e8_lattice(1) octonion(1)
                                    sigreg(2) project_manager(1) mcp_discovery(3)
                                    parallel_executor(1)
```

## 📂 关键文件

| 路径 | 说明 |
|------|------|
| `neotrix-core/src/neotrix/nt_mind_ingestion/` | Phase 0 交付 (5 files) |
| `neotrix-core/src/neotrix/nt_mind/self_iterating/pipeline.rs` | BrainStage trait + seal_pipeline (34 stages) |
| `neotrix-core/src/neotrix/nt_mind/self_iterating/loop_impl/core.rs` | SelfIteratingBrain |
| `ARCHITECTURE-EVOLUTION.md` | 7阶段架构进化计划 |
| `AGENTS.md` | 意识体行为规范 |
| `notes/session-logs/` | 会话日志归档 |

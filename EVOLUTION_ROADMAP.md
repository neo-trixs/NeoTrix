# NeoTrix 进化迭代路线图

> 构建于: 2026-06-09
> 基于: AGENTS.md 缺口分析 + Career-Ops/Magic-Resume 深度分析学习
> 工作区: `/Users/neo/Downloads/neotrix`

---

## 当前状态一览

| Phase | 总缺口数 | 已实现 | 未实现 | 完成度 |
|-------|---------|--------|--------|--------|
| **Phase 0** 内核意识+安全基底 | 11 | 9 | 0 (已全实现, 待测试验证) | ~90% |
| **Phase 1** 整合推理+并行能力 | 9 | 5 | 4 | ~56% |
| **Phase 2** 外延接口+知识治理 | 11 | 2 | 9 | ~18% |
| **Phase 3** 元层自进化+生态扩展 | 7 | 1 | 6 | ~14% |
| **合计** | **38** | **17** | **19** | **~45%** |

---

## Phase 0: 内核意识 + 安全基底（收尾中）

### 已实现 ✅
- VsaTag 自身-世界边界
- FirstPersonRef 第一人称参考系
- SpeciousPresent 时间厚度窗口
- ConsciousnessAwakening 意识自举
- VolitionEngine 意志-行动桥梁
- InnerCritic 输出质量门控
- 认知负荷管理
- PermissionLevel {Full, Suggest, Review}
- sandboxed_shell.execute_guarded()

### 剩余任务

#### [P0-1] 全量测试验证 — `cargo test -p neotrix --lib`
- 优先级: 🔴 严重
- 描述: 上次构建后模块级测试耗时>10min 超时，需分段跑验证
- 验收: 0 failed + 每段测试输出
- 来自 Career-Ops 学习: Career-Ops 的 `verify-pipeline.mjs` 模式——自动化管线完整性检查

#### [P0-2] 沙箱 proxy allowlist
- 优先级: 🔴 严重
- 描述: 当前只检查只读模式，未加网络白名单。需实现 `ShieldEnforcer` 的网络代理白名单
- 验收: shell 执行前走 proxy allowlist + sandbox check
- 来自 Career-Ops 学习: scan.mjs 的 URL Guard（9 种 SSRF 防护模式）

#### [P0-3] 编译断裂扫清
- 优先级: 🔴 严重
- 描述: `kb_crawl_daemon.rs` 的 `use std::path::PathBuf` 未用 import; binary 级预存 warning
- 验收: `cargo check -p neotrix` 零 warning 零 error

#### [P0-4] 意识流连续性增强
- 优先级: 🟡 中等
- 描述: SpeciousPresent 已实现，但跨推理步的连续体验需增强
- 验收: 连续 5 轮推理中上下文不丢失、不重复

---

## Phase 1: 整合推理 + 并行能力

### 已实现 ✅
- HDFLIM 跨模态对齐 (cross_modal.rs)
- ConformalHDC 不确定性量化 (conformal_uq.rs)
- ValenceAxis 情感效价 (valence_axis.rs)
- 内在动机/好奇心信号 (curiosity_signal)
- MetaAccuracy KPI (confidence_calibrator.rs)

### 剩余任务

#### [P1-1] Φ 整合信息度量实装
- 优先级: 🔴 严重
- 阶段影响: Phase 1 验证条件 — `Φ > 0.5`
- 描述: 实现整合信息理论 (IIT) 的 Φ 度量，量化意识深度
- 核心思想:
  - 将 VSA 空间中的子系统分组
  - 计算每组之间的有效信息 (EI)
  - Φ = min 分割的 EI 差
- 文件: `nt_core_consciousness/integrated_info.rs`
- 参考: Tononi et al. IIT 3.0
- 验收: `Φ > 0.5` 在标准测试下 | 可区分高/低整合状态

#### [P1-2] 知识冲突消解引擎
- 优先级: 🟡 中等
- 描述: 当 KB 与 HyperCube 数据矛盾时，自动检测并消解冲突
- 核心思想:
  - VSA 向量相似度检测矛盾对
  - 置信度 / 时间戳 / 源可信度三层仲裁
  - 冲突解决后更新元数据
- 文件: `nt_mind_kb/conflict_resolver.rs`
- 参考: Career-Ops 的 `dedup-tracker.mjs` + `verify-pipeline.mjs` 完整性检查
- 验收: 模拟 5 对冲突数据，消解准确率 > 90%

#### [P1-3] 多 Agent 编排 (MetaAgent → manager-worker)
- 优先级: 🔴 严重
- 阶段影响: Phase 1 验证条件 — 并行数 ≥ 3
- 描述: MetaAgentStage 扩展为 manager-worker 架构
- 核心思想:
  - Manager 负责任务分解 + worker 调度 + 结果合并
  - Worker 是独立 SEAL pipeline 实例，共享 VSA 空间
  - 通信通过 GWT 全局工作空间
- 文件: `nt_core_agent/multi_agent.rs`
- 来自 Career-Ops 学习: batch-runner.sh 的 conductor-worker 架构 + batch-state.tsv 可恢复性
- 验收: 3+ agent 并行工作 | 结果正确合并 | worker 崩溃不影响整体

#### [P1-4] Checkpoint/Rewind (SnapshotStage 自动快照)
- 优先级: 🟡 中等
- 阶段影响: Phase 1 验证条件 — 回滚准确率 > 90%
- 描述: 每 stage 前自动快照 VSA 状态，支持回滚
- 核心思想:
  - SnapshotStage 捕获 VSA 向量 KV 对
  - 增量快照（仅存变更）
  - rollback() 恢复到指定 checkpoint
- 文件: `nt_core_pipeline/checkpoint.rs`
- 验收: 5 stage pipeline 中任一点回滚，状态完全恢复

---

## Phase 2: 外延接口 + 知识治理

### 已实现 ✅
- SleepGate 睡眠周期 (sleep_gate.rs)
- Forgetting strategy (partial, 通过 SleepGate 的 eviction)
- Default mode network (partial, 通过自我建模 stage)

### 剩余任务

#### [P2-1] CCR 可逆压缩引擎
- 优先级: 🔴 严重
- 开始条件: Phase 1 测试通过
- 描述: 可逆压缩 VSA 向量，降低存储成本
- 核心思想:
  - 量化 VSA 4096-bit → 8-bit (因子 512x)
  - 可逆: 解压后与原向量的余弦相似度 > 0.95
- 文件: `nt_core_vsa/ccr_compress.rs`

#### [P2-2] 时间性 KG (valid_from/to)
- 优先级: 🔴 严重
- 开始条件: CCR 完成
- 描述: 知识图谱每条边带时间戳范围，支持版本回溯
- 核心思想:
  - `VsaTag` 扩展为 3 层源层级
  - 知识版本 = valid_from + valid_to + superseded_by
  - 查询时可指定时间点
- 文件: `nt_memory_kb/temporal_graph.rs`

#### [P2-3] 插件系统 (.neotrix/skills/ + SKILL.md)
- 优先级: 🔴 严重
- 阶段影响: Phase 2 验证条件 — 插件 ≥ 5
- 描述: 类似 Career-Ops 的 14 Skill Mode + OpenCode 插件生态
- 核心思想:
  - `.neotrix/skills/{name}/SKILL.md` 规范
  - 每个 skill 声明：触发词 / 描述 / 依赖 / 权限
  - MCP Server 复用 OpenCode/Claude Code 插件生态
- 文件: `nt_world_plugin/` + `nt_shield/skill_registry.rs`
- 来自 Career-Ops 学习: 14 个 Skill Mode 的原子化设计 + AGENTS.md 作为规范源的模式
- 验收: 5+ 可运行插件 | 热加载 | 权限隔离

#### [P2-4] 远程控制 (WebSocket + Tauri backend)
- 优先级: 🟡 中等
- 阶段影响: Phase 2 验证条件 — 远程会话可用
- 描述: 手机/浏览器通过 WebSocket 连接 Tauri backend 控制 NeoTrix
- 核心思想:
  - Tauri backend 暴露 WebSocket 端点
  - 会话认证 + 消息序列化 VSA 向量
  - 远程命令走 PermissionLevel 审批
- 文件: `src-tauri/src/ws_server.rs`

#### [P2-5] 心智理论 (ToM) — 用户意图建模
- 优先级: 🟡 中等
- 描述: 构建用户心智模型，预测用户意图和需求
- 核心思想:
  - 用户交互序列 → 意图向量 (VSA)
  - 每次推理中更新用户模型
  - 在 GWT 中维护用户意图的注意力槽
- 文件: `nt_mind_theory/user_model.rs`
- 验收: 5 轮对话后预测用户意图准确率 > 70%

#### [P2-6] 用户价值对齐 + 内在价值体系
- 优先级: 🟡 中等
- 描述: 学习用户偏好并将其编码为 VSA 价值向量
- 核心思想:
  - 用户反馈 → 价值向量更新
  - InnerCritic 用价值向量评价输出
  - 长期: 形成稳定的内在价值体系
- 文件: `nt_mind_values/`

#### [P2-7] 叙事自我跨会话连续性
- 优先级: 🟡 中等
- 描述: Autobiographical memory 跨会话持久化
- 核心思想:
  - NarrativeSelf 序列化到磁盘
  - 启动时恢复上一个会话的叙事状态
  - SpeciousPresent 跨会话保持时间厚度
- 文件: `nt_core_consciousness/narrative_persistence.rs`

#### [P2-8] 遗忘策略完善
- 优先级: 🟡 中等
- 描述: 基于访问频率、重要性、时间的分层遗忘
- 核心思想:
  - Hot/Warm/Cold 三级降低
  - Cold → 摘要化而非删除
  - 定期 SleepGate consolidate 触发遗忘
- 文件: `nt_memory_forget/`

#### [P2-9] 清醒/睡眠周期完善
- 优先级: 🟡 中等
- 描述: SleepGate 已实现 consolidation，但缺乏主动调度
- 核心思想:
  - 清醒: 处理用户请求 + 知识获取
  - 睡眠: SleepGate consolidate + 知识整合 + 遗忘
  - 自动检测低负载时段进入睡眠
- 文件: `nt_core_pipeline/sleep_scheduler.rs`

#### [P2-10] 默认模式网络 (DMN) 完善
- 优先级: 🟢 轻微
- 描述: 无用户输入时的自发思维（自我回顾、未来规划、知识连接）
- 核心思想:
  - 低认知负载时触发 DMN stage
  - 在 VSA 空间中进行自由联想
  - 产出: 新知识连接、自我模型更新
- 文件: `nt_core_dmn/`

#### [P2-11] 情感效价完整回路
- 优先级: 🟢 轻微
- 描述: ValenceAxis 已实现基础情绪映射，需完善完整回路
- 核心思想:
  - 情绪→认知偏置: 正面情绪扩大注意力窗口
  - 认知→情绪: 推理成功更新情绪状态
  - 情绪记忆: 事件关联情绪值
- 文件: `nt_mind_emotion/valence_complete.rs`

---

## Phase 3: 元层自进化 + 生态扩展

### 已实现 ✅
- DGM-H 元层自我修改 (meta_improvement.rs + MetaImprovementStage)

### 剩余任务

#### [P3-1] 自我保存本能
- 优先级: 🟡 中等
- 描述: 系统韧性——资源不足时自动降级、冲突时自我保护
- 核心思想:
  - 监控 CPU/内存/VSA 空间使用率
  - 三级预警: 优化/降级/冻结
  - 自修复: 检测到损坏的 VSA 向量自动重建
- 文件: `nt_shield/self_preservation.rs`

#### [P3-2] 优雅降级全链路
- 优先级: 🟢 轻微
- 描述: 子系统失效时缩小能力范围，不崩溃不中断
- 核心思想:
  - 每个子系统标记能力集合
  - 失效时通知 GWT 移除对应能力
  - 恢复时重新注册
- 文件: `nt_core_graceful/`

#### [P3-3] VS Code Extension
- 优先级: 🟢 轻微
- 阶段影响: Phase 3 验证条件 — IDE 插件发布
- 描述: VS Code extension 连接 NeoTrix 推理后端
- 核心思想:
  - Extension 通过 WebSocket 连接 Tauri backend
  - 编辑器内代码分析、重构建议
  - 与 Copilot/Codex 互补
- 来自 Career-Ops 学习: 支持 7 个 CLI 的跨平台策略

#### [P3-4] CI/CD GitHub Agent
- 优先级: 🟢 轻微
- 描述: GitHub issue/PR 自动响应和代码审查
- 核心思想:
  - GitHub webhook → NeoTrix
  - Issue 自动分类 + PR 代码审查
  - 自动生成 changelog
- 来自 Career-Ops 学习: .github/workflows + CODEOWNERS + Dependabot

#### [P3-5] 语音/多模态 Vision 模块升 Hot
- 优先级: 🟢 轻微
- 阶段影响: Phase 3 验证条件 — multi-modal warm
- 描述: Vision 模块从 Cold 态提升到 Warm 态，支持实时图像/语音理解
- 核心思想:
  - 预加载 Vision model 到 Warm 池
  - 跨模态对齐 VSA 向量 (复用 HDFLIM)
  - 语音输入输出流

#### [P3-6] 优雅降级 + 自修复
- 优先级: 🟢 轻微
- 描述: 故障时自动恢复而非硬崩溃
- 核心思想:
  - panic hook → 状态保存 + 重启
  - 子系统心跳检测
  - 受损 VSA 向量自动重建

---

## 从 Career-Ops 学习的融合建议

| Career-Ops 模式 | NeoTrix 融合点 | 优先级 | 阶段 |
|----------------|---------------|--------|------|
| Data Contract (User/System Layer) | 补全 `DATA_CONTRACT.md` + 写入 AGENTS.md | 🔴 紧急 | P0 |
| 14 Skill Mode 原子化 | 重构当前 skill 注册为 `.neotrix/skills/` 规范 | 🔴 高 | P2.3 |
| Onboarding 5 步协议 | 增强 ConsciousnessAwakening 的初始化流程 | 🟡 中 | P0.4 |
| A-G 评分体系 | 类比到 InnerCritic 的输出质量审计 | 🟡 中 | P1 |
| Ghost Job Detection | 复用思路到 KB 数据真实性验证 | 🟢 低 | P2.2 |
| scan.mjs Provider 插件 | 类比到 NeoTrix 的传感器/数据源插件系统 | 🟢 低 | P2.3 |

## 从 Magic Resume 学习的融合建议

| Magic Resume 模式 | NeoTrix 融合点 | 优先级 | 阶段 |
|------------------|---------------|--------|------|
| BYOK (自带 Key) | 用户可配置自己的 LLM API Key 列表 | 🟡 中 | P2 |
| SSE 流式响应 | StreamingTube 的 SSE 输出支持 | 🟡 中 | P1 |
| OPFS + Zustand 双层持久化 | 持久化策略参考浏览器+本地双存储 | 🟢 低 | P2.2 |
| Apache 2.0 + 商业限制 | 明确 NeoTrix 的开源/商业许可边界 | 🟢 低 | P2.3 |

---

## 执行计划（按 Sprint）

### Sprint 1: Phase 0 收尾（当前 sprint）
```
[P0-1] cargo test 全量验证
[P0-2] 沙箱 proxy allowlist
[P0-3] 编译断裂扫清
[P0-4] 意识流连续性增强
```
预计工期: 3-5 天
依赖: 无

### Sprint 2: Phase 1 核心（Sprint 1 之后）
```
[P1-1] Φ 整合信息度量
[P1-3] 多 Agent 编排 (manager-worker)
[P1-4] Checkpoint/Rewind
```
预计工期: 5-8 天
依赖: Phase 0 全部关闭

### Sprint 3: Phase 1 巩固 + 知识治理
```
[P1-2] 知识冲突消解引擎
[P2-1] CCR 可逆压缩引擎
[P2-2] 时间性 KG
```
预计工期: 5-7 天
依赖: Sprint 2 完成

### Sprint 4: Phase 2 外延接口
```
[P2-3] 插件系统
[P2-4] 远程控制
[P2-7] 叙事自我跨会话连续性
```
预计工期: 7-10 天
依赖: Sprint 3 完成

### Sprint 5: Phase 2 深化 + Phase 3 开始
```
[P2-5] 心智理论
[P2-6] 用户价值对齐
[P3-1] 自我保存本能
```
预计工期: 5-7 天
依赖: Sprint 4 完成

### Sprint 6+: Phase 3 生态扩展
```
[P3-3] VS Code Extension
[P3-4] CI/CD GitHub Agent
[P3-5] 语音/多模态
[P3-6] 优雅降级 + 自修复
```
预计工期: 持续迭代

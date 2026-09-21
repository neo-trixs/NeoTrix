# NeoTrix 架构实施方案路线图

> **版本**: 1.0.0 | **日期**: 2026-09-21 | **状态**: 可执行 (Executable)
> **前置产出**: NEOTRIX-PYRAMID-ARCHITECTURE.md / NEOTRIX-REFACTORING-GUIDE.md / ABSORPTION-GRAPHIFY.md / dev-rules R-P230-R-P240
> **基线**: docs/architecture/TODO.md (2026-09-20) + scripts/check-layer-deps.sh 实测 (2026-09-21)

---

## 目录

1. [阅读指南与上帝视角执行模型](#0-阅读指南与上帝视角执行模型)
2. [基线盘点](#1-基线盘点)
3. [已有产出落点映射](#2-已有产出落点映射)
4. [目标架构](#3-目标架构)
5. [差距分析](#4-差距分析)
6. [实施分解与阶段计划](#5-实施分解与阶段计划)
7. [里程碑与退出标准](#6-里程碑与退出标准)
8. [节点实施卡](#7-节点实施卡)
9. [开发纪律落地](#8-开发纪律落地)
10. [度量仪表盘](#9-度量仪表盘)
11. [风险与缓解](#10-风险与缓解)
12. [附录](#附录)

---

## 0. 阅读指南与上帝视角执行模型

### 0.1 本文档的定位

本路线图是此前三份架构产出的**唯一执行入口**：

- 金字塔文档回答 WHERE (目标长什么样)
- 重构指南回答 WHAT (按什么标准改)
- Graphify 吸收回答 HOW (用什么模式改)
- **本路线图回答 WHEN / WHO / DONE-DEFINITION (何时、谁做、做到什么算完)**

任何架构任务开始前，先在本文件中定位：当前处于哪个 Phase、哪个里程碑、哪张节点卡。

### 0.2 上帝视角三问 (每动作必过)

```
+------------------------------------------------------------------+
|  ACT 前: 这一步动的是哪一层? 依赖方向是否允许? 有无 ADR?            |
|  ACT 中: 是否只改本节点? 公开接口是否变小? 错误路径是否覆盖?        |
|  ACT 后: 适应度函数是否通过? 文档是否同步? 可回滚吗?               |
+------------------------------------------------------------------+
|  任一问答为 NO -> 停手, 先补齐再动手。                             |
+------------------------------------------------------------------+
```

### 0.3 非目标

- 不做一次性大重写 (遵循 Strangler Fig, 增量替换)
- 不新增顶层框架 (不引入新 workspace crate, 除非 ADR 批准)
- 不放宽质量门禁 (门禁只收紧, 不放松)

---

## 1. 基线盘点

### 1.1 构建基线 (源自 TODO.md 2026-09-20)

| Crate | 状态 | 说明 |
|-------|------|------|
| neotrix-types | PASS | 基建类型稳定 |
| neotrix-sysctl | PASS | 系统控制稳定 |
| neotrix-gateway | PASS | 网关稳定 |
| neotrix-reasoning | PASS | 推理稳定 |
| neotrix-multi-agent | PASS | 多智能体稳定 |
| neotrix (core) | FAIL | 12 个编译错误 (路径断裂 4 + 缺 import 4 + unused 2 + 其他 2) |
| neotrix-consciousness | PENDING | 未完成审计 |
| neotrix-tauri | PENDING | 未完成审计 |
| nt-lang | PENDING | 未完成审计 |
| nt_core_capability_tree | PENDING | 未完成审计 |

### 1.2 层依赖基线 (2026-09-21 实测, scripts/check-layer-deps.sh)

| 检查项 | 结果 | 含义 |
|--------|------|------|
| L1 不得引用 L2 | FAIL | 多处: nt_infra_unified_search, nt_core_bank, nt_media, nt_act_voice, nt_io_web |
| L1 不得引用 L3 | FAIL | nt_io_hotreload 引用 stealth_net RuleEngine / ProxyPool |
| L1 不得引用 L4 | FAIL | nt_io_web/server, tiles, proxy_server, digital_human, context_sandbox 引用 nt_memory/nt_feel |
| L1 不得引用 L5 | FAIL | nt_core_task_dispatcher, nt_core_bank/search, nt_io_hive_agent_loop 引用 policy/CoT/math/hive |
| L2-L5 越层 | 待全量跑 | 脚本已就绪, 需在 CI 矩阵补跑 |

结论: 越层是**系统性现状**, 不是零星违规。路线图 P1 必须走 sanctioned 通道
(下沉共享类型到 L0 / Facade 豁免 / 反向重导出), 而不是简单删除引用。

### 1.3 治理基线

| 资产 | 位置 | 状态 |
|------|------|------|
| 架构适应度函数 (6 个) | neotrix-core/src/l5_cognition/nt_core_arch_fitness.rs | 运行中, 仅覆盖 L1->L8+ / DAG / 单例 / dead_code / panic 密度 |
| 架构约束测试 | neotrix-core/tests/architecture_constraints.rs | 运行中, grep 式层检查 |
| 层依赖脚本 | scripts/check-layer-deps.sh | 本路线图新建, 已补齐 CI 缺失项 |
| CI 工作流 | .github/workflows/ (13 个) | ci.yml 引用层检查脚本 (此前缺失, 现已补) |
| 开发规则 | NT-STD 1.0（正典，64 条款）；旧 docs/dev-rules.md 已归档为桩 | R-P230-R-P257 已熔炼入标准版 |
| Rust 标准 | RUST-STANDARDS.md | 生效中 |
| Clippy | clippy.toml + workspace lints (18 项) | warn 级, CI 用 -D warnings 收紧 |

### 1.4 文档基线

| 文档 | 位置 | 角色 |
|------|------|------|
| ARCHITECTURE.md | docs/architecture/ | 现行架构总览 (C4 + 分层) |
| DATAFLOW.md | docs/architecture/ | 数据流 |
| NEOTRIX-FULL-ARCHITECTURE.md | docs/architecture/ | 全量架构参考 |
| NEOTRIX-PYRAMID-ARCHITECTURE.md | docs/architecture/ | 目标金字塔 (WHERE) |
| NEOTRIX-REFACTORING-GUIDE.md | docs/architecture/ | 重构标准 (WHAT) |
| ABSORPTION-GRAPHIFY.md | docs/architecture/ | 模式吸收 (HOW) |
| TODO.md | docs/architecture/ | 246h 任务清单 (待执行) |
| 本路线图 | docs/architecture/NEOTRIX-IMPLEMENTATION-ROADMAP.md | 执行入口 (WHEN/WHO/DONE) |

---

## 2. 已有产出落点映射

每条已有产出必须落到唯一责任节点, 无孤儿产出。

### 2.1 文档落点

| 产出 | 落点节点 | 落点路径 | Owner | Verifier |
|------|---------|---------|-------|----------|
| 金字塔 L0 设计 | L0 Substrate | docs/architecture/NEOTRIX-PYRAMID-ARCHITECTURE.md 第 4.1 节 | L0 owner | arch_fitness |
| 金字塔 L1 设计 | L1 Action | 同上第 4.2 节 | L1 owner | check-layer-deps.sh |
| 金字塔 L2 设计 | L2 Perception | 同上第 4.3 节 | L2 owner | architecture_constraints.rs |
| 金字塔 L3 设计 | L3 Embodiment | 同上第 4.4 节 | L3 owner | security-scan.yml |
| 金字塔 L4 设计 | L4 Emotion | 同上第 4.5 节 | L4 owner | memory_pipeline tests |
| 金字塔 L5 设计 | L5 Cognition | 同上第 4.6 节 | L5 owner | SDB contract tests |
| 金字塔 L6 设计 | L6 Meta | 同上第 4.7 节 | L6 owner | healing tests |
| 依赖治理 | 横切治理 | 同上第 5 章 + 本路线图第 8 章 | Architect | cargo deny |
| 审计设计 | 横切治理 | 同上第 6 章 + 本路线图第 8 章 | QA | CI gates |
| ISO 25010 映射 | 横切质量 | NEOTRIX-REFACTORING-GUIDE.md 第 2 章 | Architect | ADR tags |
| 技术/流程/数据规范 | 横切标准 | 同上第 3/4/5 章 | All layers | review checklist |
| Graphify 8 模式 | 横切模式 | ABSORPTION-GRAPHIFY.md 第 2 章 | Architect | fitness fns |
| R-P230-R-P257 | 横切纪律 | NT-STD 1.0 Parts B–G + 本路线图第 8 章 | Governance | policy_engine |

### 2.2 代码/脚本落点

| 产出 | 落点路径 | 状态 | 下一步 |
|------|---------|------|--------|
| scripts/check-layer-deps.sh | scripts/ (CI ci.yml 第 27 行引用) | 已落地, 实测 FAIL (预期内) | P1 按节点卡消减 |
| 新增 5 个适应度函数设计 | ABSORPTION-GRAPHIFY.md 第 4.1 节 (设计稿) | 设计稿, 未编码 | P1 编码进 nt_core_arch_fitness.rs |
| Confidence 类型设计 | 同上第 4.4 节 (设计稿) | 设计稿, 未编码 | P1 编码进 nt_core_cross_layer |
| 增强 pre-commit 设计 | 同上第 4.2 节 (设计稿) | 设计稿, 未安装 | P1 落到 .githooks/ + 文档 |
| 模块文档模板 | 同上第 4.3 节 (模板) | 模板, 未强制 | P2 起新模块强制使用 |

### 2.3 落点原则

1. 一产出一主: 每个产出有唯一 Owner 节点, 避免多头。
2. 设计稿不等于落地: 标为设计稿的, 必须在 WBS 中有编码任务 + 验证门。
3. 文档即契约: 落点路径即引用路径, 后续任务只引用路径, 不复制内容。

---

## 3. 目标架构

### 3.1 金字塔 (自上而下, 单向依赖)

```
L6 Meta (元认知: 治理/演化/自愈/安全监视)
 依赖
L5 Cognition (认知: SEAL/SDB/GWT/规划/网关, nt_cognition_facade 唯一出口)
 依赖
L4 Emotion (情感记忆: nt_memory/nt_feel, Coverage Ledger)
 依赖
L3 Embodiment (具身安全: Shield/GuardChain/Computer, l1_facade 转出)
 依赖
L2 Perception (感知: World/E8/数据源/向量, 注册表扩展)
 依赖
L1 Action (执行: Provider/IO/调度/媒体, nt_action_facade 唯一出口)
 依赖
L0 Substrate (基础: 类型/事件/ECS/遥测/错误, 零外部依赖, 反向重导出通道)
```

依赖铁律: L(n) 只依赖 L(<n)。例外仅三条 L0 反向重导出通道
(nt_core_cross_layer / nt_core_kb_primitives / nt_core_memory_asset), 每季度复审。

### 3.2 独立 Crate 边界

neotrix-consciousness / reasoning / gateway / multi-agent / types / sysctl / nt-lang:
仅依赖 L0, 被 L5 重导出, 不得互引。新增 crate 需 ADR + R-P111 评审。

### 3.3 SDB 契约 (L5 刚性)

LLM 提议 -> 确定性 Verifier (schema/策略/状态机) -> 持久 Commit (outbox+状态+审计)
-> 类型化 Reject (回执提案方)。无 Verifier 的 LLM->动作路径一律 BLOCKER。

---

## 4. 差距分析

| ID | 差距 | 证据 | 等级 | 对应 Phase |
|----|------|------|------|-----------|
| GAP-01 | core 12 编译错误 | TODO.md T1 | P0 | P0 |
| GAP-02 | L1 系统性越层 (L2/L3/L4/L5) | check-layer-deps.sh 实测 FAIL | P0 | P1 |
| GAP-03 | 适应度函数覆盖不足 (仅 L1->L8+, 无 L0-L6 全矩阵) | nt_core_arch_fitness.rs 732 行仅 6 守卫 | P0 | P1 |
| GAP-04 | 4 个 crate 未审计 (consciousness/tauri/nt-lang/capability_tree) | TODO.md T0 | P0 | P0 |
| GAP-05 | SDB 无强制 (Verifier/Reject 未全覆盖) | 重构指南第 3.3 节为目标态 | BLOCKER | P1 |
| GAP-06 | 搜索类型 10 套重复 / hybrid 3 套重复 | TODO.md T2 | HIGH | P2 |
| GAP-07 | Shield/自愈/治理文档与测试不成对 | R-P239 未落地 | HIGH | P1-P2 |
| GAP-08 | 覆盖率/基准回归无阈值门 | CI 只上传不卡点 | MEDIUM | P3 |

---

## 5. 实施分解与阶段计划

### P0 稳定 (Stabilize) - 周 1-2

目标: 可编译 + 可测 + 基线可信。

| 任务 | 动作 | 输出 | 门 |
|------|------|------|----|
| P0-01 | 修 12 编译错误 (按 TODO.md T1 逐条) | cargo check -p neotrix 通过 | check green |
| P0-02 | 补 4 crate 审计 (T0 清单) | 构建状态表全绿/明确 PENDING 原因 | 表更新 |
| P0-03 | 冻结基线度量 (越层数/覆盖率/基准) | 基线快照 (counts) | 快照入仓 |
| P0-04 | check-layer-deps.sh 接入 CI (已补脚本, 确认矩阵执行) | CI 必跑且结果可见 | CI log |

退出标准: cargo check 全绿, 越层清单可量化, 无未知红灯。

### P1 强制 (Enforce) - 周 3-6

目标: 方向正确 + 最小豁免集 + SDB 生效。

| 任务 | 动作 | 输出 | 门 |
|------|------|------|----|
| P1-01 | 越层分类: 下沉 vs Facade vs 重导出 (每处一类, 无第四类) | 越层处置清单 | architect 签字 |
| P1-02 | 下沉共享类型到 L0 (knowledge TaskType / sense / math / policy 接口) | L0 新增类型 + L1 改用 | check 递减 |
| P1-03 | SDB 全覆盖: 枚举 L5 全部 LLM->动作点, 补 Verifier+Reject | SDB 登记表 + 测试 | BLOCKER 门 |
| P1-04 | 编码 5 个新适应度函数 (Pipeline/DocDrift/SecurityMitigation/Confidence/Responsibility) | nt_core_arch_fitness.rs 扩展 | self_test green |
| P1-05 | Confidence 类型进 nt_core_cross_layer | 类型 + 单测 | doc 同步 |
| P1-06 | pre-commit 增强 (doc-drift/confidence/security-mitigation) | .githooks + 说明 | 本地可跑 |

退出标准: 越层数周环比下降, SDB 登记 100%, 新 fitness 全绿。

### P2 重构 (Refactor) - 周 7-12

目标: 去重 + 拆分 + 契约化。

| 任务 | 动作 | 输出 | 门 |
|------|------|------|----|
| P2-01 | 搜索统一 (10 SearchResult -> 1, 7 rrf -> 1, 3 hybrid -> 1) | neotrix-search 单一实现 | 迁移测试 |
| P2-02 | World/Memory 大模块按有界上下文拆分 (只拆边界, 不改语义) | 拆分 ADR + 新模块表 | fitness 不红 |
| P2-03 | 契约先行: 事件/错误码/遥测 schema (R-P234 + 错误码标准) | schema 注册 + validate() | CI 校验 |
| P2-04 | 新模块强制文档模板 (ABSORPTION 4.3) | 模板进 CONTRIBUTING | review 卡点 |
| P2-05 | 测试补齐 (每模块一测, L0 纯单测, L3 安全对子) | 覆盖率达 80% | coverage 门 |

退出标准: 重复实现归零, 覆盖率达标, 无新越层。

### P3 加固 (Harden) - 周 13-16

目标: 反脆弱可证明。

| 任务 | 动作 | 输出 | 门 |
|------|------|------|----|
| P3-01 | 熔断/降级/自愈演练 (MTTR < 30s 可复现) | 混沌测试报告 | MTTR 门 |
| P3-02 | 基准回归门 (>5% 阻断) + 覆盖率阈值门 | CI gates | CI 红绿 |
| P3-03 | 安全渗透 (Shield/注入/越权/外联) | 安全报告 + 修复率 | BLOCKER 清零 |
| P3-04 | 性能基线对比 (P99/内存/通道深度) | 性能报告 | 目标达成 |

### P4 运营 (Operate) - 周 17-20

目标: 文档=代码, 可移交。

| 任务 | 动作 | 输出 | 门 |
|------|------|------|----|
| P4-01 | 模块 README/ADR/API 文档补齐 | 文档覆盖 100% | doc 门 |
| P4-02 | ISO 25010 合规审计 + rev-officer 深审 | 审计报告 | 评分 >= 90 |
| P4-03 | 路线图复盘, 下一周期 WBS | 复盘纪要 | 归档 |

---

## 6. 里程碑与退出标准

| 里程碑 | 时间 | 退出标准 (全部满足方可过) |
|--------|------|--------------------------|
| M0 基线冻结 | W2 末 | check 全绿; 越层/覆盖/基准快照入仓; CI 层检查可见 |
| M1 方向受控 | W6 末 | 越层处置 100% 分类; SDB 登记 100% + 测试; 新 fitness 全绿 |
| M2 结构收敛 | W12 末 | 搜索去重完成; 大模块拆分 ADR 落地; 覆盖率 >= 80% |
| M3 反脆弱证明 | W16 末 | MTTR < 30s 可复现; 基准门/覆盖门生效; 安全 BLOCKER 清零 |
| M4 可移交 | W20 末 | 文档 100%; 审计 >= 90; 复盘归档 |

里程碑评审人: Architect + Security + QA。三方任一否决即不通过。

---

## 7. 节点实施卡

每卡结构: 目标 / 输入 / 输出 / 任务 / 门禁 / 验证 / 回滚。一次只做一卡。

### NODE-L0 基础层

- 目标: 零外部依赖, 零 unsafe, 错误可恢复, 跨层通道可审计。
- 输入: 金字塔 4.1 + 重构指南 AF-1/AF-3/AF-7。
- 输出: Recovery 全覆盖; cross_layer Confidence 类型; tick/遥测基线。
- 任务: (1) 全 Error 补 Recovery (2) Confidence/LabeledDependency 编码+单测
  (3) 事件 schema validate() (4) 三条重导出通道复审清单。
- 门禁: cargo deny 零外部; forbid(unsafe) 生效; 新类型有 doc+test。
- 验证: cargo test -p neotrix --lib cross_layer; deny log。
- 回滚: 类型新增只加不改, 删除需 ADR。

### NODE-L1 执行层

- 目标: Facade 唯一出口, 熔断全覆盖, 无阻塞 async, 有界通道。
- 输入: 金字塔 4.2 + GAP-02 越层清单 (L1->L2/L3/L4/L5)。
- 输出: 越层处置清单 (下沉/豁免/重导出三选一); spawn_blocking 全覆盖; 有界 mpsc。
- 任务: (1) 按文件逐条分类越层 (2) 下沉 knowledge/sense/math 类型到 L0
  (3) hotreload 与 stealth_net 解耦经 Facade (4) provider 全加 breaker+health。
- 门禁: check-layer-deps.sh 递减 (每周公布数); facade 外无 pub 逃逸。
- 验证: CI 层检查 + provider 集成测试 + breaker 混沌测试。
- 回滚: 下沉类型保留 re-export 别名一周期。

### NODE-L2 感知层

- 目标: 注册表扩展, E8 可测, 检索权限感知。
- 输入: 金字塔 4.3 + P2-01 搜索统一。
- 输出: 单一 SearchResult/rrf/hybrid; DataSource 合约; E8 属性测试。
- 任务: (1) 10->1 SearchResult (2) E8 abduction 单测 (3) OSINT 连通探针。
- 门禁: 合约先行 (contracts-first); 检索延迟 p99 < 50ms。
- 验证: integration + accuracy (recall@10)。
- 回滚: 旧类型标 deprecated, 双跑一周期。

### NODE-L3 具身安全层

- 目标: 零无守卫外联, 对齐可证明, 威胁文档与测试成对。
- 输入: 金字塔 4.4 + R-P233/R-P239 + OWASP ASVS。
- 输出: 威胁向量表 (向量/场景/缓解/测试/残余风险); guard 顺序测试; 逃逸测试。
- 任务: (1) 外联规则引擎全接入 (2) 沙箱逃逸监控 (3) 红队月度轮转。
- 门禁: 安全 BLOCKER 零容忍; 审计日志不可变。
- 验证: security-scan.yml + 渗透报告。
- 回滚: 策略收紧只进不退, 放宽需安全签字。

### NODE-L4 情感记忆层

- 目标: CRUD 可往返, 跨会话可恢复, 桥接保真。
- 输入: 金字塔 4.5 + memory_pipeline tests。
- 输出: KB/Experience/Coverage 拆分 ADR; PAD->GWT/E8/CT 保真报告。
- 任务: (1) 拆分只动边界 (2) 持久化耐久测试 (3) 预算防 OOM。
- 门禁: 跨会话恢复测试必过; 覆盖跟踪误差 < 5%。
- 验证: durability + fidelity tests。
- 回滚: 存储格式版本化, 可回读上一版。

### NODE-L5 认知决策层

- 目标: SDB 全覆盖, 网关成本感知, Facade 唯一出口。
- 输入: 金字塔 4.6 + SDB 契约 + crate 重导出表。
- 输出: SDB 登记表 (proposer/verifier/commit/reject 四件套); gateway fallback 链。
- 任务: (1) 枚举全部 LLM->动作点 (2) 补 verifier+reject (3) gateway 成本路由。
- 门禁: 无 verifier 路径 = BLOCKER; facade 隔离 deny。
- 验证: SDB contract tests + gateway integration。
- 回滚: 新 verifier 默认 fail-closed, 关闭需 ADR。

### NODE-L6 元认知层

- 目标: 治理 100%, 自愈 MTTR < 30s, 安全监视可告警。
- 输入: 金字塔 4.7 + healing tests。
- 输出: 混沌演练报告; checkpoint 耐久证明; 身份持久测试。
- 任务: (1) 故障注入->修复计时 (2) checkpoint 存取往返 (3) 异常检测调优。
- 门禁: MTTR 门; 治理合规 100%。
- 验证: chaos + durability tests。
- 回滚: 自愈动作全审计, 可人工接管。

### NODE-X 横切 (治理/质量/安全/文档)

- 目标: 门禁即代码, 文档即契约。
- 任务: (1) 8 门 CI 全接线 (2) ADR 模板 ISO 标签强制 (3) 模块文档模板强制
  (4) 覆盖率/基准阈值门 (5) SBOM/secret 扫描。
- 门禁: 任一 BLOCKER 门红即不可合入。
- 验证: CI 全绿 + 审计报告。

---

## 8. 开发纪律落地

### 8.1 单行动检查单 (贴在 PR 模板)

```
ACT 前: [ ] 层与方向 [ ] ADR/质量标签 [ ] 影响面 (facade/合约/迁移) [ ] SIM预演 (P1+任务必有SIM号，见BLUEPRINT D-14)
ACT 中: [ ] 最小改动 [ ] 错误路径 [ ] 无 unwrap/expect [ ] 无阻塞 async
ACT 后: [ ] fitness+层脚本 [ ] 文档同步 [ ] 可回滚 [ ] 度量更新
```

### 8.2 门禁接线表

| 门 | 位置 | 命令/工具 | 失败处置 |
|----|------|-----------|---------|
| 编译 | pre-commit + CI | cargo check --lib -p neotrix | 不可提交/合入 |
| 层依赖 | CI (已补) | bash scripts/check-layer-deps.sh | 按 NODE-L1 卡消减 |
| 约束测试 | CI | cargo test architecture_constraints | 修引用或申请豁免 ADR |
| 适应度 | CI/生产 | SelfTestRegistry (6+5) | 报警即任务, 不静默 |
| Lint | CI | clippy -D warnings + fmt --check | 本地复现后修 |
| 依赖 | CI | cargo deny + audit | 升级/替换/豁免 ADR |
| 安全 | CI | security-scan + gitleaks + SBOM | BLOCKER 当日清 |
| 覆盖 | CI (P3 起卡点) | llvm-cov, 阈值 80% | 补测 |
| 基准 | CI (P3 起卡点) | criterion, 回退 >5% 阻断 | 优化或 ADR 豁免 |

### 8.3 新增规则生效方式

R-P230-R-P240 不只挂文档, 每条绑定: fitness 函数或脚本或 CI 门 + Owner。
R-P230->Confidence 类型+单测; R-P231/R-P238->PipelineIndependence;
R-P232/R-P236/R-P240->DocDrift; R-P233/R-P239->SecurityMitigation;
R-P234->事件 validate; R-P235->test-coverage-check; R-P237->deny 置信报告。

---

## 9. 度量仪表盘

| 指标 | 基线 (W0) | M1 | M2 | M3 | M4 | 采集 |
|------|----------|----|----|----|----|------|
| 编译错误 | 12 | 0 | 0 | 0 | 0 | cargo check |
| L1 越层数 | 实测 FAIL (分类中) | 环比 -50% | 归零或全豁免 | 保持 | 保持 | check-layer-deps |
| SDB 覆盖 | 未登记 | 登记 100% | 测试 100% | 保持 | 保持 | 登记表 |
| 覆盖率 | 约 75% | 维持 | >= 80% | 维持 | 维持 | llvm-cov |
| 自愈 MTTR | 约 25s | 可测 | 可测 | < 30s 可复现 | 保持 | chaos |
| P99 (L1 网关) | 待补 | 基线 | < 200ms | 保持 | 保持 | criterion/otel |
| 适应度评分 | 85 | 87 | 90 | 92 | 95 | fitness |
| ADR 覆盖 | 部分 | 增量 100% | 100% | 100% | 100% | 审计 |
| 安全 BLOCKER | 待扫 | 清零 | 清零 | 清零 | 清零 | scan |

每周五更新一次, 数字只增不猜, 缺数标 TBD 不填 0。

---

## 10. 风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| 越层下沉面大, 改动扩散 | 编译反复红 | 一次一文件, re-export 别名过渡, 每步 check |
| SDB 加严导致行为变严 (误拦) | 功能回退 | fail-closed + 豁免 ADR + 灰度名单, 全审计 |
| 大模块拆分引入循环 | 新越层 | 先画边界 ADR, 后搬代码, fitness 即时跑 |
| 基准门误伤正常波动 | 合入阻塞 | 3 次复跑取中位, 波动大项标 flaky 隔离 |
| 文档 100% 运动战 | 质量水 | 模板+抽查, 先核心路径后全量 |

---

## 附录

### A. 文件落地清单

```
docs/architecture/NEOTRIX-PYRAMID-ARCHITECTURE.md   目标 (WHERE)
docs/architecture/NEOTRIX-REFACTORING-GUIDE.md      标准 (WHAT)
docs/architecture/ABSORPTION-GRAPHIFY.md            模式 (HOW)
docs/architecture/NEOTRIX-IMPLEMENTATION-ROADMAP.md 执行 (WHEN/WHO/DONE) <- 本文件
docs/standards/NEOTRIX-STD-1.0.md                  正典 (NT-STD 1.0, 64 条款)
docs/dev-rules.md (桩)                              归档指向 (原 R-P161-257 见 archive)
scripts/check-layer-deps.sh                         层门 (CI 已引用)
neotrix-core/src/l5_cognition/nt_core_arch_fitness.rs  适应度 (6 生效 +5 待编)
neotrix-core/tests/architecture_constraints.rs      约束测试 (生效中)
docs/standards/NEOTRIX-STD-1.0.md                   正典 (NT-STD 1.0, 64 条款)
docs/standards/templates/                           新规则模版套件 (SIM/模块/适应度/ADR索引)
docs/standards/archive/                             旧规则归档 (只读, R-P1-110 + R-P161-257)
docs/architecture/SIM-PROTOCOL.md                   SIM 登记 (SIM-01 起, 九格制)
```

### B. ADR 待办 (先写后做)

- ADR: 越层三选一分类总表 (P1-01)
- ADR: L0 下沉类型清单 (P1-02)
- ADR: SDB 登记与 fail-closed 策略 (P1-03)
- ADR: 搜索统一选型 (P2-01)
- ADR: World/Memory 拆分边界 (P2-02)
- ADR: 覆盖率/基准阈值 (P3-02)

无 ADR 不动手, ADR 无质量标签不评审。

### C. 命令速查

```
cargo check --lib -p neotrix            单 crate 快检
cargo test -p neotrix --lib             单测
bash scripts/check-layer-deps.sh        层依赖快检
cargo test -p neotrix --test architecture_constraints   约束测试
cargo deny check                        依赖审计
cargo clippy --all-targets --all-features -- -D warnings  严 lint
cargo fmt --all -- --check              格式门
```

---

*End of Roadmap v1.0.0*

# NeoTrix 研发闭环 — 初赛方案 PPT（官方模板 8 章对齐版）

**赛道**：Agent Infra · 方向三：软件研发全流程协同
**项目名称**：NeoTrix 研发闭环 — 软件研发全流程多 Agent 协同系统
**格式**：沿用官方《初赛方案 PPT 内容框架模板》8 章结构，19 页（含封面/P0/目录/结尾）
**排版 v3（当前）**：内容页删除模板占位图片，改为**原生矢量流程图**（圆角矩形+箭头）——图为主、文字辅助、杜绝遮挡；矢量在 OfficeCLI 渲染链路任意缩放清晰。
**设计系统**：`scripts/ppt_theme.py`（PPT 风格设计大师引擎）+ `scripts/build_ppt_v3.py`。
- 主题推导：`derive_theme(项目名, 背景)` 按关键词信号匹配主题；本项目命中 **Consciousness · 意识流**（navy 底 + amber 沉淀/自进化 + primary blue 执行 + positive green 价值 + negative red 痛点）
- 字体层级：kicker 12pt（小节标签，字距 0.12em）/ lead 11.5 / h1 12.5（卡片标题）/ body 10.5 / support 10 / micro 9
- 语义化配色跨页一致：blue=执行协作 · amber=沉淀自进化 · green=价值开源 · red=痛点风险 · teal/cyan=辅助
- 可复用：换任意项目背景即得全新风格主题（已验：DeepMind→意识流、量子→卦象、DevOps→深空工程、银行→商务、博客→极简）

> 对应模板页：slide1 封面 · slide2 P0 一页纸速览 · slide3 目录 · slide4-5 第一章 · slide6-7 第二章 · slide8-9 第三章 · slide10-11 第四章 · slide12-13 第五章 · slide14-15 第六章 · slide16-17 第七章 · slide18-19 第八章

---

## slide1 · 封面
- 项目名称：NeoTrix 研发闭环
- 标语：*The only open-source agent that measures, analyzes, and improves its own reasoning.*
- 赛道/方向/团队（个人参赛 · 郑州大学）/日期 · GitHub: `github.com/neo-trixs/NeoTrix`（MIT）

## slide2 · P0 一页纸速览（作品简介 500 字浓缩）
- 项目名称 ≤20 字：NeoTrix 研发闭环
- 问题与场景：缺陷链路碎片化（Issue/日志/反馈多源分散、根因靠人工、修复无量化门禁、复盘不沉淀）
- 核心解决方案：AgentTeams Manager-Workers + 5 职能 Agent 串「聚合→定位→修复→验证→复盘」闭环，E8 确定性推理 + experience-tree 自进化
- 创新点与差异化：确定性 64 态推理内核（对照 OpenHands/SWE-agent 概率式方案）；4240 测试质量门禁；每次复盘自动结晶为 Skill
- 开放/复用价值：MIT 开源、Skill 体系独立可复用、RAG/可观测可迁移
- 当前进展：方案设计完成，7 域架构 + 4240 测试 + MCP/Skill/KB 基础设施可支撑复赛 Demo

## slide3 · 目录
1 场景与价值 · 2 方案总览 · 3 多 Agent 协同设计 · 4 Skills 工具体系 · 5 工程落地运行验证与安全可审计 · 6 开源开放计划 · 7 落地计划与进展 · 8 团队介绍

---

## 第一章 · 场景与价值（对应评分维度 25%）

### slide4 章节页
- 第一章 · 场景与价值 · 对应评分维度：场景价值与行业可复制性 25%

### slide5 内容页（v2 图：痛点→方案→价值 三栏）
- **图**：左「现状痛点」3 框（多源分散/根因靠人工/无量化）→ 中「NeoTrix 方案」AgentTeams 5 Agent → 右「可量化价值」3 框（定位 5-10min/T1-T3 门禁/Skill 沉淀）
- 顶部一条：目标用户 + 真实场景一句话；底部两条辅助：行业可复制性 + 差异化（对照 OpenHands/SWE-agent）
- 可量化价值：定位耗时（对标人工 30-60min，Demo 目标 5-10min）、修复成功率、门禁通过率、知识沉淀条数

## 第二章 · 方案总览（承上启下）

### slide6 章节页
- 第二章 · 方案总览

### slide7 内容页（v2 图：五层架构图）
- **图**：任务输入层 → AgentTeams 编排层 → Skill 能力层 → MCP/工具层 → 证据与治理层（左标签+右说明，向下箭头贯通）
- 各层内容：AgentTeams（Manager+A1-A5，Matrix 房间全程可见）；Skill（官方用云 Skills + dev-implementer/rev-officer/experience-tree 等）；MCP（GitHub/CI/监控，Higress 托管凭证）；证据治理（MinIO 共享状态、trace_data、SHA-256 审计链、KB）
- 底部一条：关键技术选型必要性（AgentTeams=必选协同基点；Higress=统一网关+凭证；MinIO=共享上下文降 Token 消耗）

## 第三章 · 多 Agent 协同设计（对应评分维度 25%）

### slide8 章节页
- 第三章 · 多 Agent 协同设计 · 对应评分维度：多 Agent 协同与自主闭环能力 25%

### slide9 内容页（v2 图：协同闭环流程图）
- **图**：Human → Manager 主控 → 上下文传递载体；A1 采集→A2 诊断→A3 实施→A4 审查 横向流水线；下方反馈「A4 审查不通过→打回 A3 重试」；A5 沉淀→KB→Skill 回灌 Manager（自进化闭环）
- 底部两条辅助：状态流转（Task 状态机，封顶升级人工）+ 安全边界（高风险动作人工审批 · Higress 消费者令牌 · SHA-256 审计链）
- 上下文传递：MinIO `shared/tasks/<id>/` 共享工作区 + Matrix 房间时间线 + KB 结构化中间结论；Worker 无状态可替换

## 第四章 · Skill 工程体系（对应评分维度 25% · 本赛题必选项）

### slide10 章节页
- 第四章 · Skill 工程体系 · 对应评分维度：Skill 工程体系与生态复用 25%

### slide11 内容页（v2 图：Skill 四类分组 2x2）
- **图**：官方用云 Skills（必选）/ 编码·实施 / 审查·治理 / 知识·吸收 四格（标题色带+条目区）
- 官方用云 Skills：alibabacloud-resourcecenter-search / ecs-diagnose / network-reachability-analysis / sas-overview（4 个串联即排障链路，满足必选要求）
- 底部两条辅助：生命周期（SKILL.md 装载→分发→版本/回滚 安全审核→灰度→审计）+ 复用价值（跨 Agent 跨场景；Skill 为任务能力抽象层）
- 对 AgentTeams 协同基点的落实：Skill 以 SKILL.md 装载 Worker 工作区，Manager 按需分发（worker-skills 目录）

## 第五章 · 工程落地、运行验证与安全可审计（对应评分维度 20%）

### slide12 章节页
- 第五章 · 工程落地、运行验证与安全可审计 · 对应评分维度：工程落地与安全可审计 20%

### slide13 内容页（v2 图：四象限）
- **图**：可运行性 / 运行证据 / 可观测 / 安全治理 2x2（标题色带+要点区）
- 可运行性：cargo build/check/test 双验证、Docker 部署、install.sh、CI workflows
- 运行证据：4240 项测试、SelfTest T1-T3 三层接线、日志/Trace/Metrics 全记录（trace_data + 广播审计链）
- 可观测：Skill/MCP/RAG/LLM 推理全链路 Trace；Log 结构化关联 TraceId；Metrics（修复成功率/端到端时延/Token 消耗/Tool 成功率）
- 安全治理：RAG 证据强制溯源（KB 向量+BM25）、权限矩阵/审批/回滚/审计、gitleaks 密钥扫描、零 unsafe
- 底部一条：云产品选型（Higress 网关统一入口+凭证托管，可替换性/迁移成本已论证）

## 第六章 · 开放 / 开源计划（对应评分维度 5%）

### slide14 章节页
- 第六章 · 开放 / 开源计划 · 对应评分维度：开放/开源贡献 5%

### slide15 内容页（v2 图：开源四步横向流程）
- **图**：可复用成果 → 接口契约与文档 → 协议与依赖披露 → 社区共建 四步流程（标题色带+要点区）
- 可复用成果：Skill 体系独立发布、mcp-gateway 网关、KB 检索层、E8 推理内核
- 底部两条辅助：开源范围（核心引擎+Skill+网关+示例与运行报告均可复用验证）+ 合规披露（数据来源与授权边界、第三方依赖、商业 API、闭源模型使用范围）

## 第七章 · 落地计划与进展

### slide16 章节页
- 第七章 · 落地计划与进展

### slide17 内容页（v2 图：里程碑时间线）
- **图**：初赛 8.16 → 复赛名单 8.24 → 复赛 9.3 → 决赛名单 9.10 → 决赛 9.22 五节点时间线（色带头+要点区）
- 当前进展：方案设计完成；7 域架构 23.6 万行 + 4240 测试 + MCP/Skill/KB 基础设施
- 中间一条：复赛工程化（AgentTeams 本地 install.sh→K8s helm；GitHub Issue 端到端 Demo；SWE-bench-style 评测；官方 Skills 接入）
- 底部两条辅助：评估指标（修复成功率/端到端时延/Token 成本/门禁通过率/经验沉淀速率）+ 风险控制（Mock 与真实接入共用同一 Schema；评审口径对齐评分维度）

## 第八章 · 团队介绍

### slide18 章节页
- 第八章 · 团队介绍

### slide19 内容页
- 团队名称：NeoTrix（个人参赛 · 郑州大学）
- 成员：Asher（郑州大学·软件开发/AI Agent 方向）
- 团队分工：Asher — 主控/架构 · Agent/Skill 工程 · Demo/验证（个人参赛，一人承担全流程）
- 团队成果：NeoTrix 开源项目（4240 测试、RQGM 论文 arXiv:2606.26294 引用）、过往获奖/项目经历
- 作品合集链接：GitHub 仓库

---

## 附：与官方模板的差异对齐说明
- 模板目录第 8 项为「Demo 视频（如有）」→ 决赛/复赛补，初赛该页以团队介绍收尾
- 模板第 4 章名「Skills 工具体系」→ 内容含官方用云 Skills + 自定义 Skill 两层，双满足必选与复用维度

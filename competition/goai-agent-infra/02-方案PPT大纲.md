# NeoTrix 研发闭环 — 初赛方案 PPT（官方模板 8 章对齐版）

**赛道**：Agent Infra · 方向三：软件研发全流程协同
**项目名称**：NeoTrix 研发闭环 — 软件研发全流程多 Agent 协同系统
**格式**：沿用官方《初赛方案 PPT 内容框架模板》8 章结构，19 页（含封面/P0/目录/结尾）

> 对应模板页：slide1 封面 · slide2 P0 一页纸速览 · slide3 目录 · slide4-5 第一章 · slide6-7 第二章 · slide8-9 第三章 · slide10-11 第四章 · slide12-13 第五章 · slide14-15 第六章 · slide16-17 第七章 · slide18-19 第八章

---

## slide1 · 封面
- 项目名称：NeoTrix 研发闭环
- 标语：*The only open-source agent that measures, analyzes, and improves its own reasoning.*
- 赛道/方向/团队（≤3 人）/日期 · GitHub: `github.com/neo-trixs/NeoTrix`（MIT）

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

### slide5 内容页
- 目标用户与核心痛点：企业研发团队/开源维护者；缺陷定位依赖资深工程师经验、修复质量无法量化验证、复盘经验散落
- 真实场景：一个真实 GitHub Issue → 采集聚合 → E8 根因定位 → TDD 修复 → 独立审查 → 复盘沉淀
- 可量化价值收益：定位耗时（对标人工 30-60min，Demo 目标 5-10min）、修复成功率、门禁通过率、知识沉淀条数
- 行业可复制性：任何有 Issue+CI+代码仓库的组织可复制；可迁移 IT 服务/嵌入式/金融科技研发线
- 创新点与差异化（对照 OpenHands/SWE-agent）：①E8 确定性推理可复现；②SelfTest T1-T3 生产门禁；③experience-tree 自进化沉淀

## 第二章 · 方案总览（承上启下）

### slide6 章节页
- 第二章 · 方案总览

### slide7 内容页（一页架构图）
- AgentTeams 编排层（Manager + A1-A5 Worker，Matrix 房间全程可见）
- Skill 能力层（官方用云 Skills + dev-implementer/rev-officer/experience-tree 等）
- MCP/工具连接层（GitHub/CI/监控，Higress 托管凭证）
- 证据与治理层（MinIO 共享状态、trace_data、SHA-256 审计链、KB）
- 关键技术选型必要性：AgentTeams=必选协同基点；Higress=统一网关+凭证；MinIO=共享上下文降 Token 消耗

## 第三章 · 多 Agent 协同设计（对应评分维度 25%）

### slide8 章节页
- 第三章 · 多 Agent 协同设计 · 对应评分维度：多 Agent 协同与自主闭环能力 25%

### slide9 内容页
- **Agent 分工**：Manager（拆解/委派/追踪）+ A1 Collector + A2 Diagnostician + A3 Implementer + A4 Auditor + A5 Distiller（详见附录 A）
- **任务拆解**：Manager 将研发任务拆为 SubTask 队列（collect→diagnose→implement→audit→distill），映射 AgentTeams `task-management`/`task-coordination` Skill
- **上下文传递**：MinIO `shared/tasks/<id>/` 共享工作区 + Matrix 房间时间线 + KB 结构化中间结论；Worker 无状态，状态在对象存储
- **状态流转**：Task 状态机（待办→拆解→执行→审查→沉淀），Manager 经 Matrix 追踪，失败打回重试
- **异常与冲突**：审查不通过→打回 A3（重试封顶升级人工）；多方案冲突→Manager 汇总供人类裁决
- **高风险动作安全边界**：改生产/大范围重构/删数据→人工审批（approval.rs）；Worker 不持真实凭证（Higress 消费者令牌）

## 第四章 · Skill 工程体系（对应评分维度 25% · 本赛题必选项）

### slide10 章节页
- 第四章 · Skill 工程体系 · 对应评分维度：Skill 工程体系与生态复用 25%

### slide11 内容页
- **官方用云 Skills**：复用 alibabacloud-resourcecenter-search / ecs-diagnose / network-reachability-analysis / sas-overview / data-agent-skill，4 个串联即排障链路（满足必选要求）
- **核心 Skill 清单与规格**：dev-implementer / rev-officer / repair-healer / experience-tree / mcp-gateway / github-operations（详见附录 B：输入输出/依赖/失败处理/安全边界逐项）
- **复用性**：Skill 为任务能力抽象层，多 Agent 多场景复用；版本/发布/回滚经 skills_index + 安全审核→灰度→回滚
- **对 AgentTeams 协同基点的落实**：Skill 以 SKILL.md 形式装载到 Worker 工作区，Manager 按需分发（worker-skills 目录）

## 第五章 · 工程落地、运行验证与安全可审计（对应评分维度 20%）

### slide12 章节页
- 第五章 · 工程落地、运行验证与安全可审计 · 对应评分维度：工程落地与安全可审计 20%

### slide13 内容页
- **可运行性**：cargo build/check/test 双验证、Docker 部署、install.sh、CI workflows
- **运行证据**：4240 项测试、SelfTest T1-T3 三层接线、日志/Trace/Metrics 全记录（trace_data + 广播审计链）
- **可观测**：Skill/MCP/RAG/LLM 推理全链路 Trace；Log 结构化关联 TraceId；Metrics（修复成功率/端到端时延/Token 消耗/Tool 成功率）
- **RAG/检索链路**：KB nodes/edges + 向量 + BM25 混合检索；证据强制溯源
- **安全治理**：权限矩阵、审批、回滚、审计、密钥（gitleaks）、零 unsafe
- **云产品选型**：Higress（网关）必选必要性说明 + 可替换性论证

## 第六章 · 开放 / 开源计划（对应评分维度 5%）

### slide14 章节页
- 第六章 · 开放 / 开源计划 · 对应评分维度：开放/开源贡献 5%

### slide15 内容页
- 可复用成果：Skill 体系独立发布、mcp-gateway 网关、KB 检索层、E8 推理内核
- 接口契约与文档示例：README、部署说明、开源协议、示例配置、测试方法
- 开源协议与第三方依赖：MIT；披露全部依赖、商业 API 调用、闭源模型、数据授权边界

## 第七章 · 落地计划与进展

### slide16 章节页
- 第七章 · 落地计划与进展

### slide17 内容页
- **当前进展**：方案设计完成；7 域架构 23.6 万行 + 4240 测试 + MCP/Skill/KB 基础设施
- **里程碑**：8.16 初赛提交 → 8.24 复赛名单 → 9.3 复赛提交 → 9.10 决赛名单 → 9.22 决赛答辩
- **复赛落地计划**：AgentTeams 编排接入（本地 install.sh 起步→K8s helm）；GitHub Issue→修复→测试→复盘端到端 Demo；SWE-bench-style 评测集；官方 Skills 接入
- **风险控制**：Demo 环境不确定性（网络/凭证）→ Mock 数据 + 真实接入共用同一 Schema；评审口径风险→严格对齐评分维度
- **评估指标**：修复成功率、端到端时延、Token 成本、门禁通过率、经验沉淀速率

## 第八章 · 团队介绍

### slide18 章节页
- 第八章 · 团队介绍

### slide19 内容页
- 成员背景（学校/公司、岗位/专业）、核心技能
- 团队分工（≤3 人）：如 [姓名]—主控/架构，[姓名]—Agent/Skill 工程，[姓名]—Demo/验证
- 团队成果：NeoTrix 开源项目（4240 测试、RQGM 论文 arXiv:2606.26294 引用）、过往获奖/项目经历
- 作品合集链接：GitHub 仓库

---

## 附：与官方模板的差异对齐说明
- 模板目录第 8 项为「Demo 视频（如有）」→ 决赛/复赛补，初赛该页以团队介绍收尾
- 模板第 4 章名「Skills 工具体系」→ 内容含官方用云 Skills + 自定义 Skill 两层，双满足必选与复用维度

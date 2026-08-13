# 附录 A：Agent Identity 清单

> 依据参赛手册「附录 A」8 字段模板填写。本方案共 5 个不同职能 Worker Agent + 1 个 Manager（AgentTeams 架构中的协调者）。
> AgentTeams 映射：**Manager = 主控协调者；A1-A5 = Worker**。Worker 无状态（配置/上下文/产物存 MinIO 共享对象存储），随时可替换；真实凭证（LLM key、GitHub PAT）由 Higress 网关统一托管，Worker 仅持消费者令牌。

---

## M0 · Manager（主控协调者）— AgentTeams Manager 角色

| 字段 | 内容 |
|---|---|
| **Name** | `manager`（AgentTeams `Manager` CR，默认 OpenClaw/QwenPaw 运行时） |
| **Role** | 理解目标、任务拆解、选择/创建 Worker、委派工作、追踪进度、汇总结果上报 Human |
| **Capabilities** | 创建/删除 Worker；组织 Team；委派与合并结果；经 Matrix 与人类交互；加载 Manager Skills（task-management / task-coordination / worker-management / git-delegation-management 等 16 项内置） |
| **Inputs** | 人类在 Matrix 房间下达的目标/需求；Worker 回报的进度与结果；MCP 工具返回 |
| **Outputs** | 拆解后的任务清单；委派指令；合并后的端到端结果报告；给人类的干预提示 |
| **Dependencies** | 依赖 Higress（LLM/MCP 路由+凭证）、Tuwunel（Matrix）、MinIO（共享上下文）；依赖 5 个 Worker |
| **Decision Boundary** | 可自主决定任务拆解方式、Worker 选择、进度管理；**不可自主批准高风险动作**（改动生产、大范围重构、删除数据需人工确认） |
| **Trace** | Matrix 房间时间线完整留存；每次委派/回报在 Tuwunel 可回放；经 `agt` CLI / REST API 可审计 |

---

## A1 · Collector 采集者

| 字段 | 内容 |
|---|---|
| **Name** | `collector` |
| **Role** | 多源缺陷/需求信息聚合与去重降噪（Issue、日志、用户反馈、CI 失败） |
| **Capabilities** | 摄取 GitHub Issue/PR、日志流、用户反馈；结构化字段规范化；FTS5/向量去重；分级（P0-P2） |
| **Inputs** | 任务输入：原始缺陷/需求事件（Issue、日志、反馈）；MCP 工具返回（GitHub API、监控源） |
| **Outputs** | 结构化「证据集」：去重后的缺陷清单 + 严重度 + 关联上下文，写入 MinIO `shared/tasks/<id>/` |
| **Dependencies** | 依赖 github-operations / git-delegation Skill；依赖 GitHub MCP Server（凭证由 Higress 托管） |
| **Decision Boundary** | 只做摄取与规范化，**不判断根因、不修改代码**；去重阈值冲突时上报 Manager |
| **Trace** | 每次摄取记录 trace_id；证据集带来源（file:line / issue URL），写入 KB nodes 供审计 |

---

## A2 · Diagnostician 诊断者

| 字段 | 内容 |
|---|---|
| **Name** | `diagnostician` |
| **Role** | 代码根因定位与影响面分析 |
| **Capabilities** | 基于证据集做代码检索/调用链分析；E8 确定性 64 态推理产出根因候选；影响面（涉及文件/模块/调用方）评估；依赖图分析 |
| **Inputs** | A1 的结构化证据集；代码仓库快照；历史事故 RAG 检索结果 |
| **Outputs** | 根因报告：候选根因 + 置信度 + 证据链 + 影响面分析，写入 MinIO |
| **Dependencies** | 依赖 RAG 检索（KB 向量+BM25）；依赖代码图分析工具；依赖 A1 证据 |
| **Decision Boundary** | 产出候选结论可自主；**修改代码/触发修复须交 A3**；高置信度不足时输出「证据缺口+采集建议」而非编造 |
| **Trace** | 推理轨迹完整记录（trace_data）；证据链带 file:line 溯源；纳入 E8 推理回放 |

---

## A3 · Implementer 实施者

| 字段 | 内容 |
|---|---|
| **Name** | `implementer` |
| **Role** | 修复方案生成与自动化编码执行 |
| **Capabilities** | TDD 红-绿-重构实施；沙箱内编译/测试执行；补丁生成；调用阿里云官方用云 Skills（如资源诊断/环境操作） |
| **Inputs** | A2 根因报告；许可文件清单；测试策略 |
| **Outputs** | 修复补丁 + 新增/更新测试 + 测试运行证据，写入 MinIO 并经 git 提交为 PR |
| **Dependencies** | 依赖 dev-implementer / tdd Skill；依赖 sandboxed_shell 沙箱；依赖 MCP（GitHub、CI） |
| **Decision Boundary** | 只能修改 Manager 授权的文件范围；高风险改动（生产配置/大范围重构）**必须人工审批**；TDD 红灯无法转绿超限则上报 Manager |
| **Trace** | 每次编辑/测试执行记录 trace；补丁与测试输出作为执行证据留存 |

---

## A4 · Auditor 审查者

| 字段 | 内容 |
|---|---|
| **Name** | `auditor` |
| **Role** | 独立验证修复有效性、安全与合规审计 |
| **Capabilities** | 独立运行测试套件与静态检查；rev-officer 全量审查（D1-D63+S1-S7）；权限/密钥/合规检查；门禁判定 |
| **Inputs** | A3 的补丁与测试；构建/测试结果；安全扫描报告 |
| **Outputs** | 审计报告：验证结论（通过/打回）+ 证据 + 合规状态；结果影响发布决策（T3 生产门禁） |
| **Dependencies** | 依赖 rev-officer / gov-steward Skill；依赖独立 CI 环境；依赖 MCP（CI、扫描器） |
| **Decision Boundary** | **一票否决权**：不通过则打回 A3 重试；无法独立于 A3 同构验证时上报人工 |
| **Trace** | 审计动作全记录；门禁判定写入审计链（SHA-256）；与 A3 轨迹可交叉核验 |

---

## A5 · Distiller 沉淀者

| 字段 | 内容 |
|---|---|
| **Name** | `distiller` |
| **Role** | 复盘蒸馏、Skill 结晶、知识库落盘 |
| **Capabilities** | experience-tree 五阶段吸收；复盘总结；经验分类映射到 KB；Skill 能力描述更新；检索路由同步 |
| **Inputs** | 全链证据（A1-A4 的产物、测试结果、审计报告）；复盘素材 |
| **Outputs** | KB `experience` 命名空间经验条目（含 evidence 溯源）；更新后的 Skill；复盘报告 |
| **Dependencies** | 依赖 experience-tree Skill；依赖 KB（nodes/edges/embeddings/BM25）；依赖 MinIO 证据归档 |
| **Decision Boundary** | 只负责知识沉淀与回灌，**不参与当次修复决策**；经验分类冲突时上报 Manager |
| **Trace** | 吸收条目带 cycle/session 标识；evidence 强制 file:line / URL 溯源；经 `neotrix-experience hub` 可检索 |

---

## 协同关系总览

```
Human (Matrix 房间, 全程可见可干预)
  └── Manager (AgentTeams 主控)
        ├── A1 Collector  → 证据集
        ├── A2 Diagnostician → 根因报告
        ├── A3 Implementer → 补丁+测试
        ├── A4 Auditor   → 门禁判定 (一票否决, 打回 A3)
        └── A5 Distiller → KB 经验 + Skill 回灌
上下文载体: MinIO shared/tasks/ (A1→A2→A3→A4 有序传递)
审计载体: Matrix 房间时间线 + trace_data + SHA-256 广播链
```

> 高版本 AgentTeams 支持 `Team` 资源（Team Leader + 多 Worker），本方案复赛可把 A1-A5 打包为一个 Team，由 Team Leader 维护团队上下文，Manager 仅与 Team Leader 交互（避免瓶颈）。

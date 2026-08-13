# 附录 B：Skill 清单

> 依据参赛手册「附录 B」10 字段模板填写。Skill 类型：**官方用云 Skill / 自定义 Skill / 外部工具封装**。
> 说明：官方用云 Skills 从阿里云 Agent Skills 门户安装，遵循 Agent Skills 开放规范，兼容 OpenClaw 等运行时，是 AgentTeams Worker 的标准能力来源。

---

## B1 · 官方用云 Skills（复用阿里云 Agent Skills 门户，满足必选要求）

| 字段 | 内容 |
|---|---|
| **Skill 名称** | `alibabacloud-find-skills`（+ 按需安装的具体云 Skills） |
| **Skill 类型** | 官方用云 Skill |
| **使用场景** | 场景一：A3 实施者定位需操作云资源（如复现依赖环境、诊断云上服务）；场景二：A1 采集者从阿里云监控/日志服务摄取缺陷信号 |
| **输入参数** | 用户云需求自然语言描述；待查云产品/资源 ID |
| **输出结果** | 匹配的官方 Skill 列表与安装命令 / 云资源操作结果 |
| **调用条件** | 检测到任务涉及阿里云资源操作且当前 Skills 未覆盖时触发 |
| **依赖工具 / 系统** | Aliyun CLI / OpenAPI；RAM 凭证（经 Higress 消费者令牌，Worker 不持真实 key） |
| **失败处理** | `Forbidden`→提示补 RAM 权限后重试；`ServiceNotEnabled`→开通服务后重试；均失败降级为只读查询 |
| **权限与安全** | 只授予最小权限 RAM 策略；高风险写操作经 HITL 人机协同检查点确认 |
| **复用价值** | 全部 Agent 可复用；替代自建「云环境操作」Skill，降低构建成本 |

**拟复用官方 Skills 清单（复赛实际接入）**：

| 官方 Skill | 用途 | 对应 Agent |
|---|---|---|
| `alibabacloud-resourcecenter-search` | 全局资源搜索（定位受影响云资源） | A2 诊断者 |
| `alibabacloud-ecs-diagnose` | ECS 实例诊断（复现环境排查） | A3 实施者 |
| `alibabacloud-network-reachability-analysis` | 网络可达性分析（调用链中断排查） | A2 诊断者 |
| `alibabacloud-sas-overview` | 安全态势查询（合规审计输入） | A4 审查者 |
| `alibabacloud-data-agent-skill` | 数据接入与治理（日志/反馈数据摄取） | A1 采集者 |

> 4 个官方 Skill 串联即形成「资源搜索→实例诊断→网络分析→安全态势」排障链路，与方向三「根因定位→影响面→验证」闭环天然契合。

---

## B2 · 自定义核心 Skill 清单

### S1 · dev-implementer（TDD 实施）

| 字段 | 内容 |
|---|---|
| **Skill 名称** | `dev-implementer` |
| **Skill 类型** | 自定义 Skill（v2.3.0） |
| **使用场景** | A3 实施者生成修复代码（红-绿-重构）；任何需自动化编码的任务 |
| **输入参数** | 根因报告（缺陷描述、目标文件、期望行为）；技术约束；测试策略 |
| **输出结果** | 修复补丁 + 新增/更新测试 + 测试运行输出 |
| **调用条件** | A2 产出根因且文件范围已获授权时触发 |
| **依赖工具 / 系统** | 编译器/测试器；sandboxed_shell 沙箱；MCP（GitHub、CI） |
| **失败处理** | 红灯无法转绿→重写策略；borrow-checker/死桩检测失败→记录 failure archetype；重试封顶上报 |
| **权限与安全** | 仅白名单文件可改；高风险改动交审批门禁 |
| **复用价值** | 跨 A2/A3 复用；可独立发布为开源编码 Skill |

### S2 · rev-officer（全量审查）

| 字段 | 内容 |
|---|---|
| **Skill 名称** | `rev-officer` |
| **Skill 类型** | 自定义 Skill（v6.0.0） |
| **使用场景** | A4 审查者独立验证；发布前健康检查 |
| **输入参数** | 变更 diff、构建/测试结果、安全扫描输出 |
| **输出结果** | 证据化审查报告（D1-D63+S1-S7，每项带 file:line 证据） |
| **调用条件** | A3 提交补丁后由 Manager 调度 |
| **依赖工具 / 系统** | git、构建链、测试器；KB 历史经验 |
| **失败处理** | 阻断并打回 A3；不可裁决项升级人工 |
| **权限与安全** | 只读；不触碰生产 |
| **复用价值** | 通用审查器，跨项目复用 |

### S3 · repair-healer（自愈修复）

| 字段 | 内容 |
|---|---|
| **Skill 名称** | `repair-healer` |
| **Skill 类型** | 自定义 Skill |
| **使用场景** | A2/A3 协同的故障定位-修复-恢复验证闭环 |
| **输入参数** | 缺陷检测结果、系统状态 |
| **输出结果** | 修复动作 + 恢复验证证据 |
| **调用条件** | 检测到缺陷且风险等级允许自动处理 |
| **依赖工具 / 系统** | 诊断器、测试器、MCP |
| **失败处理** | 升级人工；记录 repair pattern |
| **权限与安全** | 高风险需审批 |
| **复用价值** | 运维与研发场景通用 |

### S4 · sg-diagnostician（元认知自审计）

| 字段 | 内容 |
|---|---|
| **Skill 名称** | `sg-diagnostician` |
| **Skill 类型** | 自定义 Skill |
| **使用场景** | A5 沉淀者前置健康检查；系统自检 |
| **输入参数** | 会话/周期数据 |
| **输出结果** | 健康评分、技能状态、矛盾检测 |
| **调用条件** | 周期触发 / 异常触发 |
| **依赖工具 / 系统** | KB 检索 |
| **失败处理** | 修正策略并记录 |
| **权限与安全** | 只读 |
| **复用价值** | 系统级通用 |

### S5 · experience-tree（五阶段吸收）

| 字段 | 内容 |
|---|---|
| **Skill 名称** | `experience-tree` |
| **Skill 类型** | 自定义 Skill（统一吸收协议） |
| **使用场景** | A5 沉淀者复盘蒸馏：快照→蒸馏→分类→落盘→反馈 |
| **输入参数** | 全链执行证据、复盘素材 |
| **输出结果** | KB `experience` 命名空间经验条目 + 更新后的 Skill + 检索路由同步 |
| **调用条件** | 会话/任务收尾时触发 |
| **依赖工具 / 系统** | KB kv_store；`neotrix-experience` CLI |
| **失败处理** | 幂等重放；cycle 指针守恒校验 |
| **权限与安全** | 只写 experience 命名空间 |
| **复用价值** | 全局知识沉淀标准，任何 Agent 域复用 |

### S6 · mcp-gateway（MCP 工具聚合）

| 字段 | 内容 |
|---|---|
| **Skill 名称** | `mcp-gateway` |
| **Skill 类型** | 外部工具封装 |
| **使用场景** | 聚合 GitHub/CI/监控等 MCP server 到统一接口 |
| **输入参数** | 工具名 + 参数 Schema |
| **输出结果** | 标准化返回（含幂等键、失败重试信息） |
| **调用条件** | 任何 Agent 需要外部工具时 |
| **依赖工具 / 系统** | MCP 运行时（mcporter）；Higress MCP 托管 |
| **失败处理** | 降级直连；重试退避 |
| **权限与安全** | 工具白名单；凭证 Higress 托管 |
| **复用价值** | 工具层统一复用 |

### S7 · github-operations（GitHub 操作，AgentTeams 官方分发）

| 字段 | 内容 |
|---|---|
| **Skill 名称** | `github-operations` |
| **Skill 类型** | 外部工具封装（AgentTeams `worker-skills` 官方提供） |
| **使用场景** | A1 读取 Issue；A3 创建分支/PR；A4 检查 diff |
| **输入参数** | 仓库、issue 号、分支、PR 描述 |
| **输出结果** | issue/PR/分支操作结果 |
| **调用条件** | 需要 GitHub 交互时 |
| **依赖工具 / 系统** | GitHub MCP Server（Higress 托管，Worker 看不到 PAT） |
| **失败处理** | 3 次退避重试；权限错误提示配置 |
| **权限与安全** | 按 Worker 消费者令牌限权 |
| **复用价值** | 官方分发，多 Worker 复用 |

---

## Skill 生命周期管理

- **版本/发布/回滚**：Skill 以 `SKILL.md` + scripts 形式存于 AgentTeams 工作区/镜像；经 KB `skills_index` 索引；发布经安全审核→标签灰度→运行时加载→调用审计→快速回滚。
- **质量评估**：基于 trace 数据离线评估 Skill 调用成功率、工具选择合理性，驱动 Skill 版本迭代。
- **与多 Agent 协同流程关系**：Skill 是任务能力抽象层，MCP 是工具连接层；Agent 判断 → Skill 封装 → MCP 接入 → 证据回写，形成可治理调用链。

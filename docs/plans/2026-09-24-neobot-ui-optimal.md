# neobot UI 最优解 + 能力架构 map（2026-09-24，四源熔炼）

> 源：Cumora（3.9k★，agent 队聊）/ OpenMuse（1.9k★，个人 agent）/
> OpenBot（5.5k★，治理型 coworker）/ Grok iOS（极简转录体）。
> 体：Superbody Minimal light gold（浅金 token 单一事实源，不照搬 Grok 暗黑）；
> 取四家之骨（结构/纪律），不取其皮（配色）。

## 1. 熔炼表（源 → 取用模式 → 落点）

| 源 | 模式 | 落点 |
|---|---|---|
| Grok | 转录体（assistant 无泡、user 微泡+尾角）、空即设计、promptBar 神圣置底、signature hero 面（引用卡）、surface 值代阴影、单面无 tab（历史滑出/设置推入） | 面板布局总纲 |
| Grok | 流式块光标 ▍、发送↔停止同键（停后草稿保留） | 作曲区（OpenMuse 同款已验证） |
| Cumora | 同一面 roster/DM/群聊/看板/日历；seen-cursor 新鲜门；原子认领；outbox+SKIP LOCKED；BYOA fail-closed；llm_calls 成本账 | 任务/队友/协同三件 |
| OpenMuse | durable tasks（SQL lease 灾后恢复）+ pause/resume/cancel/retry；Ideas 带证据；Goals 回退；uncertain 外写无隐藏重试；适配器 pin+契约测 | 任务中心/审批 |
| OpenBot | **gateway 唯一路径（先裁决后记录，无记录不动作）**；CEL fail-closed（deny 先行、缺策略禁行、坏规则拒行）；take-the-wheel 交接事件；secret 不进 transcript；skills=指令非能力；routines 15min 地板+10 连败自停；可读审计 | 治理层全套 |

## 2. 三缺陷 → 最优解（验收口径）

- **聚焦冗余**（chat/活动/看板/通知抢焦点）→ 单主面：会话拥有整个视口；
  队友/历史滑出、设置推入；看板/日历是会话内的 signature 卡不是 tab。
  判定：首屏可交互焦点 ≤3（作曲区/会话/一 hero 卡）。
- **扁平缺陷**（万物同权重）→ 三级 elevation：canvas → surface1（user 泡/输入）→
  surface2（hero 卡/菜单），只用面值+1pt 边，不用阴影；
  hero 卡独占视觉重心（任务卡=我方引用卡）。
- **跨域错位**（聊/机/文件/skills 各说各话）→ 一对象流：NtTask 从作曲发出，
  经 gateway 裁决，状态机 running→done/cancelled/failed 全域同 ID；
  take-the-wheel/审批/审计全挂同一 task 事件流。

## 3. 极简面板 spec（Tauri 1200×800，浅金 token）

```
┌──────────────────────────────────────────────┐
│ 顶栏：●●● + 会话标题 + 自治仪表(小) + 设置…   │  h-12, surface1
│ 会话（转录体，assistant 无泡/user 右微泡）     │  canvas, 1.55 行高, turn 间 28pt
│ hero 卡（任务/审批/引用三选一，同位唯一）      │  surface2 + 1pt 边 + 8pt 圆
│ 作曲区 pills（@队友/模式）+ 输入 + 发送/停止同键│  48pt 起，5 行内胀，send↔stop
└──────────────────────────────────────────────┘
```
- token：canvas `#FAF7F2`（雪域白暖）、surface1 `#FFFFFF`、surface2 `#F3EDE3`（浅金 tint）、
  边 `#E3DACA`、字 `#1F1B16`（正文对比 ≥7:1）、次 `#6E6459`、accent 沿用 `nt-io-500 #f0913a`
  仅用于发送键/进行态（Grok 式单点 chroma 纪律）。
- 字体：系统栈 + tabular-nums（des-ui 禁 Inter，Grok 用 Inter 恰是反例，不取）。
- 组件状态矩阵（des-ui Phase 3）：发送/停止/hero 卡/审批键必须 Default/Hover/Active/
  Focus（2px ring+2px offset）/Disabled（0.5+not-allowed)/Loading（骨架）/Error（处方式文案+重试）。
- Agent 原生（Phase 4）：透明层=tool 卡（NtTask 行内）/ 状态色区分对话流 vs 活动流 /
  长任务 stop / 失败重试回退 / 工具调用走 ApprovalPanel（手动·自动·规划三档）。

## 4. 能力架构 map（颗粒度 L0→L3，三缺陷修复点标 ★）

```text
L0 表面（单主面）
├── 会话视口（转录体）            ★治聚焦冗余：唯一主面
├── hero 卡槽（任务/审批/引用三选一）★治扁平缺陷：唯一重心
├── 作曲区（pills+发送/停止同键+草稿保持）
└── 滑出层（队友/历史）· 推入层（设置）     ★治聚焦冗余：非主面不抢位

L1 域（同一 NtTask ID 全域流转）            ★治跨域错位
├── 任务中心（durable + lease 恢复 + 重试回退）   ← OpenMuse + ntos NtTask
├── 队友（job/model/temp/joinAll + @全体）        ← ntos + Cumora roster
├── 审批（手动/自动/规划 + uncertain 外写禁默重） ← OpenBot gateway + ApprovalPanel
├── 审计（permit/refuse/fail + 规则归因可读）     ← OpenBot /admin/audit
└── 记忆/提示词库（system 注入 + builder starter）← ntos + OpenMuse personal context

L2 引擎
├── 网关（唯一路径：先裁决后记录；CEL fail-closed 三律）← OpenBot（比 ax allowlist 更细： initiator 上下文）
├── 路由（深特征 D1 + Graft push/pull）
├── 蒸馏（D2/D3：label_jev + NLL + mask LoRA）
├── 好奇（D4 L1/L2）+ 播放（NT-PLAY 游乐场）
└── 成本账（llm_calls/任务，Cumora 式）+ 熔断（circuit_breaker 已有）

L3 基建（常驻位，全齐，见 neobot-absorb-build §2）
├── 模型池（provider_manager failover）+ 本地（MLX/Qwen + sidecar :8149）
├── 市场（dsh/github 双发现 → skills intake）
├── pty/文件/剪贴板/通知/自启/单例/全局键/tray/updater
└── 秘密（keyring；secret 永不进 transcript，OpenBot 律）
```

## 5. 构建任务（接 B1–B10，UI 侧细化）

| 序 | 任务 | 验证 |
|---|---|---|
| U1 | 作曲区（发送/停止同键 + 草稿保持 + pills） | 停后草稿在（OpenMuse 级） |
| U2 | 转录体会话 + hero 卡槽（三选一互斥） | 首屏焦点 ≤3；AA 对比表 |
| U3 | NtTask 全域 ID（作曲→网关→任务中心→审计同一条） | 跨域错位抽查：改状态三处同变 |
| U4 | gateway fail-closed 三律 + take-the-wheel 事件 | 坏规则拒行演练 |
| U5 | 滑出/推入层 + 设置页备份三段式 | ntos parity |
| U6 | verify-glass 视觉门（浅金 token 锚定，对比度 debt 清零） | 截图门过 |

## 6. 定位修正：Team AI（2026-09-24 owner 拍板，个人 App 降级）

主蓝本切换：**Cumora（骨架）> OpenBot（治理）> OpenMuse（个人层模式）> Grok（皮）**。
ntos 的队友/@全体/mateNames 快照证明 Team 基因早有，定位只是追认。

Team AI 实质差异（个人版没有的 7 件）：

| # | 件 | 源 | 落点 |
|---|---|---|---|
| 1 | roster（人+agent 同表，DM/群聊同面） | Cumora | 侧栏队友条升级：presence + 当前任务态 |
| 2 | 原子认领 + seen-cursor 新鲜门（防践踏） | Cumora | NtTask 加 `claimed_by` + stale 回复 HOLD 重判 |
| 3 | 人/角两套权限（OpenBot /admin/people + SAML/OIDC 路由） | OpenBot | 设置页：成员管理 + 域路由（远期；近先单管理员工位） |
| 4 | 按成员成本账（llm_calls 按 actor 切） | Cumora | D2 周报加 actor 维度 |
| 5 | 共享面 vs 私人面（频道可见性，OpenBot private/public coworker） | OpenBot | 会话加可见性位（私/队） |
| 6 | 例会 routines（15min 地板 + 10 连败自停） | OpenBot | 后台例行（daily-intel 即第一例会） |
| 7 | 交接事件审计（control_taken/released 记名） | OpenBot | 审计行加 actor 列 |

构建顺序调整：U1–U3 不变（作曲/转录/hero 形态中立）→ 插 U3.5 roster+认领 →
U4 网关（加 actor 上下文，initiator 规则：例会≠人可做的事）→ 成员管理进远期。
单管理员工位 = owner 即全员，先跑通再开多租户（OpenBot 同款起步：`OPENBOT_SINGLE_USER` 思想）。

## 7. 经验沉淀（des-ui L1：anti-patterns）

- SLOP-RELAPSE：Grok 用 Inter——我方禁，取其结构不取其字。
- AGENT-OPAQUE：OpenBot 证明"无记录不动作"可产品化，透明层不是装饰是问责。
- STATE-BLIND：OpenMuse SQL lease 证明 durable 不是口号，重启恢复必须演示。
- FOCUS-SPLIT（新增）：主面>1 焦点即失败，hero 卡三选一互斥是硬门。
- TEAM-SPLIT（新增）：个人与 Team 模式混用同一面必串味；roster/频道/认领是 Team 入场券，缺一即退回个人版。

## 8. 实施方案（Team AI，颗粒度到文件/命令/事件）

### 8.1 token 表（primitive → semantic，浅金，AA 实测口径）

```text
primitive: gold-100 #F7F1E6 / gold-200 #F3EDE3 / gold-300 #E3DACA /
           ink-900 #1F1B16 / ink-500 #6E6459 / accent #f0913a（nt-io-500 单一事实源）
semantic:  --canvas #FAF7F2 / --surface1 #FFFFFF / --surface2 gold-200 /
           --border gold-300 / --text ink-900 / --text2 ink-500
对比度：正文 ink-900/canvas ≈15:1 ✓ / 次要 ink-500/canvas ≈5.2:1 ✓(AA) /
       accent 配深字 ≈7.27:1 ✓ / accent 配白字 2.38:1 ✗（债务律：accent 永不衬白字）
间距：4pt 基（4/8/12/16/24/32），圆角 8pt（卡）/20pt（泡尾 6pt），字阶 13/16/28pt。
```

### 8.2 数据模型增量（3 处）

```ts
// 前端 store.ts 增补（沿 ntos_*_v1 键规范）
interface NtTask { claimed_by?: string; visibility: 'private'|'team'; }  // +认领+可见性
interface RosterMember { id: string; kind: 'human'|'agent'; name: string;
  presence: 'active'|'idle'|'off'; currentTaskId?: string; }             // 新键 ntos_roster_v1（presence 常驻内存，落盘只存表）
interface CostRow { actorId: string; taskId: string; tokens: number; ms: number; ts: number; } // 新键 ntos_cost_v1，上限 500
```
```rust
// 后端：NeobotStore 任务行 +claimed_by/+actor；审计行 +actor 列；
// 网关裁决入参 +initiator{actor, channel}（OpenBot 律：例会≠人）。
```

### 8.3 新增 IPC 命令（5 个，沿 commands/neobot.rs 薄封装范式）

```text
roster_list / roster_presence        # 表 + 心跳（30s 轮询，失联→idle→off）
task_claim { task_id, actor_id }     # 原子认领（DB 唯一约束，已认领返 holder，不覆盖）
task_release { task_id, actor_id }   # 释放（holder 或 owner 可放）
cost_ledger { range }                # 按 actor 聚合（D2 周报 actor 维度同源）
member_set_visibility { task_id, visibility }  # 私/队切换（默认队，私需 owner）
```
事件（沿 harness-progress 范式）：`task-claimed` / `presence-changed` /
`control_taken, control_released`（take-the-wheel 记名）/ `routine-fired`。

### 8.4 组件矩阵（文件级落点，存量路径）

| 组件 | 落点 | props/态 | 事件 |
|---|---|---|---|
| RosterSidebar | components/ 新增 `RosterBar.tsx`（复用 AgentActivityBar 行样式） | presence 三色点 + currentTask 一行；Empty（零成员引导建队友） | presence-changed |
| TaskHeroCard | hero 卡槽（与审批/引用三选一互斥，U2） | claimed_by 署名行 + 认领键（他人已认领→置灰“XX 认领中”+ 抢占需 owner） | task-claimed |
| ApprovalPanel | 存量扩展三档（手动/自动/规划）+ actor 上下文行 | initiatior 例会/人标识 | 同 task 事件 |
| ActivityLog | 存量行加 actor 列 + take-the-wheel 交接行 | 控制中态（Bot 动作拒收不排队，OpenBot 律） | control_* |
| Composer | pills 加 @队友/@频道 + 发送/停止同键 + 草稿保持 | 私/队切换小锁（默认队） | — |
| CostView | 设置页新 tab（CostRow 聚合表） | 按成员/周切 | — |

### 8.5 阶段验收（每段可独立交付，命令级）

| 段 | 内容 | 验证 |
|---|---|---|
| U1 | 作曲区（同键+草稿+pills+私队锁） | 停后草稿在；锁默认队 |
| U2 | 转录体 + hero 三选一互斥 + token 对比表 | 首屏焦点≤3；AA 全过；截图门 |
| U3 | NtTask 全域 ID（作曲→网关→中心→审计） | 三处同变抽查 |
| U3.5 | roster + 原子认领 + presence | 双开抢认领：后者被拒并显示 holder |
| U4 | 网关 actor 上下文 + fail-closed 三律演练 | 坏规则拒行；例会越权被拒 |
| U5 | 滑出/推入 + 备份（含 roster/cost） + 成员管理（单管理员工位先行） | 备份 roundtrip |
| U6 | verify-glass + e2e + B1–B10 联验 | 全绿发版 |

缺席项先天声明：多租户/SAML/域路由进远期；单管理员工位跑通前不碰。

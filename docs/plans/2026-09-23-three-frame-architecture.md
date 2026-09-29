# 三框终局架构（侧栏 × 对话 × 设置）2026-09-23

> 目标：把 Lingee／OpenMuse／unreal／buzz／cumora／ntos 全部吸收点，熔进三个框的最优解。
> 约束：纯前端可落地部分与需后端部分分离标注；零新组件（复用 tag-pill／ss-card／NeoTag）；测试不断言破裂。

## 一、现状盘点（文件锚点）

| 框 | 文件 | 行数 | 已有 |
|---|---|---|---|
| 侧栏 | `components/Sidebar.tsx` | 1228 | 工作/开发分段、5＋4 导航、历史三 tabs＋滑块、分组会话（时间副标）、用量卡、用户条（昵称＋plan 徽） |
| 对话 | `routes/Chat.tsx` | 2697 | 双作曲区、业务 pills、档位 pill、快捷 chips、跟进队列、权限徽、上下文％、6 面板（git/tasks/terminal/timeline/sidechat/cost） |
| 设置 | `components/SettingsModal.tsx` | 764 | 9 分区＋4 分组导航、深链接（section＋marketTab）、模型档位、用量条、DNS 池、ntcode 客户端就绪 |

## 二、吸收矩阵（特性→来源→落点→状态）

| # | 特性 | 来源 | 落点框 | 状态 |
|---|---|---|---|---|
| 1 | 纵向导航 264×32＋历史 sticky＋用户 plan 徽 | Lingee 取证 | 侧栏 | ✅ 已落地 |
| 2 | 11 档＋系数（前端只暴露档位名） | Lingee 档位层 | 设置模型页＋作曲区＋ntos | ✅ 已落地 |
| 3 | runtime-data 用量（token/credit/daily） | Lingee 用量 | 侧栏卡＋模型条＋关于看板 v1（聚合过渡） | ✅ v1；⬜ 后端字段 |
| 4 | 跟进队列（可见有序、停后手动恢复） | OpenMuse | 对话作曲区上 | ✅ 已落地 |
| 5 | client_msg_id 稳定 ID | unreal inbox | `chat.send` 第三参 | ✅ 前端已发；⬜ 后端幂等 |
| 6 | Agent 即成员身份表达 | buzz＋cumora 徽 | IM／消息作者徽 | ⬜ 需作者归因后端 |
| 7 | @参与者目录 | cumora MentionList | 作曲区 @（现仅文件） | ⬜ 需 IM bots 目录 |
| 8 | 发送/停止同键＋停止保草稿 | OpenMuse | 对话发送钮 | ✅ 已有（核对过） |
| 9 | per-session 草稿持久 | OpenMuse/cumora | 同上 | ✅ 已有 |
| 10 | 头像状态命名当前工作 | OpenMuse | 侧栏用户条／AgentActivity | ⬜ 纯前端可做 |
| 11 | 不强制滚屏＋最新消息控件 | OpenMuse | 对话消息区 | ⬜ 待确认现状 |
| 12 | tone 可编辑 | OpenMuse 个人上下文 | 设置通用（昵称旁） | ⬜ 纯前端可做 |
| 13 | 答案带 receipts | buzz | 对话消息 | ⬜ 需搜索 citations 后端 |
| 14 | 11 档 business_type 透传 | Lingee 网关头 | `chat.send` 字段 | ⬜ 需后端字段 |
| 15 | 市场 3 列卡＋Beta＋产物/点数 | Lingee 市场 | 市场分区 | ⬜ 等 P2-8 字段 |

## 三、终局线框（真实锚点＋实测尺寸）

### 3.1 侧栏（280 宽，Lingee 实测对齐）

```
┌─ aside（w 280，可拖 200–460；折叠 64＋hover peek） ──┐
│ h-7 拖拽区 · 折叠钮（aria-label 折叠侧边栏）          │  ← 现状保留
│ ┌─ 工作／开发分段 ───────────────────────────────┐  │
│ │ [💬 工作(active 白 pill)] [🛠️ 开发]             │  │  ← 现状；缺滑块动画（Lingee
│ └────────────────────────────────────────────────┘  │     slider 是白块滑动，现有
│ 工作视图：                                          │     瞬切。P1 做滑块复用
│  搜索行＋＋新建（工作专属）                          │     历史 tabs 同款机制）
│  导航纵列 h-8（新对话/定时任务⌾tasks面板/         │
│    智能体→模型/技能→市场installed/发现→市场discover)│
│  历史 tabs：历史对话／定时任务⌾／已归档＋橙滑块      │  ← 现状；滑块机制已验证
│  会话列表（分组可折叠＋拖拽排序＋pin＋双行时间副标）  │
│ 开发视图：新会话/技能开发/智能体开发/应用开发        │  ← 现状；目的地全真实
│ 用量卡（调用＋成功率条＋免费数＋升级›）              │  ← 现状；token 行等后端
│ 用户条：头像（昵称首字）＋昵称＋Free Plan 徽＋⚙      │  ← 现状（stores/username 单源）
└─────────────────────────────────────────────────────┘
```

### 3.2 对话（中央列 max-w 800）

```
┌─ 主区 ─────────────────────────────────────────────┐
│ 空态 hero（问候＋SUGGESTED 大 chips）                │  ← 现状保留
│ 会话态消息流（putong 气泡；⬜ Agent 徽／receipts）    │
│ 业务 pills：日常办公／业务分析／业务执行（空＋会话双区）│ ← 现状；⬜ send 透传字段
│ .cic 作曲区：                                        │
│   textarea（草稿 per-session 持久；停止保草稿）       │  ← 现状已核对
│   工具行：＋附件／ModelSwitcher／⚡档位 pill／发送⇄停止 │ ← 现状
│ 跟进队列条（排队中 n＋发送排队＋清空＋前3＋移除）      │  ← 现状（需二进制目验）
│ 快捷 chips 行（SUGGESTED 前4，会话态）               │  ← 现状
│ 状态条：权限徽／上下文％／模型／tok／OS badge        │  ← 现状
│ 6 面板（git/tasks/terminal/timeline/sidechat/cost）  │  ← 现状；定时任务复用 tasks
└────────────────────────────────────────────────────┘
```

### 3.3 设置（弹窗，左分组导航＋右内容）

```
┌─ SettingsModal（open＋initialSection＋initialMarketTab）┐
│ 左：常规（通用/模型/网络/IM/外观）扩展（市场）          │
│     数据（数据/标签）系统（关于）                      │  ← 现状
│ 右：通用（昵称卡＋默认模型＋语言＋密钥＋偏好）          │  ← 昵称已熔入
│     模型（Hero＋11 档＋分组池；⬜ business pills 联动） │
│     网络（链路图＋代理池＋系统/DNS＋36 DNS 池）        │  ← 现状
│     市场（installed/discover＋过滤＋Beta＋可更新＋待激活）│ ← 现状；⬜ 3 列＋产物
│     数据／标签／IM／外观（外观主题已删，强制浅色）      │  ← 现状
│     关于（版本＋更新＋诊断＋用量 v1 看板）             │  ← 现状；⬜ 真 token
└──────────────────────────────────────────────────────┘
```

## 四、架构（状态流＋IPC）

```
侧栏 ──rado──▶ Chat（currentSessionId 切换清跟进队列）
  │ stores/username 单源 ──▶ 通用设置昵称卡（同屏同步）
  │ stores/tags Carter 标签筛选 ──▶ 会话过滤
对话 ──▶ chat.send(content, session_id?, client_msg_id?) ──▶ domain 后端
  │ 生成中 → followUps 队列 → done 自动 drain／取消手动恢复
  │ onDone → 释放 hold → 滑块/用量重测（60s 轮询）
设置 ──▶ initialSection/initialMarketTab 深链接（侧栏直达 7 处）
ntcode 客户端（9 命令＋4 事件）就绪，切管线待二进制联调
```

## 五、实施顺序（剩余）

- **P0（纯前端）**：分段滑块动画（复用历史 tabs 机制到工作/开发分段）＋头像状态命名（AgentActivity 文案透出）＋tone 选择（昵称卡旁）
- **P1（小后端）**：`business_type` 透传＋`client_msg_id` 幂等消费＋用量 token 回填（usage_report）
- **P2（大后端）**：IM 作者归因＋AgentBadge、@参与者目录、receipts、市场 3 列＋产物/点数、session 存储版本化

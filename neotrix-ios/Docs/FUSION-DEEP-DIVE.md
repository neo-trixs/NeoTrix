# NeoGram Fusion Deep-Dive — 深度融合架构设计

> 状态: 提议 (proposed) | 日期: 2026-08-08 | 作者: NT-CORE (des-architect)
> 前置: `FUSION-ARCHITECTURE.md` (融合矩阵 v1) — 本文档是 v2 深度融合层
> 目标: 每个功能标签 = 多来源精华融合 → **NeoGram 独有交互特性**（非复制、非叠加，而是化学反应）

---

## 0. 融合哲学（Fusion Philosophy）

### 0.1 三层融合模型

```
┌─────────────────────────────────────────────┐
│  L1 功能融合 (Feature Fusion)                 │
│  多来源功能 → 单一功能（取精华）               │
│  例: 语音转文字 = Swiftgram免费 + Turrit翻译   │
├─────────────────────────────────────────────┤
│  L2 交互融合 (Interaction Fusion)             │
│  多来源交互模式 → 新交互模式（产生新体验）      │
│  例: 长按消息 = AI编辑环 + 快速格式化 + 翻译    │
├─────────────────────────────────────────────┤
│  L3 认知融合 (Cognitive Fusion)               │
│  AI 核心 (E8/GWT/VSA) 注入 → 认知增强交互      │
│  例: AI 语义过滤 = 关键词 + E8 意图理解        │
└─────────────────────────────────────────────┘
```

**核心原则**: 每个功能标签必须达到 L2+，否则只是复制。

### 0.2 融合判定标准

| 层级 | 判定 | 示例 |
|------|------|------|
| L1 复制 | 功能相同，无新交互 | 复制官方 Poll |
| L2 融合 | 多来源交互合并 | 长按消息 → AI 编辑环 |
| L3 认知 | AI 核心注入 | 语义过滤 + E8 意图 |

---

## 2. 功能标签深度融合设计

### 2.1 消息编辑环（AI Editor Ring）

**融合来源**: 官方 Cocoon AI + Swiftgram Quick Formatting + Turrit 发送前翻译 + Nicegram Lily AI

**独有交互**: 长按消息 → 弹出 **AI 动作环**（环形菜单，非线性列表）

```
        ┌── 修语法 ──┐
        │             │
   翻译 ─┤             ├─ 改写
        │  消息文本    │
  缩短 ──┤             ├─ 正式
        │             │
        └── 友好 ──┐  └─ Zen
                   └── Viking
```

**融合逻辑**:
- 官方提供 AI 编辑基础 → 我们改为**环形菜单**（单手拇指可达，区别于线性列表）
- Swiftgram 格式化面板 → 融合为**格式化 + AI 建议**（AI 检测到语法错误时自动建议）
- Turrit 发送前翻译 → 融合为**发送前 AI 检查**（E8 判断语气/敏感词，发送前提示）

**L3 认知增强**: E8 根据对话历史预测用户最可能用的动作，环形菜单**动态排序**（常用动作靠前）

### 2.2 智能过滤（Message Filter + AI 语义）

**融合来源**: Swiftgram Message Filter + Turrit 关键词屏蔽 + Nicegram 隐藏

**独有交互**: **AI 语义过滤** — 不只匹配关键词，E8 理解消息意图

```
用户输入: "屏蔽所有广告"
  → FilterEngine 解析意图 (E8)
  → 生成语义规则: {type: ad, confidence: 0.92}
  → 实时过滤: 广告消息自动折叠 + 标记
  → 用户可查看被过滤消息 (回收站)
```

**融合逻辑**:
- Swiftgram: 关键词规则（L1）
- Turrit: 关键词屏蔽 + 频道过滤（L1）
- **NeoGram: 语义规则 + 意图理解（L3）** ← 独有

**独有交互**: 被过滤消息进入 **AI 回收站**（可恢复），AI 定期总结"本周过滤了 23 条广告"

### 2.3 Ghost Mode（隐身模式）

**融合来源**: Nicegram Ghost Mode + Swiftgram 隐藏 + 官方隐私

**独有交互**: **AI 智能隐身** —— 不只隐藏在线状态，AI 判断何时隐身

```
Ghost Mode 开启:
  → 不发送已读回执 (L1)
  → 隐藏在线状态 (L1)
  → AI 智能通知: 重要消息才打扰 (L3)
  → AI 隐身调度: 根据对话重要性自动切换隐身/在线 (L3)
```

**融合逻辑**:
- Nicegram: 手动开关（L1）
- Swiftgram: 隐藏已读（L1）
- **NeoGram: AI 判断重要性 + 自动调度（L3）** ← 独有

### 2.4 智能文件夹（AI Folders）

**融合来源**: Swiftgram Folders + Nicegram Tabs + Turrit 侧边栏

**独有交互**: **AI 自动分类** —— E8 聚类聊天，自动生成文件夹

```
聊天列表 → E8 语义聚类 → 自动文件夹:
  ├─ 工作 (Slack 风格)
  ├─ 家人 (高频联系人)
  ├─ 新闻 (频道)
  └─ 待处理 (未读 + 重要)
用户可手动覆盖 → 覆盖后 AI 学习用户偏好
```

**融合逻辑**:
- Swiftgram: 手动文件夹（L1）
- Nicegram: 自定义 Tabs（L1）
- **NeoGram: AI 聚类 + 用户覆盖学习（L3）** ← 独有

### 2.5 多账号（Multi-Account）

**融合来源**: Nicegram 无限 + Swiftgram 无限 + Turrit 10

**独有交互**: **AI 账号路由** —— 根据上下文自动切换账号

```
用户输入: "给老板发消息"
  → E8 判断: 工作账号
  → 自动切换账号 + 发送
用户输入: "发朋友圈"
  → E8 判断: 个人账号
```

### 2.6 云存储（Cloud Drive）

**融合来源**: Turrit Cloud Drive + 官方 Saved Messages

**独有交互**: **KB 语义搜索云存储** —— VSA 向量检索

```
保存消息 → KB (VSA 向量化)
搜索: "上次说的那个方案"
  → VSA 语义检索 → 找到相关消息
  → 一键分享
```

### 2.7 通知中心（Smart Notifications）

**融合来源**: Swiftgram Disable @mentions + Nicegram 隐藏 + 官方

**独有交互**: **AI 通知优先级** —— E8 判断重要性

```
通知到达 → E8 评分 (0-100)
  → >80: 立即通知 (重要)
  → 50-80: 静默通知 (摘要)
  → <50: 不通知 (折叠)
用户可自定义阈值
```

### 2.8 视频流（Video Flow）

**融合来源**: Turrit TikTok 流 + 官方 Live Photos

**独有交互**: **AI 视频推荐** —— GWT 注意力路由

```
视频流 → GWT 注意力路由 → 推荐相关视频
→ 手势控制 (音量/亮度/进度)
→ AI 视频摘要
```

---

## 3. 独有交互特性清单（Unique Interaction Patterns）

| # | 交互特性 | 融合来源 | 独有性 |
|---|---------|---------|--------|
| U1 | **AI 动作环** (环形菜单) | 官方 AI + Swiftgram | 环形 + AI 排序 |
| U2 | **AI 语义过滤** | Swiftgram + Turrit | 意图理解 |
| U3 | **AI 智能隐身** | Nicegram + Swiftgram | 主动调度 |
| U4 | **AI 自动文件夹** | Swiftgram + Nicegram | 聚类 + 学习 |
| U5 | **AI 账号路由** | Nicegram + Turrit | 上下文切换 |
| U6 | **KB 语义搜索** | Turrit + 官方 | VSA 检索 |
| U7 | **AI 通知优先级** | Swiftgram + 官方 | E8 评分 |
| U8 | **AI 视频推荐** | Turrit + 官方 | GWT 路由 |
| U9 | **AI 回收站** | Swiftgram + Turrit | 语义过滤恢复 |
| U10 | **AI 语音总结** | Swiftgram + Turrit | 要点提取 |

---

## 4. 架构支撑

### 4.1 认知层 (Cognitive Layer)

```
┌─────────────────────────────────────────┐
│  Cognitive Layer (认知层)                │
│  AIHub (AI 中枢)                        │
│  ├─ E8 推理引擎 (意图/聚类/评分)          │
│  ├─ GWT 注意力路由 (推荐/过滤)            │
│  ├─ VSA 向量存储 (语义搜索)               │
│  └─ ConsciousnessTree (自进化)           │
└─────────────────────────────────────────┘
```

### 4.2 融合管线（Fusion Pipeline）

```
用户输入 → 意图识别 (E8) → 路由 (GWT)
  → 功能执行 (Feature Layer)
  → 结果融合 (多来源)
  → 认知增强 (E8/VSA)
  → UI 渲染
```

### 4.3 状态管理

```
AppState (全局) → FeatureState (功能) → UIState (视图)
  ↓                    ↓                    ↓
  AIHub               FilterEngine          ChatUI
  (AI 中枢)           (过滤)               (视图)
```

---

## 5. 实施路线图（更新）

| 阶段 | 内容 | 验证 |
|------|------|------|
| P0 (已完成) | UI 编译清零 + AIEditorUI + PollUI | swiftc 0 error |
| P1 | AIHub 中枢 + AI 动作环 | swiftc + 功能测试 |
| P2 | FilterEngine + PrivacyEngine + 智能文件夹 | 单元测试 |
| P3 | CloudDrive + ExportEngine + Voice-to-Text | 集成测试 |
| P4 | VideoFlow + 动态主题 + 无障碍 | 全量验证 |

---

## 6. 风险与未决项

| 风险 | 缓解 |
|------|------|
| AI 功能依赖 Rust 核心 | AIHub 降级策略 (本地规则) |
| 融合过度导致膨胀 | Dark Forest 原则 |
| 云存储后端选型 | 待定 (KB vs 自建) |
| 未决: AI 动作环动画性能 | 待验证 |

---

## 7. 实现差距审计（U1-U10 ↔ 现有代码）

> 审计日期: 2026-08-08 | 方法: 逐特性对照 `Sources/Features/` 与 `Sources/UI/`

| # | 独有交互 | 现有实现 | 差距 |
|---|---------|---------|------|
| U1 | AI 动作环 | `AIEditorUI.swift` 8 动作齐全 | ⚠️ 线性列表非环形菜单；无 E8 动态排序 |
| U2 | AI 语义过滤 | `FilterEngine.swift:46` `useAISemanticFilter` + `AIHub.routeFilter` | ✅ 已实现（关键词 + E8 语义） |
| U3 | AI 智能隐身 | `PrivacyEngine.swift:12` `ghostMode` + `smartNotifications` | ⚠️ 有开关，无 AI 主动调度 |
| U4 | AI 自动文件夹 | `FolderEngine.swift` 手动文件夹 + 记忆 | ⚠️ 无 E8 语义聚类分类 |
| U5 | AI 账号路由 | 无 | ❌ 未实现（需 AccountManager 扩展） |
| U6 | KB 语义搜索 | `ExportEngine.swift:93` `storeInKB` (VSA store) | ⚠️ 只有写入，无 VSA 查询检索 UI |
| U7 | AI 通知优先级 | `PrivacyEngine.swift:44` `smartNotifications` 布尔 | ⚠️ 无 E8 0-100 评分 |
| U8 | AI 视频推荐 | 无 | ❌ 未实现（P4） |
| U9 | AI 回收站 | 无 | ❌ 未实现（被过滤消息无恢复） |
| U10 | AI 语音总结 | `VoiceToTextUI.swift:16` `summary` | ✅ 已实现（转录 + 要点提取） |

### 差距分级与建议

| 级别 | 项 | 建议 |
|------|----|------|
| **P1 补强** (半实现) | U1 环形菜单 / U3 智能调度 / U4 聚类 / U6 查询 / U7 评分 | 在现有 Engine 上加方法，改动小 |
| **P2 新增** (未实现) | U5 账号路由 / U9 回收站 | 需要新模型 + 接线 |
| **P3 延后** (依赖平台) | U8 视频推荐 | 需完整 Xcode + AVKit 验证 |

### 补强实现指引（P1 快速落地）

- **U4 聚类**: `FolderEngine` 增加 `func aiClassify(chats:) async` → 调用 `AIHub.process(.classify)` → 按返回标签建文件夹
- **U7 评分**: `PrivacyEngine` 增加 `func notificationPriority(_ message:) async -> Int` → E8 评分 0-100 → 阈值决定通知方式
- **U6 查询**: `ExportEngine` 增加 `func searchKB(query:) async -> [String]` → `vsa.search(label:similarity:)`
- **U9 回收站**: `FilterEngine` 增加 `recycleBin: [FilteredMessage]` + `restore()` 方法
- **U1 环形菜单**: `AIEditorUI` 改 `contextMenu` → 自定义环形视图 + E8 排序

> 说明: 所有补强均需先经 `swiftc -typecheck` 验证（CLT 环境无 Xcode，见 FUSION-ARCHITECTURE.md §6）
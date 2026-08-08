# NeoGram Fusion Architecture — 融合架构设计

> 状态: 提议 (proposed) | 日期: 2026-08-08 | 作者: NT-CORE (des-architect)
> 目标: 融合 Telegram 官方 2026 + Nicegram + Swiftgram + Turrit + TelegramSwift 五大来源特性，
> 以 NeoTrix AI 核心 (E8/GWT/VSA/ConsciousnessTree) 为中枢，形成 NeoGram 独有的交互特性。

---

## 1. 设计原则

| 原则 | 说明 |
|------|------|
| **AI 中枢化** | AI 不是附加功能，而是所有交互的核心路由（区别于其他客户端"AI 是插件"） |
| **融合而非复制** | 每个功能标签 = 多来源精华 + NeoTrix 独有增强 |
| **模块化单体** | 团队小 + 快速迭代 → 模块化单体（沿用现有 5 模块结构） |
| **指针守恒** | 架构文档走 KB，AGENTS.md 只存规则 |
| **Dark Forest** | 每个模块必须有消费者，否则删除 |

---

## 2. 融合矩阵（功能标签 → 来源融合 → 独有交互）

### 2.1 消息与 AI 编辑

| 功能标签 | 融合来源 | NeoGram 独有交互 |
|---------|---------|-----------------|
| **AI Editor** | 官方 Cocoon AI + Swiftgram Quick Formatting + Turrit 发送前翻译 | 长按消息 → AI 8 动作环（修语法/翻译/改写/缩短/正式/友好/Zen/Viking），E8 推理实时注入，AI 上下文感知对话历史 |
| **Rich Text** | 官方 Rich Text Editor + Swiftgram Formatting Panel | 键盘上方格式化面板（B/I/链接/代码），AI 建议格式 |
| **AI 风格** | 官方 Custom AI Styles + Nicegram Lily AI | 用户自定义 AI 风格预设（Zen/Viking/Formal…），持久化到 KB |

### 2.2 互动与内容

| 功能标签 | 融合来源 | NeoGram 独有交互 |
|---------|---------|-----------------|
| **Mighty Polls** | 官方 10+ 投票功能 | AI 建议选项（E8 生成）、投票后 AI 总结、媒体/位置附件、限时/隐藏结果/洗牌 |
| **Live Photos** | 官方 Live/Motion Photos | Live/Loop/Bounce 三样式，AI 自动推荐样式 |
| **Stories** | 官方 Stories + Swiftgram Stories 控制 + Turrit 侧边栏 | 故事 AI 摘要、隐藏/禁用滑动录制、Live Stories |
| **视频流** | Turrit TikTok 视频流 + 官方 Live Photos | AI 视频推荐（GWT 注意力路由）、手势控制（音量/亮度/进度） |

### 2.3 组织与导航

| 功能标签 | 融合来源 | NeoGram 独有交互 |
|---------|---------|-----------------|
| **智能 Tab 系统** | Nicegram 自定义 Tabs + Swiftgram Folders + Turrit 侧边/底部导航 | **AI 自动文件夹**（E8 聚类聊天）、侧边/底部导航切换、隐藏未使用 Tab、记住上次打开的文件夹 |
| **多账号** | Nicegram 无限 + Swiftgram 无限 + Turrit 10 | Keychain 会话备份、AI 账号路由（按上下文切换账号）、账号备注 |
| **群聊备注** | Turrit Group Remark | 群名备注仅自己可见、AI 自动生成备注 |

### 2.4 消息与过滤

| 功能标签 | 融合来源 | NeoGram 独有交互 |
|---------|---------|-----------------|
| **Message Filter** | Swiftgram Message Filter + Turrit 关键词屏蔽 | **AI 语义过滤**（不只关键词，E8 理解意图）、频道广告过滤、指定用户屏蔽 |
| **Ghost Mode** | Nicegram Ghost Mode + Swiftgram 隐藏 | 不发送已读回执、隐藏在线状态、AI 智能通知（重要消息才打扰） |
| **Export to LLM** | Nicegram Export to LLM | **导出到 NeoTrix KB**（知识库沉淀）、AI 总结对话、关键决策提取 |

### 2.5 媒体与存储

| 功能标签 | 融合来源 | NeoGram 独有交互 |
|---------|---------|-----------------|
| **云存储** | Turrit Cloud Drive + 官方 Saved Messages | **KB 语义搜索云存储**（VSA 向量检索）、无限空间、一键分享 |
| **下载加速** | Turrit 20x 加速 | 智能下载队列、自动续传、缓存管理（1GB/2GB 自动清理） |
| **Voice-to-Text** | Swiftgram + Turrit + Whisper | 免费语音转文字 + **AI 语音总结**（要点提取） |
| **文档扫描** | 官方 Document Scanner | 自动边缘检测、多图拼接 PDF、AI 文档分类 |

### 4.6 通知与安全

| 功能标签 | 融合来源 | NeoGram 独有交互 |
|---------|---------|-----------------|
| **智能通知** | Swiftgram Disable @mentions + Nicegram 隐藏 | AI 通知优先级（E8 判断重要性）、置顶消息控制、通知摘要 |
| **AI Guardians** | 官方 AI Guardians + NeoTrix | E8 审核入群（垃圾/骚扰自动拦截）、隐私评分 |
| **隐私检测** | Turrit Privacy Detection | 一键隐私优化、手机号泄露检测、跨客户端同步 |
| **Passcode** | 官方 + Swiftgram Instant Lock | 即时密码锁 + Face ID + AI 异常检测 |

### 4.7 界面与体验

| 功能标签 | 融合来源 | NeoGram 独有交互 |
|---------|---------|-----------------|
| **Liquid Glass** | 官方 + Swiftgram + Turrit 平铺切换 | 毛玻璃/平铺双模式切换（用户选择）、E8 动态主题 |
| **动态主题** | ThemesUI + 官方 | AI 根据对话情绪变色、自定义壁纸 |
| **Emoji 搜索** | 官方 100M+ Emoji | 36 语言搜索 + AI 表情推荐 |
| **无障碍** | Swiftgram VoiceOver | 完整 VoiceOver 支持、动态字体 |

---

## 3. 分层架构

```
┌─────────────────────────────────────────────────────────┐
│  NeoGramApp (App 入口 + 路由)                            │
│  LaunchView → Passcode → MainTabView                     │
├─────────────────────────────────────────────────────────┤
│  UI Layer (SwiftUI 视图)                                 │
│  ChatUI / ChatListUI / StoriesUI / SettingsUI /          │
│  AIEditorUI / PollUI / PasscodeUI / ThemesUI /           │
│  ReactionsUI / AnimatedEmojiUI                           │
├─────────────────────────────────────────────────────────┤
│  Feature Layer (功能域)                                  │
│  AIHub (AI 中枢) / FilterEngine / PrivacyEngine /        │
│  CloudDrive / VoiceToText / ExportHub / VideoFlow        │
├─────────────────────────────────────────────────────────┤
│  Domain Layer (领域服务)                                 │
│  NeoGramCore (协调器) / MTProto (网络) /                 │
│  AccountManager (多账号) / FolderEngine (智能文件夹)      │
├─────────────────────────────────────────────────────────┤
│  Bridge Layer (FFI)                                      │
│  NeoTrixFFI (uniffi 绑定)                                │
├─────────────────────────────────────────────────────────┤
│  Rust Core (E8 / GWT / VSA / ConsciousnessTree)          │
└─────────────────────────────────────────────────────────┘
```

### 数据流（The Spice Must Flow）

```
用户输入 → UI Layer → Feature Layer (AIHub) → Domain Layer (NeoGramCore)
  → Bridge Layer (NeoTrixFFI) → Rust Core (E8 推理)
  → 结果回传 → UI 渲染
```

### 模块依赖（无环）

```
NeoGramApp → UI → Features → Core → NeoTrixFFI
                  ↘ Premium ↗
```

---

## 4. 关键 ADR

### ADR-001: AI 中枢化（AIHub）
- **状态**: 接受
- **上下文**: 所有客户端把 AI 当插件，NeoGram 应把 AI 当核心
- **决策**: 新建 `AIHub` 域，统一路由所有 AI 请求（编辑/翻译/总结/过滤/推荐），通过 GWT 注意力路由分发到 E8/VSA/ConsciousnessTree
- **后果**: + 差异化明显；- 依赖 Rust 核心可用性（FFI 未初始化时降级为本地规则）

### ADR-002: 智能文件夹（AI 自动分类）
- **状态**: 提议
- **决策**: FolderManager 用 E8 对聊天做语义聚类，自动生成文件夹；用户可手动覆盖
- **后果**: + 组织效率；- 需要 E8 推理延迟 < 100ms（否则降级为规则分类）

### ADR-003: 融合单体
- **状态**: 已接受
- **决策**: 沿用现有 Bazel 5 模块（NeoTrixFFI/Core/UI/Premium/Features），新增 Feature 域文件
- **后果**: 快速迭代；模块边界清晰

---

## 5. 实施路线图

| 阶段 | 内容 | 验证 |
|------|------|------|
| **P0 (已完成)** | UI 编译清零 + AIEditorUI + PollUI | swiftc 0 error |
| **P1** | AIHub 中枢 + ChatUI 接线 AI Editor | swiftc + 功能测试 |
| **P2** | FilterEngine + PrivacyEngine + 智能文件夹 | 单元测试 |
| **P3** | CloudDrive + ExportEngine + Voice-to-Text | 集成测试 |
| **P4** | VideoFlow + 动态主题 + 无障碍 | 全量验证 |

---

## 6. 风险与未决项

| 风险 | 缓解 |
|------|------|
| Rust 核心未初始化时 AI 功能降级 | AIHub 统一降级策略（本地规则兜底） |
| 完整 iOS 编译需 Xcode | CLT 验证核心逻辑，Xcode 安装后全量验证 |
| 特性过多导致膨胀 | Dark Forest 原则：每个模块必须有消费者 |
| 未决: 云存储后端选型 | 待定（KB 语义搜索 vs 自建） |
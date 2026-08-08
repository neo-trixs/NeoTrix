# NeoGram App Architecture — 极简 3 Tab 融合架构

> 状态: 已接受 (accepted) | 日期: 2026-08-08 | 作者: NT-CORE (des-architect)
> 目标: 融合全部特性（聊天/Stories/联系人/通话/LiveFeed/Moments/Stream/设置/Premium/主题/隐私/AI），
> 收敛为极简 3 Tab，每个 Tab 形成独有交互特性，架构统一高效。

---

## 1. 设计原则

| 原则 | 说明 |
|------|------|
| **极简 Tab** | 5 Tab → 3 Tab（Chats/Live/Me），删除占位 Tab（Contacts/Calls 假数据） |
| **Tab = 交互中枢** | 每个 Tab 不是"页面集合"，而是有独有交互语言的场景中枢 |
| **统一设计系统** | 单一 NeoTrixTheme 令牌（颜色/圆角/间距/卡片），消灭散落的硬编码色 |
| **统一 AI 入口** | AI 助手贯穿所有 Tab（对话内 AI 编辑 / Live AI 推荐 / Me AI 设置） |
| **统一卡片语言** | LiveCard 风格（圆角 + 柔和阴影 + 类型角标）作为全 App 卡片标准 |
| **Dark Forest** | 每个视图必须有消费者；孤岛视图全部接线，占位视图删除 |

---

## 2. 3 Tab 融合矩阵

### Tab 1: Chats（对话中枢）
**独有交互**: 顶部 Stories 环 + 聊天列表 + 搜索，长按行 → 操作菜单（已读/静音/置顶/删除）

| 融合内容 | 来源 | 接线方式 |
|---------|------|---------|
| 聊天列表 | ChatListUI | 主列表（已有） |
| Stories 环 | StoriesUI | 列表顶部横向滚动环（StoryRing） |
| 联系人 | ContactsView（占位） | 融合为搜索过滤 + 列表 Section |
| 通话 | CallsView（占位） | 融合为列表 Section（最近通话） |
| AI 助手 | AIHub | 列表顶部 AI 入口（wand 按钮 → AI 对话） |

### Tab 2: Live（发现中枢）
**独有交互**: 双列瀑布流 + 类型角标 + 长按卡片菜单（收藏/隐藏/屏蔽）

| 融合内容 | 来源 | 接线方式 |
|---------|------|---------|
| Live 瀑布流 | LiveFeedUI | 主视图（已有，自研推荐算法） |
| 搜索 | LiveFeedEngine | 搜索栏（已有） |
| Moments | NeoTrix/MomentView | 瀑布流卡片类型（moment 类型） |
| Stream | NeoTrix/UnifiedStreamView | 瀑布流卡片类型（stream 类型） |

### Tab 3: Me（个人中枢）
**独有交互**: 分组列表 + 每行右侧状态角标 + 长按行快捷操作

| 融合内容 | 来源 | 接线方式 |
|---------|------|---------|
| 设置 | SettingsView | 主列表（已有） |
| Premium | PremiumIntroView | 顶部卡片入口 |
| 主题 | ThemesUI | 设置行 → 主题选择 |
| 锁屏 | PasscodeUI | 设置行 → Passcode 设置 |
| AI 设置 | AIHub | 设置行 → AI 偏好 |
| 隐私 | PrivacyEngine | 设置行 → 隐私开关 |

---

## 3. 统一设计系统（NeoTrixTheme）

```
NeoTrixTheme (enum, static tokens)
├── Colors: background / surface / accent / textPrimary / textSecondary
├── Radius: small(10) / medium(14) / large(18)
├── Spacing: xs(4) / sm(8) / md(12) / lg(16) / xl(24)
├── Fonts: title / headline / body / caption
└── Card: LiveCard 标准（圆角 + 阴影 + 角标）
```

**CLT 兼容约束**: 不使用 `Color(.systemGray6)` 等 UIKit 动态色（CLT 无 UIKit 符号），
统一用 `Color.gray.opacity()` / `Color(.white)` / `Color(.black)`。

---

## 5. 分层架构（统一）

```
NeoGramApp (@main)
  └─ MainTabView (3 Tab)
       ├─ ChatsTab: StoriesBar + ChatList + AI 入口
       ├─ LiveTab:  LiveFeedView (瀑布流 + 搜索)
       └─ MeTab:    SettingsView (分组 + 子页接线)
```

- **UI Layer**: SwiftUI 视图（ChatsUI / LiveFeedUI / SettingsUI / StoriesUI / ThemesUI / PasscodeUI）
- **Feature Layer**: 引擎（ChatListViewModel / LiveFeedEngine / ThemeManager / PrivacyEngine / AIHub）
- **Domain Layer**: NeoGramCore（协调器）
- **Bridge Layer**: NeoTrixFFI（uniffi 绑定，未初始化时降级本地规则）

---

## 6. 实施清单

| # | 变更 | 文件 |
|---|------|------|
| 1 | 统一设计系统 | `Sources/Design/NeoTrixTheme.swift`（新建） |
| 2 | MainTabView 3 Tab | `Sources/NeoGramApp.swift`（重构） |
| 3 | Chats 融合 Stories + 联系人/通话 | `Sources/UI/ChatListUI.swift`（改造） |
| 4 | Chat 融合 AI Summarize 接线 | `Sources/UI/ChatUI.swift`（接线） |
| 5 | Me 融合接线 | `Sources/UI/SettingsUI.swift`（改造） |
| 6 | 编译验证 | swiftc typecheck 0 error |

---

## 7. 风险与未决项

| 风险 | 缓解 |
|------|------|
| 完整 iOS 编译需 Xcode | CLT 验证核心逻辑，Xcode 安装后全量验证 |
| 特性过多导致膨胀 | Dark Forest：每个视图必须有消费者 |
| 未决: Moments/Stream 是否并入 Live 卡片 | 先保留 LiveFeed 单一瀑布流，Moments/Stream 作为类型标签 |
# NeoGram Live Feed Architecture — Live 标签页架构

> 状态: 提议 (proposed) | 日期: 2026-08-08 | 作者: NT-CORE (des-architect)
> 前置: `FUSION-ARCHITECTURE.md` + `SEARCH-RESOURCE-ARCHITECTURE.md`
> 目标: 接入主流社交媒体资讯（按全球活跃量排名）→ 后端分析学习 → 自研推送算法 → 双列瀑布流推送 → 点赞/不感兴趣/分享交互

---

## 1. 对标结论（调研摘要）

| 平台 | 布局 | 推荐信号 | 负反馈 |
|------|------|---------|--------|
| TikTok/抖音 | 单列全屏沉浸 | 完播>分享>评论>收藏>点赞 | 长按→Not Interested（-84% 同类） |
| Instagram Explore | 网格+Reels | 热度>save>share>like | SFPLT/Not interested |
| 小红书 | **双列瀑布流** | CES=赞1+藏1+评4+转4+关注8 | 卡片 X / 不感兴趣 |
| Telegram | 无算法发现页 | 主题级推荐 | 无 |

**NeoGram Live 页设计决策**:
- **布局**: 双列瀑布流（小红书）+ 顶部横向分类 Tab（TikTok LIVE 2026 模式）
- **推荐算法**: 自研加权模型（参考 Meta Value Model + 小红书 CES）
- **负反馈**: 三级（不感兴趣→隐藏作者→屏蔽关键词），点击即时消失 + 可见反馈
- **内容源**: 按全球活跃量排名（SimilarWeb 指数）加权

---

## 2. 内容源与活跃排名

### 2.1 全球活跃用户指数（SimilarWeb 2026）

| 平台 | 活跃指数 | 权重 |
|------|---------|------|
| YouTube | 100 | 1.00 |
| WhatsApp | 87.4 | 0.87 |
| Instagram | 80.5 | 0.81 |
| Facebook | 75.7 | 0.76 |
| TikTok | 67.4 | 0.67 |
| Telegram | 开放度最高 | 0.60 (API 加成) |

### 2.2 内容源接入

| 源 | 方式 | 状态 |
|----|------|------|
| Telegram | MTProto/Bot API（开放免费） | P0 首选 |
| YouTube | RSS (原生支持) | P0 |
| Reddit | RSS (.rss 后缀) | P0 |
| Instagram | Graph API (需审核) | P1 |
| TikTok | Display API (需审核) | P1 |
| X | 按量计费 API | P2 |

---

## 3. 推荐算法（自研 Value Model）

### 3.1 评分公式

```
score = w_platform × platformWeight(活跃指数)
      + w_engagement × engagementScore    (赞/评/转/藏加权)
      + w_recency × recencyScore          (时间衰减)
      + w_affinity × userAffinity         (用户兴趣匹配, E8)
      − w_negative × negativeSignal       (不感兴趣/隐藏/屏蔽)
```

### 3.2 信号权重（参考小红书 CES + Meta Value Model）

| 信号 | 权重 | 说明 |
|------|------|------|
| 停留时长/完播 | 1.0 | 首要信号 |
| 分享 | 0.8 | 强正信号 |
| 收藏 | 0.8 | 强正信号 |
| 评论 | 0.6 | 中正信号 |
| 点赞 | 0.4 | 弱正信号 |
| 不感兴趣 | -2.0 | 强负信号 |
| 隐藏作者 | -3.0 | 超强负信号 |
| 屏蔽关键词 | -4.0 | 永久负信号 |

### 3.3 多样性控制

- Exploit 80% / Explore 20%（抖音模式）
- 同作者降权（Instagram 同款）
- 指数遗忘策略（小红书同款）

---

## 4. 分层架构

```
┌─────────────────────────────────────────────┐
│  UI Layer                                    │
│  LiveFeedUI (双列瀑布流 + 分类 Tab)           │
│  LiveCardView (卡片 + 点赞/分享/不感兴趣)      │
├─────────────────────────────────────────────┤
│  Feature Layer                               │
│  LiveFeedEngine (Live 中枢)                  │
│  ├─ ContentSourceProvider (内容源协议)        │
│  ├─ SourceRanker (活跃排名加权)               │
│  ├─ RecommendationEngine (Value Model)       │
│  ├─ FeedbackEngine (三级负反馈)               │
│  └─ LiveFeedItem (统一内容模型)               │
├─────────────────────────────────────────────┤
│  Domain Layer (NeoGramCore / MTProto)        │
├─────────────────────────────────────────────┤
│  Bridge Layer (NeoTrixFFI)                   │
├─────────────────────────────────────────────┤
│  Rust Core (E8 / GWT / VSA)                  │
└─────────────────────────────────────────────┘
```

---

## 5. 数据流

```
内容源 (Telegram/YouTube/Reddit) → ContentSourceProvider
  → SourceRanker (活跃排名加权)
  → RecommendationEngine (E8 评分 + 探索控制)
  → LiveFeedUI (双列瀑布流)
  → 用户交互 (点赞/分享/不感兴趣)
  → FeedbackEngine (信号收集)
  → 更新用户画像 → 影响下次推荐
```

---

## 6. 实施路线图

| 阶段 | 内容 | 验证 |
|------|------|------|
| P0 | LiveFeedEngine + 推荐算法 + 内容源协议 | swiftc 0 error |
| P1 | LiveFeedUI 双列瀑布流 + 分类 Tab | swiftc 0 error |
| P2 | 点赞/分享/不感兴趣交互 + 反馈引擎 | swiftc + 功能测试 |
| P3 | MainTabView 接线 Live Tab | swiftc + 功能测试 |
| P4 | 真实内容源接入 (Telegram MTProto) | 集成测试 |

---

## 7. 风险与未决项

| 风险 | 缓解 |
|------|------|
| 官方 API 审核周期长 | Telegram/YouTube RSS 先行 |
| 内容版权/合规 | 仅聚合公开内容 + 链接跳转 |
| 推荐冷启动 | 活跃排名权重兜底 |
| 未决: 内容源认证流程 | 待 API 就绪后对接 |
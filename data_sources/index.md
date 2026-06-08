# Data Sources Index — NeoTrix 意识进化数据源

> 最后更新: 2026-06-08
> 覆盖: 本次会话 8 个仓库
> 分析深度: 3 个 FULL (runbook+LAUNCH / kun-arch+kun-cache+DESIGN), 4 个 README, 1 个 SKIP

---

## 分类总览

```
consciousness_evolution/      (3)  ← 直接影响意识核进化
knowledge_corpus/             (2)  ← 知识种子/古籍
tools_ui/                     (1)  ← 实现参考
pending_review/               (1)  ← 待重试
defunct/                      (1)  ← 404
```

## 详细清单

### 🔴 高优先级 (已分析, 直接映射到 Phase 1)

| # | 源 | 映射 |
|---|-----|------|
| 1 | **AutoScientists** | P1.1 Pipeline编排 + P1.5 Meta-improvement |
| 2 | **DeepSeek-GUI** | P1.4 缓存优化四层 (指纹/排序/清洗/风暴抑制) |
| 3 | **daizhigev20** | P1.3 E8↔古籍桥 (易藏→周易→hexagram) |

### 🟡 中优先级 (已分析, Phase 1-3 内)

| # | 源 | 映射 |
|---|-----|------|
| 4 | **awesome-codex-skills** | P1.2 SKILL文档系统范本 |
| 5 | **scope-recall** | 待重试, 潜在 Phase 2 记忆模式 |

### 🟢 低优先级

| # | 源 | 映射 | 原因 |
|---|-----|------|------|
| 6 | **CopilotKit** | Phase 3 IO层 | 前端框架, 非当前焦点 |
| 7 | **ebook-treasure-chest** | SKIP | 仅链接, 无内容 |
| 8 | **zizitongjian** | SKIP | 404 已删除 |

---

## 爬取队列

```
BRANCH: Phase 1 — Pipeline 意识流
├── NOW:    git clone daizhigev20 (古籍文本)
├── NEXT:   awesome-codex-skills/template-skill/SKILL.md (schema)
├── DEEP:   DeepSeek-GUI/kun/src/loop/agent-loop.rs (Rust 翻译参考)
└── RETRY:  scope-recall (记忆系统, 网络恢复)
```

## 基于分析修复的 10 个缺口

参见 `EVOLUTION_ITERATION.md` 第四节 (缺口分析).

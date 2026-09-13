# targeted-research-681 — Consciousness Core Dispatch Route Expansion

## Summary

Added 4 new internal dispatch routes to the consciousness core task loop in `dispatch.rs`.

## New Routes

| Capability Tag | Domain | Specialist | Chinese Keywords | Purpose |
|---|---|---|---|---|
| `visual_explainer` | NT-IO | CreativityGenerator | 可视化解释, 可视化输出, 图表生成, 图解说明 | Visual output adapter — renders data/concepts as visual artifacts |
| `deer_flow` | NT-IO | CreativityGenerator | 网关嵌入, deer flow, 嵌入运行时 | Gateway+embed runtime — unified LLM gateway with embedding pipeline |
| `crawl4ai_stealth` | NT-WORLD | PatternMatcher | stealth爬虫, 隐身爬取, 浏览器池, 反检测爬取 | Async browser pool with anti-detection stealth crawling |
| `procedural_gen` | NT-ACT | CodeAnalyzer | 过程生成, 程序化生成, 生成管线, 算法生成 | Procedural generation pipeline — terrain/level/texture modes |

## Changes Made

**File**: `neotrix-core/src/core/l5_consciousness/consciousness_core/dispatch.rs`

1. **CAPABILITY_ROUTES** (line ~287): Added 20 new keyword→capability tuples (4 capabilities x 4-5 trigger keywords each)
2. **dispatch_internal_capability match** (line ~1017): Added 4 new match arms before the `_ =>` fallback:
   - `visual_explainer` — echoes visualization scheduling with truncated query
   - `deer_flow` — echoes gateway+embed flow dispatch
   - `crawl4ai_stealth` — URL detection or keyword-based stealth crawl scheduling
   - `procedural_gen` — mode detection (terrain/level/texture/general) with mode-specific output

## Domain Mapping

- **NT-IO** (界面使徒): `visual_explainer`, `deer_flow` — both are interface/output layer capabilities
- **NT-WORLD** (虚空探索者): `crawl4ai_stealth` — stealth browser pool extends existing `crawl4ai` route
- **NT-ACT** (行动执行者): `procedural_gen` — generation pipeline is an action/output capability

## Integration Notes

- `crawl4ai_stealth` complements the existing `crawl4ai` route (line ~260) with anti-detection specialization
- `deer_flow` adds a new gateway+embed runtime path distinct from `universal_model` (line ~237)
- All routes follow the existing `(keyword, capability_tag, domain, specialist)` tuple convention
- Match arms follow the established pattern of summary echo + truncation

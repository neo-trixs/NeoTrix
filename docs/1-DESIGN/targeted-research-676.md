# Targeted Research #676 — Internal Dispatch Routes

**Date**: 2026-09-12
**File**: `neotrix-core/src/core/l5_consciousness/consciousness_core/dispatch.rs`

## Summary

Added 4 new internal dispatch routes to the consciousness core task loop, expanding the capability routing table and execution dispatch in `dispatch.rs`.

## New Routes

| Route ID | Capability Tag | Domain | Specialist | Description |
|----------|---------------|--------|------------|-------------|
| `crawl4ai` | `crawl4ai` | NT-WORLD | PatternMatcher | Async crawler architecture — dispatches URL-based or keyword-based crawl tasks |
| `seal_genstep` | `seal_genstep` | NT-MIND | KnowledgeIntegrator | SEAL phases as composable pipeline — routes distill/absorb/test/explore/iterate |
| `self_test_t3` | `self_test_t3` | NT-META | MetaCognitionAnalyst | Skill effectiveness measurement — probes T3 wiring via KB experience keys |
| `kb_governance_ostrom` | `kb_governance_ostrom` | NT-MEMORY | KnowledgeRetriever | KB governance with graduated sanctions — violation detection + severity escalation |

## CAPABILITY_ROUTES Entries (28 new keyword routes)

### crawl4ai (6 entries)
- `crawl4ai`, `异步爬虫`, `异步抓取`, `async_crawl`, `网页爬取`, `网站爬取`

### seal_genstep (6 entries)
- `seal_genstep`, `SEAL阶段`, `seal管道`, `seal组合`, `进化阶段`, `genstep`

### self_test_t3 (6 entries)
- `self_test_t3`, `T3测试`, `技能有效`, `能力度量`, `self_test`, `技能测试`

### kb_governance_ostrom (6 entries)
- `kb_governance_ostrom`, `ostrom治理`, `分级制裁`, `kb制裁`, `kb合规`, `知识库制裁`

## Dispatch Behavior

### crawl4ai
- Detects URLs in task summary (words starting with `http`)
- Falls back to keyword-based crawl if no URL found
- Routes to NT-WORLD/PatternMatcher

### seal_genstep
- Phase detection via keyword matching: `distill`/`absorb`/`test`/`explore`/`iterate`
- Calls `crate::neotrix::seal_distill()`, `seal_absorb()`, or `seal_iterate()`
- Routes to NT-MIND/KnowledgeIntegrator

### self_test_t3
- Opens KB, reads stats, probes 6 T3 capability keys
- Checks `experience:t3_effective:{cap}` for each capability
- Reports effective/total ratio with KB health stats
- Routes to NT-META/MetaCognitionAnalyst

### kb_governance_ostrom
- Reads violation/sanction counters from KB governance namespace
- Graduated sanctions: 无违规 → 警告 (1-5) → 降级 (6-20) → 封禁 (20+)
- Increments violation counter on each call
- Routes to NT-MEMORY/KnowledgeRetriever

## Files Modified

- `neotrix-core/src/core/l5_consciousness/consciousness_core/dispatch.rs` — CAPABILITY_ROUTES + match arms
- `docs/1-DESIGN/targeted-research-676.md` — this document

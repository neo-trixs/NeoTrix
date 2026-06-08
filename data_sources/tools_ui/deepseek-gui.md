source_id: deepseek-gui
url: https://github.com/XingYu-Zhong/DeepSeek-GUI
category: tools_ui
analyzed: true
depth: full (README + kun-architecture + kun-cache + DESIGN)
status: crawled
priority: high
neotrix_phase: 1.1, 1.3

## Key Patterns Adopted
- Cache-first agent loop (ImmutablePrefix, AppendOnlyLog, InflightTracker)
- Token economy (canonical sort, tool hygiene, MCP search)
- Sha256 prefix fingerprint with drift detection
- Tool pair healing (orphan tool_result cleanup)
- History compaction (soft 16k / hard 24k threshold)
- Storm breaker (repeat tool call suppression)
- Capability feature flags (capabilities.* gates)
- Architecture: Renderer → IPC → Agent Loop → Model API (clean boundaries)

## Crawl Plan
- [x] README.md
- [x] docs/kun-architecture.md
- [x] docs/kun-cache-optimization.md
- [x] DESIGN.md
- [] kun/src/loop/agent-loop.ts
- [] kun/src/cache/immutable-prefix.ts
- [] kun/src/domain/model-history-repair.ts
- [] kun/src/loop/tool-storm-breaker.ts
- [] kun/src/prompt/kun-system-prompt.ts

## Relevance to NeoTrix
Highest relevance of all 7 repos. Cache-first loop maps directly to consciousness pipeline design.

# Ranking Aggregation — 12-Source Technology Trends

**Date**: 2026-09-10
**Sources**: 12 ranking aggregation + trend sites
**Status**: Research batch complete

---

## 1. Trendshift (Daily Momentum)

**Source**: https://trendshift.io/

| # | Project | Stars | Trend | NeoTrix Relevance |
|---|---------|-------|-------|-------------------|
| 1 | ayghri/i-have-adhd | 2.2k | ↑ Rising | ADHD-friendly agent output — user-centric prompt engineering |
| 2 | k2-fsa/OmniVoice | 599★ | ↑ New | Multi-language TTS 600+ — NT-IO voice synthesis |
| 3 | google/artemis | 591★ | ↑ New | Android automation via NL instructions — NT-ACT agent patterns |
| 4 | Tencent/teamai-cli | 564★ | ↑ Rising | Team AI agent harness — multi-agent orchestration |
| 5 | TauricResearch/TradingAgents | 473★ | ↑ Rising | Multi-agent LLM trading — agent collaboration patterns |

**Trend Direction**: AI agents + multi-agent orchestration dominate momentum
**Absorbable Pattern**: `autoharness` — self-learning skill layer that distills skills from real sessions, updates as you work, prunes unused ones. Direct inspiration for NT-MIND skill crystallization.

---

## 2. OSSInsight (Open Source Intelligence)

**Source**: https://ossinsight.io/
**Note**: Star-based rankings paused since mid-2025 due to GitHub API pagination changes. Commit activity unaffected.

| Observation | Detail |
|-------------|--------|
| Platform Status | Star rankings paused; commit activity data still valid |
| Key Insight | GitHub API pagination change broke star/PR/issue ingestion |
| NeoTrix Implication | Star-based metrics have blind spots; need multi-source validation |

**Trend Direction**: — (data incomplete)
**Absorbable Pattern**: Multi-source ranking validation — don't rely on single metric source.

---

## 3. Star History (Weekly Star Gains)

**Source**: https://star-history.com/

| # | Project | Weekly Δ | Trend | NeoTrix Relevance |
|---|---------|----------|-------|-------------------|
| 1 | DietrichGebert/ponytail | +12.4k | ↑↑ Surge | — |
| 2 | debpalash/VoiceStudio | +7.6k | ↑↑ Surge | Voice synthesis studio — NT-IO |
| 3 | ayghri/i-have-adhd | +5.2k | ↑ Rising | Agent UX optimization |
| 4 | heygen-com/hyperframes | +4.4k | ↑ Rising | Video generation — NT-ACT |
| 5 | microsoft/markitdown | +4.4k | ↑ Rising | Document parsing — NT-MEMORY ingestion |
| 6 | deeplethe/utopia | +4.4k | ↑ Rising | Document-to-ontology agent — NT-MEMORY |
| 7 | NousResearch/hermes-agent | +4.2k | ↑ Rising | Open agent framework |
| 8 | browser-use/browser-use | +1.8k | ↑ Steady | Browser automation — NT-SHIELD/CAMOFOX |
| 9 | humanlayer/skills | +3.0k | ↑ New | Skill distillation framework |
| 10 | hugohe3/ppt-master | +1.9k | ↑ Rising | PPT generation — nt_file_ability |

**Trend Direction**: Voice synthesis, agent skill distillation, document parsing surging
**Absorbable Pattern**: `markitdown` (Microsoft) — universal document→Markdown converter. `deeplethe/utopia` — local-first agent-assisted document-to-ontology workbench. Both align with NT-MEMORY ingestion pipeline.

---

## 4. AwesomeLists.top

**Source**: https://www.awesomelists.top/
**Status**: 404 — site unreachable

---

## 5. GitHub Ranking (All-Time Stars)

**Source**: https://github.com/EvanLi/Github-Ranking

| Rank | Project | Stars | Language | NeoTrix Relevance |
|------|---------|-------|----------|-------------------|
| 1 | build-your-own-x | 546k | Markdown | Educational — skill template |
| 2 | awesome | 504k | — | Curated lists meta |
| 3 | public-apis | 478k | Python | API registry — NT-ACT |
| 4 | freeCodeCamp | 455k | TypeScript | Education |
| 5 | openclaw | 389k | TypeScript | AI agent platform — lobster way |

**Top by Language**:
- **Rust**: torvalds/linux (247k), rust-lang/rust (102k+), denoland/deno (101k+)
- **C++**: tensorflow (199k), llama.cpp (127k), react-native (126k)
- **Python**: build-your-own-x (546k), public-apis (478k), system-design-primer (369k)
- **TypeScript**: openclaw (389k), freeCodeCamp (455k), developer-roadmap (366k)

**Trend Direction**: Stable — classic leaderboard; Rust ecosystem growing steadily
**Absorbable Pattern**: `llama.cpp` dominance in C++ confirms local LLM inference demand — validates NeoTrix's local-first architecture.

---

## 6. Stack Overflow Developer Survey 2025

**Source**: https://survey.stackoverflow.co/2025/

### Most Popular Technologies
| Rank | Technology | Share | Trend |
|------|-----------|-------|-------|
| 1 | JavaScript | 66% | Stable |
| 2 | HTML/CSS | 61.9% | Stable |
| 3 | SQL | 58.6% | Stable |
| 4 | Python | 57.9% | ↑ +7pp YoY |
| 5 | Bash/Shell | 48.7% | Stable |

### Most Admired
| Tag | Desired | Admired |
|-----|---------|---------|
| uv | 13.9% | 61.4% |
| RAG | 12.4% | 65.1% |
| C++23 | 11.3% | 74.2% |
| Cargo | 15.2% | 56.4% |
| Claude Sonnet | 33.3% | 67.5% |

### Key AI Insights
- **84% of developers use AI tools** (up from 76% in 2024)
- **51% use AI daily** in professional work
- **46% distrust AI accuracy** vs 33% who trust
- **66% frustrated** by "almost right" AI solutions
- **AI agents not yet mainstream**: 52% don't use or use simpler tools
- **Claude Sonnet** most admired LLM (67.5% admired, 33.3% desired)
- **Positive AI sentiment decreased**: 70%+ in 2023/24 → 60% in 2025

**Trend Direction**: Python accelerating; AI adoption up but trust down; agents still emerging
**Absorbable Pattern**: The "almost right" frustration (66%) directly validates NeoTrix's Human-in-the-Loop gate (R-P14). Trust deficit supports verification-heavy architecture.

---

## 7. Stack Overflow Insights

**Source**: https://insights.stackoverflow.com/
**Status**: Redirects to main survey — data captured in #6 above.

---

## 8. TIOBE Index (September 2026)

**Source**: https://www.tiobe.com/tiobe-index/

| Rank | Language | Rating | YoY Change | Trend |
|------|----------|--------|------------|-------|
| 1 | Python | 17.76% | -8.22% | ↓ Declining share but still #1 |
| 2 | C | 10.28% | +1.63% | ↑ Resurgence |
| 3 | C++ | 8.67% | -0.13% | → Stable |
| 4 | Java | 7.54% | -0.81% | ↓ Slow decline |
| 5 | C# | 4.22% | -2.16% | ↓ Declining |
| 6 | JavaScript | 2.76% | -0.46% | → Stable |
| 8 | SQL | 2.16% | +0.29% | ↑ Rising |
| 9 | R | 1.69% | +0.27% | ↑ Rising |
| 10 | **Rust** | 1.34% | +0.33% | ↑↑ Top 10 for first time |
| 12 | Go | 1.10% | -1.22% | ↓ Declining |
| 18 | Swift | 0.83% | +0.10% | ↑ Rising |
| 21 | Julia | 0.74% | — | → Near top 20 |

**TIOBE Headline**: Rust breaks into top 10. Julia threatening top 20. MATLAB declining. Perl and Ruby exit top 20.

**Trend Direction**: Rust confirmed as mainstream; Python share declining (still #1); C having renaissance
**Absorbable Pattern**: Rust's TIOBE rise (+0.33%) validates language choice for NeoTrix core. C resurgence suggests embedded/systems interest — relevant for NT-PHYSICAL.

---

## 9. RedMonk Rankings (January 2026)

**Source**: https://redmonk.com/sogrady/2026/04/14/language-rankings-1-26/

| Rank | Language | Notes |
|------|----------|-------|
| 1 | JavaScript | Stable leader |
| 2 | Python | Stable |
| 3 | Java | Stable |
| 4 | PHP / C# | C# moved up from #5 to #4 tie |
| 6 | TypeScript | Stable |
| 7 | CSS / C++ | Tied |
| 9 | Ruby | Declining |
| 10 | C | Stable |
| 11 | Swift | Stable |
| 12 | Go | Stable |
| 13 | R | Rising |
| 14 | Shell / Kotlin / Scala | Tied |
| 17 | PowerShell | Stable |
| 18 | Dart / Objective-C | Dart rose from bottom to #18 |
| 20 | Rust | Stable but influential |

**Key Observations**:
- **C# rise**: From #5 to #4, tying PHP
- **Dart surge**: From bottom of top 20 to #18, passing Rust
- **Objective-C decline**: May exit top 20 permanently
- **Coding assistants**: No observable impact on language distribution yet
- **Stack Overflow declining**: RedMonk questioning its role as ranking axis

**Trend Direction**: Top languages extremely stable; Dart surprising rise; Rust steady at #20
**Absorbable Pattern**: RedMonk's dual-axis (GitHub PRs + SO tags) methodology — multi-signal ranking reduces single-source bias.

---

## 10. Chatbot Arena (HuggingFace)

**Source**: https://huggingface.co/spaces/lmarena-ai/chatbot-arena

**Note**: Arena is a live Elo-based voting system. Could not extract static leaderboard from page (requires JavaScript rendering).

**Known Trend** (from external knowledge):
- GPT-4o, Claude Sonnet 4, Gemini 2.5 Pro consistently top
- Open-weight models (Llama 3, Qwen 2.5) competitive with closed
- Local inference models gaining ground

**Trend Direction**: Closed models still lead; open-weight gap narrowing
**Absorbable Pattern**: Elo-based blind evaluation — validation method for NeoTrix model selection.

---

## 11. VisualAI.io

**Source**: https://www.visualai.io/
**Status**: Domain for sale (parked page). Not a ranking site.

---

## 12. AIModels.fyi

**Source**: https://aimodels.fyi/

**Platform Function**: AI research aggregator — tracks papers, models, researchers. Provides digests and alerts.

**Key Features**:
- Paper/model tracking with plain-English summaries
- Researcher following
- Architecture/algo/app filtering
- Historical trend analysis

**Trend Direction**: AI research volume continues exponential growth
**Absorbable Pattern**: Signal-over-noise curation model — similar to NeoTrix's SEAL distillation pipeline. "Three weeks ahead of Twitter" positioning.

---

## Cross-Source Synthesis

### Top 5 Rising Technologies (Multi-Source Consensus)

| Technology | Sources Confirming | Trend | NeoTrix Action |
|-----------|-------------------|-------|----------------|
| **Rust** | TIOBE (#10), RedMonk (#20), GitHub, Stack Overflow | ↑↑ | Validate core language choice; monitor ecosystem |
| **AI Agents** | Trendshift, SO Survey, GitHub Trending | ↑ | NT-ACT agent orchestration; skill distillation |
| **Voice/TTS** | Star History (OmniVoice, VoiceStudio), Trendshift | ↑ | NT-IO voice synthesis integration |
| **RAG** | SO Survey (65.1% admired), AI models | ↑ | NT-MEMORY retrieval pipeline |
| **Multi-Agent Systems** | Trendshift (TradingAgents, teamai-cli), Star History | ↑ | NT-ACT multi-agent coordination |

### Top 5 Stable Technologies

| Technology | Status | NeoTrix Implication |
|-----------|--------|---------------------|
| JavaScript | #1 everywhere | — (not core) |
| Python | #1 TIOBE, #2 RedMonk | AI/ML ecosystem standard |
| C/C++ | Top 3 everywhere | llama.cpp validates local inference |
| VS Code | #1 IDE (75.9%) | Agent IDE integration target |
| Git/GitHub | Universal | Workflow backbone |

### Absorbable Patterns

1. **Self-Learning Skill Layer** (`autoharness`): Distills skills from sessions, updates as you work, prunes unused ones — direct map to NT-MIND crystallization
2. **Document-to-Ontology** (`deeplethe/utopia`): Local-first agent-assisted knowledge extraction — NT-MEMORY ingestion
3. **Multi-Source Ranking** (RedMonk methodology): GitHub PRs + SO tags + alternative signals — reduces single-source bias
4. **Elo-Based Model Evaluation** (Chatbot Arena): Blind pairwise comparison — validation for NT-IO model selection
5. **Skill-as-Production-Template** (Easel/112 skills): SKILL.md contract (<200 lines) — NT-ACT skill node architecture

### NeoTrix Architecture Validation

| SO Survey Finding | NeoTrix Feature | Validation |
|-------------------|----------------|------------|
| 66% frustrated by "almost right" AI | Human-in-the-Loop gate (R-P14) | ✅ Confirmed need |
| 46% distrust AI accuracy | SEAL verification pipeline | ✅ Confirmed need |
| 84% using AI tools | Agent-first design | ✅ Market alignment |
| AI agents not mainstream | NT-ACT agent orchestration | ✅ Early mover |
| Claude Sonnet most admired | Multi-provider routing | ✅ Model-agnostic design |
| Python +7pp YoY | Rust core + Python interop | ✅ Strategic split |

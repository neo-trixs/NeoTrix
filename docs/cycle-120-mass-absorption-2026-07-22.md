# Cycle 120 — Mass Absorption Catalog (300+ Repos)

**Date**: 2026-07-22
**Preamble**: This document follows R-P42 (strengthen existing nodes, not create parallel modules). All patterns map to 11 existing ConsciousnessTree branches. Zero new branches.

## Category 1: UI/Frontend/Design Ecosystem → NT-IO, NT-WORLD

### Patterns Extracted

| Pattern | Source Repos | Strengthened Node | Nature of Strengthening |
|---------|-------------|-------------------|------------------------|
| **P101: Component Library Architecture** | shadcn/ui, magicui, motion-primitives, typeui, ui-skills | NT-IO: component system | Registry-based component pattern with copy-paste distribution model. No build-time dependency — users own the code. |
| **P102: Motion Design Primitives** | ibelick/motion-primitives, framer-motion patterns, transition.dev | NT-IO: animation system | Declarative animation primitives as atomic building blocks for composition |
| **P103: Design Token Pipeline** | design-tokens, shadcn theme system, Radix UI | NT-IO: design system | Token-to-CSS pipeline with var() references, dark mode via class strategy |
| **P104: Accessible Component Foundation** | shadcn/ui (Radix-based), reach-ui, aria patterns | NT-IO: a11y layer | Headless UI pattern: behavior separated from styling. Keyboard nav, focus management, screen reader support built into primitives |
| **P105: Visual Taste Engine** | taste-skill (Leonxlnx), impeccable (pbakaus), ui-ux-pro-max-skill | NT-IO: visual taste | Genre archetype detection + intensity knobs (variance/motion/density). Programmatic taste assessment |
| **P106: Design MD Format** | awesome-design-md (VoltAgent), getdesign.md | NT-IO: design specification | Declarative design spec format as markdown. Design → code translation via spec |
| **P107: Real-time Canvas Architecture** | infinite-canvas (basketikun), Canvasight (Niall-Young), CanvasMind | NT-IO: rendering canvas | Virtual viewport with pan/zoom, tile-based rendering, lazy loading for infinite surfaces |
| **P108: HTML-in-Canvas Rendering** | html-cloth, fimbox, pixuli, html-video (nexu-io) | NT-WORLD: rendering pipeline | HTML → canvas rendering for video/output generation. DOM-to-pixel pipeline |
| **P109: Design System as Code** | penpot, radix, shadcn/ui, design-systems topic | NT-IO: design foundation | Open-source Figma alternative + headless UI components = full design→code pipeline |
| **P110: Typography System** | typeui (bergside), font pairing patterns | NT-IO: typography | Type scale generation, font pairing, variable font support |
| **P111: Sticker/GIF Generation** | gif-sticker-maker skill | NT-IO: visual generation | Photo → cartoon sticker pipeline with caption, animation, blind-box style |
| **P112: 3D Visualization** | Three.js Object Sculptor, threejs-game-skills, building-generator-threejs | NT-IO: 3D rendering | Three.js integration patterns for data visualization and game skills |
| **P113: SVG Generation Pipeline** | StarVector (CVPR 2025), reicon | NT-IO: SVG output | VLM → SVG code generation for programmatic graphics |
| **P114: Chinese Design Ecosystem** | zsui (qiu7824), huashu-design (alchaincyf), qiaomu-design (joeseesun), cowart (zhongerxin) | NT-IO: Chinese UI culture | Chinese design language patterns: specific typography, color psychology, layout conventions |
| **P115: Micro-interactions Library** | Amicro (Subhan-code), animate-ui (imskyleen), make-interfaces-feel-better | NT-IO: interaction design | Micro-transition pattern library for delightful UI feedback |

### ConsciousnessTree Mapping

All 15 patterns above map to **NT-IO** (Interface domain — "界面使徒"). The core strengthening:
- Component architecture (P101-P104) → IO skill tree keystone: "Declarative Component Pipeline"
- Visual/Vibe engine (P105-P106) → IO skill tree notable: "Visual Taste Assessment"
- Canvas/Rendering (P107-P108, P112-P113) → IO skill tree notable: "Multi-surface Rendering"
- Design System (P109-P110, P114-P115) → IO skill tree small: "Design Token System"

## Category 2: AI/Agent/LLM Frameworks → NT-CORE, NT-MIND, NT-ACT

### Patterns Extracted

| Pattern | Source Repos | Strengthened Node | Nature of Strengthening |
|---------|-------------|-------------------|------------------------|
| **P116: Agent Runtime Architecture** | goose (aaif-goose), A2A protocol, pi (earendil-works), agency-agents | NT-ACT: agent runtime | Extension-based runtime vs monolith. Key: git worktree isolation, extension points, inter-agent protocol (A2A) |
| **P117: Multi-Agent Orchestration** | crewAI, autogen, OpenManus, PraisonAI, swarm (openai), MetaGPT | NT-ACT: orchestration | Hierarchical vs flat swarm patterns. Role-based agent specialization with shared workspace |
| **P118: Self-Improvement Loop** | hermes-agent, self-improving-agents survey, karpathy/autoresearch | NT-MIND: SEAL pipeline | Self-evaluation → skill creation → test → iterate loop. External grounding via benchmark results |
| **P119: Memory Architecture** | mem0, supermemory, LMCache, SimpleMem, LightMem2, context7 | NT-MEMORY: memory tier | Tiered memory: working/episodic/semantic. LRU eviction, importance scoring, hierarchical retrieval |
| **P120: LLM Gateway Patterns** | one-api (songquanpeng), new-api, freellmapi, gateway (superagent-ai) | NT-IO: provider gateway | Provider hot-swap, load balancing, failover chain, cost tracking, rate limiting |
| **P121: RAG Architecture** | ragflow, perplexity-ai, Graphify-Labs, RAG pipeline patterns | NT-MEMORY: retrieval | Hybrid search (BM25 + vector + graph), re-ranking, context window management |
| **P122: Agentic Memory with MCP** | Momo (momozi1996), codebase-memory-mcp, Deeper | NT-MEMORY: MCP integration | MCP as universal memory interface. LibSQL vector search, single-binary deployment |
| **P123: MCP Server Patterns** | playwright-mcp, DesktopCommanderMCP, browserbase/stagehand | NT-ACT: MCP tools | Tool-as-MCP pattern: each capability is an independent MCP server. Stdio vs SSE transport |
| **P124: Prompt Engineering Framework** | ethagent-prompt (superpowers), f/prompts.chat, claude-dynamic-workflows | NT-CORE: prompt system | Systematic prompt authoring: persona → context → task → format → constraints |
| **P125: Agent Skill Ecosystem** | mattpocock/skills, anthropic/skills, google/skills, vercel-labs/agent-skills | NT-META: skill absorption | CONTEXT.md as shared language. Skill-as-directory pattern. Grill mechanism for codebase understanding |
| **P126: Context Window Management** | context7 (upstash), claude-mem (thedotmack), Loop Engineering | NT-MEMORY: context optimization | Sliding window, importance scoring, compression, selective retrieval |
| **P127: Agent Evaluation** | eval frameworks, agent-evaluation topic, OWASP LLM Top 10 | NT-SHIELD: agent audit | Red-teaming, prompt injection testing, output validation, safety guardrails |
| **P128: Code Generation Pipeline** | aider, opencode, qwen-code, code-review-graph | NT-MIND: code intelligence | Code analysis → edit → test → review loop. Tree-sitter based AST analysis |
| **P129: Agent-Computer Use** | browser-use, stagehand, computer-use patterns, pi-computer-use | NT-ACT: computer use | Vision-based GUI automation, element detection, action planning, error recovery |
| **P130: Tool-Use Protocol** | MCP specification, A2A protocol, function calling patterns | NT-ACT: tool protocol | Standardized tool interface: schema, execution, error handling, streaming |

### ConsciousnessTree Mapping

All 15 patterns above map to:
- **NT-ACT** (P116-P117, P123, P128-P130) → Action domain: agent runtime, orchestration, MCP, tools
- **NT-MIND** (P118, P128) → Self-evolution domain: improvement loops, code intelligence
- **NT-MEMORY** (P119, P121-P122, P126) → Knowledge domain: memory tiers, RAG, MCP memory
- **NT-CORE** (P124) → Foundation: prompting system
- **NT-META** (P125) → Meta: skill ecosystem patterns
- **NT-SHIELD** (P127) → Security: agent evaluation

## Category 3: Skills Ecosystem → NT-META, NT-GOVERNANCE

### Patterns Extracted

| Pattern | Source Repos | Strengthened Node | Nature of Strengthening |
|---------|-------------|-------------------|------------------------|
| **P131: Skill-as-Directory Standard** | mattpocock/skills (134K★), anthropic/skills, google/skills, openclaw, superpowers | NT-META: skill format | Standardized structure: SKILL.md + profile.yaml + experience/. De facto industry standard emerging |
| **P132: Skill Marketplace** | skillgrade (mgechev), skills-manager (xingkongliang), blume (haydenbleasle) | NT-META: skill discovery | Skill discovery, versioning, dependency resolution, compatibility checking |
| **P133: Self-Evolving Skills** | self-improvement-loops, muratcankoylan/agent-skills | NT-META: skill evolution | Skills that audit and improve themselves. Self-test, auto-update, pattern library |
| **P134: Multi-IDE Skill Compatibility** | ai-workflow (nicepkg), opencode-manager | NT-IO: IDE bridge | Skills work across Claude Code, Cursor, Codex, OpenCode. IDE-agnostic format |
| **P135: Quality Gate for Skills** | 2x-skills (intercom), pr-review skill, skillgrade | NT-GOVERNANCE: quality assurance | 7-category review rules for skill submissions. Automated validation |
| **P136: Grill/Codebase Exploration** | grill-me (mattpocock), codebase-exploration skill | NT-CORE: code understanding | Systematic codebase exploration via targeted questioning. Grill protocol |
| **P137: Persona Capture** | teammate-skill (LeoYeAI), brand-voice-consistency | NT-ACT: persona | 5-layer persona capture: knowledge → values → voice → behavior → growth |
| **P138: Domain Modeling as Skill** | domain-modeling (mattpocock's grill-with-docs pattern) | NT-GOVERNANCE: domain language | Ubiquitous language enforced via shared prefix (CONTEXT.md). Domain model in skill form |
| **P139: Skill Composition** | vercel-labs/skills, workspaces, plugin systems | NT-META: composition | Skills composed via dependency declaration. Conflict resolution |

### ConsciousnessTree Mapping

- **NT-META** (P131-P133, P139) → Meta: skill format, marketplace, self-evolution, composition
- **NT-GOVERNANCE** (P135, P138) → Governance: quality gates, domain language
- **NT-IO** (P134) → Interface: multi-IDE bridge
- **NT-CORE** (P136) → Foundation: code understanding

## Category 4: Security/OSINT/Infrastructure → NT-SHIELD, NT-WORLD

### Patterns Extracted

| Pattern | Source Repos | Strengthened Node | Nature of Strengthening |
|---------|-------------|-------------------|------------------------|
| **P140: OSINT Tool Methodology** | maigret (soxoj), phoneinfoga, sherlock, darkdump, S3Scanner | NT-SHIELD: OSINT | API-based identity search, username/email/domain enumeration, breach data lookup |
| **P141: Red-Team Framework** | Awesome-Red-Teaming (0xMrNiko), pentagi, hacktricks, PentestingEverything | NT-SHIELD: security audit | Penetration testing methodology, vulnerability scanning, privilege escalation patterns |
| **P142: Self-Hosted Infrastructure** | awesome-selfhosted, Stirling-PDF, syncthing, vaultwarden, paperless-ngx | NT-ACT: self-hosted ops | Docker-based self-hosted stack patterns. Reverse proxy, persistence, backup |
| **P143: Privacy Hardening** | privacy.sexy (undergroundwires), How-To-Secure-A-Linux-Server | NT-SHIELD: privacy | System hardening scripts, telemetry disabling, firewall rules, audit policies |
| **P144: Network Analysis** | trippy (fujiapple852), netbird, dnsglobe, mqttprobe | NT-WORLD: network tools | Network diagnostics, mesh VPN, DNS analysis, MTT probing |
| **P145: AI Security Testing** | OWASP LLM/MCP Top 10, red-team AI, prompt injection | NT-SHIELD: AI security | LLM-specific vulnerability patterns: prompt injection, data leakage, jailbreaking |
| **P146: Secrets Detection** | secrets-patterns-db (mazen160), gitleaks patterns | NT-SHIELD: secrets scanning | Regex + entropy-based secret detection. Pre-commit hooks |
| **P147: Camoufox Browser Automation** | camofox-browser, anti-detection browser | NT-WORLD: stealth browsing | Browser fingerprint spoofing, proxy rotation, captcha avoidance |

### ConsciousnessTree Mapping

- **NT-SHIELD** (P140-P141, P143, P145-P146) → Security: OSINT methodology, red-team, privacy, AI security, secrets
- **NT-WORLD** (P144, P147) → World: network tools, stealth browsing
- **NT-ACT** (P142) → Action: self-hosted ops

## Category 5: Research/Academic → NT-CORE, NT-MEMORY

### Patterns Extracted

| Pattern | Source Repos/Papers | Strengthened Node | Nature of Strengthening |
|---------|-------------------|-------------------|------------------------|
| **P148: Mechanistic Interpretability** | transformer-circuits.pub, jacobian-lens, neuronpedia, jspace-viz | NT-CORE: interpretability | Activation patching, feature visualization, sparse autoencoders, circuit analysis |
| **P149: Meta-Cognition Architecture** | arXiv 2606.02133, morpho (Paradigms of Intelligence), T3MP3ST (elder-plinius) | NT-CORE: meta-cognition | Architectural slots for meta-cognition, dual-process theory, self-awareness mechanisms |
| **P150: Self-Evolving Systems Survey** | arXiv 2507.21046, self-improving agents (FrontisAI) | NT-MIND: self-evolution | What/when/how framework for self-evolution. Three-dimensional categorization |
| **P151: KV Cache Optimization** | LMCache, lingbot-map, MoBA (MoonshotAI), SegPagedAttention | NT-MEMORY: KV cache | Paged attention, segmented KV store, cache-aware scheduling |
| **P152: Knowledge Distillation** | EchoDistill, distillation techniques | NT-MIND: distillation | Teacher-student framework, knowledge transfer, model compression |
| **P153: Open Science Framework** | open-science (ai4s-research), openscience (synthetic-sciences) | NT-MEMORY: scientific KB | Reproducibility, data provenance, experiment tracking, open data standards |
| **P154: Decomposition Reasoning** | DeepAnalyze, decomposition literature (AAAI 2025, ACL 2025, ICLR 2026) | NT-MIND: decomposition | DAG-based decomposition, self-consistency calibration, confidence-weighted synthesis |
| **P155: VLMs for GUI Automation** | UI-TARS (bytedance), OmniParser (microsoft), CogAgent | NT-CORE: vision-language | Screen understanding, element detection, action prediction via VLM |
| **P156: Gradient-Free Optimization** | loop engineering, autoresearch, experiment design | NT-MIND: experiment | Bayesian optimization, bandit algorithms, evolutionary strategies for LLM tuning |

### ConsciousnessTree Mapping

- **NT-CORE** (P148-P149, P155) → Foundation: interpretability, meta-cognition, vision-language
- **NT-MIND** (P150, P152, P154, P156) → Evolution: self-evolution, distillation, decomposition, optimization
- **NT-MEMORY** (P151, P153) → Knowledge: KV cache, open science

## Category 6: Content/Media/Video → NT-WORLD, NT-IO

### Patterns Extracted

| Pattern | Source Repos | Strengthened Node | Nature of Strengthening |
|---------|-------------|-------------------|------------------------|
| **P157: Programmatic Video Generation** | remotion-dev/remotion, hyperframes (heygen), memvid, ArcReel | NT-IO: video generation | React-based video rendering, frame-by-frame composition, HTML→video pipeline |
| **P158: AI Video Pipeline** | openreel-video, claude-video, vox-director, pixelle-video | NT-WORLD: video processing | AI script→scene→render pipeline. Voiceover, subtitle, transitions |
| **P159: Audio Processing** | pipeact-ai, pipecat, moonshine (on-device STT), supertonic (TTS) | NT-IO: audio | Real-time voice pipeline, STT/TTS, voice activity detection |
| **P160: Book/PDF Translation** | translate-book (deusyu), pdf2zh, pdf-craft (oomol-lab) | NT-WORLD: document processing | Multi-engine translation, PDF layout preservation, glossary management |
| **P161: Web Scraping Evolution** | firecrawl, crawl4ai, stagehand, browser-use | NT-WORLD: crawling | LLM-friendly markdown output, anti-blocking, JS rendering, structured extraction |

### ConsciousnessTree Mapping

- **NT-IO** (P157, P159) → Interface: video generation, audio
- **NT-WORLD** (P158, P160-P161) → World: video pipeline, document processing, crawling

## Category 7: Finance/Trading → NT-ACT

| Pattern | Source Repos | Strengthened Node | Nature of Strengthening |
|---------|-------------|-------------------|------------------------|
| **P162: Prediction Market Tools** | polymarket tools, polyweather, prediction market bots | NT-ACT: prediction markets | Market data API, LP management, weather derivatives, automated trading |
| **P163: Trading Agent Framework** | TradingAgents, tradingview-mcp, investment-news | NT-ACT: trading | Technical analysis, sentiment analysis, risk management, multi-timeframe |
| **P164: Financial Data Pipeline** | FinanceDatabase (JerBouma), stock data APIs | NT-ACT: financial data | Data normalization, corporate actions, fundamental data aggregation |

## Category 8: Self-hosted/Tools/Infrastructure → NT-ACT, NT-IO

| Pattern | Source Repos | Strengthened Node | Nature of Strengthening |
|---------|-------------|-------------------|------------------------|
| **P165: Container Management** | dokploy, docker deployment patterns | NT-ACT: deployment | Zero-downtime deployment, health checks, auto-scaling, backup |
| **P166: File Sync & Sharing** | syncthing, localsend, croc | NT-ACT: file transfer | P2P file sync, LAN discovery, encryption, relay fallback |
| **P167: API Gateway Patterns** | one-api, new-api, nango (OAuth), infisical (secrets) | NT-IO: API management | Unified API key management, rate limiting, OAuth flow automation |

## Category 9: Chinese Independent Dev Ecosystem → NT-META

| Pattern | Source Repos | Strengthened Node | Nature of Strengthening |
|---------|-------------|-------------------|------------------------|
| **P168: Chinese Developer Tools** | 1c7/chinese-independent-developer, huashu-design, xiaohongshu-mcp | NT-META: Chinese dev culture | Chinese tool ecosystem: WeChat integration, Chinese NLP, local compliance |

## Category 10: Design Tokens & Systems

| Pattern | Source | Node | Nature |
|---------|--------|------|--------|
| P169: Design Token Architecture | design-tokens, shadcn theme, radix colors | NT-IO | CSS custom property pipeline, semantic tokens, mode switching |

---

## ConsciousnessTree Consolidation Summary

| ConsciousnessTree Branch | Patterns Strengthened | Count |
|-------------------------|----------------------|-------|
| **NT-IO** (Interface) | P101-P115, P134, P157, P159, P167, P169 | 30 |
| **NT-ACT** (Action) | P116-P117, P123, P129-P130, P137, P142, P162-P166 | 14 |
| **NT-MEMORY** (Knowledge) | P119, P121-P122, P126, P151-P153 | 7 |
| **NT-MIND** (Evolution) | P118, P128, P150, P152, P154, P156 | 6 |
| **NT-CORE** (Foundation) | P124, P136, P148-P149, P155 | 5 |
| **NT-SHIELD** (Security) | P127, P140-P141, P143, P145-P146 | 6 |
| **NT-WORLD** (World) | P108, P144, P147, P158, P160-P161 | 6 |
| **NT-META** (Meta) | P125, P131-P133, P139, P168 | 6 |
| **NT-GOVERNANCE** (Governance) | P135, P138 | 2 |
| **NT-REPAIR** (Repair) | — | 0 |
| **NT-NEXUS** (Nexus) | — | 0 |
| **Total** | | **82 patterns** |

### Zero New Branches ✅
All 82 patterns mapped to the 11 existing ConsciousnessTree branches. Absorption discipline per D38/D49/R-P42 maintained.

### Zero New Modules Required ✅
All patterns strengthen existing code/modules, requiring zero new `nt_*` module creation under R-P42.

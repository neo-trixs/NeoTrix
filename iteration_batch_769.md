# Iteration Batch 769 — Free LLM Pool Architecture Audit

**Date**: 2026-09-07
**Predecessor**: Batch 768 (GGUF↔llama.cpp coupling, Q4_K_M dominant format, AWQ Marlin 10.9× speedup)

---

## Search 1: Free LLM API Landscape (2026)

### Providers & Free Tiers (verified Sep 2026)

| Provider | Free Models | RPM | RPD | Context | OpenAI Compat | Key Advantage |
|----------|-------------|-----|-----|---------|---------------|---------------|
| **Groq** | Llama 3.3 70B, Llama 4 Scout, Qwen3 32B, DeepSeek R1 Distill | 30 | 14,400 | 128K | Yes | 500–3,000 tok/s via LPU silicon |
| **OpenRouter** | 20+ free models (multi-provider) | 20 | 50 (1,000 with $10 top-up) | Up to 1M | Yes | Single key, auto-failover |
| **Together AI** | Llama 3.3 70B, DeepSeek R1 Distill, FLUX.1, Llama Vision | ~60 | Rate-limited | 128K | Yes | Chat+reasoning+vision+image gen on one key |
| **Google AI Studio** | Gemini 3.7 Flash, Gemma variants | 5–15 | 20–1,500 | 1M | Partial | 1M token context, multimodal |
| **Mistral** | Codestral, Mistral Small/Large ($10/mo free credits) | Variable | ~1B tokens/mo | 32K–256K | Yes | Vibe Code agentic environment included |
| **Cerebras** | Llama 3.3 70B, Llama 4 Scout | 30 | ~1M tokens/day | 1M | Yes | 2,600 tok/s, high daily volume |
| **Cloudflare Workers AI** | Llama 4 Scout, Mistral Small 3.1, Qwen3 | High | 10K neurons/day | 2K–8K | Partial | Edge deployment, zero infra |
| **GitHub Models** | GPT-4o, Claude 3.5 Sonnet, Llama, Phi | 15 | 150–1,000/day | 8K–128K | Yes | Frontier model access |
| **HuggingFace** | 100K+ OSS models | Variable | Community-rate limited | Model dependent | Partial | Widest task variety |

### Key Pricing Points (Sep 2026)

| Model | Input/M | Output/M | Notes |
|-------|---------|----------|-------|
| DeepSeek V4-Flash | $0.14–$0.22 | $0.28 | Price rose Aug 17, 2026 |
| DeepSeek V4-Pro | $0.435–$0.66 | $0.87 | Also rose Aug 17 |
| GPT-5.4 Nano | $0.10–$0.20 | $0.40 | Budget frontier floor |
| Llama 3.3 70B (Together) | $0.88 | $0.88 | Blended |
| Claude Sonnet 4.6 | $3.00 | $15.00 | Cache read: $0.30 |
| Claude Opus 4.8 | $5.00 | $25.00 | Cache read: $0.50 |

### Speed Benchmarks

| Provider | Hardware | Llama 3.3 70B tok/s | Latency |
|----------|----------|---------------------|---------|
| Groq | Custom LPU | 500–3,000 | Sub-second TTFT |
| Cerebras | Custom silicon | ~2,600 | Very low |
| Fireworks | GPU (FireAttention) | Moderate-high | Good |
| Together AI | GPU cluster | 50–200 | Moderate |
| Replicate | On-demand GPU | Variable | Cold starts |

---

## Search 2: LLM Gateway Landscape (2026)

### Gateway Comparison Matrix

| Tool | Type | Hosting | License | Self-Hosted Free? | P95 Overhead | Best For |
|------|------|---------|---------|-------------------|--------------|----------|
| **LiteLLM** | Proxy gateway | Both | Open source (MIT) | Yes | 0.66ms (Rust core) | 140+ providers, cost tracking, guardrails |
| **OpenRouter** | Aggregator | Managed | Proprietary | No | N/A | One API + billing across hundreds of models |
| **Portkey** | Proxy + router | Both | Open source core | Yes | Low | Routing + caching + guardrails + observability |
| **Bifrost** | Proxy gateway | Self-hosted | Open source | Yes | Low | Low-overhead routing, hierarchical budgets |
| **Cloudflare AI Gateway** | Edge proxy | Managed | Proprietary | No | N/A | Edge caching, zero infra |
| **Helicone** | Proxy + observability | Both | Open source | Yes | Low | Drop-in proxy logging |
| **Martian** | Smart router | Managed | Proprietary | No | N/A | Dynamic cheapest/best model per request |
| **Not Diamond** | Smart router | Managed | Proprietary | No | N/A | Prompt-to-model quality matching |
| **Kong AI Gateway** | API gateway | Both | Open source | Yes | N/A | Enterprise API governance |

### LiteLLM Key Specs (2026)

- **Core**: Rust with Python SDK
- **Providers**: 140+ LLM providers, 1,800+ models
- **Latency**: 0.66ms P99 added overhead, 2,800+ req/s at ~21% CPU
- **Memory**: ~22 MB at rest (vs ~199 MB for competitors)
- **Features**: Virtual keys, spend tracking, guardrails, load balancing, A2A agent support, MCP tool bridging
- **Deploy**: Docker, Helm, Terraform, one-click AWS/GCP/Azure
- **OpenAI compatible**: Drop-in replacement

### Semantic Routing (OpenZiti LLM Gateway)

- Three-layer cascade: keyword heuristics → embedding similarity → LLM classifier
- Multi-endpoint load balancing with weighted round-robin
- Health checks with passive failover
- zrok zero-trust integration (no exposed ports)
- OpenTelemetry metrics (Prometheus)

### Cost Optimization with LiteLLM Router

- **14× cost reduction** demonstrated: route 70% requests to DeepSeek V4-Flash ($0.14/M), escalation to V4-Pro for hard tasks
- **Usage-based-routing-v2**: spreads traffic across keys based on real-time usage
- **Fallback chains**: fast-tier → escalation-tier → long-context-tier
- **Cost tracking**: per-request tier, token count, estimated cost to SQLite

---

## Search 3: LLM Cost Optimization (2026)

### Prompt Caching State (Sep 2026)

| Provider | Cache Read Discount | Cache Write Premium | Min Cacheable | TTL | Auto? |
|----------|--------------------|--------------------|---------------|-----|-------|
| **Anthropic** | 90% off | 1.25× (5min), 2× (1hr) | 512–4,096 tokens | 5min default, 1hr at 2× | Explicit `cache_control` |
| **OpenAI** | 90% off (GPT-5.x) | 1.25× (GPT-5.6+) | 1,024 tokens | ≥30min (GPT-5.6+) | Automatic |
| **Google** | 90% off (Gemini 2.5+) | None (storage fee: $1–$4.50/MTok/hr) | 2,048–4,096 | Configurable | Implicit default |
| **DeepSeek** | 96.7–96.8% off | None | Automatic | "Hours to days" | Automatic |
| **xAI** | 75–85% off | Not published | Automatic | Eviction-based | Automatic |

### Critical Caching Pitfall

**Thomson Reuters finding (2025, still relevant)**: Parallel calls against the same document with caching enabled produced only **4.2% cache hit rate** — near zero. Race condition: cache creation takes 2–4 seconds, parallel calls launch before any cache exists. One warming call before parallel batch cuts cost **59%**.

### Batch API Discount

All three major providers offer ~50% off for batch processing (up to 24hr turnaround):
- **Anthropic batch + cache read**: $0.15/MTok effective input (95% off list)
- **OpenAI batch + cache**: GPT-5+ models support cached input inside Batch
- **Gemini batch**: Discounted context-caching rates for explicit caching

### Model Routing Savings

| Traffic Split | Daily Cost (100K requests) | Savings |
|--------------|---------------------------|---------|
| 100% Opus 4.8 ($25/M out) | $750/day | Baseline |
| 70% Haiku + 25% Sonnet + 5% Opus | $425/day | 43% |
| With caching on medium+complex tiers | ~$225–300/day | 60–70% |

### Self-Hosting Break-Even

| Volume | Best Option | Cost/M Output |
|--------|-------------|---------------|
| <1M tokens/day | Together AI API | $0.22–$0.88 |
| 1–5M tokens/day | Evaluate packet.ai H100 | $0.18–$0.73 |
| >5M tokens/day | Self-host H200 vLLM FP8 | $0.10–$0.15 |

Break-even: **2–5M tokens/day** on reserved GPU over 12-month window.

### Token Efficiency GPT-5.5 vs GPT-5.4

- GPT-5.5 is ~40% more token-efficient on coding tasks
- Price doubled ($5/$30 vs $2.50/$15) but effective per-task cost only ~20% higher
- Structured JSON responses are 30–60% shorter than natural language
- "Be concise" system prompt reduces output tokens 20–40%

---

## Defects Found for Free LLM Pool Architecture

### D769-1: Rate Limit Fragmentation (CRITICAL)

**Finding**: Each free provider has independent, incompatible rate limit structures. Groq: 30 RPM / 14,400 RPD. OpenRouter: 20 RPM / 50 RPD (unfunded). Together AI: ~60 RPM but model-specific. Mistral: variable.

**Defect**: NeoTrix's `nt_core_llm` gateway selection must implement per-provider rate limit tracking AND cross-provider budget allocation. Current architecture (single provider selection) cannot handle this — a single provider's quota exhausts in hours at production volume.

**Impact**: Without token bucket or sliding window rate limiting per provider, the free pool collapses to single-provider dependency, negating the multi-provider resilience goal.

### D769-2: Cache Incompatibility Across Providers (HIGH)

**Finding**: Each provider implements caching differently — Anthropic requires explicit `cache_control`, OpenAI caches automatically, Google charges storage fees. Cache TTLs range from 5min to 24hr. DeepSeek's cache is "hours to days, not guaranteed."

**Defect**: A unified caching layer for the Free LLM Pool cannot assume consistent cache semantics. Request routing to different providers invalidates cached state. A prompt cached on Anthropic is not warm on OpenAI.

**Impact**: Cross-provider failover breaks cache benefits. The 90% cache discount is only achievable within a single provider, not across the pool. Architecture must cache per-provider, not globally.

### D769-3: OpenAI Compatibility Is Not Universal (HIGH)

**Finding**: Google AI Studio is only "partial" OpenAI compatible. Cloudflare Workers AI is partial. HuggingFace is partial. Only Groq, Together, OpenRouter, Cerebras, and Mistral are fully compatible.

**Defect**: NeoTrix's assumption that all providers speak OpenAI format is false for 30–40% of the free pool. Provider-specific SDK adapters needed for Google (native SDK), Cloudflare (Workers format), HuggingFace (Inference API).

**Impact**: Unified `nt_io` LLM interface cannot be a thin OpenAI wrapper — must include provider-specific translation layers.

### D769-4: Free Tier Business Model Instability (MEDIUM)

**Finding**: DeepSeek raised prices Aug 17, 2026 (V4-Flash from $0.14 to $0.22/M). Free tier terms change frequently. Google AI Studio free tier "outside EU/UK/EEA" trains on data. OpenRouter free models change frequently.

**Defect**: The Free LLM Pool cannot treat free tiers as stable infrastructure. Terms change without notice. A provider can remove models, change rate limits, or add training clauses.

**Impact**: Architecture needs: (1) monitoring of provider terms changes, (2) automatic failover when a provider degrades or changes terms, (3) data policy awareness (some free tiers train on inputs).

### D769-5: Context Window Asymmetry (MEDIUM)

**Finding**: Free tiers offer wildly different context windows: Google/Gemini 1M tokens, Groq 128K, Cloudflare 2K–8K, OpenRouter varies by model. Task routing must account for this.

**Defect**: A request requiring 200K context cannot be routed to Groq (128K) or Cloudflare (8K). The router needs context-aware routing, not just quality/cost routing.

**Impact**: Missing context-aware routing means requests silently fail or get truncated when routed to short-context providers.

### D769-6: Parallel Cache Warming Race Condition (MEDIUM)

**Finding**: Thomson Reuters documented parallel cache calls achieving only 4.2% hit rate. Cache creation takes 2–4 seconds. Parallel calls launch before cache exists.

**Defect**: NeoTrix's concurrent request patterns (agent loops, batch evaluations) will hit this race condition. Architecture must implement sequential cache warming or lock-based serialization for shared prefixes.

**Impact**: Without cache warming protocol, the 90% cache discount degrades to near-zero effective savings on parallel workloads.

### D769-7: No Semantic Routing in Current Gateway (LOW)

**Finding**: OpenZiti's LLM gateway demonstrates three-layer semantic routing (heuristics → embeddings → LLM classifier) that auto-selects the best model when `model` field is omitted.

**Defect**: NeoTrix's current gateway selection is rule-based (provider priority). No prompt-aware routing to match request complexity to model capability.

**Impact**: Simple queries waste expensive models; complex queries get routed to underpowered free-tier models. Estimated 20–40% cost waste from misrouting.

### D769-8: Data Training Policy Risk (LOW)

**Finding**: Google AI Studio free tier trains on data (outside EU/UK/EEA). Mistral Experiment tier trains on data. Other providers (Groq, Cerebras) do not train on customer inputs.

**Defect**: NeoTrix's Egress Privacy Guard does not track per-provider data training policies. Sensitive prompts could be routed to providers that train on them.

**Impact**: Privacy violation for users with data sensitivity requirements. Architecture needs data classification → provider policy matching.

---

## Sources Cited

1. DEV Community — "Together AI Free API" (May 2026)
2. OpenRouter Blog — "Free LLM API in 2026: 13 Options Ranked" (Jun 2026)
3. Dataiku — "Best free LLM APIs for developers" (Aug 2026)
4. KDnuggets — "5 Free LLM API Providers" (Sep 2026)
5. GetAIPerks — "Groq Free Tier 2026" (May 2026)
6. machinelearningplus — "Groq vs Fireworks vs Together AI Speed Benchmark" (Mar 2026)
7. GrizzlyPeakSoftware — "Every AI API with a Free Tier in 2026" (Apr 2026)
8. GitHub BerriAI/litellm — LiteLLM AI Gateway README
9. litellm.ai — Official website
10. aiprosol.com — "LLM Gateway & Router Index 2026" (Jun 2026)
11. tech-insider.org — "Cut AI API Costs 14x With LiteLLM Router" (Aug 2026)
12. Ramp — "Best LLM Gateways for AI Cost Control" (Aug 2026)
13. CloudZero — "LLM token cost explained" (Sep 2026)
14. Stochastic Sandbox — "LLM Token Costs: Practitioner's Guide" (Jun 2026)
15. leanlm.ai — "Prompt Caching in 2026" (Jun 2026)
16. TrueFoundry — "LLM Cost Optimization Guide" (Aug 2026)
17. Json House — "LLM Cache Pricing 2026" (Aug 2026)
18. packet.ai — "LLM Inference Cost 2026" (Jul 2026)
19. aiarch.dev — "LLM cost optimization: caching, routing, budgets" (Jun 2026)
20. openziti/llm-gateway — Zero trust LLM gateway (Feb 2026)

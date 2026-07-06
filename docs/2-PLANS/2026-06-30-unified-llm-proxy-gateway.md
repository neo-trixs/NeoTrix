# Unified LLM Provider + IP Proxy Gateway Architecture

## Problem

NeoTrix has **5 disconnected routers**, **4 stub streaming providers**, **zero rate limiting**, **zero circuit breakers**, **zero failover wiring**, and a **complete disconnect between the proxy pool and LLM provider layer**. FreeApi maps to local Ollama instead of actual free APIs.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        GatewayProvider                          │
│                   (implements LlmProvider trait)                  │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │               Middleware Pipeline                          │    │
│  │                                                           │    │
│  │  1. Token Bucket Rate Limiter  (per-provider)             │    │
│  │  2. Circuit Breaker           (per-provider state machine)│    │
│  │  3. Provider Pool Manager     (health + scoring)          │    │
│  │  4. Proxy Integration         (per-request proxy inject)  │    │
│  │  5. Fallback Chain            (provider→proxy→model)      │    │
│  │  6. Streaming Pipeline        (real SSE parsing)          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │               Scoring Engine                               │    │
│  │  S(provider, proxy) = (success²/latency) × cost_factor^w  │    │
│  │  Thompson Sampling for provider+proxy pair selection        │    │
│  └─────────────────────────────────────────────────────────┘    │
└──────────────────────────┬──────────────────────────────────────┘
                           │
              ┌────────────┼────────────┬───────────┐
              ▼            ▼            ▼           ▼
        ┌──────────┐ ┌──────────┐ ┌────────┐ ┌──────────┐
        │ OpenAI   │ │ Anthropic│ │ Gemini │ │ Free     │
        │ (paid)   │ │ (paid)   │ │ (free)  │ │ Providers│
        └──────────┘ └──────────┘ └────────┘ └──────────┘
              │            │            │           │
              └────────────┼────────────┼───────────┘
                           │
              ┌────────────▼──────────────────────┐
              │   Proxy-Aware HTTP Client          │
              │   (reqwest::Client + per-request   │
              │    proxy injection from proxy_pool) │
              └───────────────────────────────────┘
```

## Key Design Decisions

### 1. GatewayProvider implements LlmProvider
Zero changes to all 10+ call sites. `create_provider()` returns a `GatewayProvider` that wraps all middleware internally. All existing code that calls `.complete()` or `.stream_complete()` works unchanged.

### 2. Provider Abstraction via `ProviderHandle`
Each provider (OpenAI, Anthropic, Gemini, Groq, OpenRouter, Ollama) becomes a `ProviderHandle` with:
- Inner `Box<dyn LlmProvider>` for the actual HTTP calls
- `CircuitBreaker` state machine (Closed/Open/HalfOpen)
- `TokenBucket` rate limiter
- Health score (EMA of success rate × latency)
- Cost tracking (USD per token)

### 3. Proxy Injection at HTTP Client Level
The `nt_io_http_factory.rs` gains a `build_client_with_proxy()` function. Each provider request can optionally route through a proxy node from `global_pool()`. The proxy selection is integrated into the scoring engine.

### 4. Unified Scoring
`Score(provider, proxy) = (success_rate^2 / p95_latency) × (1 / cost_per_token)^β × health_penalty^γ`

Where:
- `success_rate` = EMA over last 50 requests
- `p95_latency` = rolling p95 latency
- `cost_per_token` = provider cost per 1K tokens
- `health_penalty` = 0.0 (circuit open), 0.5 (half-open), 1.0 (closed)
- `β` = cost sensitivity (default: 0.3)
- `γ` = health penalty exponent (default: 2.0)

### 5. Free Provider Integration
Three tiers of free providers:
- **Tier 1 (Keyless)**: Pollinations.ai, OVHcloud, Kilo Gateway — no API key needed
- **Tier 2 (Keyed-Free)**: Groq (rate-limited free tier), Google Gemini (free tier), OpenRouter (free models), Cerebras, SambaNova
- **Tier 3 (Trial/Community)**: DeepSeek, Mistral, Cohere, Together AI — free tier with API key

Each free provider is a proper `LlmProvider` implementation with the correct base_url, auth headers, and response parsing.

## Module Structure

```
nt_io_provider/
├── mod.rs                    # Module root (add new module declarations)
├── types.rs                  # LlmProvider trait, types (fix Message fields)
├── factory.rs                # Provider factory (use GatewayProvider)
├── gateway.rs                # NEW: Unified GatewayProvider
├── circuit_breaker.rs        # NEW: Circuit breaker state machine
├── rate_limiter.rs           # NEW: Token bucket rate limiter
├── provider_pool.rs          # NEW: Provider pool manager + scoring
├── free_providers.rs         # NEW: Real Groq/OpenRouter/Pollinations providers
├── openai.rs                 # Fix: real streaming
├── anthropic.rs              # Fix: real streaming, assistant messages
├── gemini.rs                 # Fix: real streaming, headers auth
├── ollama.rs                 # Fix: OpenAI-compatible endpoint
├── discovery.rs              # FIX: register in mod.rs
├── free_catalog.rs           # FIX: register in mod.rs
├── agent_routing.rs          # FIX: wire failover to GatewayProvider
├── search_router.rs          # FIX: connect to GatewayProvider
└── compaction.rs             # FIX: register in mod.rs

nt_shield_stealth_net/
├── proxy_pool.rs             # FIX: real check_proxy()
└── ...

nt_io_http_factory.rs         # FIX: add build_client_with_proxy()
```

## Implementation Order

1. Fix `types.rs` — add tool_calls, tool_call_id to Message
2. Create `circuit_breaker.rs` — state machine
3. Create `rate_limiter.rs` — token bucket
4. Create `free_providers.rs` — Groq, OpenRouter, Gemini free endpoints
5. Create `provider_pool.rs` — multi-provider management
6. Create `gateway.rs` — Unified GatewayProvider
7. Fix `factory.rs` — use GatewayProvider, fix FreeApi
8. Fix `mod.rs` — register all modules
9. Fix `nt_io_http_factory.rs` — add proxy integration
10. Fix `proxy_pool.rs` — real check_proxy()
11. Fix provider implementations — real streaming

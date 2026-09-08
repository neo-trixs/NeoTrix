# Iteration Batch 623 — Testing Strategy, Contract Testing, Integration Testing

## What's NEW (622→623)

### 1. Testing Strategy: Trophy/Pyramid Hybrid Is Now Default Practice
- **Key finding**: Teams no longer debate pyramid vs trophy — the 2026 consensus is **architecture-shaped hybrid**. Domain-heavy backend = pyramid-heavy; API-centric SPA = trophy-heavy; microservices = pyramid + heavy contract testing.
- **Google's size model** (Small ≤60s / Medium ≤300s / Large ≤900s) replaces subjective "unit vs integration" labels with **enforced resource constraints**. A test claiming Small but opening a socket fails the build — the most practical upgrade most pyramid explainers skip.
- **Diff coverage** (80-90% on new/changed lines) replaces repo-wide coverage mandates. The 80% industry CI gate targets code most likely to contain fresh bugs, not legacy dead code.
- **AI test generation**: Strong fit for unit layer (low flakiness risk), partial fit for integration, weakest fit for E2E (highest flakiness — agents draft then heal). vitest (96% retention) and Playwright (91% retention) are the 2026 standard tools.
- **Flaky test quarantine**: Track first-attempt pass rate per test; quarantine below 95%. Treat as a **budget line**, not a nuisance.
- **New metric**: Defect detection per pipeline minute, not defect detection per test.

**Sources**:
- digitalapplied.com (2026-06-17): Google's size model, coverage tiers 60/75/90
- yoo.be (2026-08-15): Test Pyramid vs Testing Trophy for CI/CD pipeline
- ankurm.com (2026-04-04): JUnit 6 hybrid model with slice tests
- onpathtesting.com (2026-03-25): 5-layer extended pyramid (unit/component/integration/system/acceptance)

### 2. Contract Testing: Pact V4 + Bidirectional + `can-i-deploy` as Mandatory Gate
- **Key finding**: Pact is the de facto standard. V4 spec supports HTTP, async message, AND synchronous message contracts — single tool for REST, GraphQL, Kafka, RabbitMQ.
- **Bidirectional contracts** (2024+): Provider can publish its own contract (OpenAPI-derived) instead of running live verification. Broker compares consumer expectations against provider spec — trades some fidelity for speed and decoupling. Critical for legacy/vendor providers.
- **`can-i-deploy`** is now the mandatory CI/CD gate, not an optional nicety. Without it, Pact is "most of the cost, little of the benefit."
- **Pending/WIP pacts**: Enable `enablePending: true` in verifier — new unverified pacts don't fail provider builds. Once verified, they graduate out of pending. This is how you roll Pact across independent teams without grinding pipelines.
- **GraphQL contract testing**: Works best when contracts focus on actual operations/fragments used by consumers, not the entire schema graph.
- **Provider states as API surface**: When consumer changes "a user exists" → "a user in state paid exists", provider handler must also change. Treat state handlers as part of the contract.

**Sources**:
- docs.pact.io (2026-08-25): Official Pact docs
- qaskills.sh (2026-06-04): Complete Pact reference 2026
- qaskills.sh (2026-06-23): Practical microservices Pact walkthrough
- apiscout.dev (2026-03-09): Consumer-driven API contract testing
- sqaexperts.com (2026-04-30): Microservices QA guide for 2026
- aiwisdom.dev (2026-03-29): Contract testing with PactFlow

### 3. Integration Testing: WireMock + Testcontainers as Standard Infrastructure
- **Key finding**: WireMock + Testcontainers module (`wiremock-testcontainers-module`) is the canonical pattern for external API mocking. WireMock runs as Docker container → real HTTP network hop → catches serialization, timeout, and circuit breaker behavior that Java-level mocks miss.
- **`@ServiceConnection`** (Spring Boot 3.1+) for databases; `@DynamicPropertySource` for WireMock (not a Spring-managed connection type — this is the one exception).
- **Stateful stubs** via WireMock scenarios: Model retry/timeout/circuit-breaker by setting state transitions (STARTED → first-retry → second-retry). Tests exercise real retry logic.
- **JSON mapping files** vs Java DSL: Both used together — JSON for shared/reusable/non-Java-team stubs, Java DSL for test-specific stubs.
- **Anti-pattern confirmed**: H2 in tests while PostgreSQL in production. Testcontainers with real DB images is the fix. Pin real version tags (`postgres:18`), never `latest`.
- **Container reuse**: `@Testcontainers` singleton pattern + Testcontainers' `reuse` flag. Container startup adds 5-15s per unique image. Reuse on locally (with tests that tolerate leftover data), off in CI.
- **Async testing**: Awaitility for async assertions, never `Thread.sleep`.

**Sources**:
- testcontainers.com (guide): WireMock + Testcontainers REST API testing
- tutorialq.com (2026-02-15): Spring Boot + Testcontainers + WireMock
- github.com/wiremock: wiremock-testcontainers-java (alpha-13→alpha-16)
- qaskills.sh (2026-06-23): WireMock API Mocking complete guide
- stevenpg.com (2026-07-22): Ultimate Guide Testcontainers + Spring Boot
- wiremock.org: Official Testcontainers docs

---

## Defects Found

### D1: NeoTrix SelfTest Framework Lacks Google-Style Size Constraints
**Severity**: HIGH | **Category**: Testing Architecture
NeoTrix's `SelfTest` trait (T1/T2/T3 tiers) defines existence/registration/production-wiring but has **no resource-constraint enforcement** (timeouts, network/filesystem boundaries). Without Google's Small/Medium/Large constraints, there's no objective gate to prevent a "unit" test from silently becoming an integration test.

**Fix**: Extend `SelfTest` with a `TestSize` enum (Small/Medium/Large) with enforced timeout + resource constraints. A test claiming Small but opening a socket or accessing filesystem should fail at the registry level.

### D2: Contract Testing Gap — NT-ACT ↔ NT-IO Boundaries Are Unguarded
**Severity**: HIGH | **Category**: Inter-Domain Contracts
NT-ACT (action execution) calls NT-IO (LLM providers, CLI, web server) over internal interfaces. No consumer-driven contracts exist between these domains. A provider state change in NT-IO (e.g., LLM response format) can silently break NT-ACT without detection.

**Fix**: Define Pact-style contracts for NT-ACT→NT-IO boundaries. At minimum, schema-based contracts for LLM provider response shapes and CLI command output formats. Wire into SelfTest T2 registration.

### D3: Flaky Test Budget Not Tracked in ConsciousnessTree Health
**Severity**: MEDIUM | **Category**: Meta-Cognition Blind Spot
`HeartbeatAggregator` collects compilation/test/KB health but has no metric for **flaky test ratio** or **first-attempt pass rate**. A test suite that's 100% passing but 20% flaky is indistinguishable from a stable one in the health snapshot.

**Fix**: Add `flaky_test_ratio` and `first_attempt_pass_rate` to `SystemHealthSnapshot`. Integrate into ConsciousnessTree health chain as a new dimension. Quarantine tests below 95% first-attempt pass rate.

### D4: Integration Test Infrastructure Missing for NT-WORLD Crawler
**Severity**: MEDIUM | **Category**: Testing Infrastructure
NT-WORLD (UnifiedCrawler) fetches external web content but has no WireMock/Testcontainers pattern for mocking HTTP dependencies in tests. Tests either call real APIs (flaky, rate-limited) or mock at the Java level (miss serialization issues).

**Fix**: Establish a WireMock Testcontainer pattern for NT-WORLD. Use `WireMockContainer` with JSON mapping files for crawler test stubs. Add to SelfTest T1 as crawler integration test baseline.

### D5: Contract Testing Not Part of Constellation Maturity (C0-C6)
**Severity**: LOW | **Category**: Maturity Framework Gap
Constellation maturity ladder (C0=compiles → C5=self-healing) doesn't include contract testing verification as a maturity gate. A module can reach C4 (integrated into pipeline) without any contract validation against its dependencies.

**Fix**: Add C3.5 or equivalent gate: "Contract tests defined and passing for all inter-domain boundaries" before C4 integration. This prevents shipping modules that compile and pass unit tests but break integration contracts.

### D6: `can-i-deploy` Equivalent Missing for NT Module Deployment
**Severity**: MEDIUM | **Category**: Deployment Safety
NeoTrix module deployment has no equivalent to Pact's `can-i-deploy` gate. Modules can be deployed without verifying compatibility with currently-deployed versions of their dependencies.

**Fix**: Implement a `nt-deploy-check` command that queries KB for deployed module versions and verifies contract compatibility before allowing deployment. Modeled after Pact Broker's `can-i-deploy` pattern.

### D7: AI Test Generation Quality Not Calibrated for NeoTrix Rust Codebase
**Severity**: LOW | **Category**: AI Tooling
Current AI test generation works well for unit layer but NeoTrix's Rust codebase with `#![forbid(unsafe_code)]` and complex type constraints may produce AI-generated tests that compile but don't exercise meaningful behavior paths. The "strong fit" claim from JS/TS research doesn't directly transfer.

**Fix**: Benchmark AI test generation against NeoTrix's existing SelfTest suite. Calibrate fitness function for Rust-specific constraints (ownership, lifetime, trait bounds). Track mutation score on AI-generated tests specifically.

---

## Sources Cited

1. digitalapplied.com — "Software Testing Strategy 2026: The Engineering Guide" (2026-06-17)
2. yoo.be — "Test Pyramid vs Testing Trophy: Choosing the Right Test Strategy for Your 2026 CI/CD Pipeline" (2026-08-15)
3. ankurm.com — "Test Pyramid vs Test Trophy: What Actually Works in Production" (2026-04-04)
4. onpathtesting.com — "The Test Pyramid in 2026: Still Relevant, Still Necessary" (2026-03-25)
5. codesnatch.io — "Testing Pyramid vs Trophy: Pick the Right Shape" (2026-03-04)
6. aiwisdom.dev — "Test Architecture Strategy: Pyramid, Trophy, and CI Optimization" (2026-04-02)
7. docs.pact.io — Pact official documentation (2026-08-25)
8. qaskills.sh — "Pact Consumer-Driven Contract Reference 2026" (2026-06-04)
9. qaskills.sh — "Pact Contract Testing Guide 2026" (2026-06-23)
10. apiscout.dev — "Consumer-Driven API Contract Testing with Pact 2026" (2026-03-09)
11. sqaexperts.com — "Consumer-Driven Contract Testing with Pact: Microservices QA Guide 2026" (2026-04-30)
12. aiwisdom.dev — "Contract Testing with Pact" (2026-03-29)
13. oneuptime.com — "How to Handle Contract Testing Between Services" (2026-01-24)
14. testcontainers.com — "Testing REST API integrations using WireMock" (guide)
15. tutorialq.com — "Java Integration Testing: Spring Boot, TestContainers & WireMock 2026" (2026-02-15)
16. github.com/wiremock/wiremock-testcontainers-java (alpha-13 to alpha-16)
17. qaskills.sh — "WireMock API Mocking & Service Virtualization: Complete Guide 2026" (2026-06-23)
18. stevenpg.com — "Ultimate Guide to Testcontainers with Spring Boot" (2026-07-22)
19. wiremock.org/docs/solutions/testcontainers — Official Testcontainers docs
20. blog.devops-monk.com — "WireMock — Testing External REST API Integrations" (2026-05-23)

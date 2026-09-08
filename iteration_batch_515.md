# Iteration Batch 515 — External Research Loop

**Date**: 2026-09-06
**Focus**: Web frameworks, Web APIs, Web Security — latest 2026 advances
**Method**: Multi-source web research → design doc defect identification

---

## Sources Cited

### Web Frameworks
1. [React 19 vs Vue 3.6 vs Svelte 5: 2026 Framework Convergence](https://byteiota.com/react-19-vs-vue-3-6-vs-svelte-5-2026-framework-convergence/) — Jan 2026
2. [Is React Still the King in 2026?](https://svar.dev/blog/react-vs-vue-vs-svelte-for-modern-web-apps/) — Aug 2026
3. [JavaScript Framework Trends in 2026](https://www.nucamp.co/blog/javascript-framework-trends-in-2026-what-s-new-in-react-next.js-vue-angular-and-svelte) — Jan 2026
4. [Modern Frontend Framework Guide 2026](https://releaserun.com/modern-frontend-framework-guide-vue-js-angular-and-react-compared-2026/) — Mar 2026
5. [Vue 3.6 Vapor Mode Hits Feature-Complete](https://blog.imseankim.com/vue-3-6-beta-vapor-mode-virtual-dom-solidjs-svelte-feature-complete-2026/) — Apr 2026

### Web APIs
6. [What's New in WebGPU (Chrome 149-150)](https://developer.chrome.com/blog/new-in-webgpu-149-150) — Jun 2026
7. [WebGL+WebGPU — SIGGRAPH LA Jul 2026](https://www.khronos.org/assets/uploads/developers/presentations/WebGL%2BWebGPU_-_SIGGRAPH_Jul26.pdf) — Jul 2026
8. [Towards a Component Model 1.0 — Luke Wagner @ Wasm I/O 2026](https://www.youtube.com/watch?v=qq0Auw01tH8) — Mar 2026
9. [@huggingface/kernels: 200+ WebGPU Kernels for Local AI](https://huggingface.co/blog/webgpu-kernels) — Sep 2026
10. [WebGPU & WGSL Ecosystem 2026](https://www.youngju.dev/blog/culture/2026-05-16-webgpu-wgsl-ecosystem-2026-chrome-safari-firefox-wgpu-naga-tint-three-js-babylon-webllm-transformers-deep-dive.en) — May 2026
11. [The State of WebAssembly 2025-2026](https://platform.uno/blog/the-state-of-webassembly-2025-2026/) — Jan 2026
12. [WASI-WebGPU 0.3 RC](https://www.webgpu.com/news/wasi-webgpu-03-rc-gpu-compute/) — Aug 2026
13. [WebAssembly in 2026: Edge Computing](https://zylos.ai/research/2026-02-05-webassembly-ecosystem-2026/) — Feb 2026

### Web Security
14. [CSP, CORS, and Secure Headers — Patterns & Anti-Patterns](https://www.sachith.co.uk/csp-cors-and-secure-headers-patterns-anti%e2%80%91patterns-practical-guide-jul-12-2026/) — Jul 2026
15. [OWASP XSS Prevention Cheat Sheet 2026](https://xuro.net/blog/xss-prevention-cheatsheet/) — Jun 2026
16. [CORS Is Not a Security Feature](https://axeploit.com/blog/cors-is-not-a-security-feature-and-other-things-your-config-is-lying-about) — Jul 2026
17. [XSS Prevention Guide 2026: CSP, Encoding, TT](https://aliazlan.net/blog/cross-site-scripting-xss-prevention-a-complete-guide-for-2026) — Feb 2026
18. [HTTP Security Headers 2026 — What Changed](https://httpfixer.dev/changelog/http-security-headers-2026/) — Apr 2026
19. [XSS Prevention Guide for Developers 2026](https://zeriflow.com/blog/xss-prevention-guide-developers-2026) — Apr 2026
20. [XSS Prevention Playbook: Defense in Depth](https://www.securecodinghub.com/blog/xss-prevention-defense-in-depth-developer-guide) — Apr 2026
21. [Content Security Policy: Complete Guide 2026](https://csp-guide.com/posts/content-security-policy-complete-guide/) — Mar 2026

---

## Defects Found

### DEFECT-01: No WebGPU Compute Integration for AI Inference
**Severity**: HIGH
**Design Doc Gap**: NT-IO and NT-PHYSICAL have no WebGPU compute pipeline. CONTEXT.md defines `ResourceBudgetManager` and `ParallelTaskManager` for GPU resource management, but these target local GPU via native APIs only.
**2026 Reality**: WebGPU reached Candidate Recommendation (W3C). All major browsers (Chrome, Firefox, Safari) support it as GA. Hugging Face released 200+ WebGPU kernels for browser-local AI inference (Sep 2026). WebLLM runs LLMs entirely in-browser via WebGPU. WASI-WebGPU 0.3 RC enables portable GPU compute outside the browser.
**Impact**: NeoTrix cannot perform browser-side AI inference, limiting its reach to desktop-only deployments. Tauri app misses opportunity to offload ML workloads to WebGPU.
**Suggestion**: Add `nt_physical::webgpu_compute` module bridging native wgpu to browser WebGPU via Tauri's IPC. Define a `WebGpuKernel` trait implementing Hugging Face's kernel contract format. Register in `ParallelTaskManager` as a dispatch target alongside native CUDA/Metal.

### DEFECT-02: Missing Trusted Types Policy for NT-IO Web Surface
**Severity**: HIGH
**Design Doc Gap**: NT-IO (界面使徒) includes web server and ACP/LSP interfaces but has no Trusted Types policy defined. No `require-trusted-types-for 'script'` CSP directive is specified anywhere.
**2026 Reality**: Trusted Types is the single most impactful new XSS prevention primitive. Supported in Chrome/Edge, shipping in Firefox. Forces DOM sink assignments (`innerHTML`, `outerHTML`, `document.write`) to accept only validated typed objects, not raw strings. OWASP 2026 recommends it as Layer 5 of defense-in-depth.
**Impact**: NT-IO's web surface is vulnerable to DOM-based XSS via framework escape hatches (`dangerouslySetInnerHTML`, `v-html`). Any user-generated content rendered through NT-IO lacks the enforcement layer that prevents entire classes of injection.
**Suggestion**: Define `nt_io::trusted_types_policy` as a central HTML/sink policy. Enforce via CSP: `require-trusted-types-for 'script'; trusted-types neoTrixPolicy`. Wrap all NT-IO HTML output through `TrustedHTML` objects. Add to CONTEXT.md as a new security term.

### DEFECT-03: CSP Configuration Not Specified for Tauri/Desktop
**Severity**: MEDIUM
**Design Doc Gap**: `src-tauri/` is listed as the desktop app, but no CSP policy is defined for the Tauri webview. NT-SHIELD has `Egress Privacy Guard` for outbound LLM filtering, but no browser-enforced CSP for the Tauri webview surface.
**2026 Reality**: CSP Level 3 with nonces and `strict-dynamic` is the standard. Tauri supports CSP injection. Without it, any content loaded in the webview (including third-party scripts, iframes, or injected content) runs unrestricted. The `X-XSS-Protection` header is deprecated across all browsers — CSP is the sole remaining browser-enforced XSS defense.
**Impact**: Tauri desktop app has no browser-enforced script execution controls. If NeoTrix loads any external content (LLM responses, web-crawled pages, plugin UIs), arbitrary script execution is possible.
**Suggestion**: Define `nt_shield::csp_policy` with a default-deny policy: `default-src 'self'; script-src 'self' 'nonce-{random}'; object-src 'none'; frame-ancestors 'none'`. Inject via Tauri's `Content-Security-Policy` config. Use per-response nonces for inline scripts.

### DEFECT-04: No WebAssembly Component Model Integration
**Severity**: MEDIUM
**Design Doc Gap**: NeoTrix is pure Rust. No mention of WebAssembly Component Model for cross-language sandboxing, WASI 0.3 async I/O, or portable module composition.
**2026 Reality**: WASI 0.3 released Feb 2026 with native async I/O (futures, streams). WASI 1.0 expected late 2026. Component Model enables polyglot module composition — write business logic in Rust, data processing in Python, glue in JavaScript, all composable as Wasm components with capability-based security. Wasmtime achieved Core Project status with 95% native performance.
**Impact**: NeoTrix cannot embed untrusted third-party code (plugins, user-contributed skills) in a sandboxed Component Model container. Currently relies on process-level isolation or trust. Plugin ecosystem security is weaker than it needs to be.
**Suggestion**: Add `nt_act::wasm_component_host` implementing the Component Model host API. Use `wasmtime` (already Rust-native, Bytecode Alliance Core Project). Define a `SkillComponent` WIT interface that all skill plugins must implement. This aligns with R-P1 (no unsafe) since Wasmtime provides memory-safety guarantees.

### DEFECT-05: CORS Strategy Not Defined for Cross-Origin KB/API Access
**Severity**: MEDIUM
**Design Doc Gap**: NT-MEMORY (KB) and NT-IO (web server) serve data across origins, but no CORS policy is specified. The `Egress Privacy Guard` covers outbound filtering but not inbound cross-origin access control.
**2026 Reality**: CORS is misunderstood as a security feature — it only restricts browser-initiated cross-origin responses, not server-to-server requests. The common misconfiguration is dynamically reflecting the request origin in `Access-Control-Allow-Origin` while setting `Allow-Credentials: true`, creating a permissive credential-accepting policy. `COEP: credentialless` (Chrome 96+, Firefox 119+) is the modern approach for isolation.
**Impact**: If NT-IO serves KB data or API responses to a web frontend, misconfigured CORS could allow credentialed cross-origin requests from any origin. If NeoTrix embeds third-party content, missing COOP/COEP prevents SharedArrayBuffer usage (needed for WebGPU compute).
**Suggestion**: Define `nt_io::cors_policy` with explicit origin allowlist (never dynamic reflection). Add COOP/COEP headers for cross-origin isolation: `Cross-Origin-Opener-Policy: same-origin; Cross-Origin-Embedder-Policy: credentialless`. Add to NT-SHIELD's audit dimensions.

### DEFECT-06: No Subresource Integrity for Third-Party Dependencies
**Severity**: LOW-MEDIUM
**Design Doc Gap**: No mention of SRI (Subresource Integrity) for any third-party JavaScript/CSS loaded in NT-IO web surfaces or Tauri webview.
**2026 Reality**: Supply chain attacks via compromised CDN scripts are a documented XSS vector. CSP host-based allowlists are demonstrably bypassable because popular CDNs serve JSONP endpoints and script gadgets. SRI pins static assets to known hashes. OWASP 2026 recommends SRI for version-pinned CDN libraries.
**Impact**: If NeoTrix loads any third-party scripts (analytics, UI libraries, LLM provider widgets), a compromised CDN can inject arbitrary code that CSP allows because the host is on the allowlist.
**Suggestion**: Add SRI hash verification for all third-party `<script>` and `<link>` tags in NT-IO templates. Use `integrity` + `crossorigin` attributes. Integrate with NT-SHIELD's supply chain audit (D10). Define in `dev-rules.md` as a new R-P81.

### DEFECT-07: Framework Reactivity Model Not Aligned with 2026 Convergence
**Severity**: LOW
**Design Doc Gap**: CONTEXT.md defines many architectural patterns (Skill Tree, Rune Socketing, Constellations) but does not specify the web frontend reactivity model for NT-IO's dashboard/admin UI.
**2026 Reality**: All major frameworks converged on fine-grained reactivity (Signals/Runes), server-first rendering, and compiler-driven optimization. Svelte 5 Runes ($state/$derived/$effect), Vue 3.6 Vapor Mode (no VDOM), React 19 Compiler (auto-memoization), Angular 21 zoneless Signals. Virtual DOM is effectively deprecated for performance-critical paths.
**Impact**: If NeoTrix builds a web UI without adopting signals-based reactivity, it will carry unnecessary runtime overhead and miss compiler optimizations. The NT-IO web dashboard could use React/Vue/Svelte with signals, but no guidance exists.
**Suggestion**: Document in CONTEXT.md that NT-IO web surfaces use signals-based frameworks (Svelte 5 or Vue 3.6 Vapor when stable). Define a `ReactiveBridge` pattern connecting Tauri IPC events to framework signals. This aligns with the "Spice Must Flow" axiom — clear event→signal→render pipeline.

### DEFECT-08: WebGPU Bindless Resources Not Accounted for in HyperCube Design
**Severity**: LOW
**Design Doc Gap**: VSA HyperCube operations (vector symbolic architecture) are CPU-bound. No plan for WebGPU compute acceleration of high-dimensional vector operations.
**2026 Reality**: WebGPU bindless resources (in active standardization, mid-2026) will allow freely using any resource in any shader via a large index space (65536 slots). Subgroup-based cooperative matrix multiplication for ML is in active standardization. WebGPU compute shaders can accelerate matrix multiply, attention, and vector operations 2-10000x over CPU (Hugging Face benchmarks show 2.57x geometric mean speedup for ML kernels).
**Impact**: VSA HyperCube's high-dimensional vector operations (cosine similarity, binding, bundling) could be dramatically accelerated via WebGPU compute shaders. Currently limited to CPU-bound Rust operations.
**Suggestion**: Prototype `nt_core_hcube::webgpu_backend` using WebGPU compute shaders for vector operations. Define WGSL shaders for HyperCube binding/bundling/unbinding. This would make the consciousness engine viable for browser deployment via Wasm + WebGPU.

---

## Summary

| Category | Defects Found | Severity Distribution |
|----------|--------------|----------------------|
| Web APIs (WebGPU/Wasm) | 3 | 1 HIGH, 1 MEDIUM, 1 LOW |
| Web Security (CSP/TT/CORS) | 3 | 1 HIGH, 2 MEDIUM |
| Web Frameworks | 2 | 1 LOW-MEDIUM, 1 LOW |

**Total**: 8 defects identified across 21 sources.

**Critical Path Items** (address first):
1. **DEFECT-01** (WebGPU compute) — enables browser-side AI inference, unblocks Tauri ML workloads
2. **DEFECT-02** (Trusted Types) — prevents DOM XSS at the browser level, highest-impact security primitive
3. **DEFECT-03** (Tauri CSP) — hardens desktop app against injected content attacks

**Cross-cutting Theme**: NeoTrix's Rust-native architecture is strong for backend reasoning, but its web-facing surfaces (NT-IO, Tauri) lack 2026-era browser security primitives and GPU compute integration. The design is optimized for native deployment but not for the browser-first reality of modern web development.

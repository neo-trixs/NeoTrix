// DEPRECATED — scheduled for removal. Zero consumers confirmed.
/// # Zero-Dependency Binary Strategy
///
/// ## Current State
///
/// neotrix-core/Cargo.toml declares ~70 direct dependencies:
///   - 57 always-on (unconditional)
///   - 14 optional (behind feature flags)
///
/// Workspace: 15 members (neotrix-types, nt-lang, ne-surface, nt-proxy-daemon, ...)
/// Total Rust source: ~440,000 lines across all binaries
///
/// ## Dependency Map
///
/// ### Core (always needed for VSA reasoning engine — 23 crates)
/// | Crate | Purpose | Use site |
/// |-------|---------|----------|
/// | serde | serialization framework | core, neotrix runtime |
/// | serde_json | JSON persistence | config, state save/load |
/// | tokio | async runtime (full features) | everywhere |
/// | chrono | timestamp aware types | timestamps everywhere |
/// | uuid | unique identifiers | entities, sessions |
/// | dirs | platform directory paths | config, data dirs |
/// | rand | randomness for VSA encoding | hypercube, E8, VSA ops |
/// | async-trait | async trait support | handlers, agents |
/// | lru | LRU caches | agent identity, VSA cache |
/// | regex | pattern matching | prompt guard, text extraction |
/// | base64 | encoding (images, data URIs) | vision, crypto, stealth |
/// | hex | hex encoding | content hashing |
/// | url | URL parsing | crawl, navigation |
/// | walkdir | filesystem traversal | code search, file ops |
/// | glob | file pattern matching | config, discovery |
/// | toml | TOML config parsing | neotrix config |
/// | shellexpand | tilde path expansion | config paths |
/// | fastrand | fast non-crypto randomness | internal ops |
/// | sha1 | file/content hashing | content addressing |
/// | sha2 | general purpose hashing | integrity checks |
/// | log | logging facade | diagnostics |
/// | filetime | file timestamp manipulation | file sync |
/// | egg | EGraph term rewriting | reasoning core, inference |
/// | flate2 | zlib decompression | PDF extraction, crawl data |
///
/// ### CLI/UI (binary-only — 6 crates)
/// | Crate | Purpose | Dependency chain |
/// |-------|---------|-----------------|
/// | clap | CLI argument parsing | main.rs, neotrix-web |
/// | clap_complete | shell completion generation | entry/mod.rs |
/// | ratatui | TUI framework | cli/tui/, ne_dialog, ne_dashboard |
/// | crossterm | terminal control | cli/tui/, ne_dialog, ne_dashboard |
/// | colored | terminal color output | entry/mod.rs, proxy_cmd |
/// | indicatif | progress bars | cli/progress.rs |
///
/// ### Network (optional — 5 crates)
/// | Crate | Purpose | Use site |
/// |-------|---------|----------|
/// | reqwest | HTTP client | social, crawl, fetcher |
/// | axum | HTTP server framework | server/, a2a, api, auth |
/// | tokio-tungstenite | WebSocket client | web_navigator |
/// | futures | async stream utilities | sandbox, a2a, server |
/// | futures-util | sink/stream extensions | web_navigator |
///
/// ### Crypto/Identity (optional — 11 crates)
/// | Crate | Purpose | Use site |
/// |-------|---------|----------|
/// | hmac | HMAC signatures | JWT, auth |
/// | sha3 | Keccak-256 | wallet (tx.rs, wallet.rs) |
/// | aes | AES block cipher | crypto provider |
/// | aes-gcm | AEAD encryption | vault, keyvault, channel |
/// | k256 | ECDSA/ECDH (secp256k1) | wallet, signed_card, ohttp |
/// | argon2 | password hashing | credential storage |
/// | hkdf | key derivation | crypto_provider |
/// | ml-kem | post-quantum KEM | crypto_provider |
/// | zeroize | secure memory zeroing | crypto_provider |
/// | rcgen | TLS certificate generation | ca_cert |
/// | rustls | TLS client config | ca_cert, ohttp_gateway |
/// | num-bigint | big integer math | crypto (gram) |
///
/// ### Storage (optional — 3 crates)
/// | Crate | Purpose | Feature |
/// |-------|---------|---------|
/// | rusqlite | SQLite knowledge graph | always-on (bundled) |
/// | rkyv | zero-copy serialization | rkyv-storage |
/// | memmap2 | memory-mapped IO | rkyv-storage |
///
/// **Note:** rusqlite with `bundled` adds ~40s to first compile. It is always-on
/// but could be feature-gated for a truly minimal binary that uses only JSON files.
///
/// ### Telemetry/Observability (optional — 6 crates)
/// | Crate | Purpose | Feature |
/// |-------|---------|---------|
/// | tracing | structured diagnostics | always-on (unused in core) |
/// | tracing-subscriber | log subscriber | always-on (unused in core) |
/// | opentelemetry | OpenTelemetry API | telemetry |
/// | opentelemetry_sdk | OTel SDK | telemetry |
/// | opentelemetry-otlp | OTLP exporter | telemetry |
/// | tracing-opentelemetry | bridge | telemetry |
/// | sentry | error reporting | telemetry |
///
/// **Note:** tracing+tracing_subscriber are always-on but `neotrix/nt_io_telemetry.rs`
/// is the only user. Minimal profile should gate these. They add ~40 deps transitively.
///
/// ### Sandbox (optional — 2 crates)
/// | Crate | Purpose | Feature |
/// |-------|---------|---------|
/// | wasmtime | WebAssembly runtime | sandbox |
/// | agent-sandbox | sandbox API | sandbox |
///
/// ### Evolution/Dev (optional — 5 crates)
/// | Crate | Purpose | Feature/Status |
/// |-------|---------|----------------|
/// | holon | SIMD Holon VSA | simd-vsa |
/// | self_update | binary self-update | self-update |
/// | notify | filesystem watcher | always-on (hot-reload) |
/// | syn | Rust syntax parsing | always-on (graph build) |
/// | proc-macro2 | proc macro support | always-on (transitive) |
/// | chromiumoxide | headless Chrome | stealth-net |
///
/// ### Workspace Members (always-on — 3 crates)
/// | Crate | Purpose |
/// |-------|---------|
/// | neotrix-types | shared type definitions |
/// | nt-lang | Ne programming language |
/// | ne-surface | VSA surface/interaction layer |
///
/// ## Profile Definitions
///
/// ### `minimal` — VSA reasoning core only
/// **Size target:** ~5-8 MB release binary
/// **Gates removed:**
///   - Network (reqwest, axum, tokio-tungstenite, futures, futures-util)
///   - CLI TUI (ratatui, crossterm, indicatif, colored, clap_complete)
///   - Crypto (hmac, sha3, aes, aes-gcm, k256, argon2, hkdf, ml-kem, zeroize, num-bigint)
///   - TLS (rcgen, rustls)
///   - Telemetry (tracing, tracing-subscriber, sentry, opentelemetry*)
///   - Sandbox (wasmtime, agent-sandbox)
///   - Evolution (holon, self_update, notify, syn, proc-macro2)
///   - Storage (rusqlite — replace with JSON files)
///   - Stealth (chromiumoxide)
///   - Keyring
/// **Retained:** serde, serde_json, tokio, chrono, uuid, dirs, rand, async-trait,
///               lru, regex, base64, hex, url, walkdir, glob, toml, shellexpand,
///               fastrand, sha1, sha2, log, filetime, egg, flate2
/// **Required workspace:** neotrix-types, nt-lang, ne-surface
///
/// ### `headless` — daemon/server profile
/// **Size target:** ~10-15 MB release binary
/// **Adds to minimal:**
///   - Network (reqwest, axum, tokio-tungstenite, futures, futures-util)
///   - CLI-only (clap for argv parsing; no TUI: ratatui/crossterm/indicatif removed)
///   - Core crypto for auth (hmac, sha2, aes-gcm — not full wallet stack)
/// **Still removed:**
///   - TUI (ratatui, crossterm, colored, indicatif, clap_complete)
///   - Wallet crypto (k256, sha3, num-bigint, ml-kem)
///   - TLS certificate gen (rcgen, rustls)
///   - Telemetry (tracing*, opentelemetry*, sentry)
///   - Sandbox (wasmtime, agent-sandbox)
///   - Evolution (holon, self_update, notify, syn, proc-macro2)
///   - Storage (rusqlite → JSON fallback)
///   - Stealth (chromiumoxide)
///
/// ### `full` — everything enabled (current default)
/// **Size target:** ~30-40 MB release binary
/// **All features enabled:**
///   - stealth-net, sandbox, telemetry, simd-vsa, self-update
///   - rkyv-storage, keyring
///   - All 57 always-on + 14 optional crates
///   - All 15 workspace members
///
/// ## Feature Flag Design
///
/// ```toml
/// [features]
/// default = ["full"]
///
/// # ——— Profiles ———
/// minimal = []
/// headless = ["network", "cli-core"]
/// full = ["stealth-net", "sandbox", "telemetry", "simd-vsa",
///         "self-update", "rkyv-storage", "keyring"]
///
/// # ——— Feature groups ———
/// network = ["dep:reqwest", "dep:axum", "dep:tokio-tungstenite",
///            "dep:futures", "dep:futures-util"]
/// cli-core = ["dep:clap"]
/// cli-tui = ["dep:ratatui", "dep:crossterm", "dep:colored",
///            "dep:indicatif", "dep:clap_complete"]
/// crypto-wallet = ["dep:k256", "dep:sha3", "dep:num-bigint",
///                  "dep:aes-gcm", "dep:hmac", "dep:argon2",
///                  "dep:hkdf", "dep:ml-kem", "dep:zeroize"]
/// crypto-tls = ["dep:rcgen", "dep:rustls"]
/// storage-sqlite = ["dep:rusqlite"]
/// storage-rkyv = ["dep:rkyv", "dep:memmap2"]
/// telemetry = ["dep:opentelemetry", "dep:opentelemetry_sdk",
///              "dep:opentelemetry-otlp", "dep:tracing-opentelemetry",
///              "dep:sentry"]
/// sandbox = ["dep:wasmtime", "dep:agent-sandbox"]
/// evolution = ["dep:holon", "dep:self_update", "dep:notify",
///              "dep:syn", "dep:proc-macro2"]
/// stealth-net = ["dep:chromiumoxide"]
/// keyring = ["dep:keyring"]
/// ```
///
/// ## Migration Path
///
/// 1. **Gate rusqlite** behind `storage-sqlite` feature (currently always-on with
///    `bundled` feature, adding ~40s compile time and 10+ transitive deps).
///    Fallback: JSON-file based knowledge store when sqlite is disabled.
///
/// 2. **Gate tracing/tracing-subscriber** behind `telemetry` feature. Currently
///    always-on but only `nt_io_telemetry.rs` uses them. They pull in ~40
///    transitive dependencies (matchers, regex-automata, etc.).
///
/// 3. **Gate notify** behind `evolution` feature. Only used in hot-reload.
///
/// 4. **Gate syn/proc-macro2** behind `evolution` feature. Only used in
///    `graph_build.rs` for Rust AST analysis.
///
/// 5. **Gate crypto** into `crypto-wallet` and `crypto-tls` feature groups.
///    Core integrity (sha2/sha1) stays always-on. Wallet/TLS go behind features.
///
/// 6. **Split CLI**: `cli-core` (clap only) vs `cli-tui` (ratatui+crossterm).
///    Headless server needs clap but not TUI.
///
/// 7. **Move futures/futures-util** to network feature. These are only used
///    in async networking contexts (sandbox, a2a, server, websocket).
///
/// ## Compilation Time Impact
///
/// | Profile | Dependencies | Est. clean build | Est. binary size |
/// |---------|-------------|------------------|------------------|
/// | minimal | ~28 direct | 60-90s | 5-8 MB |
/// | headless | ~38 direct | 90-150s | 10-15 MB |
/// | full | ~71 direct | 300-480s | 30-40 MB |
///
/// ## Status
/// Strategy documented. Implementation deferred.
///
/// ## Risk Assessment
///
/// | Risk | Impact | Mitigation |
/// |------|--------|------------|
/// | rusqlite→JSON fallback perf | Knowledge queries 10-100× slower | Keep rusqlite as default, JSON as optional fallback |
/// | tracing removal hides errors | Silent failures in non-telemetry paths | Ensure `log` + `eprintln!` cover all error paths |
/// | futures gating breaks sandbox | Sandbox depends on futures-streams | futures needed within sandbox feature anyway |
/// | Wallet crypto missing | `neotrix wallet` commands fail | Feature-gate entire wallet module, show helpful error |
/// | Workspace members still link | ne-surface pulls in tokio via tonic | Audit ne-surface deps; consider feature-gating tonic |

/// Three binary profiles for controlled dependency inclusion.
///
/// Each profile represents a distinct use case with a different
/// dependency footprint:
///
/// | Profile | Use case | Binary size target |
/// |---------|----------|-------------------|
/// | `Minimal` | Embedded/CI VSA reasoning only | ~5-8 MB |
/// | `Headless` | Server/daemon (HTTP API, no TUI) | ~10-15 MB |
/// | `Full` | Everything (current default) | ~30-40 MB |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DependencyProfile {
    /// Core VSA engine + config loading + logging.
    /// No network I/O, no CLI TUI, no crypto wallet, no database.
    Minimal,
    /// Core + HTTP server + CLI argument parsing.
    /// No TUI, no wallet, no sandbox, no telemetry.
    Headless,
    /// Every feature enabled. Current default behavior.
    Full,
}

/// Returns the set of Cargo feature flags to enable for the given profile.
///
/// These correspond to the `[features]` section proposed in the strategy above.
///
/// # Example
///
/// ```ignore
/// let features = features_for_profile(DependencyProfile::Minimal);
/// // => &["minimal"]
/// ```
pub fn features_for_profile(profile: DependencyProfile) -> &'static [&'static str] {
    match profile {
        DependencyProfile::Minimal => &["minimal"],
        DependencyProfile::Headless => &["headless", "network", "cli-core"],
        DependencyProfile::Full => &[
            "full",
            "stealth-net",
            "sandbox",
            "telemetry",
            "simd-vsa",
            "self-update",
            "rkyv-storage",
            "keyring",
            "crypto-wallet",
            "crypto-tls",
            "storage-sqlite",
            "cli-tui",
            "evolution",
        ],
    }
}

/// Returns `true` if a dependency crate name is mandatory across all profiles.
///
/// Core dependencies are always linked regardless of profile selection.
/// They form the irreducible kernel of the VSA reasoning engine.
pub fn is_core_dependency(name: &str) -> bool {
    matches!(
        name,
        "serde"
            | "serde_json"
            | "tokio"
            | "chrono"
            | "uuid"
            | "dirs"
            | "rand"
            | "async-trait"
            | "lru"
            | "regex"
            | "base64"
            | "hex"
            | "url"
            | "walkdir"
            | "glob"
            | "toml"
            | "shellexpand"
            | "fastrand"
            | "sha1"
            | "sha2"
            | "log"
            | "filetime"
            | "egg"
            | "flate2"
            | "neotrix-types"
            | "nt-lang"
            | "ne-surface"
    )
}

/// Returns a structured report of all dependencies grouped by category.
///
/// Useful for generating documentation or verifying the dependency map
/// against the actual Cargo.toml.
pub fn dependency_report() -> Vec<DependencyGroup> {
    vec![
        DependencyGroup {
            category: "Core (always needed)",
            crates: &[
                "serde",
                "serde_json",
                "tokio",
                "chrono",
                "uuid",
                "dirs",
                "rand",
                "async-trait",
                "lru",
                "regex",
                "base64",
                "hex",
                "url",
                "walkdir",
                "glob",
                "toml",
                "shellexpand",
                "fastrand",
                "sha1",
                "sha2",
                "log",
                "filetime",
                "egg",
                "flate2",
                // Workspace members
                "neotrix-types",
                "nt-lang",
                "ne-surface",
            ],
            count: 27,
        },
        DependencyGroup {
            category: "CLI/UI (binary-only)",
            crates: &[
                "clap",
                "clap_complete",
                "ratatui",
                "crossterm",
                "colored",
                "indicatif",
            ],
            count: 6,
        },
        DependencyGroup {
            category: "Network (optional)",
            crates: &[
                "reqwest",
                "axum",
                "tokio-tungstenite",
                "futures",
                "futures-util",
            ],
            count: 5,
        },
        DependencyGroup {
            category: "Crypto/Identity (optional)",
            crates: &[
                "hmac",
                "sha3",
                "aes",
                "aes-gcm",
                "k256",
                "argon2",
                "hkdf",
                "ml-kem",
                "zeroize",
                "rcgen",
                "rustls",
                "num-bigint",
            ],
            count: 12,
        },
        DependencyGroup {
            category: "Storage (optional)",
            crates: &["rusqlite", "rkyv", "memmap2"],
            count: 3,
        },
        DependencyGroup {
            category: "Telemetry/Observability (optional)",
            crates: &[
                "tracing",
                "tracing-subscriber",
                "opentelemetry",
                "opentelemetry_sdk",
                "opentelemetry-otlp",
                "tracing-opentelemetry",
                "sentry",
            ],
            count: 7,
        },
        DependencyGroup {
            category: "Sandbox (optional)",
            crates: &["wasmtime", "agent-sandbox"],
            count: 2,
        },
        DependencyGroup {
            category: "Evolution/Dev (optional)",
            crates: &[
                "holon",
                "self_update",
                "notify",
                "syn",
                "proc-macro2",
                "chromiumoxide",
            ],
            count: 6,
        },
        DependencyGroup {
            category: "Keychain/Secrets (optional)",
            crates: &["keyring"],
            count: 1,
        },
    ]
}

/// A named group of related dependencies with a count.
#[derive(Debug, Clone)]
pub struct DependencyGroup {
    /// Human-readable category name (e.g. "Network (optional)")
    pub category: &'static str,
    /// Crate names in this group
    pub crates: &'static [&'static str],
    /// Number of crates (can differ from `crates.len()` in grouped entries)
    pub count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_features() {
        let f = features_for_profile(DependencyProfile::Minimal);
        assert!(f.contains(&"minimal"));
        assert!(!f.contains(&"network"));
        assert!(!f.contains(&"cli-tui"));
    }

    #[test]
    fn test_headless_features() {
        let f = features_for_profile(DependencyProfile::Headless);
        assert!(f.contains(&"headless"));
        assert!(f.contains(&"network"));
        assert!(!f.contains(&"cli-tui"));
    }

    #[test]
    fn test_full_features() {
        let f = features_for_profile(DependencyProfile::Full);
        assert!(f.contains(&"full"));
        assert!(f.contains(&"sandbox"));
    }

    #[test]
    fn test_core_dependency_identification() {
        assert!(is_core_dependency("serde"));
        assert!(is_core_dependency("tokio"));
        assert!(is_core_dependency("egg"));
        assert!(is_core_dependency("neotrix-types"));
        assert!(!is_core_dependency("axum"));
        assert!(!is_core_dependency("ratatui"));
        assert!(!is_core_dependency("wasmtime"));
        assert!(!is_core_dependency("rusqlite"));
    }

    #[test]
    fn test_report_covers_major_groups() {
        let report = dependency_report();
        let names: Vec<&str> = report.iter().map(|g| g.category).collect();
        assert!(names.contains(&"Core (always needed)"));
        assert!(names.contains(&"Network (optional)"));
        assert!(names.contains(&"CLI/UI (binary-only)"));
        assert!(names.contains(&"Crypto/Identity (optional)"));
        assert!(names.contains(&"Storage (optional)"));
        assert!(names.contains(&"Telemetry/Observability (optional)"));
        assert!(names.contains(&"Sandbox (optional)"));
    }

    #[test]
    fn test_report_totals() {
        let report = dependency_report();
        let total: usize = report.iter().map(|g| g.count).sum();
        // All always-on (27) + optional (43) = ~70
        assert!(
            total >= 60 && total <= 80,
            "Expected ~70 dependencies, got {}. If this fails, update the report.",
            total
        );
    }

    #[test]
    fn test_all_core_deps_in_report() {
        let report = dependency_report();
        let all_crates: Vec<&str> = report
            .iter()
            .flat_map(|g| g.crates.iter())
            .copied()
            .collect();
        // Spot-check that key deps are listed somewhere
        for dep in &[
            "serde", "tokio", "axum", "clap", "rusqlite", "wasmtime", "k256",
        ] {
            assert!(all_crates.contains(dep), "Missing from report: {}", dep);
        }
    }
}

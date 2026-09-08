# ADR: NeoTrix-Telegram iOS Hybrid Architecture

**Status**: Accepted
**Date**: 2026-08-07
**Context**: Building an iOS app that fuses NeoTrix core AI capabilities with Telegram iOS source code, implementing all Telegram Premium features.

## Context

We need to build an iOS application that:
1. Uses NeoTrix's Rust core (E8 Hexagram reasoning, VSA HyperCube, GWT attention routing, ConsciousnessTree, SEAL pipeline)
2. Adopts Telegram iOS architecture patterns (274 modules, Bazel build, MTProto, AsyncDisplayKit)
3. Implements ALL Telegram Premium features (double limits, no ads, voice-to-text, peer colors, animated emoji, stories, reactions, etc.)
4. Adds NeoTrix AI features as unique differentiators

## Decision

### Hybrid Architecture: "NeoGram" Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        NeoGram iOS App                          │
├─────────────────────────────────────────────────────────────────┤
│  SwiftUI / UIKit Layer (Telegram UI Patterns)                   │
│  ├── ChatUI (ChatController, Message Bubbles, Reactions)       │
│  ├── ChatListUI (Chat List, Filters, Folders)                  │
│  ├── SettingsUI (Premium, Privacy, Themes, Passcode)           │
│  ├── PremiumUI (Intro, Limits, Gifts, Boost Levels)            │
│  ├── StoriesUI (Story Camera, Viewer, Replies)                 │
│  ├── ContactsUI, CallUI, MediaPickerUI                         │
│  └── ComponentFlow (Declarative UI Components)                 │
├─────────────────────────────────────────────────────────────────┤
│  NeoTrix Bridge Layer (Swift ↔ Rust FFI)                        │
│  ├── NeoTrixFFI (C-ABI bridge, uniffi-generated)               │
│  ├── CapabilityBridge (E8, HyperCube, GWT, ConsciousnessTree)  │
│  ├── SEALPipelineBridge (Evolution, Distillation, SelfTest)    │
│  └── KBridge (Knowledge Base, Embeddings, Search)              │
├─────────────────────────────────────────────────────────────────┤
│  Telegram Core Layer (Ported from Telegram-iOS)                 │
│  ├── MTProtoKit (Network, Encryption, Transport)               │
│  ├── TelegramCore (Engine, Account, Postbox, Network)          │
│  ├── TelegramApi (TL Schema → Swift Types)                     │
│  └── SwiftSignalKit (Reactive Primitives)                      │
├─────────────────────────────────────────────────────────────────┤
│  NeoTrix Rust Core (Compiled to staticlib + uniffi)             │
│  ├── nt_core_self (E8, GWT, HyperCube, ConsciousnessTree)      │
│  ├── nt_mind (SEAL Pipeline, Skill Tree, Evolution)            │
│  ├── nt_memory (KB, Spatial Memory, Historian)                 │
│  ├── nt_world (Absorber, Sensors, Models)                      │
│  └── nt_act, nt_shield, nt_io (Capabilities)                   │
└─────────────────────────────────────────────────────────────────┘
```

### Key Architectural Decisions

| Decision | Rationale |
|----------|-----------|
| **Bazel Build System** | Matches Telegram iOS; enables 274-module parallel builds, reproducible builds, remote caching |
| **uniffi for Rust→Swift FFI** | Type-safe, generates Swift bindings automatically, supports async, handles memory management |
| **Telegram UI Modules as Swift Packages** | Modular, testable, matches Telegram's submodule architecture |
| **AsyncDisplayKit (Texture) for Chat** | 60fps scrolling with complex message layouts; Telegram's proven choice |
| **SwiftSignalKit for Reactive** | Telegram's battle-tested reactive framework; integrates with MTProto |
| **MTProtoKit for Networking** | Full Telegram compatibility; end-to-end encryption, perfect forward secrecy |
| **NeoTrix Core as Static Library** | Zero-copy FFI calls; Rust performance for AI reasoning |

### Module Mapping (Telegram → NeoGram)

| Telegram Module | NeoGram Module | Status |
|-----------------|----------------|--------|
| `ChatController` | `NeoGramChatController` | Port + NeoTrix AI integration |
| `ChatListController` | `NeoGramChatListController` | Port + AI-powered filtering |
| `PremiumUI` | `NeoGramPremiumUI` | Full port + NeoTrix premium tiers |
| `SettingsUI` | `NeoGramSettingsUI` | Port + NeoTrix config |
| `StoriesUI` | `NeoGramStoriesUI` | Port + AI story generation |
| `ComponentFlow` | `NeoGramComponentFlow` | Port for declarative UI |
| `MTProtoKit` | `NeoGramMTProto` | Direct port (core protocol) |
| `TelegramCore` | `NeoGramCore` | Port + NeoTrix engine injection |
| `Postbox` | `NeoGramPostbox` | Port + VSA HyperCube storage |
| `SwiftSignalKit` | `NeoGramSignalKit` | Direct port |

### NeoTrix AI Integration Points

1. **Chat Intelligence**: E8 Hexagram analyzes conversation patterns → suggests replies, summarizes threads
2. **Smart Filtering**: GWT attention routing prioritizes messages → "NeoTrix Priority Inbox"
3. **Knowledge Injection**: VSA HyperCube retrieves relevant context → inline knowledge cards
4. **ConsciousnessTree Monitoring**: Self-health dashboard in Settings → "AI Health"
5. **SEAL Pipeline**: Background evolution → "Auto-Improving AI" toggle
6. **Skill Tree Visualization**: POE-style passive tree → "AI Capabilities" screen

### Premium Feature Implementation Strategy

| Premium Feature | Implementation Approach |
|-----------------|------------------------|
| Double Limits | Config flags in `NeoGramCore`; UI gates in `PremiumUI` |
| No Ads | Ad module stubbed out; `PremiumStatus.noAds` gate |
| Voice-to-Text | Integrate Apple Speech + Whisper.cpp via Rust |
| Peer Colors | `PeerColorManager` + theme engine |
| Animated Emoji | Lottie + Telegram's `.tgs` format support |
| Stories | Full `StoriesUI` port + AI story composer |
| Reactions | `ReactionEngine` + custom reaction packs |
| Emoji Status | `EmojiStatusManager` + profile integration |
| App Icons | `AppIconManager` with dynamic icons |
| Chat Folders | `ChatFolderManager` with AI-suggested folders |
| Translation | `TranslateUI` + NeoTrix multilingual models |
| Wallpapers | `WallpaperEngine` with animated gradients |

## Consequences

### Positive
- Full Telegram compatibility (can connect to Telegram servers)
- NeoTrix AI as unique differentiator
- Modular, maintainable architecture
- Proven Telegram UI patterns
- Reproducible builds via Bazel

### Negative
- High initial complexity (274+ modules)
- Bazel learning curve
- Rust↔Swift FFI debugging complexity
- Large binary size (~100MB+)
- App Store review scrutiny for Telegram-compatible app

### Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Telegram API changes | Abstract `TelegramEngine` protocol; version pinning |
| FFI performance | Benchmark critical paths; use `uniffi` zero-copy |
| App Store rejection | Clear differentiation: "AI-enhanced messaging"; own branding |
| Build complexity | CI/CD with Bazel remote cache; prebuilt Rust artifacts |

## Implementation Phases

### Phase 1: Foundation (Week 1-2)
- [ ] Bazel workspace setup
- [ ] Rust core → staticlib + uniffi bindings
- [ ] Swift package structure mirroring Telegram modules
- [ ] MTProtoKit port + basic connection

### Phase 2: Core UI (Week 3-4)
- [ ] ChatListUI + ChatController port
- [ ] Message rendering pipeline (bubbles, media, reactions)
- [ ] ComponentFlow declarative UI
- [ ] Theme engine + dark mode

### Phase 3: Premium Features (Week 5-6)
- [ ] PremiumUI (Intro, Limits, Gifts, Boost)
- [ ] StoriesUI (Camera, Viewer, Replies)
- [ ] Voice-to-Text, Translation, Peer Colors
- [ ] Animated Emoji, Emoji Status, App Icons

### Phase 4: NeoTrix AI Integration (Week 7-8)
- [ ] E8 Reasoning in chat (smart replies, summarization)
- [ ] GWT Attention Routing (priority inbox)
- [ ] VSA HyperCube (knowledge cards)
- [ ] ConsciousnessTree Dashboard
- [ ] SEAL Pipeline Background Evolution

### Phase 5: Polish & Ship (Week 9-10)
- [ ] Passcode/FaceID, App Lock
- [ ] App Extensions (Share, Widget, Notifications)
- [ ] TestFlight beta + feedback
- [ ] App Store submission

## Validation

- **rev-officer**: D3 (layer compliance), D4 (production safety), D17 (SelfTest coverage)
- **dev-implementer**: Feasibility spike for Rust→Swift FFI + MTProto connection
- **Benchmark**: 60fps chat scrolling, <100ms message send latency, <50MB memory baseline

---

*This ADR follows the NeoTrix architecture decision process. See `dev-rules.md` for review requirements.*
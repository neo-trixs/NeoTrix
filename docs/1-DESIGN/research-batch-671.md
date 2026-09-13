# Research Batch 671 — Game/3D/VTT Repository Analysis

**Date**: 2026-09-13
**Repos**: 5 | **Theme**: 3D worlds, desktop agents, RPG resources, game assets, interactive maps

---

## 1. cclank/clay-safari

**URL**: https://github.com/cclank/clay-safari
**Stars**: 168 | **License**: MIT | **Stack**: Three.js r185, Vite 8, Blender 5.x, Web Audio API

### Architecture Patterns

| Pattern | Detail |
|---------|--------|
| **Procedural Generation Pipeline** | Blender Python scripts → GLB export → Three.js runtime. 19 models built in ~10s, zero external assets. |
| **Joint-Pivot Animation** | Moving parts on Empty nodes with `matrix_parent_inverse` pivots. No skeleton needed — Three.js rotates at joints directly. |
| **Noise-Based Texture Synthesis** | Fractal noise generates seamless bump maps at runtime. `MeshPhysicalMaterial` sheen for clay feel. |
| **Grid A\* Pathfinding** | 0.5-unit grid, 8-directional, no diagonal corner cutting. Robot guide uses this for auto-tour. |
| **Web Audio Procedural Sound** | Oscillators + noise + biquad filters + waveshapers synthesize 17 animal sounds. Zero audio files. |
| **Performance Tiers** | Desktop: 4096 shadow maps, Bloom. Mobile: no post-processing, shadow 2048, pixel ratio ≤1.5. |

### Fusion Opportunities with NeoTrix

| Opportunity | NeoTrix Module | Implementation |
|-------------|----------------|----------------|
| **Procedural asset generation pipeline** | `nt_world::media_asset_registry` | Extend AssetRegistry to support Blender script → GLB → runtime material pipeline. Add `ProceduralAssetGenerator` node that chains script execution, export, and material synthesis. |
| **Audio synthesis for NT-FEEL** | `nt_feel` emotion engine | Adopt Web Audio oscillator/filter pattern for procedural emotion soundscapes. Each EmotionLabel variant triggers a unique synthesis chain (Joy→major chord arpeggios, Fear→dissonant tremolo). |
| **Mobile/desktop performance tiering** | `nt_io::platform_gateway` | Generalize the desktop↔mobile degradation pattern into a `PerformanceTierRouter` that automatically adjusts rendering quality, shadow maps, and post-processing based on detected platform. |
| **A\* pathfinding for NT-ACT** | `nt_act::production_orchestrator` | Use grid-based A\* for task dependency graph traversal. Nodes = tasks, edges = dependencies, weight = estimated cost. Robot-guide pattern → auto-sequencer for batch production. |
| **Zero-asset philosophy** | `nt_file_ability` | All content (models, textures, audio) generated procedurally. Aligns with R-P79 (no external dead code). Extend `NtFileAbility` domain with `ProceduralContentGenerator` capability. |

---

## 2. rullerzhou-afk/clawd-on-desk

**URL**: https://github.com/rullerzhou-afk/clawd-on-desk
**Stars**: 6.2k | **License**: AGPL-3.0 | **Stack**: Electron, Node.js, multi-agent hooks

### Architecture Patterns

| Pattern | Detail |
|---------|--------|
| **Multi-Agent State Machine** | 12 animated states (idle/thinking/typing/building/groove/juggle/error/happy/notification/sweeping/carrying/sleeping). State driven by hook events from 20+ AI coding agents. |
| **Hook-Based Agent Integration** | Each agent (Claude Code, Codex, Copilot, Gemini, Cursor, etc.) registers command hooks or HTTP permission hooks. Clawd observes lifecycle events. |
| **Permission Bubble Architecture** | In-app floating card for permission review. Allow/Deny/Always. Remote approval via Telegram/Feishu. Stackable layout. Auto-dismiss if answered in terminal first. |
| **Session Intelligence** | Multi-session tracking, subagent awareness (headphones=groove for 1, juggling for 2+), process liveness detection, startup recovery. |
| **Theme System** | Codex Pet import, custom themes via `theme.json` + assets (SVG/GIF/APNG/WebP). Capability badges per theme. Theme validation script. |
| **Mini Mode** | Edge-docking with peek-on-hover, mini alerts, parabolic jump transitions. Position memory across restarts. |

### Fusion Opportunities with NeoTrix

| Opportunity | NeoTrix Module | Implementation |
|-------------|----------------|----------------|
| **Agent State Visualization** | `nt_feel` + `nt_io` | Map NeoTrix 7-domain health signals to Clawd-style animated states. NT-CORE→idle, NT-MIND→thinking, NT-ACT→typing/building, NT-SHIELD→sweeping. Desktop pet becomes live ConsciousnessTree avatar. |
| **Permission Bubble for NT-SHIELD** | `nt_shield::sandbox` | Adopt Clawd's permission bubble pattern for sandbox egress requests. When `EgressPrivacyGuard` blocks a request, show floating card with Allow/Deny/Always. Remote approval via Telegram. |
| **Multi-Agent Session Tracking** | `nt_meta::cross_module_audit` | Clawd's session intelligence (multi-session, subagent awareness, process liveness) maps to NeoTrix's cross-module audit. Track SEAL pipeline stages as "sessions" with subagent spawning. |
| **Theme System for NT-IO** | `nt_io` (CLI/Tauri) | Adopt theme.json + capability badges pattern for NeoTrix Tauri desktop. Themes control emotion expression display, GWT attention visualization, KB search UI. |
| **Hook Pattern for EventBus** | `nt_core::eventbus` | Clawd's hook registration (command hooks + HTTP hooks) is a clean EventSource pattern. Adapt for NeoTrix EventBus: modules register hooks for specific event types, auto-cleanup on crash. |

---

## 3. neovatar/awesome-rpg-resources

**URL**: https://github.com/neovatar/awesome-rpg-resources
**Stars**: 8 | **License**: None | **Content**: Curated RPG tool list

### Architecture Patterns

| Pattern | Detail |
|---------|--------|
| **Procedural Map Generation** | Watabou suite (dungeon, village, mansion, city generators) + Azgaar's Fantasy Map Generator + Red Blob Games mapgen4. All produce structured map data from seed parameters. |
| **VTT Plugin Ecosystem** | FoundryVTT self-hosted + marketplace modules. Roll20 freemium + marketplace. Fantasy Grounds with automation. Pattern: core engine + extension marketplace. |
| **Tile-Based Rendering** | gdal2tiles pattern for splitting high-res images into zoom-level chunks. Leaflet.js for interactive rendering with pan/zoom. |
| **GeoJSON Marker System** | Structured marker data in GeoJSON format. Marker logic separate from marker data. Edit pane for interactive marker placement. |

### Fusion Opportunities with NeoTrix

| Opportunity | NeoTrix Module | Implementation |
|-------------|----------------|----------------|
| **Procedural Map Generation for NT-WORLD** | `nt_world::crawl` | Adopt Watabou/Azgaar patterns for procedural world generation. Seed → biome map → dungeon/city/village. Maps become KB spatial index for crawled content. |
| **VTT Plugin Model for Skill System** | `nt_mind::skill_engine` | FoundryVTT's core+extension pattern maps to NeoTrix skill nodes. Each skill = module with `SKILL.md` contract (Easel pattern). Marketplace = KB `experience` namespace with versioned skill snapshots. |
| **Tile-Based Content Indexing** | `nt_memory::kb` | Use gdal2tiles-style chunking for large document indexing. Split documents into semantic "tiles" at multiple zoom levels (paragraph/sentence/token). Enables multi-resolution retrieval. |
| **GeoJSON for Knowledge Graph** | `nt_memory::kb` | Extend KB node schema with GeoJSON geometry fields. Enable spatial queries: "find all knowledge nodes within semantic distance X of concept Y." Pattern from interactive map template. |

---

## 4. Huu-Yuu/StardewValley-Assets

**URL**: https://github.com/Huu-Yuu/StardewValley-Assets
**Stars**: 84 | **License**: MIT (non-commercial use) | **Content**: Game asset collection (maps, NPCs, items)

### Architecture Patterns

| Pattern | Detail |
|---------|--------|
| **Asset Taxonomy** | 4 categories: Map assets (farm/town/mine/forest), NPC assets (sprites/expressions/actions), Item assets (icons/details), UI assets (menus/dialogs). |
| **Versioned Asset Releases** | Assets packaged as RAR, distributed via GitHub Releases. Version-tagged for reproducibility. |
| **Cross-Reference Indexing** | Links to sister repo (PixelSRPG-Forge) for SRPG assets. Pattern: asset family with shared taxonomy. |

### Fusion Opportunities with NeoTrix

| Opportunity | NeoTrix Module | Implementation |
|-------------|----------------|----------------|
| **Asset Taxonomy for MediaAssetRegistry** | `nt_world::media_asset_registry` | Adopt 4-category taxonomy (Map/NPC/Item/UI) as `AssetCategory` enum. Extend MediaAssetRegistry with typed asset slots per category, version tracking, and cross-family links. |
| **Versioned Asset Snapshots** | `nt_memory::kb` | Each asset version = KB node with `version` edge. Enables "find all assets at version X" queries. Pattern from GitHub Releases → KB `experience` namespace versioning. |
| **Non-Commercial License Gate** | `nt_shield::risk_assessor` | Extend RiskAssessor to check license compliance. Asset ingestion scans for license headers, flags commercial-use-incompatible assets. Aligns with R-P82 (risk grading). |

---

## 5. interactive-game-maps/template

**URL**: https://github.com/interactive-game-maps/template
**Stars**: 25 | **License**: None | **Stack**: Leaflet.js, GeoJSON, gdal2tiles

### Architecture Patterns

| Pattern | Detail |
|---------|--------|
| **Tile Pyramid Generation** | gdal2tiles splits high-res images into zoom-level tile pyramids. `-z 0-5` controls depth. Output: directory tree of tiles. |
| **Layer-Data Separation** | `map.js` defines layers/metadata. `marker/` holds GeoJSON data. `marker_logic/` holds behavior. Clean MVC separation. |
| **Interactive Marker Editor** | In-browser edit pane for marker placement. Export to GeoJSON. Import replaces example markers. |
| **Common Utilities** | `common/` directory for shared functions. `map_utils.js` for map manipulation helpers. |

### Fusion Opportunities with NeoTrix

| Opportunity | NeoTrix Module | Implementation |
|-------------|----------------|----------------|
| **Tile Pyramid for KB Visualization** | `nt_memory::kb` | Use tile pyramid pattern for visual KB exploration. Each zoom level = different abstraction (L1: raw text, L2: key concepts, L3: topic clusters, L4: domain summary). Leaflet.js integration for Tauri desktop. |
| **Layer-Data Pattern for Skill Rendering** | `nt_mind::skill_engine` | Separate skill definition (layer) from skill data (marker) from skill behavior (marker_logic). Clean separation enables hot-swapping skill implementations without data migration. |
| **Marker Editor for KB Node Editing** | `nt_io` | In-browser/CLI marker editor pattern → KB node editor. Place nodes spatially on concept map, edit properties, export as KB batch import. |
| **GeoJSON Spatial Index** | `nt_memory::kb` | Add spatial dimension to KB. Nodes have lat/lng/elevation. Queries: "find nodes near point X" enables geographic knowledge organization. |

---

## Cross-Repo Synthesis

### Shared Patterns Across All 5 Repos

| Pattern | Repos | NeoTrix Mapping |
|---------|-------|-----------------|
| **Procedural Generation** | clay-safari (models/textures/audio), RPG-resources (maps/dungeons), interactive-maps (tiles) | `nt_world::procedural` — unified procedural generation engine |
| **Layer-Data Separation** | interactive-maps (map/marker/logic), clawd-on-desk (state/animation/theme) | `nt_mind::skill_engine` — skill definition/data/behavior separation |
| **Zero-Asset Philosophy** | clay-safari (all procedural), RPG-resources (generators not assets) | `nt_file_ability` — prefer synthesis over storage |
| **Multi-Format Support** | clawd-on-desk (SVG/GIF/APNG/WebP), StardewValley (PNG/RAR), interactive-maps (GeoJSON/tiles) | `nt_io::platform_gateway` — format-agnostic content pipeline |
| **Performance Tiering** | clay-safari (desktop/mobile), clawd-on-desk (mini/full) | `nt_io::performance_router` — adaptive quality based on platform |

### Priority Implementation Recommendations

| Priority | Task | Source Pattern | NeoTrix Target |
|----------|------|----------------|----------------|
| P0 | Agent State Dashboard | clawd-on-desk state machine | `nt_feel` emotion visualization in Tauri |
| P0 | Permission Bubble for Egress | clawd-on-desk permission system | `nt_shield::sandbox` floating approval |
| P1 | Procedural Content Pipeline | clay-safari generation chain | `nt_world::procedural` engine |
| P1 | Tile Pyramid KB Explorer | interactive-maps tile system | `nt_memory::kb` visual explorer |
| P2 | Asset Taxonomy System | StardewValley-Assets categorization | `nt_world::media_asset_registry` typed slots |
| P2 | Skill Layer Separation | interactive-maps MVC | `nt_mind::skill_engine` hot-swap |
| P3 | Procedural Audio for Emotion | clay-safari Web Audio synthesis | `nt_feel` emotion soundscapes |
| P3 | GeoJSON Spatial KB | interactive-maps + RPG-resources | `nt_memory::kb` spatial queries |

---

## Appendix: Repo Quick Stats

| Repo | Stars | Language | Key Innovation |
|------|-------|----------|----------------|
| clay-safari | 168 | JavaScript | Zero-asset 3D world, procedural everything |
| clawd-on-desk | 6.2k | JavaScript | 20+ AI agent desktop pet, permission bubbles |
| awesome-rpg-resources | 8 | Markdown | Curated RPG tool taxonomy |
| StardewValley-Assets | 84 | Binary | Game asset categorization pattern |
| interactive-game-maps/template | 25 | JavaScript | Tile pyramid + GeoJSON marker system |

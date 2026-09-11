# NeoTrix-SIM Rankings & Scan — V6

**Date**: 2026-09-11

---

## Part 1: GitHub Rankings

### Rust Trending (Weekly)

| # | Repo | Stars | Stars/Week | Description |
|---|------|-------|------------|-------------|
| 1 | clash-verge-rev/clash-verge-rev | 143.7k | 1,737 | Modern GUI proxy client (Tauri) |
| 2 | rtk-ai/rtk | 79.9k | 1,520 | CLI proxy reducing LLM token consumption 60-90% |
| 3 | lencx/ChatGPT | 54.5k | 62 | ChatGPT Desktop Application |
| 4 | astral-sh/ruff | 49.6k | 129 | Extremely fast Python linter/formatter |
| 5 | ajeetdsouza/zoxide | 39.4k | 228 | Smarter cd command |
| 6 | jj-vcs/jj | 31.5k | 168 | Git-compatible VCS |
| 7 | lbjlaq/Antigravity-Manager | 31.2k | 288 | Antigravity account manager (Tauri) |
| 8 | CapSoftware/Cap | 22.1k | 480 | Open-source Loom alternative |
| 9 | jlcodes99/cockpit-tools | 17.5k | 493 | Universal AI IDE account manager |
| 10 | eythaann/Seelen-UI | 17.8k | 122 | Customizable desktop environment (Win 10/11) |

### Bevy Ecosystem (Top 10 by Stars)

| # | Repo | Stars | Description |
|---|------|-------|-------------|
| 1 | bevyengine/bevy | 48.1k | Data-driven game engine in Rust |
| 2 | valence-rs/valence | 3.3k | Rust framework for Minecraft servers |
| 3 | avianphysics/avian | 3.2k | ECS-driven 2D/3D physics engine |
| 4 | bevy-cheatbook/bevy-cheatbook | 2.4k | Unofficial Bevy reference book |
| 5 | fishfolk/jumpy | 1.9k | Tactical 2D shooter (Rust + Bevy) |
| 6 | vladbat00/bevy_egui | 1.4k | Egui integration for Bevy |
| 7 | djeedai/bevy_hanabi | 1.4k | GPU particle system plugin |
| 8 | zkat/big-brain | 1.3k | Utility AI library for Bevy |
| 9 | NiklasEi/bevy_game_template | 1.1k | Bevy game template (CI/CD for all platforms) |
| 10 | cBournhonesque/lightyear | 1.1k | Multiplayer networking for Bevy |

### Game-AI (Top 10 by Stars)

| # | Repo | Stars | Language | Description |
|---|------|-------|----------|-------------|
| 1 | recastnavigation/recastnavigation | 7.9k | C++ | Industry-standard nav-mesh toolset |
| 2 | kwai/DouZero | 4.7k | Python | Dou Dizhu AI (ICML 2021) |
| 3 | tomlooman/ActionRoguelike | 4.6k | C++ | Co-op Action Roguelike (Unreal) |
| 4 | datamllab/rlcard | 3.5k | Python | RL/AI bots in card games |
| 5 | Tencent/behaviac | 3.0k | C# | Game AI framework (BT/FSM/HTN) |
| 6 | sturdyspoon/unity-movement-ai | 2.1k | C# | Unity movement AI library |
| 7 | Equim-chan/Mortal | 1.5k | Rust | Riichi mahjong AI (deep RL) |
| 8 | YGYOOO/WorldX | 1.5k | TypeScript | AI-driven world generation |
| 9 | Mugen87/yuka | 1.4k | JavaScript | Game AI library |
| 10 | Yuan-ManX/ai-game-devtools | 1.3k | JS | AI Game Dev Hub (LLMs, agents, tools) |

---

## Part 2: neotrix-sim Codebase Scan

### Build Status

```
cargo check -p neotrix-sim
```

**Result**: Clean — no errors, no warnings.

### Test Results

```
test result: ok. 325 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

**Result**: 325/325 passing, 0.04s wall time.

### File Metrics

| Metric | Value |
|--------|-------|
| Agent source files (`src/agents/*.rs`) | 16 |
| `world_sim.rs` line count | 1,550 |

---

## Summary

| Check | Status |
|-------|--------|
| `cargo check` | Clean |
| `cargo test --lib` | 325/325 pass |
| Agent modules | 16 .rs files |
| World sim size | 1,550 lines |

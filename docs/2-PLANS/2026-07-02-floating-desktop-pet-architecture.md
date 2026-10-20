# Floating 3D Desktop Pet Architecture

## Overview

The NeoTrix floating pet is a self-evolving, anthropomorphic 3D-pseudo desktop companion embedded in `preview-ui-v2.html`. It externally manifests all NeoTrix system capabilities (E8, GWT, SEAL, Memory, Tool, Evolve) as orbiting skill nodes around a Canvas-rendered character that grows through interaction.

## Architecture

### Layer Mapping (9-Layer Framework)

| Pet Component | NeoTrix Layer | Description |
|--------------|---------------|-------------|
| Canvas Character (body/face) | L1 Body | Physical manifestation of consciousness |
| VAD Mood Engine | L5 Consciousness | Valence-Arousal-Dominance emotional state |
| Orbiting Skill Nodes | L7 Capability | External display of routing/ability layer |
| Evolution State Machine | L8 Autonomic | Self-evolution without conscious control |
| XP/Leveling System | L3 Memory | Experience → growth → new capabilities |
| Idle Mood → Dim | L9 Transcendent | Self-observation of neglect triggers withdrawal |

### Core State (`pet` Object)

```
pet.state = {
  level: 1,           // 1-∞, determines evolution tier
  xp: 0,              // current XP toward next level
  xpNext: 100,        // XP needed (grows by 1.5x each level)
  evolution: 0,       // 0-5: 卵/幼体/成长体/成熟体/完全体/超验体
  mood: 'neutral',    // happy | neutral | tired | excited
  v: 0.5,             // Valence (pleasure)
  a: 0.5,             // Arousal (activation)
  d: 0.5,             // Dominance (control)
  isDragging: false,
  tick: 0,            // frame counter
  nodeAngle: 0,       // orbital rotation angle
  lastInteraction: timestamp,
  blinkFrame: 0
}
```

### Evolution Tiers

| Level | Evolution | Name | Visual Features | Color |
|-------|-----------|------|-----------------|-------|
| 1-2 | 0 | 卵 (Egg) | Simple shape, no ears, minimal eyes | `#f0d6b0` |
| 3-5 | 1 | 幼体 (Baby) | Round face, ears appear, big eyes | `#b8e0c0` |
| 6-8 | 2 | 成长体 (Child) | Full eyes+blush, inner ear shown | `#80c8e0` |
| 9-11 | 3 | 成熟体 (Teen) | Arms, glow effect, cyber lines | `#c090f0` |
| 12-14 | 4 | 完全体 (Adult) | Full cyber body, strong glow | `#f0c040` |
| 15+ | 5 | 超验体 (Transcended) | Crown/halo, stars, max glow | `#ff80e0` |

## Interactions

### Direct Character
| Action | Response | XP |
|--------|----------|-----|
| Click (pet head) | Mood→happy, headpat animation | 5-15 |
| Double-click | Status display (name/level/XP) | 0 |
| Drag (canvas) | Free reposition, pause float anim | 0 |

### Orbiting Nodes
| Node | Color | Symbol | Click Response |
|------|-------|--------|---------------|
| E8 | `#8b5cf6` | ⟐ | "E8: 状态引擎" |
| GWT | `#06b6d4` | ◈ | "GWT: 全局工作空间" |
| SEAL | `#f59e0b` | ◉ | "SEAL: 自我进化" |
| 记忆 | `#10b981` | ◆ | "记忆: 记忆矩阵" |
| 工具 | `#ef4444` | ⚡ | "工具: 工具血脉" |
| 进化 | `#ec4899` | ✧ | "进化: 自我进化" |

### Passive
| Trigger | Response | XP |
|---------|----------|-----|
| 30s idle | +1 XP, auto-learning | 1 |
| 15s idle | Mood→neutral | — |
| 30s idle | Mood→tired, dim opacity | — |
| Click node | +2 XP, mood→happy | 2 |

## Mood Expressions (VAD-driven)

| Mood | Eye Width | Eye Height | Mouth | Brow | Blur | V | A | D |
|------|-----------|------------|-------|------|------|---|---|---|
| happy | 0.65 | 0.55 | smile | up | 5 | 0.9 | 0.8 | 0.7 |
| neutral | 0.55 | 0.50 | line | flat | 8 | 0.5 | 0.5 | 0.5 |
| tired | 0.35 | 0.30 | line | down | 15 | 0.2 | 0.2 | 0.3 |
| excited | 0.75 | 0.60 | open | up | 3 | 0.8 | 1.0 | 0.9 |

## Animation Loop

`requestAnimationFrame` at ~60fps:
1. `blinkTick()` — random blink every ~200 frames (6-frame closure)
2. `updateMood()` — check idle time → dim/undim
3. `updateNodePositions()` — orbit nodes CCW around character center
4. `renderPet()` — clear canvas → draw body → face → eyes → mouth → blush → crown → badge → XP bar

## CSS Design System Alignment

- **CCW constraint**: Node orbit uses `+speed` (counter-clockwise consistent with all NeoTrix rotation animations)
- **Float animation**: `petFloat` 4s ease-in-out infinite (−6px Y oscillation)
- **Dim state**: `brightness(0.6) saturate(0.4)` over 0.8s transition
- **Node glow**: `box-shadow` scales with evolution level (6px + evo*2px)

## Integration Points

```
preview-ui-v2.html
├── CSS:  #neoPet, .pet-node, .pet-tooltip, @keyframes petBreathe/Float
├── HTML: <div id="neoPet"> → <canvas id="petCanvas"> + 6× .pet-node
└── JS:   pet object + renderPet() + drag/click/touch handlers + petLoop()
```

## Future Extensions

- **E8 State Sync**: Pet mood mirrors E8 hexagram state via keyboard/fetch-driven updates
- **GWT Resonance Waves**: Orbiting nodes resonate (pulse glow) when GWT broadcasts occur
- **SEAL Aging**: Pet visual shows wear if neglected > 24h, needs interaction to heal
- **Procedural Memory**: Pet learns frequent user interaction patterns → predicts next action
- **Multi-Pet Ecosystem**: Multiple pets for different workspace contexts (dev/chat/research)
- **Pet-to-Pet Communication**: A2A protocol between pet instances across sessions

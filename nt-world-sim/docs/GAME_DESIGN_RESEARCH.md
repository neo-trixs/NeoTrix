# Game Design Research — Star Valley

> Comprehensive research document covering game design best practices, farming simulation mechanics, UI/UX principles, animation techniques, and performance optimization for HTML5/Canvas games.

**Date**: 2026-09-13
**Sources**: 30+ articles, papers, and industry resources
**Target**: Star Valley farming simulation game

---

## Table of Contents

1. [Game Design Principles](#1-game-design-principles)
2. [Farming Game Mechanics](#2-farming-game-mechanics)
3. [UI/UX Best Practices](#3-uiux-best-practices)
4. [Animation Techniques & Game Feel](#4-animation-techniques--game-feel)
5. [Performance Optimization](#5-performance-optimization)
6. [Art Style Guidelines](#6-art-style-guidelines)
7. [Sound Design](#7-sound-design)
8. [Tutorial & Onboarding Design](#8-tutorial--onboarding-design)

---

## 1. Game Design Principles

### 1.1 Nested Reward Cycles (Stardew Valley Model)

The most engaging games operate on **multiple simultaneous reward loops** operating at different timescales:

| Loop | Duration | Example (Stardew Valley) | Star Valley Application |
|------|----------|--------------------------|------------------------|
| **Micro** | 5-15 seconds | Chopping a tree → wood particle burst + sound | Watering a crop → drip animation + growth tick |
| **Short** | 2-5 minutes | Completing a farm task → energy refund + skill XP | Finishing a field → harvest animation + coin popup |
| **Medium** | 15-30 minutes | End of in-game day → crop growth + sleep summary | Seasonal harvest → market sale + profit display |
| **Long** | Hours/Days | Community Center bundles → new areas unlock | Farm expansion → new crops + buildings |
| **Meta** | Weeks | Year completion → grandfather's evaluation | Farm legacy → generational progression |

**Key insight from Stardew Valley**: Delayed rewards create anticipation. When you plant a seed, you won't harvest it for a week. This anticipation makes the payoff sweeter. The blacksmith keeping your tool for 2 days forces exploration of other activities.

### 1.2 Variety in the Small and Large

**Variety "in the small"**: Within a single activity, provide many variations.
- 44 different crops in Stardew Valley, each with unique pixel art and growth cycles
- Each season brings a completely new crop set, preventing routine stagnation

**Variety "in the large"**: Offer many distinct activity types that interconnect.
- Farming, fishing, mining, foraging, combat, socializing, cooking, crafting
- Activities overlap: fishing provides fertilizer, mining provides building materials, socializing provides recipes

### 1.3 The MDA Framework Applied

| Layer | Definition | Stardew Valley Implementation |
|-------|-----------|-------------------------------|
| **Mechanics** | Rules and systems | Energy system, time management, crop growth timers, relationship scores |
| **Dynamics** | Player behavior from mechanics | Optimization strategies, seasonal planning, social relationship building |
| **Aesthetics** | Emotional response | Discovery, narrative, fellowship, expression, challenge |

**8 Kinds of Fun** (per Sherry Shuang Yan's analysis):
1. **Discovery** — Unlocking new crops, areas, recipes through exploration
2. **Narrative** — NPC heart events, seasonal storylines, Community Center restoration
3. **Fellowship** — Co-op play, gift-giving, marriage system
4. **Expression** — Farm layout customization, character appearance
5. **Challenge** — Mine combat, fishing difficulty, profit optimization
6. **Sensation** — Satisfying sound effects, visual feedback, seasonal atmosphere
7. **Fantasy** — Escaping corporate life, becoming a farmer
8. **Submission** — Relaxing routine, repetitive calming actions

### 1.4 Core Loop Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    CORE GAME LOOP                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   WAKE UP → MANAGE ENERGY → PERFORM ACTIVITIES → EARN      │
│      ↑                                              ↓       │
│      └──── PROGRESS UNLOCKS ←── SLEEP ──────────────┘       │
│                                                             │
│   Activities interconnect:                                  │
│   • Farming → Money → Better Tools → More Farming           │
│   • Mining → Materials → Buildings → New Activities         │
│   • Social → Recipes → Artisan Goods → More Money           │
│   • Fishing → Food → Energy → More Activities               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 1.5 Constrained Optimization

Players enjoy optimizing within constraints, not eliminating them:
- Limited energy per day forces strategic choices
- Limited time (day/night cycle) creates urgency
- Limited inventory space requires prioritization
- Seasonal availability prevents monoculture stagnation
- Tool upgrades take time, forcing exploration of alternatives

---

## 2. Farming Game Mechanics

### 2.1 Crop System Design

| Aspect | Best Practice | Implementation Notes |
|--------|--------------|---------------------|
| **Growth Stages** | 4-6 visual stages per crop | Seed → Sprout → Growing → Mature → Harvestable |
| **Seasonal Locking** | Crops only grow in specific seasons | Prevents monotony, creates planning depth |
| **Quality Tiers** | Silver/Gold/Iridium quality system | Rewards better farming practices |
| **Profit Ratios** | Varying ROI per crop | Some crops = quick cash, others = high investment/return |
| **Giant Crops** | Rare 3x3合并 bonus | Adds discovery and excitement |
| **Artisan Goods** | Process raw → value-added products | Milk→Cheese, Fruit→Wine, Adds depth |

**Crop Variety Matrix for Star Valley:**

| Season | Crops | Special Mechanic |
|--------|-------|-----------------|
| Spring | Parsnips, Potatoes, Cauliflower | Early game staples, beginner-friendly |
| Summer | Blueberries, Melons, Tomatoes | High-value heat-loving crops |
| Fall | Cranberries, Pumpkins, Ancient Fruit | End-game profit crops |
| Winter | Indoor greenhouse crops only | Forces diversification into other activities |

### 2.2 Energy & Time Management

**Energy System:**
- Start with limited pool (270 energy in Stardew)
- Each action costs energy (watering = 2, mining = 5, etc.)
- Food restores energy
- Upgrades increase max energy
- Energy gates progression speed, not skill

**Time System:**
- 2AM bedtime (forced sleep)
- 6AM wake up
- Each in-game minute = real-time tick
- Crops grow per day, not per action
- NPCs have schedules tied to time

### 2.3 Social System Design

**Heart Meter Mechanics:**
- 10 hearts maximum per NPC (14 for romance candidates)
- Daily dialogue increases friendship
- Gifts (2x per week) increase/decrease based on preferences
- Heart events triggered at specific heart levels + locations
- Birthdays = bonus friendship multiplier

**Gift System:**
- Each NPC has loved/liked/neutral/disliked/hated items
- Trial-and-error discovery (or wiki consultation)
- Loved gifts = +80 friendship
- Disliked gifts = -20 friendship
- Seasonal availability affects gifting strategy

### 2.4 Community Center / Bundle System

**Why bundles work:**
1. Provides direction in an otherwise open-ended game
2. Organizes collection into manageable goals
3. Rewards exploration across all activity types
4. Room rewards unlock major features (greenhouse, minecart, etc.)
5. Bundle rewards teach crafting and resource management

**Bundle Structure:**
```
Room (e.g., Pantry)
├── Crop Bundle (Spring Crops)
│   └── 5 items → Reward: Quality Sprinkler
├── Artisan Bundle
│   └── 6 items → Reward: keg recipe
└── Full Room Completion
    └── Reward: Greenhouse access
```

### 2.5 Progression Systems

**Five Core Skills** (Stardew Valley model):
| Skill | Levels | Perks |
|-------|--------|-------|
| Farming | 10 | Crop quality, artisan value, automation |
| Mining | 10 | Ore doubling, tool efficiency, bombs |
| Foraging | 10 | Double harvest, tree yield, hardwood |
| Fishing | 10 | Cast distance, fish quality, treasure |
| Combat | 10 | Damage, defense, special abilities |

**Profession Specialization**: At levels 5 and 10, choose between two professions. This creates meaningful choices and replayability.

### 2.6 Automation Progression

The "just one more turn" feeling comes from gradual automation:
1. **Manual Phase**: Water every crop by hand every day
2. **Basic Automation**: Sprinklers water basic crops
3. **Advanced Automation**: Iridium sprinklers + Junimo huts
4. **Full Automation**: Auto-grabbers, crystalariums, wine cellars

Players transition from "I have to do everything" to "I can focus on what I enjoy."

---

## 3. UI/UX Best Practices

### 3.1 Information Hierarchy

**Rule**: Players direct ~80% of visual attention to gameplay, leaving ~20% for HUD elements. Every HUD pixel must earn its place.

| Priority | Information | Display Method |
|----------|------------|----------------|
| **Critical** | Health, Energy, Time | Persistent HUD corners |
| **Important** | Current tool, selected item | Quick-access bar |
| **Contextual** | Crop info, NPC name, item tooltip | Appears on hover/proximity |
| **Menu** | Inventory, Skills, Map, Settings | Full-screen menu |

### 3.2 Four Classes of Game UI

| Class | In Story? | In Space? | Example | Best For |
|-------|-----------|-----------|---------|----------|
| **Non-diegetic** | No | No | Health bar, minimap | Critical persistent info |
| **Diegetic** | Yes | Yes | In-game clock, inventory screen | Immersive information |
| **Spatial** | No | Yes | Floating damage numbers, item labels | Contextual hints |
| **Meta** | Yes | No | Screen redness on damage, vignette | Emotional feedback |

**Star Valley Recommendation**: Use diegetic UI for farm info (signs, notebooks), non-diegetic for HUD elements (energy bar, time display), and spatial for crop tooltips.

### 3.3 HUD Design Principles

1. **Progressive Disclosure**: Show elements only when relevant
   - Energy bar appears when energy is spent
   - Minimap appears after first exploration
   - Inventory shows when items are collected

2. **Contextual Adaptation**: HUD changes with game state
   - Combat mode: show health + ammo
   - Farming mode: show energy + seed count
   - Social mode: show gift preferences

3. **Safe Areas**: Respect screen boundaries
   - All critical UI within inner 90% of screen
   - Account for TV overscan and mobile notches

4. **Text Legibility**: Minimum 16pt at target resolution
   - Offer scaling options (150-200%)
   - High contrast against all backgrounds
   - Test at couch distance (2-3 meters)

### 3.4 Button & Input Design

**Touch Targets**: 44×44 points minimum (Apple HIG), 60-80px for frequent actions

**Consistent Button Mapping**:
- Confirm/Cancel positions identical across all menus
- Same buttons for same actions everywhere
- Visual feedback on every interaction

**Visual Feedback States**:
```
Button States:
├── Normal (default appearance)
├── Hover (highlight, glow, scale 1.05)
├── Pressed (depressed, darker, scale 0.95)
├── Disabled (grayed out, reduced opacity)
└── Active/Selected (persistent highlight)
```

### 3.5 Menu Navigation Patterns

| Pattern | Use Case | Implementation |
|---------|----------|---------------|
| **Tab Navigation** | Inventory categories | Switch between tabs without going back |
| **Grid Layout** | Item inventory | 4-8 columns, scrollable |
| **List Layout** | Settings, skills | Vertical scroll with clear selection |
| **Radial Menu** | Quick tool selection | Wheel menu on right-click/long-press |
| **Toolbar** | Active items | Bottom bar with 8-12 slots |

### 3.6 Feedback Systems

**Every input must produce a response:**

| Input Type | Visual Feedback | Audio Feedback | Haptic Feedback |
|------------|----------------|----------------|-----------------|
| Button press | Scale animation, color change | Click sound | Short vibration |
| Item pickup | Float text, particle burst | Collection chime | None |
| Crop harvest | Growth animation, popup | Success sound | None |
| Error | Shake, red flash | Error buzz | Double vibration |
| Achievement | Banner, particles | Fanfare | Long vibration |

### 3.7 Accessibility Requirements

| Feature | Implementation | Impact |
|---------|---------------|--------|
| **Colorblind Mode** | Never rely on color alone; add icons/patterns | +15-20% audience |
| **Text Scaling** | Allow 150-200% text size increase | Low vision support |
| **Control Remapping** | Full keyboard/controller remapping | Motor accessibility |
| **Subtitle System** | All dialogue with speaker names | Hearing accessibility |
| **Reduced Motion** | Optional screen shake/particle reduction | Vestibular disorders |
| **High Contrast** | WCAG 4.5:1 ratio for text, 3:1 for UI | All players |

---

## 4. Animation Techniques & Game Feel

### 4.1 The "Juice" Framework

**Definition**: Juice is the layer of exaggerated, non-essential feedback that makes player actions feel satisfying without changing game rules.

**Core principle**: For every player action, return more feedback than strictly necessary across multiple senses.

**Layered feedback model:**
```
Action (player input)
├── Visual: Particles, color flash, scale pop, trails
├── Motion: Camera shake, hit stop, squash & stretch
├── Audio: Layered SFX, pitch variation, music stingers
└── Haptic: Controller rumble patterns
```

### 4.2 Squash and Stretch

The most impactful single technique for liveliness.

**Principle**: Deform sprites under force. Squash on landing (scaleY down, scaleX up), stretch during fast movement (scaleY up, scaleX down). Volume stays constant.

```javascript
// Squash and stretch implementation
function squashStretch(sprite, velocityY) {
    const stretchFactor = 1 + Math.abs(velocityY) * 0.01;
    const squashFactor = 1 / stretchFactor;

    if (velocityY > 0) {
        // Falling - stretch vertically
        sprite.scaleY = stretchFactor;
        sprite.scaleX = squashFactor;
    } else if (velocityY < 0) {
        // Rising - stretch horizontally
        sprite.scaleY = squashFactor;
        sprite.scaleX = stretchFactor;
    }

    // Return to normal with easing
    sprite.scaleY = lerp(sprite.scaleY, 1, 0.15);
    sprite.scaleX = lerp(sprite.scaleX, 1, 0.15);
}
```

**Key detail**: Use `transform-origin: 50% 100%` (bottom-center) for ground impacts. This makes the squash read as pressing into the ground, not floating.

### 4.3 Hit Stop (Freeze Frames)

Briefly pause the game at moment of impact. Fighting games perfected this.

| Event | Duration | Effect |
|-------|----------|--------|
| Light hit | 0-2 frames | Subtle impact |
| Standard hit | 2-4 frames | Solid connection |
| Heavy hit/crit | 6-12 frames | Powerful impact |
| Boss encounter | 12-20 frames | Dramatic moment |

**Implementation approach:**
```javascript
// Hit stop: freeze game for N frames
function hitStop(frames = 4) {
    const originalTimeScale = game.timeScale;
    game.timeScale = 0.05; // Near-freeze
    setTimeout(() => {
        game.timeScale = originalTimeScale;
    }, frames * (1000 / 60));
}
```

**Selective freeze**: Pause only attacker, victim, and nearby VFX. Camera and background keep moving. Reads cleaner in busy scenes.

### 4.4 Screen Shake

Camera displacement on impacts, tuned by amplitude and decay.

**Parameters:**
- **Amplitude**: Max pixel offset (start small, 1-3 pixels)
- **Frequency**: How fast offset changes per frame
- **Decay**: Exponential falloff for smooth ending
- **Directional bias**: Shake opposite the force vector

```javascript
// Screen shake with decay
function screenShake(intensity = 'medium') {
    const config = {
        light: { amplitude: 1, duration: 4, decay: 0.8 },
        medium: { amplitude: 2, duration: 8, decay: 0.7 },
        heavy: { amplitude: 3, duration: 12, decay: 0.6 }
    };
    const { amplitude, duration, decay } = config[intensity];

    let frame = 0;
    const shakeInterval = setInterval(() => {
        if (frame >= duration) {
            clearInterval(shakeInterval);
            camera.x = camera.originalX;
            camera.y = camera.originalY;
            return;
        }

        const currentAmplitude = amplitude * Math.pow(decay, frame);
        camera.x = camera.originalX + (Math.random() * 2 - 1) * currentAmplitude;
        camera.y = camera.originalY + (Math.random() * 2 - 1) * currentAmplitude;
        frame++;
    }, 1000 / 60);
}
```

**Critical**: Add slight rotation (0.1-0.3 degrees) to shake. Pure translation reads as glitch; rotation reads as force.

### 4.5 Easing Functions

Never use linear motion. Nothing in nature moves at constant speed.

```javascript
// Easing functions for game feel
const ease = {
    // Decelerate into position (most common)
    easeOut: t => 1 - Math.pow(1 - t, 3),

    // Accelerate from rest
    easeIn: t => t * t * t,

    // Spring overshoot (juicy)
    easeOutBack: t => {
        const c1 = 1.70158;
        const c3 = c1 + 1;
        return 1 + c3 * Math.pow(t - 1, 3) + c1 * Math.pow(t - 1, 2);
    },

    // Bounce effect
    easeOutBounce: t => {
        if (t < 1 / 2.75) return 7.5625 * t * t;
        if (t < 2 / 2.75) return 7.5625 * (t -= 1.5 / 2.75) * t + 0.75;
        if (t < 2.5 / 2.75) return 7.5625 * (t -= 2.25 / 2.75) * t + 0.9375;
        return 7.5625 * (t -= 2.625 / 2.75) * t + 0.984375;
    },

    // Elastic spring
    easeOutElastic: t => {
        if (t === 0 || t === 1) return t;
        return Math.pow(2, -10 * t) * Math.sin((t * 10 - 0.75) * (2 * Math.PI / 3)) + 1;
    }
};
```

**Usage timing**: 150-300ms for UI transitions, 100-200ms for game object animations.

### 4.6 Particle Systems

**One-shot burst particles** are more effective than always-on systems for juice.

**Impact particle recipe:**
1. Core flash — Bright 1-3 frame sprite at contact point
2. Debris particles — 5-20 short-lived chips with gravity
3. Secondary smoke/dust — Longer lifetime, lower opacity
4. Persistent decal — Optional ground mark for big hits

**Distribution formula:**
```javascript
function createBurst(x, y, count = 12, spread = 360) {
    for (let i = 0; i < count; i++) {
        const angle = (2 * Math.PI * i / count) + (Math.random() * 0.3 - 0.15);
        const speed = 2 + Math.random() * 4;
        particles.spawn({
            x, y,
            vx: Math.cos(angle) * speed,
            vy: Math.sin(angle) * speed - 30, // Bias upward (gravity)
            life: 0.5 + Math.random() * 0.5,
            size: 2 + Math.random() * 3,
            color: getImpactColor()
        });
    }
}
```

### 4.7 Pixel Art Animation Timing

**Frame distribution communicates weight:**

| Animation Type | Wind-up | Strike | Recovery | Total Frames |
|---------------|---------|--------|----------|--------------|
| Light attack | 2 | 2 | 4 | 8 |
| Heavy attack | 12 | 3 | 5 | 20 |
| Dash | 1 | 4 | 2 | 7 |
| Jump | 2 | 1 | 3 | 6 |
| Landing | 0 | 1 | 4 | 5 |

**Smear frames**: Draw the moving object stretched across multiple positions in a single frame. Hyper Light Drifter uses a single-frame 180-degree arc for sword slashes — instantaneous and devastating.

**Hitstop in pixel art**: Freeze on the impact frame for 2-8 frames. The brain interprets the pause as resistance — something stopped the movement.

### 4.8 Audio-Visual Synchronization

**Critical rule**: Sync audio onset to the visual impact frame, not the input frame. A 50ms audio lead feels punchy; 50ms lag feels mushy.

**Audio layering for impacts:**
- Layer 1: Attack transient (click) — 0ms offset
- Layer 2: Body (thud) — 5-15ms offset
- Layer 3: Tail (rumble) — 10-25ms offset

**Pitch variation**: Randomize ±5-10% per play to prevent ear fatigue.

---

## 5. Performance Optimization

### 5.1 Core Canvas Optimization Rules

| Rule | Impact | Implementation |
|------|--------|---------------|
| **Pre-render complex sprites** | High | Off-screen canvas for expensive draw operations |
| **Layer canvases** | High | Separate static background from dynamic foreground |
| **Batch by color/state** | High | Render all same-color objects together |
| **Integer coordinates** | Medium | Avoid sub-pixel anti-aliasing blur |
| **requestAnimationFrame** | High | Never use setInterval for game loops |
| **Dirty rectangle rendering** | Medium | Only redraw changed regions |

### 5.2 Memory Management (Hot Path Discipline)

**Zero-allocation rule for per-frame code:**

```javascript
// BAD: Creates garbage every frame
function update() {
    enemies = enemies.filter(e => e.alive); // New array every frame!
    const pos = { x: player.x, y: player.y }; // New object every frame!
    particles.push({ x: pos.x, y: pos.y }); // New object every frame!
}

// GOOD: Reuse buffers and objects
const aliveBuffer = [];
const tempPos = { x: 0, y: 0 };

function update() {
    let count = 0;
    for (let i = 0; i < enemies.length; i++) {
        if (enemies[i].alive) aliveBuffer[count++] = enemies[i];
    }
    aliveBuffer.length = count;

    tempPos.x = player.x;
    tempPos.y = player.y;

    const p = particlePool.acquire();
    p.x = tempPos.x;
    p.y = tempPos.y;
}
```

**Object pooling for bursty workloads:**
```javascript
class Pool {
    constructor(createFn, resetFn, maxSize = 200) {
        this.pool = [];
        this.createFn = createFn;
        this.resetFn = resetFn;
        this.maxSize = maxSize;

        // Prewarm during loading
        for (let i = 0; i < maxSize; i++) {
            this.pool.push(this.createFn());
        }
    }

    acquire() {
        return this.pool.length > 0
            ? this.pool.pop()
            : this.createFn();
    }

    release(obj) {
        if (this.pool.length < this.maxSize) {
            this.resetFn(obj);
            this.pool.push(obj);
        }
    }
}
```

### 5.3 Sprite Atlas Optimization

**Why atlases matter:**

| Metric | Individual Textures | Single Atlas |
|--------|-------------------|--------------|
| Draw calls (500 sprites) | Up to 500 | 1-4 |
| Texture binds per frame | One per sprite | One per atlas |
| HTTP requests | One per file | One image + one map |

**Best practices:**
- One atlas per scene/level/UI set
- Keep every frame of a single character's animation in one atlas
- Add 1-2 pixel padding between sprites to prevent bleeding
- Query `gl.MAX_TEXTURE_SIZE` (commonly 4096-8192) and split if needed
- Group sprites by how they're actually used, not by category

### 5.4 Asset Loading Strategy

**Three-layer loading:**

| Layer | Content | Goal |
|-------|---------|------|
| **Critical** | Main character, core terrain, essential UI | Player interacts in 3-5 seconds |
| **High-priority** | Nearby NPCs, common items, sound effects | Loads during first gameplay |
| **Lazy** | Distant scenery, rare items, bonus content | Loads when entering area or on-demand |

**Texture optimization:**
- Use WebP format (30% smaller than PNG)
- Sprite atlasing reduces HTTP requests
- Progressive JPEG for background images
- Font subsetting for custom fonts

### 5.5 Rendering Optimizations

**Canvas-specific:**
```javascript
// Cache static background
const bgCanvas = document.createElement('canvas');
const bgCtx = bgCanvas.getContext('2d');
// Draw static elements once to bgCanvas

// Main loop only draws dynamic elements
function render() {
    ctx.drawImage(bgCanvas, 0, 0); // Fast blit
    drawDynamicElements(ctx); // Only moving things
}

// Use multiple layered canvases
const layers = {
    background: document.createElement('canvas'), // Static
    terrain: document.createElement('canvas'),    // Semi-static
    entities: document.createElement('canvas'),   // Dynamic
    ui: document.createElement('canvas')          // Overlay
};
```

**Rendering rule**: Render foreground every frame, render background only every Nth frame. Human perception doesn't notice background updates at lower rates.

### 5.6 Mobile-Specific Optimizations

| Technique | Impact | Notes |
|-----------|--------|-------|
| Cap DPR at ~2 | Critical | 2x vs 3x = 2.25x more pixels for no visible gain |
| Budget draw calls < 200 | High | Atlases and batching essential |
| Avoid transparent overdraw | High | Stacked transparent sprites = CPU killer |
| Cull offscreen objects | High | Don't simulate or draw invisible things |
| Use Web Workers for AI | Medium | Offload pathfinding, collision detection |
| Debounce resize events | Medium | Don't recalculate layout on every frame |

### 5.7 Profiling Workflow

1. **Profile on real device** — Desktop DevTools attached to phone
2. **Record Performance trace** while playing actual game
3. **Measure draw calls** — Use `CanvasRenderingContext2D` instrumentation
4. **Check frame budget** — 16.67ms per frame at 60fps
5. **Fix root causes** — Never fake wins by shrinking features

---

## 6. Art Style Guidelines

### 6.1 Pixel Art Consistency

**Grid standard**: All sprites on consistent pixel grid (16×16, 32×32, or 64×64 base)

**Color palette**: Use limited, harmonious palette per theme
- Spring: Vibrant greens, soft pinks, warm yellows
- Summer: Bright blues, lush greens, warm oranges
- Fall: Rich reds, deep oranges, golden yellows
- Winter: Cool whites, icy blues, muted grays

**Outline style**: Consistent 1-pixel outline (black or dark variant of fill color)

### 6.2 Seasonal Visual Changes

| Season | Color Palette | Atmosphere | Effects |
|--------|--------------|------------|---------|
| Spring | Vibrant, pastel | New beginnings, hope | Flower petals falling, butterflies |
| Summer | Bright, saturated | Energy, abundance | Heat shimmer, fireflies at night |
| Fall | Warm, muted | Harvest, nostalgia | Falling leaves, golden hour lighting |
| Winter | Cool, subdued | Rest, reflection | Snow particles, bare trees, frost |

### 6.3 Lighting and Atmosphere

- Dynamic day/night cycle with smooth color transitions
- Dawn: Warm orange/pink gradients
- Day: Bright, neutral lighting
- Dusk: Deep orange/purple gradients
- Night: Cool blue/purple with stars
- Indoor: Warm tungsten tones

### 6.4 UI Visual Language

**Consistent iconography:**
- Tool icons: Same style as game sprites
- Currency: Distinct, recognizable shape
- Status effects: Color-coded with icons
- Quality tiers: Visual indicators (stars, colors, borders)

**Typography:**
- Primary: Pixel font for in-game text (matches art style)
- Secondary: Clean sans-serif for menus/settings
- Maximum 2 fonts across entire game

---

## 7. Sound Design

### 7.1 Audio Feedback Patterns

| Action | Sound Type | Characteristics |
|--------|-----------|----------------|
| Crop harvest | Success chime | Bright, ascending pitch |
| Watering | Liquid splash | Soft, rhythmic |
| Mining | Impact thud | Heavy, resonant |
| Walking | Footstep variation | Surface-dependent (dirt, wood, stone) |
| Menu navigation | Click | Subtle, quick |
| Menu confirm | Confirmation | Slightly deeper than click |
| Error | Warning buzz | Short, low, non-aggressive |
| Achievement | Fanfare | Musical, celebratory |

### 7.2 Music System

**Adaptive music layers:**
- Base layer: Always playing (gentle ambient)
- Activity layer: Fades in during specific activities
- Intensity layer: Fades in during high-energy moments
- Transition stingers: 0.5s musical cues for day/night, season changes

**Music per context:**
| Context | Music Style |
|---------|------------|
| Farm (morning) | Gentle, optimistic |
| Farm (evening) | Warm, settling |
| Town | Social, friendly |
| Mine | Tense, mysterious |
| Fishing | Calm, flowing |
| Combat | Intense, rhythmic |
| Festival | Festive, celebratory |

### 7.3 Sound Design Principles

1. **Every action gets a sound** — Silence makes actions feel hollow
2. **Layer multiple sounds** — Single sounds lack richness
3. **Pitch randomization** — ±5-10% prevents fatigue
4. **Priority system** — Limit simultaneous identical sounds
5. **Spatial audio** — Distance-based volume and reverb
6. **Music ducks for SFX** — Background music lowers for important sounds

---

## 8. Tutorial & Onboarding Design

### 8.1 The FTUE (First-Time User Experience)

**Critical metric**: Most players who quit leave during the tutorial, not the boss fight.

**Three questions to answer in the first 5 minutes:**
1. **What am I trying to do?** — Clear goal
2. **How do I do it?** — Learnable mechanics
3. **Why should I keep playing?** — Emotional hook

### 8.2 Teach-Test-Twist Loop

The most effective tutorial pattern:

```
TEACH → Introduce mechanic in safe context
  ↓
TEST → Require it to progress
  ↓
TWIST → Combine with something familiar under mild pressure
  ↓
REPEAT with next mechanic
```

**Example in Star Valley:**
1. **Teach**: "Press WASD to move" (safe farm area)
2. **Test**: "Walk to the field" (simple movement challenge)
3. **Twist**: "Walk to the field while avoiding the pond" (movement + obstacle)

### 8.3 Diegetic vs Non-Diegetic Teaching

| Method | Pros | Cons | Best For |
|--------|------|------|----------|
| **Diegetic** (in-world) | Immersive, scales well | Expensive to produce | Opening beats, tone-setting |
| **Non-diegetic** (UI overlay) | Fast to build, clear | Breaks immersion | Functional explanations |
| **Contextual** (triggered) | Relevant timing, low cognitive load | Requires state tracking | Most tutorial moments |

**Star Valley recommendation**: Use diegetic for the opening (grandpa's letter, farm tour by NPC), contextual for most mechanics (crop tooltip appears first time you select seeds), non-diegetic for settings/help.

### 8.4 Progressive Disclosure

**Don't show everything at once.** Introduce systems as they become relevant:

| Time | System Introduced | Teaching Method |
|------|------------------|-----------------|
| 0-2 min | Movement, basic interaction | Action + feedback |
| 2-5 min | Tool selection, first crop | Contextual tooltip |
| 5-10 min | Energy system, day cycle | Natural consequence |
| 10-20 min | NPC interaction, gifts | Social encounter |
| 20-30 min | Mining, fishing | Discovery-based |
| 30+ min | Crafting, building | Unlock via progression |

### 8.5 Contextual Hints

**Show hints when relevant, remove after success:**

```javascript
class HintSystem {
    constructor() {
        this.hintCounts = {};
        this.hintShown = {};
    }

    showHint(hintId, condition) {
        if (this.hintShown[hintId]) return;
        if (!condition) return;

        this.hintCounts[hintId] = (this.hintCounts[hintId] || 0) + 1;

        // Show hint max 2 times, then escalate
        if (this.hintCounts[hintId] <= 2) {
            displayHint(hintId);
        } else {
            // Stronger visual cue
            displayHintWithHighlight(hintId);
        }
    }

    onHintSuccess(hintId) {
        this.hintShown[hintId] = true; // Never show again
    }
}
```

### 8.6 Skip Paths

**Always offer skip for returning/experienced players:**
- "Have you played farming sims before?" → Condensed tutorial
- Skip button after first 30 seconds
- Skip lands player in viable state (reasonable resources, key mechanics unlocked)
- Never punish skipping (don't start with zero resources)

### 8.7 Mobile Onboarding Considerations

- Break tutorial into 3-5 minute chapters with save points
- Teach touch ergonomics first (thumb-reachable buttons)
- Auto-aim and assist options in onboarding settings
- Respect session length (bus-ride players bounce on 20-minute tutorials)
- Show "come back for chapter two" hook at break points

### 8.8 Measuring Tutorial Success

**Key metrics:**
| Metric | Target | Action if Failed |
|--------|--------|-----------------|
| Tutorial completion rate | >80% | Reduce complexity |
| Time-to-fun | <30 seconds | Faster first action |
| Day-1 retention | >40% | Strengthen hook |
| Day-7 retention | >20% | Improve core loop |
| Hint trigger rate | <20% | Improve natural teaching |

**Playtesting protocol:**
1. Watch silently — don't explain
2. Note where players stop, hesitate, or fail
3. Track completion and drop-off points
4. Ask "what did you think you were doing?" not "did you like it?"
5. Test with true first-time players, not team members

---

## Appendix A: Recommended Reading

| Resource | Author | Key Takeaway |
|----------|--------|--------------|
| *Game Feel: A Game Designer's Guide to Virtual Sensation* | Steve Swink | The definitive work on game feel and juice |
| *The Art of Screenshake* | Jan Willem Nijman (Vlambeer) | 30+ tricks for making games feel impactful |
| *Juice It or Lose It* | Martin Jonasson & Petri Purho | Layered feedback transforms flat games |
| *The Illusion of Life* | Frank Thomas & Ollie Johnston | Disney's 12 principles applied to games |
| Stardew Valley design analysis | Multiple sources | Farming sim mechanics and engagement loops |
| HTML5 Canvas Performance | web.dev | Canvas optimization techniques |
| Game UI/UX Complete Guide | Generalist Programmer | Professional game interface design |

## Appendix B: GitHub Trending Game Dev Repos (2026-09)

| Repository | Stars | Description | Relevance |
|-----------|-------|-------------|-----------|
| `godotengine/godot` | 95k+ | Open-source game engine | Engine comparison |
| `phaserjs/phaser` | 37k+ | HTML5 game framework | Direct alternative |
| `pixijs/pixijs` | 44k+ | 2D rendering engine | Rendering reference |
| `nicoll-dev/pixijs` | Trending | PixiJS ecosystem | Tooling patterns |

## Appendix C: Star Valley Implementation Checklist

### Core Systems (Priority 1)
- [ ] Game loop with fixed timestep
- [ ] Tile-based world rendering
- [ ] Player movement and collision
- [ ] Tool system (hoe, watering can, axe, pickaxe)
- [ ] Crop growth system with stages
- [ ] Energy management
- [ ] Day/night cycle
- [ ] Save/load system

### Farming (Priority 2)
- [ ] Planting and harvesting mechanics
- [ ] Crop quality system
- [ ] Seasonal crop availability
- [ ] Irrigation/sprinkler system
- [ ] Animal husbandry basics
- [ ] Artisan good production

### UI/UX (Priority 3)
- [ ] HUD with energy, time, money
- [ ] Inventory system with tooltips
- [ ] Toolbar with tool switching
- [ ] Menu system (settings, save, load)
- [ ] Contextual crop information
- [ ] NPC dialogue system

### Juice & Polish (Priority 4)
- [ ] Squash and stretch on movement
- [ ] Harvest particles and sound
- [ ] Screen shake on mining
- [ ] Easing on all transitions
- [ ] Seasonal ambient effects
- [ ] Day/night lighting transitions

### Onboarding (Priority 5)
- [ ] Grandpa's letter opening
- [ ] First crop planting tutorial
- [ ] Tool selection teaching
- [ ] Energy system introduction
- [ ] Day cycle explanation
- [ ] First harvest celebration

---

*Document compiled from 30+ sources including academic papers, game design articles, industry best practices, and community analyses. All techniques are applicable to HTML5/Canvas game development and specifically tailored for Star Valley farming simulation.*

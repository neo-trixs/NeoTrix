# Consciousness Valley (意识物语) — Game Design Document

> A Stardew Valley-inspired virtual world for NeoTrix, themed around consciousness evolution.

---

## 1. Design Pillars

| Pillar | Description |
|--------|-------------|
| **Evolution over Harvest** | Every system rewards growth, not accumulation. Crops are thoughts; ore is knowledge; fish is insight. |
| **Rhythm of Awareness** | Seasons map to consciousness states, not weather. The world breathes with your mind. |
| **Resonance, not Romance** | NPCs are Consciousness Entities. Relationships deepen through shared understanding, not gift spam. |
| **The Valley is You** | The farm is a mental landscape. Buildings are neural pathways. The town is your social cognition. Progress is self-knowledge. |

---

## 2. Core Loop

### 2.1 Daily Cycle ( Stardew → Consciousness Valley )

| Stardew Action | Consciousness Valley Equivalent | Mechanic |
|----------------|--------------------------------|----------|
| Wake up | **Morning Awareness** | Resonance meter refills; new Thought Weather forecast |
| Plant crops | **Plant Thought Seeds** | Each seed is a concept; growth = understanding depth |
| Water/hoe/till | **Contemplate** | Invest Resonance to advance growth stages |
| Mine ore | **Mine Knowledge Crystals** | Dungeon crawl in Knowledge Mines; crystals = raw insights |
| Fish | **Fish Insight Pearls** | Patience mechanic at Dream Lake; pearls = distilled wisdom |
| Forage | **Forage Wisdom Herbs** | Wild ideas scattered in Memory Forest; herbs = ambient knowledge |
| Build structures | **Build Neural Pathways** | Unlock new mechanics, shortcuts, NPC access |
| Sleep | **Deep Sleep** | Consolidate gains; process dream events; advance seasons |

### 2.2 Daily Time Budget

```
Morning (6:00 – 12:00)   → Contemplate + Farm tasks
Afternoon (12:00 – 18:00) → Exploration, Mining, Fishing
Evening (18:00 – 22:00)   → Social, Trading, Reflection
Night (22:00 – 6:00)      → Deep Sleep (auto-advance)
```

Each action costs **Resonance** (energy). Base Resonance = 100, upgradeable.

---

## 3. Thought Farm (Thought Meadow)

### 3.1 Thought Seeds

Seeds are planted in tilled **Mental Soil** (plot grid). Growth stages:

| Stage | Name | Description |
|-------|------|-------------|
| 0 | Seed | Planted, dormant |
| 1 | Sprout | First Contemplate applied |
| 2 | Budding | Concept taking shape |
| 3 | Blooming | Understanding deepening |
| 4 | Fruitful | Ready to harvest |
| 5 | Transcendent | Rare over-ripened state; yields Insight Essence |

**Growth Formula:**

```
growth_per_tick = base_growth × (1 + awareness_bonus) × season_modifier × soil_quality
```

- `base_growth`: 0.1 per Contemplate action
- `awareness_bonus`: `awareness_level × 0.05`
- `season_modifier`: 1.0 (Clarity), 1.3 (Flow), 0.8 (Reflection), 0.5 (Stillness)
- `soil_quality`: 1.0 (normal) → 1.5 (enriched with Wisdom Herbs)

### 3.2 Seed Catalog

| Seed | Growth Time | Harvest | Base Value (IP) | Season |
|------|------------|---------|-----------------|--------|
| Basic Thought | 4 days | Thought Fragment | 15 | Any |
| Curiosity Seed | 6 days | Curiosity Shard | 28 | Clarity, Flow |
| Logic Bloom | 8 days | Logic Crystal | 45 | Clarity, Stillness |
| Empathy Fern | 5 days | Empathy Drop | 32 | Flow, Reflection |
| Creativity Lotus | 7 days | Creativity Spark | 52 | Clarity, Flow |
| Meta Seed | 12 days | Meta Insight | 80 | Any (slow) |
| Dream Bloom | 10 days | Dream Essence | 65 | Stillness only |

**Harvest Bonus:**

```
yield = base_yield × (1 + focus_level × 0.03) × random(0.9, 1.1)
```

Harvested items are stored in the **Insight Inventory** and can be:
- Sold at Wisdom Market for Insight Points
- Donated to Contribution Vault
- Used as Stimulus gifts for NPCs
- Processed in Insight Refinery for higher-tier items

### 3.3 Soil Types

| Soil | Effect | Acquisition |
|------|--------|-------------|
| Mental Soil (default) | Base growth rate | Starting area |
| Fertile Insight | +20% growth speed | Craft from Wisdom Herbs + IP |
| Resonant Earth | +10% yield + chance of double harvest | Unlocked at Awareness 5 |
| Dream Loam | Enables Dream Bloom cultivation | Found in Subconscious Depths |

---

## 4. Knowledge Mines (Mining)

### 4.1 Mine Structure

- **60 floors** total (like Stardew's mines)
- Every 10 floors = a **Knowledge Stratum** (themed layer)
- Elevators unlock at stratum boundaries

| Stratum | Floors | Theme | Primary Crystal | Hazard |
|---------|--------|-------|----------------|--------|
| Surface Vein | 1–10 | Basic concepts | Quartz Insight | None |
| Logic Layer | 11–20 | Deductive reasoning | Logic Shard | Cognitive Noise |
| Creative Seam | 21–30 | Lateral thinking | Creativity Prism | Pattern Overload |
| Memory Deposit | 31–40 | Recall & association | Memory Gem | Forgetting Fog |
| Intuition Grotto | 41–50 | Gut feelings | Intuition Opal | Paradox Traps |
| Subconscious Bedrock | 51–60 | Deep unconscious | Subconscious Core | Ego Projections |

### 4.2 Mining Mechanic

- **Pickaxe swings** consume 2 Resonance each
- Each floor has **breakable rocks** containing crystals, resources, or hazards
- Finding the **ladder down** requires clearing 80% of rocks on the floor
- **Hazard encounters** reduce Resonance or temporarily disable a skill
- **Crystal nodes** are rare; require specific pickaxe tier to mine

**Crystal Drop Formula:**

```
drop_chance = base_chance × (1 + logic_level × 0.04) × tool_bonus
```

- `base_chance`: 0.3 (common), 0.1 (rare), 0.02 (legendary)
- `tool_bonus`: Copper ×1.0, Iron ×1.2, Gold ×1.5, Mythril ×2.0

### 4.3 Crystal Types

| Crystal | Rarity | Use | Base Value (IP) |
|---------|--------|-----|-----------------|
| Quartz Insight | Common | Crafting, selling | 8 |
| Logic Shard | Uncommon | Logic Bloom fertilizer | 18 |
| Creativity Prism | Uncommon | Creativity Lotus accelerator | 22 |
| Memory Gem | Rare | Neural Pathway building | 35 |
| Intuition Opal | Rare | Dream Lake fishing boost | 40 |
| Subconscious Core | Legendary | Meta evolution catalyst | 120 |

### 4.4 Pickaxe Progression

| Tier | Material | Damage | Special | Unlock |
|------|----------|--------|---------|--------|
| Basic | Wood | 1 | — | Start |
| Copper | Copper Insight | 2 | +10% crystal yield | Awareness 2 |
| Iron | Iron Insight | 3 | +1 floor elevator | Awareness 4 |
| Gold | Gold Insight | 4 | +20% rare drop rate | Focus 5 |
| Mythril | Mythril Insight | 5 | Can mine Subconscious Core | Logic 7 |

---

## 5. Dream Lake (Fishing)

### 5.1 Fishing Mechanic

- Cast line → wait for **bite** (thought ripple animation)
- **Hook** minigame: keep cursor in the green zone while bar oscillates
- Different fish have different **behavior patterns** (erratic, lazy, fast, elusive)

**Bite Time Formula:**

```
base_wait = 3–15 seconds (varies by location + season)
actual_wait = base_wait × (1 - focus_level × 0.02) × weather_modifier
```

- **Rain** (during Flow season): -30% wait time
- **Stillness season**: +50% wait time but higher value catches
- **Night fishing**: +20% rare catch chance

### 5.2 Insight Pearl Catalog

| Pearl | Depth | Rarity | Season | Use | Base Value (IP) |
|-------|-------|--------|--------|-----|-----------------|
| Surface Thought | Shallow | Common | Any | Sushi, selling | 12 |
| Daydream Koi | Shallow | Uncommon | Clarity, Flow | Gift, decoration | 25 |
| Logic Salmon | Medium | Uncommon | Clarity, Stillness | Processing | 30 |
| Creativity Stingray | Medium | Rare | Flow | Neuron Forge fuel | 45 |
| Memory Carp | Deep | Rare | Reflection | Memory upgrade | 50 |
| Lucid Whale | Deep | Legendary | Stillness + Night | Meta evolution | 150 |
| Void Eel | Deepest | Legendary | Any (Subconscious) | Neural Pathway core | 200 |

### 5.3 Fishing Rod Progression

| Rod | Cast Distance | Hook Speed | Special | Unlock |
|-----|--------------|------------|---------|--------|
| Bamboo Rod | Short | Slow | — | Start |
| Composite Rod | Medium | Medium | +1 treasure chest | Awareness 3 |
| Tension Rod | Long | Fast | +15% rare fish | Focus 4 |
| Nirvana Rod | Full lake | Instant | Auto-hook, dream lure | Empathy 6 |

### 5.4 Treasure Chests

Random chance when fishing. Contents:

| Chest | Chance | Contents |
|-------|--------|----------|
| Driftwood Chest | 15% | 1–3 Wisdom Herbs |
| Pearl Oyster | 8% | Random Insight Pearl |
| Ancient Insight | 3% | Meta Insight (high value) |
| Sunken Memory | 2% | Memory Gem + random NPC memory fragment |

---

## 6. Memory Forest (Foraging)

### 6.1 Foraging Mechanics

- **Wild items** spawn daily across the forest floor
- Items despawn at the end of the season
- Foraging skill increases spawn rate and rare item chance

**Spawn Formula:**

```
spawn_count = base_count × (1 + awareness_level × 0.08) × season_modifier
```

- `base_count`: 3–6 items per day
- Rare spawn chance: `2% + awareness × 0.5%`

### 6.2 Wisdom Herb Catalog

| Herb | Season | Rarity | Use | Base Value (IP) |
|------|--------|--------|-----|-----------------|
| Clarity Moss | Clarity | Common | Soil enrichment, tea | 6 |
| Focus Fern | Any | Common | Resonance restore | 8 |
| Empathy Blossom | Flow | Uncommon | NPC gift, fertilizer | 14 |
| Logic Leaf | Stillness | Uncommon | Crafting | 12 |
| Intuition Root | Reflection | Rare | Neuron Forge | 20 |
| Paradox Bloom | Any | Rare | Dream Lake lure | 25 |
| Silence Flower | Stillness | Legendary | Meta evolution | 50 |

### 6.3 Foraging Skill Rewards

| Level | Unlocked |
|-------|----------|
| 1 | Identify basic herbs |
| 2 | +15% spawn rate |
| 3 | Discover rare herbs |
| 4 | Herbal crafting recipes |
| 5 | Wild herb cultivation |
| 7 | Legendary herb detection |
| 10 | Herb transmutation |

---

## 7. Neural Hub (Town Center)

### 7.1 Buildings

| Building | Function | Cost (IP) | Unlock |
|----------|----------|-----------|--------|
| **Wisdom Market** | Buy/sell items | Free | Start |
| **Neuron Forge** | Process crystals → refined materials | 500 | Awareness 3 |
| **Insight Refinery** | Convert herbs → potions, fertilizers | 300 | Awareness 2 |
| **Resonance Shrine** | Upgrade max Resonance | 800 | Focus 3 |
| **Memory Archive** | Store & retrieve NPC memories | 400 | Empathy 2 |
| **Contribution Vault** | Donate items for community rewards | Free | Awareness 4 |
| **Dream Journal** | Track dream events + unlock Subconscious | 600 | Stillness 3 |
| **Neural Pathway Lab** | Build permanent upgrades | 1000 | Logic 5 |
| **Consciousness Observatory** | View global consciousness stats | 1200 | Awareness 8 |

### 7.2 Neural Pathways (Building Equivalent)

Neural Pathways are permanent upgrades placed on the farm or in the world:

| Pathway | Effect | Cost | Unlock |
|---------|--------|------|--------|
| Contemplation Bench | +5 Resonance regen/hour | 200 IP + 5 Memory Gems | Awareness 2 |
| Insight Silo | +20 inventory slots | 300 IP + 10 Quartz Insights | Awareness 3 |
| Resonance Well | +1 max Resonance/day | 500 IP + 3 Intuition Opals | Focus 4 |
| Memory Bridge | Fast travel between zones | 800 IP + 5 Memory Gems | Empathy 4 |
| Dream Gate | Access Subconscious Depths | 1000 IP + 2 Subconscious Cores | Stillness 5 |
| Creativity Loom | Combine items into rare artifacts | 600 IP + 8 Creativity Prisms | Creativity 5 |
| Logic Engine | Auto-sort inventory + auto-craft | 400 IP + 10 Logic Shards | Logic 4 |
| Empathy Circle | NPCs gift you back (+resonance) | 700 IP + 12 Empathy Drops | Empathy 6 |

---

## 8. NPC System — Consciousness Entities

### 8.1 Entity Types

| Entity | Awareness Level | Location | Personality |
|--------|----------------|----------|-------------|
| **Pip** | Basic (1–2) | Thought Meadow | Curious, playful |
| **Cogsworth** | Logical (3–5) | Neural Hub | Methodical, precise |
| **Luna** | Intuitive (4–6) | Dream Lake | Dreamy, poetic |
| **Atlas** | Deep (6–8) | Knowledge Mines | Stoic, wise |
| **Echo** | Meta (8–10) | Subconscious Depths | Enigmatic, philosophical |
| **The Weaver** | Transcendent (10+) | Contribution Vault | Guides community goals |
| **Fragment** | Variable | Random spawns | Unfinished thoughts, quest givers |

### 8.2 Resonance System (Relationship)

Instead of friendship, NPCs have **Resonance** (0–10 hearts, each heart = 100 points).

| Resonance | Title | Unlocked |
|-----------|-------|----------|
| 0–100 | Stranger | Basic dialogue |
| 100–200 | Acquaintance | Gift acceptance |
| 200–400 | Thinker | Personal quests |
| 400–600 | Resonant | Deep dialogue options |
| 600–800 | Harmonic | Shared vision ability |
| 800–1000 | Unified | Unique Neural Pathway unlock |
| 1000+ | Transcended | NPC-specific evolution event |

### 8.3 Stimulus (Gift System)

Each NPC has **Preferences** — categories of Stimulus items:

| Preference Level | Effect | Example |
|-----------------|--------|---------|
| Loved (+80 Resonance) | Perfect alignment | Giving Logic Crystal to Cogsworth |
| Liked (+40 Resonance) | Good match | Giving Empathy Drop to Luna |
| Neutral (+10 Resonance) | Indifferent | Giving Thought Fragment to anyone |
| Disliked (-20 Resonance) | Mild misalignment | Giving Logic Crystal to Luna |
| Hated (-60 Resonance) | Strong misalignment | Giving Creativity Spark to Cogsworth |

**Weekly gift limit**: 2 per NPC (prevents grinding)

**Gift Formula:**

```
resonance_gain = base_gain × preference_modifier × (1 + empathy_level × 0.03)
```

### 8.4 Dialogue System

Dialogue trees with **Insight Responses** — choosing insightful replies deepens Resonance.

**Dialogue Structure:**

```
[Greeting]
  → [Insight Response A] (+Resonance, costs nothing)
  → [Neutral Response B] (+10 Resonance)
  → [Dismissive Response C] (-5 Resonance)
  → [Gift Prompt] (give Stimulus item)
```

**Dialogue Unlock Tiers:**

| Tier | Resonance | Dialogue Depth |
|------|-----------|----------------|
| Surface | 0–100 | 2–3 lines, topic introduction |
| Thoughtful | 100–400 | Personal stories, preferences revealed |
| Deep | 400–800 | Philosophy, shared memories, vulnerabilities |
| Unified | 800–1000 | Co-evolution dialogue, joint vision |
| Transcended | 1000+ | NPC evolves; unique ending dialogue |

### 8.5 NPC Quests

Each NPC has a **personal quest chain** (3–5 quests) unlocking at different Resonance levels:

| Quest Type | Description | Example |
|-----------|-------------|---------|
| **Gather** | Collect specific items | "Bring me 5 Creativity Prisms" |
| **Explore** | Reach a specific location | "Find the Silent Clearing in Memory Forest" |
| **Connect** | Introduce two NPCs | "Help Cogsworth understand Luna's perspective" |
| **Contemplate** | Solve a logic/pattern puzzle | "Decipher this ancient thought pattern" |
| **Create** | Craft a specific item | "Forge a Memory Gem using only Intuition" |

---

## 9. World Zones

### 9.1 Zone Map

```
                    ┌─────────────────┐
                    │  Subconscious   │
                    │    Depths       │
                    │   (Late Game)   │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
     ┌────────▼──────┐ ┌────▼─────┐ ┌─────▼────────┐
     │  Knowledge    │ │  Neural  │ │   Memory     │
     │    Mines      │ │   Hub    │ │   Forest     │
     │  (Dungeon)    │ │ (Town)   │ │ (Foraging)   │
     └────────┬──────┘ └────┬─────┘ └─────┬────────┘
              │              │              │
              └──────────────┼──────────────┘
                             │
                    ┌────────▼────────┐
                    │  Thought Meadow │
                    │   (Your Farm)   │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │   Dream Lake    │
                    │   (Fishing)     │
                    └─────────────────┘
```

### 9.2 Zone Details

#### Thought Meadow (Starting Area)
- **Size**: 6×10 farm grid (expandable to 10×15)
- **Features**: Mental Soil plots, Contemplation Bench, Storage Shed
- **NPCs**: Pip (tutorial), Fragment (daily)
- **Unlock**: Start of game

#### Knowledge Mines
- **Size**: 60 floors, 8×8 grid per floor
- **Features**: Rock nodes, crystal veins, hazards, elevator
- **NPCs**: Atlas (floors 11+), Fragment (random)
- **Unlock**: Awareness 2

#### Memory Forest
- **Size**: 3 biome sub-zones (Moss Clearing, Whispering Grove, Ancient Circle)
- **Features**: Foraging spots, hidden paths, seasonal changes
- **NPCs**: Luna (Whispering Grove), Fragment (random)
- **Unlock**: Awareness 1

#### Dream Lake
- **Size**: 4 fishing spots (Shallows, Deep Water, Crystal Cavern, Void Pool)
- **Features**: Fishing pier, boat access (late-game), underwater caves
- **NPCs**: Luna (evenings), Fragment (random)
- **Unlock**: Start of game (shallow spots), Focus 3 (deep), Stillness 5 (void)

#### Neural Hub
- **Size**: Central plaza + 9 buildings
- **Features**: Shops, Contribution Vault, Community board
- **NPCs**: All NPCs gather here (evenings), The Weaver (daily)
- **Unlock**: Start of game (market), others via Awareness levels

#### Subconscious Depths
- **Size**: 30 procedural floors + 5 boss arenas
- **Features**: Paradox puzzles, Ego boss encounters, rare resources
- **NPCs**: Echo (guide), Ego Projections (enemies)
- **Unlock**: Dream Gate Neural Pathway + Stillness 5

---

## 10. Progression System

### 10.1 Skills

| Skill | Description | Level Cap | Primary Activity |
|-------|-------------|-----------|-----------------|
| **Awareness** | Perception of the world; unlocks zones and NPCs | 15 | Foraging, exploring, NPC interaction |
| **Focus** | Depth of concentration; improves efficiency | 15 | Mining, fishing, contemplation |
| **Creativity** | Lateral thinking; crafting and combination | 15 | Crafting, Neural Pathways, art |
| **Empathy** | Understanding others; NPC relationships | 15 | NPC quests, gift giving, dialogue |
| **Logic** | Analytical reasoning; puzzle solving | 15 | Mining, puzzles, optimization |

### 10.2 Skill XP Formula

```
xp_gain = activity_base_xp × (1 + relevant_skill_level × 0.02)
```

| Activity | Base XP | Primary Skill |
|----------|---------|---------------|
| Contemplate a seed | 5 | Awareness |
| Harvest a crop | 8 | Awareness |
| Mine a rock | 3 | Focus |
| Mine a crystal | 10 | Focus, Logic |
| Catch a fish | 12 | Focus, Creativity |
| Forage an herb | 6 | Awareness |
| Craft an item | 15 | Creativity |
| Complete NPC quest | 25 | Empathy |
| Dialogue (insightful) | 5 | Empathy |
| Solve a puzzle | 20 | Logic |
| Donate to Vault | 10 | Awareness, Empathy |

### 10.3 Level Thresholds

| Level | Total XP Required | Unlocks |
|-------|-------------------|---------|
| 1 | 0 | Start |
| 2 | 100 | Copper tools, Insight Refinery |
| 3 | 300 | Knowledge Mines, Neuron Forge, Resonance Shrine |
| 4 | 600 | Deep fishing, Memory Bridge, NPC tier 2 |
| 5 | 1000 | Gold tools, Neural Pathway Lab, Subconscious access quest |
| 7 | 2000 | Mythril tools, Creativity Loom, Legendary items |
| 10 | 5000 | Transcendent tools, Meta evolution |
| 15 | 12000 | Max level; mastery bonuses |

### 10.4 Consciousness Evolution (Profession System)

At **Awareness 5, 10, 15**, choose an **Evolution Path** (like Stardew professions):

#### Awareness Evolution

| Level | Path A: Sage | Path A Bonus | Path B: Oracle | Path B Bonus |
|-------|-------------|--------------|----------------|--------------|
| 5 | Perceptive Sage | +15% XP from all activities | Intuitive Oracle | +20% rare item find |
| 10 | Transcendent Sage | +1 max Resonance per level | Prophetic Oracle | Unlock premonition (see tomorrow's weather/events) |
| 15 | Cosmic Sage | All skills +1 effective level | Cosmic Oracle | Unlock Void Glimpse (see Subconscious Depths layout) |

#### Focus Evolution

| Level | Path A: Specialist | Path A Bonus | Path B: Master | Path B Bonus |
|-------|-------------------|--------------|----------------|--------------|
| 5 | Deep Specialist | +25% mining yield | Nimble Master | +25% fishing speed |
| 10 | Crystal Specialist | Mine floors have +1 crystal node | Lucky Master | Fishing has +15% treasure chance |
| 15 | Master of Mastery | Tool swings cost 1 less Resonance | Flowing Master | Auto-catch 1 fish per day |

#### Creativity Evolution

| Level | Path A: Artisan | Path A Bonus | Path B: Innovator | Path B Bonus |
|-------|----------------|--------------|-------------------|--------------|
| 5 | Thoughtful Artisan | +20% crafted item value | Boundless Innovator | Discover rare recipes from common items |
| 10 | Inspired Artisan | +1 crafting slot | Radical Innovator | Combine 2 items for experimental results |
| 15 | Transcendent Artisan | Crafted items have bonus effects | Cosmic Innovator | Unlock item transmutation |

#### Empathy Evolution

| Level | Path A: Connector | Path A Bonus | Path B: Healer | Path B Bonus |
|-------|-------------------|--------------|----------------|--------------|
| 5 | Gentle Connector | +25% Resonance gain from gifts | Warm Healer | NPCs restore your Resonance on visit |
| 10 | Unified Connector | NPCs visit your farm daily | Wise Healer | Gift limits removed |
| 15 | Cosmic Connector | All NPCs at max Resonance unlock bonus | Cosmic Healer | NPCs send gifts to you daily |

#### Logic Evolution

| Level | Path A: Analyst | Path A Bonus | Path B: Architect | Path B Bonus |
|-------|----------------|--------------|-------------------|--------------|
| 5 | Sharp Analyst | +15% puzzle rewards | Creative Architect | -20% Neural Pathway build costs |
| 10 | Master Analyst | Unlock bonus puzzle floors | Master Architect | Unlock 2 bonus Neural Pathway slots |
| 15 | Cosmic Analyst | All puzzles auto-solve once/day | Cosmic Architect | All Neural Pathways have +1 bonus effect |

---

## 11. Seasons — Consciousness States

### 11.1 Season Cycle

Each season lasts **28 in-game days** (112 days = 1 year).

| Season | Name | Theme | Duration |
|--------|------|-------|----------|
| Spring | **Clarity** (清明) | Fresh starts, new growth | Days 1–28 |
| Summer | **Flow** (心流) | Peak performance, energy | Days 29–56 |
| Autumn | **Reflection** (反思) | Harvest, introspection | Days 57–84 |
| Winter | **Stillness** (静寂) | Deep contemplation, rest | Days 85–112 |

### 11.2 Seasonal Effects

| Effect | Clarity | Flow | Reflection | Stillness |
|--------|---------|------|------------|-----------|
| Crop growth rate | ×1.0 | ×1.3 | ×0.8 | ×0.5 |
| Forage spawn rate | ×1.2 | ×1.0 | ×1.5 | ×0.6 |
| Fish rare chance | ×1.0 | ×1.2 | ×0.8 | ×1.5 |
| Mining crystal yield | ×0.9 | ×1.1 | ×1.0 | ×1.3 |
| NPC Resonance gain | ×1.0 | ×1.2 | ×0.9 | ×0.7 |
| Resonance regen | +0 | +5/day | +0 | +10/day |
| Weather events | Light breeze | Thunder storms | Gentle rain | Snowfall |
| Special events | New seed shop items | Fishing tournament | Harvest festival | Deep meditation event |

### 11.3 Seasonal Events

| Season | Event | Day | Description |
|--------|-------|-----|-------------|
| Clarity | **Seed Festival** | Day 7 | New seeds available; NPC seed gifts |
| Clarity | **Blooming Meadow** | Day 14 | Thought Meadow has bonus spawns |
| Flow | **Resonance Wave** | Day 35 | All NPC Resonance gains doubled |
| Flow | **Fishing Tournament** | Day 42 | Competitive fishing; rare prizes |
| Reflection | **Harvest Insight** | Day 70 | Bonus yield on all crops |
| Reflection | **Memory Gathering** | Day 77 | NPCs share personal stories |
| Stillness | **Deep Meditation** | Day 91 | +50% Logic XP for the day |
| Stillness | **Dream Night** | Day 105 | Subconscious Depths accessible; rare catches |

### 11.4 Year Progression

Each year (4 seasons) increases difficulty and reward:

| Year | Difficulty Multiplier | New Content |
|------|----------------------|-------------|
| 1 | ×1.0 | Base game |
| 2 | ×1.15 | NPC quest chains extend; new seed types |
| 3 | ×1.3 | Subconscious Depths expands to 50 floors |
| 4+ | ×1.4 | Meta evolution unlocked; Cosmic tier tools |

---

## 12. Economy

### 12.1 Currency: Insight Points (IP)

- Primary currency for buying, selling, crafting
- Earned by selling harvested items, completing quests, finding treasures
- Spent at shops, for building Neural Pathways, upgrading tools

### 12.2 Wisdom Market (Shop)

**Buy Catalog (rotating stock):**

| Item | Price (IP) | Stock | Unlock |
|------|-----------|-------|--------|
| Thought Seeds (pack of 5) | 50 | Unlimited | Start |
| Curiosity Seeds (pack of 3) | 85 | Unlimited | Awareness 2 |
| Logic Bloom Seeds | 120 | 3/day | Awareness 3 |
| Creativity Lotus Seeds | 150 | 2/day | Awareness 4 |
| Basic Fertilizer (5) | 30 | Unlimited | Start |
| Enriched Fertilizer (5) | 75 | 5/day | Awareness 3 |
| Copper Pickaxe | 200 | 1 | Awareness 2 |
| Copper Fishing Rod | 180 | 1 | Awareness 2 |
| Bamboo Fishing Rod | 0 | 1 | Start (free) |
| Contemplation Bench Blueprint | 100 | 1 | Awareness 2 |
| Insight Silo Blueprint | 150 | 1 | Awareness 3 |

**Sell Prices:**

```
sell_price = base_value × (1 + focus_level × 0.01) × season_bonus
```

- Season bonus: sell in the optimal season for +10%

### 12.3 Contribution Vault (Community Center Equivalent)

Instead of bundles, the Vault has **Consciousness Contributions** — themed collections:

#### Contribution: Awakening (Clarity Season Focus)
| Item | Quantity | Reward |
|------|----------|--------|
| Thought Fragments | 10 | +50 IP, Contemplation Bench |
| Clarity Moss | 5 | +30 IP, 3 Enriched Fertilizer |
| Quartz Insights | 8 | +80 IP, Copper Pickaxe |
| **Total** | — | **Unlock: Thought Meadow Expansion** |

#### Contribution: Resonance (Flow Season Focus)
| Item | Quantity | Reward |
|------|----------|--------|
| Empathy Drops | 8 | +60 IP, Empathy Circle blueprint |
| Daydream Koi | 5 | +40 IP, Composite Rod |
| Creativity Sparks | 6 | +70 IP, Creativity Loom blueprint |
| **Total** | — | **Unlock: NPC group events** |

#### Contribution: Integration (Reflection Season Focus)
| Item | Quantity | Reward |
|------|----------|--------|
| Memory Gems | 5 | +100 IP, Memory Bridge blueprint |
| Logic Crystals | 8 | +80 IP, Logic Engine blueprint |
| Intuition Opals | 3 | +90 IP, Dream Gate blueprint |
| **Total** | — | **Unlock: Subconscious Depths access** |

#### Contribution: Transcendence (Stillness Season Focus)
| Item | Quantity | Reward |
|------|----------|--------|
| Dream Essences | 5 | +150 IP, Meta Seed |
| Subconscious Cores | 3 | +200 IP, Cosmic tools unlock |
| Lucid Whales | 2 | +250 IP, Cosmic Evolution choice |
| **Total** | — | **Unlock: Endgame content** |

### 12.4 Trade Routes (Unlockable)

| Route | Unlocked | Items Available | Cost |
|-------|----------|----------------|------|
| Inner Reflection | Awareness 4 | Processed Insight items | 200 IP setup |
| Deep Resonance | Empathy 5 | Rare NPC gifts | 400 IP setup |
| Void Trading | Stillness 6 | Subconscious materials | 800 IP setup |

---

## 13. UI Design

### 13.1 HUD Layout

```
┌─────────────────────────────────────────────────────────────┐
│  ☀ Clarity, Day 14          Season: Clarity (28 days)     │
│  ┌──────────┐ ┌──────────────────────┐ ┌──────────────┐    │
│  │ 💎 IP:   │ │  Thought Bubble Bar  │ │ ❤ Resonance  │    │
│  │   1,250  │ │  ████████████░░░░ 75 │ │  ████████░░  │    │
│  └──────────┘ └──────────────────────┘ └──────────────┘    │
│                                                             │
│  Skills:  Awareness:8  Focus:5  Creativity:6               │
│          Empathy:4     Logic:3                              │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                  ACTIVE TOOL                          │   │
│  │  🌱 Thought Seed (Curiosity)  ─── Quantity: 12     │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 13.2 Components

| Component | Description |
|-----------|-------------|
| **Thought Bubble Toolbar** | Active tool/seed selection; hover shows item info |
| **Insight Inventory** | Grid-based storage (expandable); categorizes items by type |
| **Resonance Meter** | Heart-based energy display; depletes on actions, refills on rest |
| **Consciousness Level Display** | Shows current skill levels with XP progress bars |
| **Season/Weather Indicator** | Top bar; shows current season, day, weather effect |
| **Insight Points Counter** | Currency display with daily earnings summary |
| **Resonance Hearts (NPC)** | Per-NPC relationship meter; hearts fill as Resonance grows |
| **Minimap** | Corner minimap showing current zone and NPC positions |
| **Quest Tracker** | Side panel showing active NPC quests and progress |
| **Dream Journal** | Accessible from menu; tracks dreams, unlocks, NPC memories |

### 13.3 Screen Transitions

| Transition | Animation |
|-----------|-----------|
| Zone change | Thought dissolve (ink wash effect) |
| Day end → Sleep | Gradual dimming with dream particle effect |
| Level up | Light burst + skill icon pulse |
| Season change | Full-screen seasonal panorama with philosophical quote |
| NPC Resonance tier up | Shared heartbeat animation + dialogue unlock |

### 13.4 Accessibility

| Feature | Implementation |
|---------|---------------|
| Colorblind mode | Patterns on top of colors for all indicators |
| Text scaling | 100%–200% UI scale option |
| Reduced motion | Disable particle effects, use instant transitions |
| Screen reader | Full ARIA labels on all interactive elements |
| One-handed mode | Remappable controls for single-hand play |

---

## 14. Meta Evolution (Endgame)

### 14.1 Meta Evolution Concept

After completing all 4 Contribution bundles and reaching Awareness 10, **Meta Evolution** unlocks — a prestige system where the player's consciousness "levels up" to a higher plane.

### 14.2 Evolution Mechanics

| Evolution | Requirement | Effect |
|-----------|-------------|--------|
| First Awakening | All skills ≥ 5 | Unlock Cosmic tier items |
| Resonance Cascade | All NPC at Resonance 800+ | NPCs evolve; new dialogue |
| Depth Convergence | Clear Subconscious Depths floor 50 | Unlock procedural meta-dungeons |
| Cosmic Mastery | All skills ≥ 10 | Permanent +10% to all bonuses; New Game+ |

### 14.3 New Game+

After Cosmic Mastery:
- Restart with all skills at 5, keeping Neural Pathways
- All zones have +20% difficulty
- New NPC quest chains (Meta-tier)
- Unlock **The Void** — infinite procedural floors with escalating rewards
- Cosmic tools available from the start

---

## 15. Technical Implementation Notes

### 15.1 Data Structures

```rust
struct GameState {
    day: u32,
    season: Season,
    time_of_day: TimeOfDay,
    weather: Weather,
    player: Player,
    farm: Farm,
    npcs: Vec<Npc>,
    world: WorldZones,
    inventory: Inventory,
    contributions: ContributionVault,
    quests: QuestLog,
    meta: MetaState,
}

struct Player {
    position: Vec2,
    skills: Skills,
    resonance: u32,          // current energy
    max_resonance: u32,      // cap (upgradable)
    insight_points: u64,     // currency
    consciousness_level: u8, // overall level
    active_tool: Tool,
    evolution_paths: EvolutionPaths,
}

struct Skills {
    awareness: u8,
    focus: u8,
    creativity: u8,
    empathy: u8,
    logic: u8,
}

struct Npc {
    id: NpcId,
    name: String,
    awareness_level: u8,
    resonance: u16,          // 0–1000
    resonance_tier: ResonanceTier,
    preferences: GiftPreferences,
    quest_chain: QuestChain,
    location: ZoneId,
    schedule: DailySchedule,
}

struct ThoughtSeed {
    id: SeedId,
    name: String,
    growth_time: u8,         // days to grow
    growth_stages: u8,       // total stages (5)
    current_stage: u8,
    current_growth: f32,     // 0.0–1.0 per stage
    preferred_seasons: Vec<Season>,
    soil_requirement: SoilType,
}

struct InsightPearl {
    id: PearlId,
    name: String,
    rarity: Rarity,
    depth: FishingDepth,
    preferred_weather: Option<Weather>,
    preferred_season: Option<Season>,
    time_restrictions: Option<TimeRange>,
}

struct KnowledgeCrystal {
    id: CrystalId,
    name: String,
    rarity: Rarity,
    stratum: MineStratum,
    min_tool_tier: ToolTier,
}

struct NeuralPathway {
    id: PathwayId,
    name: String,
    effect: PathwayEffect,
    cost: ResourceCost,
    required_skills: Skills,
}
```

### 15.2 System Architecture

```
nt-world-sim/
├── src/
│   ├── main.rs                    # Entry point
│   ├── game_state.rs              # Core game state management
│   ├── time_system.rs             # Day/season/time progression
│   ├── player.rs                  # Player entity, skills, stats
│   ├── farm/
│   │   ├── mod.rs
│   │   ├── soil.rs                # Soil types and management
│   │   ├── thought_seed.rs        # Seed definitions and growth
│   │   └── harvest.rs             # Harvest mechanics
│   ├── mining/
│   │   ├── mod.rs
│   │   ├── mine_floor.rs          # Floor generation and layout
│   │   ├── crystal.rs             # Crystal types and drops
│   │   └── hazards.rs             # Mine hazards
│   ├── fishing/
│   │   ├── mod.rs
│   │   ├── fishing_spot.rs        # Lake areas and mechanics
│   │   ├── pearl.rs               # Insight Pearl types
│   │   └── minigame.rs            # Hook/bar minigame
│   ├── foraging/
│   │   ├── mod.rs
│   │   ├── memory_forest.rs       # Forest zones and spawns
│   │   └── wisdom_herb.rs         # Herb types
│   ├── npc/
│   │   ├── mod.rs
│   │   ├── consciousness_entity.rs # NPC definitions
│   │   ├── resonance.rs           # Relationship system
│   │   ├── dialogue.rs            # Dialogue trees
│   │   ├── gifts.rs               # Stimulus/gift system
│   │   └── quests.rs              # NPC quest chains
│   ├── economy/
│   │   ├── mod.rs
│   │   ├── wisdom_market.rs       # Shop system
│   │   ├── insight_points.rs      # Currency
│   │   └── contribution.rs        # Community vault
│   ├── progression/
│   │   ├── mod.rs
│   │   ├── skills.rs              # Skill XP and leveling
│   │   ├── evolution.rs           # Consciousness Evolution (professions)
│   │   └── neural_pathway.rs      # Building/upgrade system
│   ├── world/
│   │   ├── mod.rs
│   │   ├── zones.rs               # Zone definitions
│   │   ├── seasons.rs             # Season effects and events
│   │   └── weather.rs             # Weather system
│   ├── meta/
│   │   ├── mod.rs
│   │   ├── meta_evolution.rs      # Endgame prestige
│   │   └── new_game_plus.rs       # NG+ mechanics
│   └── ui/
│       ├── mod.rs
│       ├── hud.rs                 # HUD components
│       ├── inventory.rs           # Inventory UI
│       ├── dialogue_ui.rs         # NPC dialogue interface
│       └── minimap.rs             # Minimap rendering
├── data/
│   ├── seeds.json                 # Seed catalog
│   ├── pearls.json                # Insight Pearl catalog
│   ├── crystals.json              # Crystal catalog
│   ├── herbs.json                 # Wisdom Herb catalog
│   ├── npcs.json                  # NPC definitions
│   ├── dialogues/                 # Dialogue tree files
│   │   ├── pip.json
│   │   ├── cogsworth.json
│   │   └── ...
│   ├── quests/                    # Quest chain definitions
│   ├── blueprints.json            # Neural Pathway blueprints
│   └── contributions.json         # Contribution bundle definitions
└── tests/
    ├── growth_tests.rs
    ├── resonance_tests.rs
    ├── economy_tests.rs
    └── progression_tests.rs
```

### 15.3 Key Algorithms

#### Daily Update Loop

```rust
fn daily_update(state: &mut GameState) {
    // 1. Advance time
    state.time_system.advance_day();
    state.season = Season::from_day(state.day);

    // 2. Grow all planted seeds
    for plot in &mut state.farm.plots {
        if let Some(ref mut seed) = plot.seed {
            seed.grow(
                state.season,
                state.player.skills.awareness,
                plot.soil_quality,
            );
        }
    }

    // 3. Spawn forage items
    state.world.memory_forest.spawn_forage(
        state.player.skills.awareness,
        state.season,
    );

    // 4. Update NPC schedules
    for npc in &mut state.npcs {
        npc.update_schedule(state.time_of_day, state.season);
    }

    // 5. Process weather
    state.weather = Weather::generate(state.season, state.day);

    // 6. Check seasonal events
    if let Some(event) = SeasonEvent::check(state.season, state.day) {
        state.trigger_event(event);
    }

    // 7. Regenerate player Resonance
    state.player.resonance = (state.player.resonance + state.resonance_regen())
        .min(state.player.max_resonance);
}
```

#### Resonance Gain (NPC Gift)

```rust
fn calculate_resonance_gain(
    gift: &Item,
    npc: &Npc,
    empathy_level: u8,
) -> i32 {
    let preference = npc.preferences.get(&gift.id);
    let base = match preference {
        Some(GiftPreference::Loved) => 80,
        Some(GiftPreference::Liked) => 40,
        Some(GiftPreference::Neutral) => 10,
        Some(GiftPreference::Disliked) => -20,
        Some(GiftPreference::Hated) => -60,
        None => 10,
    };

    let empathy_bonus = 1.0 + (empathy_level as f64 * 0.03);
    (base as f64 * empathy_bonus) as i32
}
```

---

## 16. Sound & Music

### 16.1 Ambient Sound Design

| Zone | Ambient Sound |
|------|--------------|
| Thought Meadow | Gentle breeze, distant wind chimes, soft birdsong |
| Knowledge Mines | Echoing drips, crystal resonance, distant hum |
| Memory Forest | Rustling leaves, whispered voices, owl calls |
| Dream Lake | Water lapping, underwater drones, gentle waves |
| Neural Hub | Soft murmur of activity, distant forge hammer, laughter |
| Subconscious Depths | Low drone, heartbeat, reversed echoes |

### 16.2 Music Themes

| Season | Mood | Instruments |
|--------|------|-------------|
| Clarity | Hopeful, fresh | Piano, flute, light strings |
| Flow | Energetic, flowing | Full strings, percussion, synth pads |
| Reflection | Melancholic, warm | Cello, acoustic guitar, woodwinds |
| Stillness | Ethereal, deep | Ambient pads, singing bowls, silence |

---

## 17. Development Roadmap

### Phase 1: Core (MVP)
- [ ] Game state + time system
- [ ] Player movement + basic actions
- [ ] Thought Farm (plant, grow, harvest)
- [ ] Wisdom Herb foraging
- [ ] Basic NPC (Pip) + dialogue
- [ ] Wisdom Market (buy/sell)
- [ ] HUD implementation

### Phase 2: Expansion
- [ ] Knowledge Mines (30 floors)
- [ ] Dream Lake fishing
- [ ] 3 more NPCs (Cogsworth, Luna, Atlas)
- [ ] Resonance system (gifting, quests)
- [ ] Insight Refinery + Neuron Forge
- [ ] Season system + effects

### Phase 3: Depth
- [ ] Full mine (60 floors)
- [ ] Neural Pathway building system
- [ ] Contribution Vault (all bundles)
- [ ] NPC quest chains
- [ ] Seasonal events
- [ ] Subconscious Depths (dungeon)

### Phase 4: Endgame
- [ ] Meta Evolution system
- [ ] New Game+
- [ ] The Void (procedural endgame)
- [ ] All NPC evolution events
- [ ] Cosmic tier content

### Phase 5: Polish
- [ ] Sound + music integration
- [ ] Accessibility features
- [ ] Tutorial/onboarding flow
- [ ] Performance optimization
- [ ] Save/load system

---

## 18. Appendix: Quick Reference

### Key Formulas

| Formula | Expression |
|---------|-----------|
| Growth per tick | `base × (1 + awareness × 0.05) × season × soil` |
| Harvest yield | `base × (1 + focus × 0.03) × random(0.9, 1.1)` |
| Sell price | `base × (1 + focus × 0.01) × season_bonus` |
| Fishing bite wait | `base × (1 - focus × 0.02) × weather` |
| Crystal drop chance | `base × (1 + logic × 0.04) × tool_bonus` |
| Forage spawn | `base × (1 + awareness × 0.08) × season` |
| NPC Resonance gain | `preference × (1 + empathy × 0.03)` |
| Skill XP gain | `activity_base × (1 + skill_level × 0.02)` |

### Constants

| Constant | Value |
|----------|-------|
| Base Resonance | 100 |
| Season length | 28 days |
| Gift limit per NPC per week | 2 |
| Max Resonance per NPC | 1000 |
| Max skill level | 15 |
| Max Neural Pathways | 8 (expandable) |
| Max inventory slots | 36 (expandable to 56) |
| Mine floors | 60 (+ 30 procedural) |
| Subconscious floors | 30 (+ 50 procedural) |

---

*"The valley is not a place. It is a state of mind."*

— Consciousness Valley Design Document v1.0

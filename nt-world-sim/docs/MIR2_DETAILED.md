# Legend of Mir 2 (热血传奇) — Comprehensive Implementation Reference

## Table of Contents

1. [Mir2 Graphics Resources](#1-mir2-graphics-resources)
2. [Mir2 Game Mechanics](#2-mir2-game-mechanics)
3. [Mir2 Map Format](#3-mir2-map-format)
4. [Mir2 Combat System](#4-mir2-combat-system)
5. [Mir2 Character System](#5-mir2-character-system)
6. [Mir2 Monster System](#6-mir2-monster-system)
7. [Mir2 Item System](#7-mir2-item-system)
8. [Mir2 UI Design](#8-mir2-ui-design)

---

## 1. Mir2 Graphics Resources

### 1.1 Asset Downloads

| Resource | URL | Size | Description |
|----------|-----|------|-------------|
| **LOMCN 2TB Asset Collection** | https://www.lomcn.net/forum/threads/2tb-of-mir-more-updated-15th-june.111818/ | 2TB+ | Monster sprites, armours, UI, wings, effects — the most comprehensive collection |
| **itch.io Mir2 Assets** | https://duqingnian.itch.io/assets-of-legend-of-mir2 | 58MB+ | UI elements (传奇UI.rar) |
| **Crystal Map Libraries** | https://mirfiles.co.uk/resources/mir2/ | Various | Map tilesets, object libraries (.lib format) |
| **HD Assets (HDAssets.rar)** | https://mirfiles.com/resources/mir2/Data/HDAssets.rar | ~1.75GB | HD character models with wedding models |
| **Wemade Mir2 Official** | https://mir2.mironline.co.kr/ | — | Original Korean client files |

### 1.2 Monster Sprite Sets (by region)

| Region | Monsters | Download |
|--------|----------|----------|
| **Bichon Town/Cave** | ChestnutTree, Skeleton, BoneFighter, BoneWarrior, AxeSkeleton, BoneElite, CrawlingZombie, Zombie, PriestZombie, Ghoul | https://mega.nz/file/MdVxRAZR |
| **WoomyonWoods** | Deer, Scarecrow, Yob, RakingCat, HookingCat, CannibalPlant, SpittingSpider, ForestYeti, Oma, OmaFighter, OmaWarrior | https://mega.nz/file/cBMg2S6Y |
| **Mongochon** | Sheep, Wolf, RedSnake, YellowSnake, SkyStinger, VisceralWorm, ShellNipper, Keratoid, GiantKeratoid | https://mega.nz/file/wFlUxLTZ |
| **Zuma Temple** | WedgeMoth, BugbatMaggot, GiantRat, ZumaArcher, ZumaStatue, ZumaGuardian, ZumaTaurus | https://mega.nz/file/cN8CQTIT |
| **AngelStoneTomb** | BlackMaggot, SnakeScorpion, RedBoar, BlackBoar, WhiteEvilBoar, KingHog | https://mega.nz/file/EYdlQaBb |
| **Bug Cave** | WhimperingBee, Centipede, Tongs, GiantWorm, EvilTongs, EvilCentipede | https://mega.nz/file/1A0EHRbK |
| **Prajna Island** | RoninGhoul, ToxicGhoul, BoneBladeMan, BoneSpearMan, BoneArcher, BoneCaptain, BoneLord | https://mega.nz/file/pZ8hBLgT |
| **Prajna Temple** | Minotaur, IceMinotaur, ElectricMinotaur, WindMinotaur, RightGuard, LeftGuard, FireMinotaur, MinotaurKing | https://mega.nz/file/YMkD3KoR |
| **Past Bichon** | AxeOma, CrossbowOma, FlailOma, OmaGuard, SwordOma, WingedOma, OmaKing | https://mega.nz/file/td8T2Bja |
| **Red Moon Valley** | VenomSpider, GangSpider, GreatSpider, LureSpider, SpiderBat, RootSpider, BigApe, EvilApe, RedEvilApe, RedMoonEvil | https://mega.nz/file/xZ0CjIYC |
| **BDD** | EvilTongs, EvilSnake, KingScorpion, IncarnatedWT, KingHog, IncarnatedZT, DarkDevil | https://mega.nz/file/AR8THSBa |

### 1.3 Chinese Asset Sources (传奇素材)

| Source | URL | Content |
|--------|-----|---------|
| **爱给网 (Aigei)** | https://m.aigei.com/legend/legend/ | 传奇素材合集：怪物、技能、称号、装备、地图 |
| **魂罪游戏素材** | https://www.hunzui.com/category/2.html | 传奇素材下载：地图、武器、衣服、套装、首饰、怪物、道具 |
| **39传奇素材网** | https://39sc.com/ | 武器素材(485)、衣服素材(272)、套装(285)、时装(1127)、怪物(771)、地图(883)、技能特效(149) |
| **传奇素材网 (MirScz)** | https://www.mirscz.com/ | 时装、复古怪物、地砖地图、首饰、法宝、盾牌、翅膀坐骑 |

### 1.4 Sprite Format Details

- **Crystal format**: `.lib` files (MLibrary) — used by Crystal client
- **Korean format**: `.wil` / `.wix` files — original Korean client
- **Shanda format**: `.wil` files — Chinese version
- **Sprite sheets**: 8-direction sprites, typically 48×32 pixel tile units
- **Offsets**: Each sprite has X/Y offset values for alignment (e.g., `-50 -105 48 7 -44`)
- **Animation frames**: Characters have Idle, Move, Attack (1/2/3), Cast, Death, Struck, Harv frames

---

## 2. Mir2 Game Mechanics

### 2.1 Overview

Legend of Mir 2 (1999, WeMade Entertainment/ActozSoft) is a sprite-based isometric 3D MMORPG. The game uses a tile-based map system with 48×32 pixel units creating a "checkerboard" movement feel.

### 2.2 Core Systems

| System | Description |
|--------|-------------|
| **Movement** | 8-direction grid-based (48×32 tile units), client-side obstacle detection |
| **Combat** | Real-time auto-attack + skill system, accuracy vs agility hit calculation |
| **Leveling** | Experience-based, each class has proficiency-based skill leveling |
| **PK System** | Player killing tracked with points, name color changes (White→Yellow→Red) |
| **Guild System** | Guild wars, castle sieges, guild storage |
| **Trading** | Player-to-player trades, NPC shops, auction house |
| **Fishing** | Dedicated fishing system with special map attributes |

### 2.3 Game World Structure

```
Bichon Province (starting area)
├── Bichon Town (safe zone, shops)
├── Bichon Wall (PvP zone)
├── Woomyon Woods (low-level mobs)
├── Wooma Temple (mid-level cave)
├── Mongchon Province
│   ├── Mongchon Town
│   └── Mongchon Cave
├── Zuma Temple (high-level)
├── AngelStone Tomb
├── Bug Cave
├── Prajna Island
│   ├── Prajna Cave
│   └── Prajna Temple
├── Red Moon Valley
├── Past Bichon
├── Sabuk (castle siege area)
└── Endless World (level 35+)
```

### 2.4 Server Architecture

- **TCP socket communication** between client and server
- **Packet-based protocol**: Each packet has 2-byte length + 2-byte ID header
- **Game loop**: Server processes packets in while loop, broadcast updates to nearby players
- **Database**: Server.MirADB (Access/SQL) stores accounts, characters, items

---

## 3. Mir2 Map Format

### 3.1 Map File Structure

Maps are stored as `.map` binary files. The Crystal engine supports **10+ map formats**:

| Type | Header Signature | Description |
|------|-----------------|-------------|
| **Type 0** | Default | Original Mir2 format |
| **Type 1** | `Map 2010 Ver 1.0` | Wemade 2010 format |
| **Type 2** | Bytes[4]=0x0F, 14 bytes/tile | Shanda older format |
| **Type 3** | Bytes[4]=0x0F, 14 bytes/tile, >52+W×H×14 bytes | Shanda 2012 format |
| **Type 4** | `Mir2 AntiHack` title | Wemade anti-hack labyrinth maps |
| **Type 5** | First byte = 0 | Wemade Mir3 format |
| **Type 6** | `(C) SNDA, MIR3.` title | Shanda Mir3 format |
| **Type 7** | `0x0D 0x4C` header | 3/4 Heroes format |
| **Type 100** | `0x43 0x23` (C#) | Crystal custom format |

### 3.2 Cell Structure (Type 100 — Crystal format)

Each cell contains **38 bytes** of data:

```
BackIndex     : int16    (2 bytes)  — tile library index
BackImage     : int32    (4 bytes)  — tile image index
MiddleIndex   : int16    (2 bytes)  — middle layer library
MiddleImage   : int16    (2 bytes)  — middle layer image
FrontIndex    : int16    (2 bytes)  — front layer library
FrontImage    : int16    (2 bytes)  — front layer image
DoorIndex     : byte     (1 byte)   — door reference
DoorOffset    : byte     (1 byte)   — door animation offset
FrontAnimFrame: byte     (1 byte)   — front animation frame count
FrontAnimTick : byte     (1 byte)   — front animation speed
MiddleAnimFrame: byte    (1 byte)   — middle animation frame count
MiddleAnimTick: byte     (1 byte)   — middle animation speed
TileAnimImage : int16    (2 bytes)  — tile animation image
TileAnimOffset: int16    (2 bytes)  — tile animation offset
TileAnimFrames: byte     (1 byte)   — tile animation frame count
Light         : byte     (1 byte)   — lighting level (100-119 = fishing)
```

### 3.3 Map Layers

Maps use **3 rendering layers**:

1. **Back Layer** — Ground tiles (grass, dirt, stone, water)
2. **Middle Layer** — Ground decorations (paths, puddles, mud patches)
3. **Front Layer** — Objects (buildings, trees, walls, furniture)

**Layer conventions differ between Mir2 and Mir3**:
- **Mir2 maps**: Front layer for most objects, middle for ground decorations
- **Mir3 maps**: Middle layer for most objects, front only for tag collisions

### 3.4 Cell Attributes (Flags)

| Flag | Bit | Meaning |
|------|-----|---------|
| `HighWall` | BackImage bit 29 | Can fire over (ranged attacks pass) |
| `LowWall` | FrontImage bit 15 | Can't walk, can't fire over |
| `FishingCell` | Light 100-119 | Fishing attribute zone |

### 3.5 Map Dimensions

- Typical town: 200×200 tiles
- Province maps: up to 700×700 tiles
- Each tile = 48×32 pixels (isometric diamond)
- Example: Bichon Province = 33,600 × 22,400 pixels

### 3.6 Tile Libraries

```
Data/Map/
├── WemadeMir2/
│   ├── Tiles.lib        — ground tiles
│   ├── Smtiles.lib      — small tiles
│   ├── Objects.lib       — object library 1
│   ├── Objects2.lib      — object library 2
│   └── ...Objects26.lib  — up to 26 libraries
├── ShandaMir2/
│   ├── Tiles.lib
│   ├── Tiles2.lib ... Tiles199.lib
│   ├── Smtiles.lib
│   ├── Objects.lib ... Objects500.lib
│   └── AniTiles1.lib    — animated tiles
└── Maps/
    └── *.map             — actual map files
```

---

## 4. Mir2 Combat System

### 4.1 Hit Calculation

**Accuracy vs Agility** is the core hit/miss mechanic:

```
// Generate random number between 0 and Target Agility
random = Random(0, Target.Agility)

// If attacker's Accuracy > random, hit lands
if (Attacker.Accuracy > random):
    hit lands → damage calculation
else:
    miss → 0 damage
```

**Example**: 115 Accuracy vs 230 Agility → 50% hit rate

### 4.2 Damage Calculation

**Physical (Warrior)**:
```
Damage = Random(DC_min, DC_max) × SkillModifier
FinalDamage = Damage - Target.AC
```

**Magic (Wizard)**:
```
Damage = Random(MC_min, MC_max) × SkillModifier
FinalDamage = Damage - Target.MAC
```

**Soul (Taoist)**:
```
Damage = Random(SC_min, SC_max) × SkillModifier
FinalDamage = Damage - Target.MAC
```

### 4.3 Skill Damage Formulas (from Korean data)

| Class | Skill | Formula | Notes |
|-------|-------|---------|-------|
| Warrior | Flaming Sword | 2.5 × DC_max | Single target burst |
| Warrior | DaySlash (日斩) | 5.0 × DC_max | Ultimate single target |
| Wizard | HellFire | 1.2 × MC_max + 19 | Close range flame |
| Wizard | Thunderbolt | 1.5 × MC_max + 36 | AoE lightning |
| Wizard | IceStorm | 1.0 × MC_max + 26 | AoE ice |
| Wizard | FireExplosion | 1.2 × MC_max + 12 | 3×3 AoE |
| Taoist | PoisonCloud | 1.0 × SC_min + 0.5 × SC_max | DoT damage |
| Taoist | Plague | 2 × SC_max | Mana drain + debuff |

### 4.4 Critical & Elemental Damage

- **Critical Rate**: Random chance for bonus damage
- **Critical Damage**: Multiplier on critical hits
- **Elemental Damage**: Dark/Holy/Ice/Fire/Lightning — adds bonus to spells using that element
- **Ignore Target Defense**: Chance to bypass AC/MAC
- **Reflect Damage**: Chance to reflect incoming damage back

### 4.5 Status Effects

| Effect | Source | Effect |
|--------|--------|--------|
| **Poison (Grey)** | Taoist | Damage over time |
| **Poison (Yellow)** | Taoist | Reduces defenses |
| **Paralysis** | Wizard Electric Shock | Target immobilized temporarily |
| **Freeze** | Wizard Ice spells | Target frozen, can't move/attack |
| **Slow** | Taoist Curse | Reduced movement/attack speed |
| **Stun** | Warrior skills | Target can't act |
| **Hide** | Taoist | Invisible to monsters |
| **Blessed Armour** | Taoist | +AC buff |
| **Soul Shield** | Taoist | +MAC buff |

---

## 5. Mir2 Character System

### 5.1 Character Classes

| Class | Role | Primary Stats | Description |
|-------|------|---------------|-------------|
| **Warrior (战士)** | Tank/DPS | DC (Destructive Class), AC | Highest defense and HP. Melee attacks. Skills: Fencing, Slaying, Thrusting, HalfMoon, FlamingSword, ProtectionField, Rage |
| **Wizard (法师)** | Ranged DPS | MC (Magic Class), AMC | Powerful AoE magic. Low HP/defense. Skills: Fireball, HellFire, Thunderbolt, IceStorm, FireExplosion, Lightning |
| **Taoist (道士)** | Healer/Support | SC (Soul Class), DC, AC, AMC | Heals, buffs, summons pets, poisons. Skills: Healing, Poisoning, SummonSkeleton, SummonShinsu, SoulShield, BlessedArmour |
| **Assassin (刺客)** | Melee DPS | DC, Accuracy | High single-target damage, low defense. Skills: DoubleSlash, FlashDash, backstab abilities |

### 5.2 Character Stats

| Stat | Full Name | Purpose |
|------|-----------|---------|
| **AC** | Armour Class | Physical defense (melee damage reduction) |
| **MAC** | Magic Armour Class | Magic defense |
| **DC** | Destructive Class | Physical attack power |
| **MC** | Magic Class | Magic attack power |
| **SC** | Soul Class | Taoist spell power |
| **HP** | Health Points | Physical strength |
| **MP** | Mana Points | Magical power |
| **Accuracy** | Precision | Hit rate (vs Agility) |
| **Agility** | Evasion | Evasion rate (vs Accuracy) |
| **Carrying Weight** | — | Total equipment weight |
| **Weight of Hands** | — | Hand-held weapon weight |

### 5.3 Leveling System

- Experience gained from killing monsters
- Each level requires more XP
- Skill proficiency increases through use (practice-based leveling)
- Skills have 3 levels of mastery, each requiring proficiency points
- Level cap varies by server (original: ~40, Crystal: up to 200+)

### 5.4 Skill Proficiency System

```
Skill Level 0 → Level 1: Requires X proficiency + Player Level Y
Skill Level 1 → Level 2: Requires X2 proficiency + Player Level Y2
Skill Level 2 → Level 3: Requires X3 proficiency + Player Level Y3
```

Example Warrior Fencing:
- Level 0: Available at level 7
- Level 1: 270 proficiency → +3 Accuracy
- Level 2: 600 proficiency → +6 Accuracy
- Level 3: 1300 proficiency → +9 Accuracy

---

## 6. Mir2 Monster System

### 6.1 Monster Data Structure

Each monster in the database has:

| Field | Description |
|-------|-------------|
| **Name** | Monster display name |
| **Race** | Monster behavior type (AI) |
| **RaceImg** | Visual appearance index |
| **ImgIndex** | Sprite library index |
| **Level** | Monster level (determines XP, drops) |
| **HP** | Health points |
| **DC/AC/MAC** | Attack/defense stats |
| **Experience** | XP reward |
| **Drop Items** | Loot table |

### 6.2 Monster Race Types (AI Behaviors)

| Race ID | Behavior | Description |
|---------|----------|-------------|
| 0 | Passive | Doesn't attack unless attacked |
| 1 | Aggressive | Attacks players on sight |
| 2 | Guard | Attacks red-named players |
| 3 | GuardArcher | Ranged guard, attacks red players |
| 4 | AngryOma | Special Oma behavior |
| 5 | Scorpion | Special scorpion AI |
| 6 | Tree | Static tree, becomes hostile |
| 7-10 | Various | Specialized AI types |

### 6.3 Monster Animation Structure (Mir2)

Each monster sprite file contains:

```
Idle:     4 frames × 8 directions = 32 images
Move:     5 frames × 8 directions = 40 images
Attack:   5 frames × 8 directions = 40 images
Struck:   2 frames × 8 directions = 16 images
Death:    10 frames × 8 directions = 80 images
Harvest:  1 frame × 8 directions = 8 images
Cast1:    6 frames × 8 directions = 48 images
Cast2:    6 frames × 8 directions = 48 images
```

### 6.4 Monster Database

The Crystal server uses a database (ServerDB/MirDB) containing all monster definitions. Key fields from the "Almost Complete Monster Database":

| Monster | Race | RaceImg | Notes |
|---------|------|---------|-------|
| Yimoogi | — | 100 | Set RaceImg to 100 for full effect |
| OmaKingSpirit | — | 78 | Set RaceImg to 78 |
| KingScorpion | 129 | 60 | Special scorpion behavior |
| BoneSpearman | — | 68 | RaceImg 68 |
| BoneBlademan | — | 66 | RaceImg 66 |
| BoneArcher | — | 69 | RaceImg 69 |
| ChestnutTree | 121 | — | Trees use Race 121 |
| DarkDevil | 181 | — | Ranged attack behavior |
| TurtleBoss | 164 | — | Turtle boss Race 164 |

---

## 7. Mir2 Item System

### 7.1 Equipment Slots

| Slot | Types | Notes |
|------|-------|-------|
| **Weapon** | Swords, Axes, Maces, Wands, Bows, Spears | 1-handed or 2-handed |
| **Armour** | Robes, Leather, Chain, Plate | Class-restricted |
| **Helmet** | Various helmets | Some class-specific |
| **Boots** | Various footwear | Some provide special effects |
| **Belt** | Various belts | Weight capacity |
| **Necklace** | Amulets, Pendants | SC/MC/DC bonuses |
| **Ring (L/R)** | Rings | Dual-wearable |
| **Bracelet (L/R)** | Bracelets, Gloves | Dual-wearable |
| **Torch** | Light sources | 2-handed or 3-handed slots |
| **Stone** | Magic stones | Embedded stat bonuses |

### 7.2 Item Attributes

**Basic Stats**:
| Attribute | Description |
|-----------|-------------|
| **AC** | Armour Class — physical defense |
| **MAC** | Magic Armour Class — magic defense |
| **DC** | Destructive Class — physical attack |
| **MC** | Magic Class — magic attack |
| **SC** | Soul Class — taoist attack |
| **Accuracy** | Hit rate bonus |
| **Agility** | Evasion bonus |

**Advanced Stats**:
| Attribute | Description |
|-----------|-------------|
| **HP/MP Recovery** | Health/mana regeneration rate |
| **Bag Weight** | Increases carrying capacity |
| **Durability** | Item wear (Strong = reduced loss) |
| **Luck** | Critical hit rate |
| **Curse** | Weapon curse from PK kills |

**Elemental Stats**:
| Attribute | Description |
|-----------|-------------|
| Dark/Holy/Ice/Fire/Lightning Damage % | Elemental damage bonus |
| Dark/Holy/Ice/Fire/Lightning Resist % | Elemental resistance |
| Physical/Magic Damage Reduction % | Flat damage reduction |
| Reflect Damage % | Chance to reflect damage |
| Poison Resistance % | Poison resistance |
| Ignore Target Defense % | Bypass AC/MAC chance |

### 7.3 Item Special Properties

| Property | Bit Value | Effect |
|----------|-----------|--------|
| Para | 1 | Paralyze on hit |
| Teleport | 2 | Teleport ability |
| ClearRing | 4 | Clear nearby monsters |
| ProtectionRing | 8 | Protection aura |
| RevivalRing | 16 | Revive on death |
| MuscleRing | 32 | +Damage |
| FlameRing | 64 | Fire damage |
| HealingRing | 128 | HP regen |
| Probe | 256 | Detect hidden |
| Skill | 512 | Skill bonus |
| NoDurabilityLoss | 1024 | Never degrades |

### 7.4 Item Upgrading

**Weapon Upgrading Process**:
1. Level weapon through use (hitting monsters)
2. Collect Black Ore (mining in Desolate Mine)
3. Collect upgrade materials (bracelets/necklaces of matching level)
4. Visit Blacksmith NPC (Bichon or Sabuk)
5. Submit weapon + materials → success rate based on bet time

**Upgrade Tiers**:
- Level 1 → 2: 3 items + ore
- Level 2 → 3: 20 items + ore
- Level 3+: Higher material counts

**Accessory Upgrading**:
- Level 1: 5 identical items
- Level 2: 20 identical items
- Level 3: 40-80 identical items

### 7.5 Item Drop System

Items drop from monsters based on:
- Monster level (higher level = better drops)
- Monster type (bosses drop rare items)
- Region (some items are region-specific)
- Random chance (RNG-based)

**Drop Table Structure**:
```
MonsterID → Level → DropRate → [ItemList with weights]
```

---

## 8. Mir2 UI Design

### 8.1 Original UI Layout

```
┌─────────────────────────────────────────────────────┐
│  [Minimap]                    [Chat Window]          │
│  ┌──────┐                   ┌──────────────────┐    │
│  │  ◉   │                   │ [System Messages] │    │
│  │      │                   │ [Player Chat]     │    │
│  └──────┘                   └──────────────────┘    │
│                                                      │
│                    GAME WORLD                        │
│                 (Isometric View)                     │
│                                                      │
│                                                      │
│  ┌──────────────┐  ┌──────────┐  ┌──────────────┐  │
│  │   HP Globe    │  │  Skills  │  │   MP Globe    │  │
│  │   (Red)       │  │  Bar     │  │   (Blue)      │  │
│  └──────────────┘  └──────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────┘
```

### 8.2 UI Elements

| Element | Description |
|---------|-------------|
| **HP/MP Globes** | Circular displays showing health (red) and mana (blue) |
| **Skill Bar** | Quick-access skill buttons at bottom center |
| **Minimap** | Top-left corner, shows nearby area |
| **Chat Window** | Bottom area, shows messages and system text |
| **Inventory** | Grid-based backpack (40 slots max, weight-limited) |
| **Equipment Panel** | Shows equipped items with stats |
| **Character Panel** | Shows AC/DC/MC/SC/HP/MP/Level/Class |
| **Trade Window** | Player-to-player trading interface |
| **NPC Dialog** | Text-based NPC interaction |
| **PK Indicator** | Name color changes (White→Yellow→Red) |

### 8.3 UI Rendering (from JS recreation)

The JS implementation uses **3 layered canvases**:
1. **Bottom layer**: Map tiles (ground)
2. **Middle layer**: Sprites (characters, monsters, items, effects)
3. **Top layer**: UI elements (inventory, HP bar, skill bar)

**Performance optimization**: 40 FPS target, 25ms per frame budget, 10ms for JS logic after rendering.

### 8.4 Map Tile Rendering

- Split map into 480×320 pixel tiles (10×10 game tiles)
- Load 4×4 = 16 tiles in a container
- As player moves, shift container and swap tiles at boundaries
- Preload tiles outside visible area for smooth scrolling

---

## 9. Implementation Resources

### 9.1 Crystal Source Code (C#)

| Resource | URL |
|----------|-----|
| **Crystal Server+Client** | https://github.com/Suprcode/Crystal |
| **Crystal Database** | https://github.com/Suprcode/Crystal.Database |
| **Crystal Map Editor** | https://github.com/Suprcode/Crystal.MapEditor |
| **LOMCN Wiki** | https://www.lomcn.net/wiki/index.php/Main_Page |
| **Build Guide** | https://www.lomcn.net/wiki/index.php/Getting_Started |
| **Tutorials** | https://www.lomcn.net/forum/forums/crystalm2-tutorials.634/ |

### 9.2 JavaScript Recreation

| Resource | URL | Description |
|----------|-----|-------------|
| **c-zhuo/Mir2** | https://github.com/c-zhuo/Mir2 | JS+Canvas+Node recreation, character/equipment/spawning/combat/inventory |
| **Easycanvas library** | https://github.com/chenzhuo1992/easycanvas | Data-driven Canvas rendering library |
| **Client file analysis** | https://github.com/jootnet/mir-client-analyzer | Mir2 client file format analysis |
| **Technical article** | https://www.mo4tech.com/javascript-high-imitation-of-legend-of-mir-ii-game.html | Detailed implementation guide (map rendering, UI, sprites) |

### 9.3 C++ Recreation

| Resource | URL | Description |
|----------|-----|-------------|
| **mir2x** | https://github.com/jhuix-games/mir2x | Actor-model parallelism MMORPG, C++20 coroutine, includes client+server+mapeditor+animaker |

### 9.4 Unity Port

| Resource | URL | Description |
|----------|-----|-------------|
| **Unity remake analysis** | https://medium.com/@Sou1gh0st/legend-of-mir2-source-code-analysis-and-remake-in-unity-1-4b9ffd4de3a8 | Detailed source code analysis, porting guide |

### 9.5 Map Tools

| Tool | URL | Description |
|------|-----|-------------|
| **Crystal Map Editor** | https://github.com/Suprcode/mir2-mapeditor | Official map editor |
| **Xiyue MapEditor** | https://www.lomcn.net/forum/threads/xiyue-mapeditor-akaras-mod-updated-by-m2p.106376/ | Modified map editor with improvements |
| **MapConverter** | https://github.com/EliteMir/MapConverter | Convert maps between formats |
| **MapTileSet** | https://mirfiles.co.uk/resources/mir2/Maps/Tools/ | Adjust map tile references |
| **16BitMapEditor** | https://mirfiles.co.uk/resources/mir2/Maps/Tools/16BitMapEditor.rar | 16-bit color map editor |

### 9.6 Data Tools

| Tool | URL | Description |
|------|-----|-------------|
| **EnigmaWare Config Editor** | https://enigmaware.co.uk/mir2/configeditor.php | Server config editor with item database |
| **C#Mir Dev Resource** | http://mir2.ueuo.com/ | Server/client configs, import/export, item images |
| **Library Editor** | Included in Crystal | Edit .lib sprite libraries |

---

## 10. Key Implementation Notes

### 10.1 Rendering Pipeline

1. **Map Layer**: Load tile from `.lib` library by index → draw at isometric position
2. **Object Layer**: Load sprite animation frame → draw with offset at entity position
3. **UI Layer**: Draw HP/MP bars, inventory, skill bar as overlay

### 10.2 Network Protocol

- TCP socket, async send/receive
- Packet format: `[2 bytes length][2 bytes packet ID][payload]`
- Client sends: C.Walk, C.Attack, C.UseSpell, C.PickUp, C.Trade, etc.
- Server broadcasts: S.UserLocation, S.ObjectWalk, S.ObjectAttack, etc.

### 10.3 Map Loading Algorithm

```python
# Pseudo-code for map loading
for x in range(map_width):
    for y in range(map_height):
        cell = read_cell(file_bytes, offset)
        map_cells[x][y] = Cell(
            back_index = cell.BackIndex,
            back_image = cell.BackImage,
            middle_index = cell.MiddleIndex,
            middle_image = cell.MiddleImage,
            front_index = cell.FrontIndex,
            front_image = cell.FrontImage,
            door = cell.DoorIndex,
            light = cell.Light,
            is_walkable = check_flags(cell)
        )
        offset += CELL_SIZE
```

### 10.4 Sprite Rendering

```python
# Pseudo-code for character rendering
def render_character(char, direction, frame):
    sprite = lib_loader.get_sprite(char.sprite_index)
    image = sprite.get_frame(direction, frame)
    offset_x, offset_y = sprite.get_offset(direction, frame)
    
    screen_x = char.x * TILE_WIDTH / 2 - camera_x
    screen_y = char.y * TILE_HEIGHT / 2 - camera_y
    
    canvas.draw(image, screen_x + offset_x, screen_y + offset_y)
```

---

*Report compiled from: LOMCN community, GitHub Crystal project, Chinese game asset sites, Mir2 wiki, and technical implementation articles.*
*Last updated: 2026-09-13*

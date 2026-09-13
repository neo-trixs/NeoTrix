# 鬼谷八荒 (Tale of Immortal) — Comprehensive Game Research

## 1. Game Overview

**鬼谷八荒** (Guigubahuang / Tale of Immortal) is an open-world sandbox cultivation RPG developed by 鬼谷工作室 (Ghost Valley Studio). It combines the traditional Chinese cultivation (修仙) system with the mythological setting of the Classic of Mountains and Seas (山海经).

**Core Concept:** The player starts as an ordinary mortal and must cultivate their way through ten major realms to become an immortal. The game features:
- Procedurally generated world with grid-based exploration
- Deep cultivation system with 10 major realms × 3 sub-stages each
- Skill/equipment hybrid system (skills ARE gear)
- Rich NPC social simulation (relationships, dual cultivation, sect politics)
- Three starting artifacts: 昊天眼 (Heavenly Eye), 双鱼佩 (Twin Fish Pendant), 炼妖壶 (Demon Refining Cauldron)

**Genre:** Open-world sandbox cultivation RPG
**Engine:** Unity
**Platform:** PC (Steam), Mobile (手游)

---

## 2. Character System

### 2.1 Appearance
- Full character customization: face, hairstyle, clothing, accessories
- Beautiful art style with detailed character portraits

### 2.2 Core Attributes

| Category | Attributes |
|----------|-----------|
| **General Stats** | Lifespan, Mood, Health, Stamina, Vitality (HP), Energy (MP), Focus |
| **Combat Stats** | ATK, DEF, Martial/Spiritual RES, CRIT, CRIT RES, Agility, CRIT DMG, CRIT DR |
| **Proficiency Stats** | Weapon aptitudes (Sword, Blade, Spear, Fist, Palm, Finger), Spirit Root aptitudes (Fire, Water, Thunder, Wind, Earth, Wood) |
| **Social Stats** | Charisma (7 levels), Luck, Insight, Mood, Reputation |
| **Economy Stats** | Spirit Stones (currency), Sect Contribution Tokens |
| **Alignment** | Righteous (正道) vs Demonic (魔道) |

### 2.3 Spirit Root System (灵根)

Six elemental spirit roots determine which elemental techniques you can learn:
- **火灵根** (Fire Spirit Root)
- **水灵根** (Water Spirit Root)
- **雷灵根** (Thunder Spirit Root)
- **风灵根** (Wind Spirit Root)
- **土灵根** (Earth Spirit Root)
- **木灵根** (Wood Spirit Root)

Spirit roots are increased through spiritual fruits (灵果) from sect stores, or by defeating specific bosses.

### 2.4 Weapon Aptitudes (功法资质)

Six weapon types with separate aptitude values:
- **剑** (Sword) — highest single-target DPS
- **刀** (Blade) — close-range AoE, lifesteal
- **枪** (Spear) — multi-projectile, piercing
- **拳** (Fist) — shield generation
- **掌** (Palm) — mid-range AoE
- **指** (Finger) — high hit count, multi-hit

### 2.5 Prior Birth Destinies (先天气运)

At character creation, the player rolls 9 destinies from a pool of hundreds, organized by rarity:

| Color | Rarity | Examples |
|-------|--------|---------|
| White | Common | 左撇子 (Insight+15), 木匠 (Insight+20, ATK-1) |
| Green | Uncommon | 吼声如雷 (ATK+5), 浩然正气 (Reputation+100) |
| Blue | Rare | 鹰眼 (Crit+30), 天生灵体 (Energy+100) |
| Purple (圣) | Holy | 火灵体 (Fire+25), 剑仙转世 (Sword+25) |
| Orange (仙) | Immortal | 武器大师 (Blade/Sword/Spear+20), 天命之孙 (Luck+30) |
| Red (神) | Divine | 人族圣体 (ATK+8, HP+150), 武圣转世 (ATK+10, DEF+5) |

Key stats to prioritize: **Insight** (reduces learning time, improves comprehension) and **Luck** (affects all random mechanics).

### 2.6 Retroactive Fate (逆天改命)

Earned at each realm breakthrough. Provides new destiny/fate options. Some are upgradeable through tiers (e.g., 小李飞剑 → 中李飞剑 → 大李飞剑). Key options include:

- **剑灵入门/进阶/精通** — Auto-attacking spectral sword
- **武法入门/进阶/精通** — Reduced skill cooldown
- **双灵/三灵/四灵共生** — Chance to cast skills extra times
- **红尘剑匣/红尘剑魂** — Summon sword spirits after dealing damage
- **兵刃专精** — Remove cast requirements for weapon ultimates

---

## 3. Cultivation System (境界系统)

### 3.1 Ten Major Realms

| # | Realm | Chinese | Unlock Condition |
|---|-------|---------|-----------------|
| 1 | Qi Refining | 炼气 | Starting realm |
| 2 | Foundation Establishment | 筑基 | Breakthrough with treasures + pills |
| 3 | Crystallization | 结晶 | Specific treasures + Crystallization Pill |
| 4 | Golden Core | 金丹 | Grade 1-5 path with treasures + Qi Pearls |
| 5 | Spirit Formation | 具灵 | Divine Souls from 5 bosses + materials |
| 6 | Nascent Soul | 元婴 | Tao Souls with purity + materials |
| 7 | Soul Formation | 化神 | Specialized breakthrough materials |
| 8 | Enlightenment | 悟道 | Tao Realm clearing + Tao Seeds |
| 9 | Transcendence | 羽化 | Advanced materials |
| 10 | Ascension | 登仙 | Final breakthrough |

Each realm has **Early, Mid, Late** sub-stages, totaling 30 levels.

### 3.2 Breakthrough Mechanics

**Three Paths per Breakthrough (Foundation example):**

| Path | Difficulty | Requirements | Reward |
|------|-----------|-------------|--------|
| Mortal Path | Easy | 1 treasure + pill | 1 attribute option |
| Earthly Path | Medium | 2 treasures + 3 elemental Qi | 3 attribute options |
| Heavenly Path | Hard | 3 treasures + all 6 elemental Qi | 6 attribute options |

**Breakthrough Flow:**
1. Cultivate on Qi Spots to reach bottleneck
2. Use appropriate pill (洗髓丸/涤髓丸 for sub-stages)
3. Gather required treasures and materials
4. Attempt breakthrough (has failure chance)
5. On failure: lightning tribulation or stat penalty
6. On success: choose from Fates (逆天改命 options)

### 3.3 Experience Sources

- **Qi Spots** — Map locations rich in cultivation energy
- **Sect Spirit Pavilion** — Faster cultivation in sect
- **Dual Cultivation** — Cultivate with partner for bonus EXP
- **Events/Quests** — Random events provide EXP

### 3.4 Cultivation Efficiency

- Matching spirit root attribute doubles cultivation efficiency
- Higher Insight reduces learning time
- Spiritual fruits boost corresponding aptitudes
- Low-level maps can be revisited for experience farming

---

## 4. Martial Arts System (功法系统)

### 4.1 Five Skill Types

| Type | Chinese | Description |
|------|---------|-------------|
| **Basic Attack** | 武技/灵技 | Primary attack skill (LMB). Weapon-type = 武技, Elemental = 灵技 |
| **Special Skill** | 绝技 | Active skill with longer cooldown and higher damage (RMB) |
| **Movement** | 身法 | Dash/mobility skill (Space). Used for dodging and activating ultimates |
| **Mind Technique** | 心法 | Passive stat boosts. Requires Dao Points (道点) to equip. 8 slots max |
| **Ultimate** | 神通 | Powerful ultimate with prerequisite conditions to activate (R key) |

### 4.2 Skill Quality & Affixes

**Quality Tiers:** White → Green → Blue → Purple → Orange → Red (highest)

**Affix System (词条):**
- Each skill has 6 affix slots that unlock with proficiency
- Affixes have their own quality tiers (White through Red)
- "Perfect" (完美) and "Near Perfect" (接近完美) are the target affix rolls
- Use Comprehension (参悟) to reroll affix quality
- Rollback points (回溯点) let you restore good affixes after bad rerolls

### 4.3 Skill Proficiency

Proficiency levels (5 tiers):
1. 乍炼 (Initial)
2. 娴熟 (Familiar)
3. 通宵 (Proficient)
4. 小成 (Minor Mastery)
5. 灵动 (Spiritual)

Gained through combat or training (spending Soul Stones). Higher proficiency unlocks more affixes.

### 4.4 Weapon System Details

**Each weapon type has:**
- 3 Basic Attacks (武技/灵技)
- 6 Special Skills (绝技)
- 1 Movement Skill (身法)
- 2 Ultimates (神通)

**Weapon-Specific Mechanics:**

| Weapon | Unique Mechanic |
|--------|----------------|
| Sword (剑) | Sword Slash stacks, Flowing Blood debuff, Extra damage below 50% HP |
| Blade (刀) | Lifesteal (7%), Blood Blade debuff, Execution threshold |
| Spear (枪) | Charge mechanic, Breaking Momentum buff, Armor Break |
| Fist (拳) | Shield generation per room, Iron Skin buff, Bone Break stun |
| Palm (掌) | Strength buff, Mid-range AoE |
| Finger (指) | Multi-hit, Highest hit count |

### 4.5 Build Archetypes

| Build | Core Skills | Playstyle |
|-------|------------|-----------|
| **Wind Sword (风剑)** | Clear Wind Sword + Wind Movement + Absolute Shadow Sword | High mobility, invincibility frames, safe |
| **Water Sword (水剑)** | Moon Shadow Sword + Water Clone + Myriad God Sword | Control + burst, clone distraction |
| **Fire Sword (火剑)** | Fire Spirit + Burning Stacks + Single-target burst | High single-target DPS |
| **Sword Spirit (剑灵)** | Sword Spirit chain + Auto-attack summon | No-mana, beginner-friendly |
| **Wufa CD (武法CD)** | Wufa chain + Cooldown reduction | Extreme burst, attack speed |
| **Fire Cultivator (火修)** | Fire Pill + AoE Fire + Phoenix Ultimate | Sustained AoE burn |
| **Thunder Cultivator (雷修)** | Thunder Light + Thunder Cloud + Thunder Punishment | CC + burst |

---

## 5. Combat System

### 5.1 Controls

| Key | Action |
|-----|--------|
| WASD | Movement |
| Left Click | Basic Attack (武技/灵技) |
| Right Click | Special Skill (绝技) |
| Space | Movement Skill (身法) |
| R | Ultimate (神通) |
| + Button | Use elixirs before combat |

### 5.2 Combat Resources

| Resource | Purpose |
|----------|---------|
| **Vitality** | HP. Reaches 0 = death |
| **Energy** | Mana for skills. Empty = can only use basic attack |
| **Focus** | Used for items during combat. Also affects steal chance and artifact equip limit |
| **Soul Stones** | Currency |
| **Spirit Stones** | General currency |

### 5.3 Combat Flow

1. **Pre-battle**: Use + buttons to consume elixirs for Vitality/Energy/Focus restoration
2. **Engagement**: WASD + LMB/RMB/Space/R for skill rotation
3. **Resource Management**: Monitor Energy (mana), use elixirs when low
4. **Movement**: Use 身法 for dodging, positioning, and activating certain ultimates
5. **Ultimate Activation**: Meet prerequisite conditions (e.g., deal X damage, accumulate stacks) then press R

### 5.4 Special Combat Mechanics

- **Dual Cultivation**: Partner cultivation for EXP and relationship
- **Sparring**: Friendly combat with NPCs for Knowledge points
- **Lethal Battle**: Can occur between Righteous and Demonic cultivators
- **Arena/Tournament**: Sect competitions and the "Rise to Immortality" tournament

### 5.5 Elixir/Item System

Equip up to 5 battle items (丹药/法宝) in hotbar. Each type stacks to 5. Using items costs Focus.

---

## 6. World System

### 6.1 Map Structure

The world is divided into 9 regions, each corresponding to cultivation stages:

| Region | Realm | Key Features |
|--------|-------|-------------|
| 白源区 (Baiyuan) | Tutorial | Starting area, tutorial encounters |
| 永宁州 (Yongning) | Qi Refining–Foundation | First major region, 16 sects |
| 雷泽 (Lei Ze) | Foundation transition | Dangerous swamp, elemental bosses |
| 华封州 (Huafeng) | Crystallization–Golden Core | Second major region |
| 十万大山 (100k Mountains) | Golden Core transition | Dangerous mountain range |
| 云陌州 (Yunmo) | Spirit Formation–Nascent Soul | Third major region |
| 永恒冰原 (Eternal Ice) | Higher realms | Frozen region |
| 暮仙州 (Muxian) | Late game | Advanced area |
| 迷途荒漠 (Lost Desert) | Endgame | Final region |

### 6.2 Exploration Mechanics

- **Grid-based movement**: Each step costs 1 day
- **Random events**: Encounters marked with ? on map
- **Kan'yu (堪舆) skill**: Detect hidden locations, treasures, NPCs
- **Treasure maps**: Can be purchased or found
- **Special zones**: Arcane Lands (1000 stones), Spirit Realm dungeons
- **Hidden Li Si (李四)**: Each region has a hidden NPC at specific coordinates giving rare items

### 6.3 Time System

- **30 days per month**
- **12 months per year**
- Monthly cycle with events
- "Skip Month" button for fast-forwarding
- Certain events only trigger at specific times

### 6.4 Map Discovery

- Use 寻灵符 (Spirit-seeking Talisman) to reveal 40 tiles around player
- Find map beacons to reveal towns and sects
- Towns enable fast travel between discovered locations
- Sect teleportation requires sufficient reputation

---

## 7. NPC System

### 7.1 NPC Types

- **Sect Members**: Varying ranks and allegiances
- **Roaming Cultivators**: Encounter on map
- **Shopkeepers**: Town markets
- **Quest Givers**: Various NPCs with tasks
- **Bosses**: Region bosses and hidden bosses

### 7.2 Relationship System

**Favorability Range:** -300 to +300 (displayed as hearts/flames, 60 points per heart)

| Favorability | Icon | Effect |
|-------------|------|--------|
| < -60 | 🔥🔥+ | Enemy, may attack you |
| -60 to 60 | 陌 | Stranger, neutral |
| ≥ 60 | ❤️ | Friend, positive interactions |

**Relationship Types:**

| Relationship | Requirements | Benefits |
|-------------|-------------|---------|
| Friends (好友) | Mutual favor ≥ 60 | Positive interaction bonuses |
| Enemies (仇人) | One-sided favor < -60 | May attack, hostile |
| Sworn Siblings (结义) | Favor + personality match | Faction benefits, protection |
| Master/Disciple (师徒) | Favor + personality | Training bonuses |
| Spouse/Dao Companion (道侣) | Favor + personality | Dual cultivation, inheritance |
| Adopted Parent/Child | Favor + personality | Family bonuses |

### 7.3 Interaction Actions

1. **Talk (交谈)** — Basic interaction, builds favor
2. **Gift (赠送)** — Give items to increase favor
3. **Request (索取)** — Ask for items (may decrease favor)
4. **Sparring (切磋)** — Friendly combat for Knowledge
5. **Dual Cultivation (双修)** — Joint cultivation for EXP
6. **Debate (论道)** — Earn Dao Points
7. **Steal (偷窃)** — Risky, based on Focus stat
8. **Attack (攻击)** — Combat, may trigger lethal battle

### 7.4 NPC Personality System

Each NPC has inner and outer personalities affecting interactions:

| Personality | Effect |
|-------------|--------|
| Selfless (无私) | Most sociable, +favor with everyone |
| Righteous (正直) | +favor with righteous, -favor with demonic |
| Loyal (忠贞) | Won't dual cultivate outside marriage |
| Passionate (情种) | Easy to dual cultivate with |
| Family-loving (爱家) | Protects family bonds |
| Vengeful (睚眦) | Never forgets enemies |

### 7.5 Favor Decay

Every 12 months (December), all favorability drifts toward 0:
- Higher favor decays slower
- Personality modifiers can resist decay (义气 = preserve friends, 情种 = preserve partners, 睚眦 = preserve enemies)

---

## 8. Sect System (宗门系统)

### 8.1 Sect Types

- **Branch Sects (分舵)**: Cannot become sect leader, auto-transfer to next region
- **Main Sects**: Can become sect leader, take over and control

### 8.2 Sect Alignment

- **Righteous (正道)**: Blue map markers
- **Demonic (魔道)**: Red map markers
- Opposite alignments are enemies
- Mismatched alignment causes contribution tax

### 8.3 Sect Buildings

| Building | Purpose |
|----------|---------|
| 议事大厅 | Sect management hub |
| 藏经阁 | Skill manual library |
| 任务大厅 | Quest board |
| 疗伤院 | Heal injuries |
| 演武堂 | Training ground |
| 聚灵阵 | Cultivation boost |
| 炼丹房 | Alchemy lab |
| 炼器房 | Artifact forging |
| 炼符房 | Talisman crafting |
| 风水楼 | Feng Shui research |
| 秘境 | Sect dungeon |
| 招贤堂 | Recruit new members |
| 聚宝仙楼 | Spirit Fruit store |

### 8.4 Sect Ranks

| Rank | Requirements | Benefits |
|------|-------------|---------|
| Outer Disciple (外门弟子) | Join sect | Basic access |
| Inner Disciple (内门弟子) | Contribution + Promotion trial | Better access |
| True Disciple (真传弟子) | Higher contribution | Advanced access |
| Elder (长老) | High contribution + reputation | Major access |
| Grand Elder (大长老) | Maximum contribution | Full access |
| Sect Leader (宗主) | Defeat current leader | Full control |

### 8.5 Sect Economy

- **Contribution Tokens (宗门贡献)**: Earned through sect quests, spent at sect stores
- **Sect Quests**: Collect herbs, refine pills, expand territory, eliminate enemies, collect ores, subdue demons
- **Protection Tax**: Sects collect taxes from their territory
- **Sect Wars**: Can conquer rival sects and take them over

### 8.6 Sect Management (as Sect Leader)

- Recruit cultivators via 招贤堂
- Assign members to production tasks
- Upgrade buildings (every 3 years)
- Set sect techniques (refreshes every 6 months)
- Manage diplomacy with other sects

---

## 9. Crafting System (炼丹/炼器系统)

### 9.1 Alchemy (炼丹)

**Requirements:**
1. **Recipes (丹方)**: Purchase from town workshops or find through events
2. **Materials**: Herbs gathered from regions, purchased from shops, or obtained from quests
3. **Furnace (丹炉)**: Required, obtained from sect stores (green/blue/purple tiers)
4. **Alchemy Aptitude**: Must meet minimum to use higher-tier furnaces

**75 Total Recipes** across all realms:

| Realm | Recipe Count | Example Pills |
|-------|-------------|---------------|
| Qi Refining–Foundation | 9 | Foundation Pill, Jade Cream Pill |
| Crystallization–Golden Core | 15 | Crystallization Pill, Cloud Pill |
| Spirit Formation–Nascent Soul | 19 | Spirit Formation Pill, Baby Pill |
| Soul Formation–Enlightenment | 16 | Enlightenment Pill, Soul Pill |
| Transcendence–Ascension | 16 | Return-to-Life Pill |

**Alchemy Process:**
1. Select recipe
2. Add materials to furnace (more slots = higher purity)
3. Balance elemental energies (Yin/Yang + elements)
4. Avoid wrong materials (creates impurities → explosion risk)
5. Higher purity = better pill quality

### 9.2 Pill Grades

Pills come in 6 grades (1st = highest quality, 6th = lowest). Higher grade = higher success rate for breakthroughs.

### 9.3 Artifact Forging (炼器)

- Craft weapons and accessories
- Upgrade equipment
- Requires specific materials and recipes

### 9.4 Talisman Crafting (炼符)

- Create combat talismans
- Teleportation talismans
- Various utility items

### 9.5 Feng Shui (风水)

- Detect hidden locations
- Find treasure
- Discover monster lairs
- Locate graves (risky but rewarding)

---

## 10. UI Design

### 10.1 Main Game Screen

**Layout:**
```
┌─────────────────────────────────────────────────────────┐
│  [Mini Map]                    [Region Name]    [Time]   │
│                                                         │
│                                                         │
│                    MAIN GAME VIEW                       │
│                 (Grid-based world map)                  │
│                                                         │
│                                                         │
│  [Character]  [Status Bars]              [Actions Menu]  │
├─────────────────────────────────────────────────────────┤
│  [Mail] [Quest] [Skills] [Items] [Craft] [Map] [Cultivate] [Skip Month]  │
│  [Favorability] [Current Realm] [Spirit Stones]         │
└─────────────────────────────────────────────────────────┘
```

### 10.2 Character Panel

**Tab 1: Attributes (属性)**
- General stats (HP, MP, Focus, etc.)
- Combat stats (ATK, DEF, etc.)
- Prior/Post Birth Destinies list

**Tab 2: Skills (技能)**
- 4 active skill slots: 武技/灵技, 绝技, 身法, 神通
- 8 passive mind technique slots (心法)
- Skill training interface
- Skill comprehension (参悟) interface
- Proficiency levels and affix management

**Tab 3: Artisanship (技艺)**
- Alchemy interface
- Feng Shui interface
- Elixir configuration

**Tab 4: Inventory (物品)**
- All items with filter/sort
- Equipment comparison
- Item details

**Tab 5: Career Stats (生涯统计)**
- Character history record
- Achievement tracking

**Tab 6: Relationships (人际关系)**
- All NPC relationships
- Family tree
- Friend/Enemy lists

### 10.3 Skill Interface Details

**Training Tab:**
- Select skill to train
- Drag slider to set training duration
- Cost: Soul Stones + Time
- Proficiency progress bar

**Comprehension Tab:**
- View all unlocked affixes
- Each affix shows quality tier (color)
- Rollback points to restore good affixes
- Cost: Spirit Stones + 悟心砂

**Mind Technique Tab:**
- View equipped mind techniques
- Dao Points budget display (total/used)
- 8 slots: 经(HP), 诀(Mana), 录(Focus), 劲(ATK), 式(DEF), 大法(Crit/CD), 神功(Magic DEF), 秘卷(Physical DEF)

### 10.4 Town/Market Interface

- **Market (坊市)**: Buy/sell items, potions, talismans
- **Workshop (工坊)**: Purchase recipes, craft pills
- **Manual Pavilion (琅琊阁)**: Buy skill manuals
- **Auction House (拍卖会)**: Bi-annual auction for rare items

### 10.5 Sect Interface

- Sect overview (alignment, rules, benefits)
- Building list with upgrade status
- Quest board
- Contribution store
- Rank/promotion interface
- Member management

### 10.6 Dialogue System

- Multiple choice dialogue options
- Options affect: favorability, alignment, outcomes
- Some choices locked behind stat checks
- Personality and relationship affect available options

---

## 11. Key Game Mechanics Summary

### 11.1 Core Loop

```
Explore World → Find Resources → Learn Skills → Fight Enemies →
Gain EXP → Break Through Realm → Get New Destinies → Repeat
```

### 11.2 Progression Gates

1. **Realm Breakthrough**: Requires specific materials + success chance
2. **Skill Learning**: Requires matching aptitude + Insight stat
3. **Sect Access**: Requires meeting elemental requirements
4. **Region Access**: Requires reaching certain realm thresholds
5. **Sect Rank**: Requires Contribution Tokens + Promotion trials

### 11.3 Resource Economy

| Resource | Source | Use |
|----------|--------|-----|
| Spirit Stones (灵石) | Quests, Selling, Stealing | Currency for all transactions |
| Soul Stones (魂石) | Monster drops, Combat | Skill training |
| Contribution Tokens | Sect quests | Sect store purchases |
| Dao Points (道点) | NPC debates, Events | Equip mind techniques |
| Herbs | Gathering, Monster drops | Alchemy materials |
| Spiritual Fruits | Sect stores, Boss drops | Boost aptitudes |

### 11.4 Difficulty Modes

- **Normal (普通)**: Standard experience
- **Hard (困难)**: Increased challenge
- **Chaos (混沌)**: Cannot reroll destinies, maximum difficulty

### 11.5 Save/Load

- Manual save/load system
- Auto-save on monthly transitions
- S/L (Save/Load) is a core strategy for optimizing outcomes
- Important for: breakthrough success, destiny rerolls, loot drops

---

## 12. Design Patterns for Simulation Reference

### 12.1 Character Creation Pattern

```
Appearance Customization → Birth Destiny Roll (9 slots, rarity tiers) →
Attribute Assignment → Spirit Root Selection → Background Choice
```

### 12.2 Progression Pattern

```
Cultivation (EXP Gain) → Bottleneck → Gather Materials → Attempt Breakthrough →
Success: Choose Destiny + Realm Up | Failure: Retry with penalties
```

### 12.3 Combat Pattern

```
Basic Attack (resource-free) → Special Skill (energy cost, cooldown) →
Movement (positioning, dodge) → Ultimate (prerequisite conditions) →
Item Use (Focus cost, limited)
```

### 12.4 Social Pattern

```
Meet NPC → Interact (Talk/Gift/Gift/Request) → Favorability Change →
Threshold Crossed: Unlock Relationship Type → Relationship Benefits
```

### 12.5 Economy Pattern

```
Earn Spirit Stones (Quests/Selling) → Buy Resources (Market/Sect) →
Craft Items (Alchemy/Forge) → Use for Progression → Repeat
```

---

## 13. References

- 3DM Game Full Guide: https://www.3dmgame.com/gl/3824864.html
- Tale of Immortal Wiki: https://tale-of-immortal.fandom.com/
- TapTap Community Guides: https://www.taptap.cn/moment/
- Gamersky System Guides: https://www.gamersky.com/handbook/
- 17173 Beginner Guide: https://news.17173.com/z/ggbh/
- Steam Community Guide: https://steamcommunity.com/sharedfiles/filedetails/?id=2597185200

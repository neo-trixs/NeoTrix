# 鬼谷八荒 (Guigubahuang / Tale of Immortal) — Mod & Feature Research Report

> Research date: 2026-09-13
> Sources: 3DM, Steam Workshop, Nexus Mods, Bilibili, GamerSky, TapTap, GitHub

---

## 1. Game Overview

- **Developer**: 鬼谷工作室 (Guigu Studio)
- **Publisher**: 鬼谷工作室 / Lightning Games
- **Engine**: Unity (IL2CPP backend)
- **Genre**: Open-world sandbox cultivation RPG
- **Theme**: Xianxia (cultivation/immortality) + Shan Hai Jing (Classic of Mountains & Seas) mythology
- **Platforms**: PC (Steam), Switch, iOS, Android
- **EA Launch**: 2021-01-27 | **1.0 Release**: 2023-05-26
- **DLCs**: 不归玄境 (Survival Roguelike), 八荒博物志, 五朵金花 (5 new female characters + stories)
- **Total Steam Workshop mods**: 543+ (3DM site), 66+ (Nexus Mods)
- **Mod install path**: `<Game Folder>/ModExportData/` or Steam Workshop

---

## 2. Popular Mods (3DM & Steam Workshop)

### 2.1 Mod Frameworks (Prerequisites)

| Mod | Author | Purpose | Downloads |
|-----|--------|---------|-----------|
| **Fatury Framework** | Sorry | Core mod framework: dynamic/static portrait replacement, UI replacement, audio replacement, video dual-cultivation images, webm video support for artifact spirits | ~3,000+ Workshop |
| **【大鬼】UI Framework** | 八荒大鬼 | Provides UI creation API for other mods. Must be loaded at top of mod order | 2,994 Workshop ratings |
| **SaiLL Framework** | — | Alternative portrait/clothing framework, open wardrobe system | Popular on Bilibili |
| **Mod Manager (补丁管理器)** | — | One-click mod install manager, includes built-in patches | 7,293 downloads (3DM) |

### 2.2 Top Functional Mods

| Mod Name | Description | Size |
|----------|-------------|------|
| **自动战斗** (Nuyoah) | Full auto-combat: auto-cast martial skills, ultimate skills, movement skills, divine arts, domain. Auto-cast Shuangyu Pei, Haotian Eye, Lianyao Hu. Auto-positioning in dungeons, auto-advance to next room | 1.34MB |
| **功法系统大改** | Overhauls the skill system with XP-based progression, skill upgrading, name customization, quick learning, affix modification | 4,430 downloads |
| **八荒绝色榜 (BHJSB) / Virgin** | Character beauty ranking + virginity system patch | 6,751 downloads |
| **境界与寿命突破寿命修复补丁** | Fixes breakthrough/lifespan calculation bugs | 3,009 downloads |
| **万古神话 (Eternal Myth)** | Large story MOD: original cultivation story based on Chen Dong's novel trilogy, adds custom portraits, large dungeon battles (百断山, 海神岛, 绵心阁), unique female protagonist | 96.82MB |
| **八荒大乱斗** | Large-scale combat expansion mod | Part of mod packs |
| **合欢宗** | Adds the Hehuan Sect faction with related storylines and mechanics | Popular mod pack component |
| **鬼谷地牢 1.54** | Dungeon crawler expansion with custom illustrations | 3,378 downloads |
| **蛊真人 (Gu Zhenren)** | "I, Fang Yuan, am back" — player gets assimilated by Fang Yuan character from Reverend Insanity | 3,425 downloads |
| **再现斗罗大陆** | Douluo Dalu (Soul Land) expansion: custom ranks, soul rings, storyline, open map | 277MB |
| **灵活组队** | Flexible party system for companion management | 0.96MB |
| **AI强化MOD** | Enhanced NPC AI behavior | 295KB |
| **法宝增强** | Enhanced artifact/treasure system | 211KB |
| **魅魔系统** | Succubus system expansion | 25.6MB |
| **废柴少爷的逆天系统** | "Trash Young Lord's Cheat System" — distribute breakthrough materials to sect members, manage sect development | 3.12MB |

### 2.3 Utility / QoL Mods

| Mod | Author | Function |
|-----|--------|----------|
| **小地图任意传送** | 轻抚含羞草 | Double-click any icon on minimap to teleport |
| **便利性调优** | 轻抚含羞草 | Auto-learn herb/feng shui/equipment books, battle stats display, one-click warehouse deposit, free soul switching, expanded dungeon pickup range (500→5000) |
| **背包助手** (sigma) | sigmaplus | Quick sell/all sell, auto-convert to spirit stones, Jianmu storage management |
| **成长型功法体系** (sigma) | sigmaplus | XP-based skill system, skill upgrade/rename, quick learn, affix modification |
| **宗门管家** | 八荒大鬼 | Use all 3 artifacts (Shuangyu Pei + Haotian Eye + Lianyao Hu) simultaneously |
| **宗门自由存取灵石** | 轻抚含羞草 | Deposit/withdraw spirit stones from sect treasury |
| **自动修复存档** | 八荒大鬼 | Auto-fix corrupt save data from removed mods (invalid items, qi yun, tasks, map points) |
| **远程查看NPC时可交互** | 轻抚含羞草 | Interact with NPCs remotely without traveling to their location |
| **过月免打扰** | 青云 | Disable NPC monthly interactions via tavern UI |
| **副本扫荡** | — | Sweep dungeon for rewards (reduced vs manual) |
| **大鬼工具盒** | 八荒大鬼 | Free built-in modifier: F1 = 3x speed, F3 = full fire sword build |

### 2.4 Visual / Portrait Mods

| Mod | Description |
|-----|-------------|
| **好看小姐姐立绘** | Beautiful female character portraits (430MB) |
| **仙子堕凡尘 v2.7** | Fairy descending to mortal world portraits (111MB) |
| **古风NPC动态立绘** | Animated ancient Chinese style NPC portraits |
| **原版水墨风格静态立绘** | Original ink-wash style static portraits |
| **动物角头饰** | Animal horn headwear cosmetic (444KB) |
| **全新性感动态立绘** | New dynamic sexy portraits (24MB) |
| **绝美静态女生服装立绘** | Beautiful static female clothing portraits (100MB) |
| **发型美化** | Hairstyle beautification (13.9MB) |
| **白的立绘包** | "White" portrait pack (9.79MB) |
| **立绘整合大礼包** | Portrait mega-pack (202MB) |

---

## 3. Map System — "八荒地图" (Eight Desolations Map)

### 3.1 Map Structure

- **Grid-based open world map** — each tile is one day of travel on foot
- **Terrain types**: Plains (1 day), Mountains/Waterfalls (2-3 days), Snow, Desert, Swamp, Secret Realms, Ancient Ruins
- **Biome regions**: Each region (州/Province) has distinct geography
- **Fog of war**: Unexplored areas covered in black mist, revealed by exploration
- **Random generation**: Every new game generates a unique map layout
- **Named locations**: Mountain peaks and ranges have special names from Shan Hai Jing

### 3.2 Map Regions (Provinces)

The game world is divided into 5 major provinces, progressively unlocked:

| Province | Unlock | Content |
|----------|--------|---------|
| **愚村** (Fool's Village) | Start | Tutorial area |
| **永宁州** (Yongning Province) | Early game | First town, sects, basic cultivation |
| **华封州** (Huafeng Province) | After 筑基 (Foundation Building) | Mid-game content |
| **云陌州** (Yunmo Province) | After 具灵 (Spirit Gathering) | Late-game |
| **慕仙州/天元山** (Muxian Province / Tianyuan Mountain) | Endgame | Final story, 5 endings |

### 3.3 Map Features

- **Towns** contain: Bounty Board, Tavern, Market (坊市), Auction House, Lounge, Black Market, Workshop, Secret Realm, Langya Pavilion
- **Sects** (宗门) on map: Blue = Righteous, Red = Demonic territory
- **Teleportation Arrays** between towns (costs spirit stones)
- **Spirit Points** (灵气点): Randomly generated, boost cultivation speed, can be checked with Feng Shui skill for hidden treasures
- **Special Buildings** spawn randomly: Sword Tomb (剑冢), Ancient Ruins (远古遗址), Ten-Thousand-Year Tree Demon (万年树妖)
- **World Events**: Ancient Secret Realm openings, town auctions — NPCs gather near event locations
- **Monthly NPC AI updates**: NPCs move, cultivate, and interact autonomously each month
- **路标 (Waymarkers)**: Show nearby towns and sects
- **探灵符**: Reveal Spirit Spirit beasts and spirit energy locations (indirectly reveals town positions)

### 3.4 Map Navigation

- **Foot travel**: Base speed, affected by terrain
- **Flying Sword (飞剑)**: Purchased from market, increases step count
- **Mounts (坐骑)**: Significantly reduce travel time, some terrain requires specific mount stats
- **Teleportation Talismans**: Instant travel to known locations
- **Escape**: Can escape from random encounters to bypass terrain
- **Mod: 小地图任意传送**: Double-click any minimap icon to teleport directly

---

## 4. Combat System

### 4.1 Combat Basics

- **Action RPG combat** with real-time mechanics
- **Controls**: Left-click = Basic Attack, Right-click = Skill, Space = Dodge, R = Ultimate (大招)
- **Skill Slots**: 4 slots total:
  - 2x Martial Skills (武技/武灵技) — left-click and right-click bound
  - 1x Movement Skill (身法) — dash/teleport
  - 1x Ultimate Skill (绝技)
- **Domain (领域)**: Activated via separate key
- **Artifact Skills**: Shuangyu Pei (resurrection), Haotian Eye (vision), Lianyao Hu (beast capture) — each with dedicated keys

### 4.2 Skill System (功法)

**Skill Tiers** (by color): White → Green → Blue → Purple → Orange → Red (best)

**Skill Types**:
| Category | Variants |
|----------|----------|
| **Martial Arts (功法)** | Sword (剑), Blade (刀), Spear (枪), Fist (拳), Palm (掌), Finger (指) |
| **Elemental Roots (灵根)** | Fire (火), Water (水), Thunder (雷), Wind (风), Earth (土), Wood (木) |
| **Craft Skills (技艺)** | Alchemy (炼丹), Forging (炼器), Feng Shui (风水), Talisman Drawing (画符) |

**Skill Proficiency Levels**: 初学(70%) → 熟练(80%) → 小成(90%) → 大成(100%) → 圆满(110%) → 化境(120%)

**Equipment Loadout**:
- 3 Equipment Slots: Ring, Mount, Talisman
- 5 Combat Item Slots: Only pills and artifacts (usable in combat)

### 4.3 Combat Mods

| Mod | Features |
|-----|----------|
| **自动战斗** (Nuyoah) | Full auto: martial skills, ultimates, movement, domain, artifacts, beast summoning, auto-positioning, auto-advance rooms |
| **AI强化** | Enhanced enemy/companion AI |
| **灵活组队** | Flexible party management |
| **神将相随** | Divine general companions |
| **所有雷系功法增强** | All thunder skills buffed |
| **所有火系技能增强** | All fire skills buffed |
| **让枪法枪修跟上剑修强度** | Spear mastery rebalanced to match sword |
| **双持武法灵技** | Dual-wield two martial skills simultaneously |
| **自创剑武技三才剑** | Custom sword skill: Sancai Sword |
| **追踪武灵技** (勺子) | Auto-tracking projectiles (player only), fixes projectile count bugs |

### 4.4 Breakthrough System (境界突破)

**10 Major Realms** (each with 初期/中期/后期 = 30 stages total):

| # | Realm | Chinese |
|---|-------|---------|
| 1 | Qi Refining | 练气 |
| 2 | Foundation Building | 筑基 |
| 3 | Crystal Formation | 结晶 |
| 4 | Golden Core | 金丹 |
| 5 | Spirit Gathering | 具灵 |
| 6 | Nascent Soul | 元婴 |
| 7 | Spirit Transformation | 化神 |
| 8 | Dao Comprehension | 悟道 |
| 9 | Feather Ascension | 羽化 |
| 10 | Immortal Ascension | 登仙 |

**Breakthrough Types** (at Foundation Building):
- **人道筑基 (Human)**: No materials needed, 1 attribute option
- **地道筑基 (Earth)**: 1 elemental spirit + Earth Pill, 3 attribute options
- **天道筑基 (Heavenly)**: 6 different elemental spirits + Heaven Pill, 6 attribute options

**逆天改命 (Fate Defiance)**: Special talent system triggered at each breakthrough, providing unique passive abilities (e.g., 后羿射日, 悬壶御妖)

---

## 5. UI Features

### 5.1 Main Game UI

- **Overworld Map UI**:
  - Top-right: Quest tracker
  - Top-left: History/letter notifications
  - Minimap with zoom and drag (added in 2022 update)
  - Task pointer lines guide to objectives

- **Character Panel**:
  - Personal attributes: Lifespan, Mood, Health, Stamina, Spirit, Mind Power, Luck, Comprehension
  - Combat stats: ATK, DEF, Speed, Physical/Magical Immunity, Crit, Crit Resist, Move Speed
  - Skill qualifications (12 types)
  - Personality system (7 inner + 12 outer personality traits)
  - Reputation levels (7 tiers)
  - Charm levels: 仙姿 (Immortal) → 憎呕 (Disgusting)

- **Skill Interface** (press X):
  - Skill slots display
  - Growth system (from mod)
  - Affix viewing/modification

### 5.2 Town UI

Buildings accessible from town screen:
- Bounty Board (悬赏榜) — daily tasks, city lord tokens
- Tavern (酒馆) — gossip, quest information, buff drinks
- Market (坊市) — buy/sell items, pills, books
- Auction House (拍卖行) — not in all towns
- Workshop (工坊) — alchemy, forging
- Black Market (黑市) — rare items
- Secret Realm (秘境) — dungeon entrance
- Langya Pavilion (琅琊阁) — skill book exchange

### 5.3 Sect UI

- Sect Overview: Mission, rules, enemy sects, entry requirements
- Buildings: Hall, Scripture Library, Mission Hall, Healing Hall, Training Hall, Spirit Gathering Array, Alchemy Lab, Forging Hall, Talisman Room, Feng Shui Tower, Secret Realm
- Sect Rank: Outer Disciple → Inner Disciple → True Disciple → Elder → Grand Elder → Sect Leader
- Sect contributions for skill purchases, pills, promotions

### 5.4 Combat UI

- Health/Mana/Stamina bars
- Skill cooldown indicators
- Buff/Debuff display
- ESC menu: escape, pause, stats display
- **Mod: 战斗更多信息**: Shows attack count, damage dealt, dungeon time in ESC

### 5.5 UI Mods

| Mod | Author | Effect |
|-----|--------|--------|
| **界面美化** (无极) | 名取早耶香 | UI beautification: fortune destiny icons, realm icons |
| **UI美化-界面按钮** | Aiifba | Button styling overhaul |
| **UI美化-过场图片** | Aiifba | Cutscene image replacement |
| **重写八荒 任务栏UI** | 擎天 | Task bar UI and button beautification |
| **大鬼UI框架** | 八荒大鬼 | Foundation UI framework for other mods |
| **Fatury Framework** | Sorry | Full UI replacement capability (portraits, cutscenes, audio, video) |
| **修真聊天群** | SolidStateLight | In-game chat system with NPC interaction via chat window |

---

## 6. Game Progression System

### 6.1 Core Loop

```
Start in 愚村 → Explore map → Join sect → Learn skills → Cultivate →
Breakthrough → Defeat bosses → Discover secrets → Reach 登仙 (Immortal)
```

### 6.2 Progression Pillars

1. **Cultivation (修炼)**: Absorb spirit energy at spirit points, meditate, use pills
2. **Skill Acquisition (功法)**: Buy from sect/market, find in dungeons, steal from NPCs
3. **Breakthrough (突破)**: Gather materials (spirit energy, heavenly treasures), face tribulation
4. **Fate Defiance (逆天改命)**: Choose special talents at each breakthrough
5. **Social (社交)**: Dual cultivation, master-disciple, sworn siblings, marriage
6. **Sect Management (宗门)**: Rise through ranks, manage sect, collect tributes
7. **Story (剧情)**: Main quest through Tianyuan Mountain, 5 different endings

### 6.3 Difficulty Modes

| Mode | Unlock Requirement |
|------|--------------------|
| Normal (普通) | Default |
| Hard (困难) | Reach Golden Core in Normal |
| Extreme (极难) | Reach Nascent Soul in Hard |
| Hell (地狱) | Reach Spirit Transformation in Extreme |

### 6.4 Key Items

- **双鱼佩 (Shuangyu Pei)**: Resurrection artifact — collect life/death energy to charge, each death increases charge requirement
- **昊天眼 (Haotian Eye)**: Vision artifact — reveals hidden information
- **炼妖壶 (Lianyao Hu)**: Beast capture artifact — capture, assign as guard, send on missions, summon in battle

---

## 7. Visual Style & Art

### 7.1 Art Direction

- **Chinese ink-wash painting (水墨画) aesthetic** — the signature visual identity
- **Color palette**: Cool tones dominate — jade green bamboo, pale pink lotus, clear water, rugged mountain peaks
- **"Cold aesthetic" (性冷淡风格)**: Delicate, refined, minimalist color usage
- **Character portraits**: Large, detailed character illustrations — often praised as "好大" (impressive) in quality
- **连环画 (Lianhuanhua)**: Sequential art / picture-story style for narrative scenes
- **Environment**: Map and dungeon scenes feel like being inside a "画卷" (painting scroll)
- **Destructible environments**: Reed marshes destroyed by combat techniques, echoing Sekiro's芦苇场 aesthetic
- **2000+ unique encounter illustrations** in 1.0 release

### 7.2 Combat Visual Effects

- **Bullet hell (弹幕) style**: Enemies fill screen with projectile patterns
- **Damage number popups**: Large, flashy damage numbers
- **Skill particle effects**: Color-coded by element (fire = red/orange, water = blue, thunder = yellow, wind = green, earth = brown, wood = green)
- **Equipment glow**: Visual rarity indicators (white/green/blue/purple/orange/red)
- **Diablo-like feel**: Simplified gameplay with strong visual feedback — "嗑瓜子" (sunflower seed) addictive sensation

### 7.3 Mod Visual Enhancements

| Mod | Enhancement |
|-----|-------------|
| **4K立绘重制** (逍遥阁) | 4K portrait remaster with dynamic skeletal animation + particle effects |
| **山河社稷图 4K材质包** | 4K texture pack |
| **动态立绘** | Animated character portraits with breathing, blinking, hair movement |
| **视频立绘 (webm)** | Video-based animated portraits using webm format |
| **M框架 / SaiLL框架** | Advanced portrait animation frameworks |
| **双修视频DIY** | Custom dual-cultivation video sequences |
| **过场图替换** | Custom cutscene illustrations |
| **BGM替换** | Custom background music replacement |
| **音效替换** | Custom sound effect replacement |

### 7.4 Technical Art Details

- **Engine**: Unity with IL2CPP backend
- **Mod asset types**:
  - Static portraits (PNG)
  - Dynamic portraits (animated sprites via framework)
  - Video portraits (webm format)
  - Texture replacements (UI elements, cutscenes)
  - Audio replacements (BGM, SFX)
  - Asset bundles (custom 3D models via mods like "再现斗罗")

---

## 8. Modding Ecosystem

### 8.1 Mod Tools

| Tool | Purpose |
|------|---------|
| **Official Mod Creator** | Built-in in-game mod editor (v1.0 BETA), supports adventures, stories, NPC generation |
| **MelonLoader** | Unity mod loader for advanced code mods |
| **ILSpy / dnSpyEx** | .NET decompilers for mod development |
| **Unity Explorer** | In-game runtime inspection and hooking |
| **TaleOfImmortalTool** | CLI tool for packing/unpacking mods, encrypting/decrypting mod files |
| **Asset Bundle Extractor** | Modify Unity asset bundles |

### 8.2 Mod Creation Categories

1. **Mod Creator (GUI)**: In-game editor for adventures, destinies, NPC generation, world events
2. **ModExcel**: JSON/Excel data modification for game parameters
3. **ModCode**: C# code injection via MelonLoader for advanced functionality

### 8.3 Mod Distribution

- **Steam Workshop**: Primary distribution, 543+ mods
- **3DM Mod Site**: Chinese community hub, extensive collection
- **Nexus Mods**: 66+ mods
- **Bilibili**: Mod showcase videos, integration packs
- **GitHub**: Open-source mod projects (nozwock/tale-of-immortal-mods)

### 8.4 Integration Packs

Popular modpacks (60-66GB with game):
- 万古神话 + 八荒大乱斗 + 合欢宗 + 3000+ portraits
- 仙剑/遮天/诛仙 IP crossovers
- 200+ visual mods + functional mods + NPC beautification

---

## 9. Key Takeaways for Simulation Design

### What Makes 鬼谷八荒's Map & Combat Interesting

1. **Random generation creates emergent narratives** — no two playthroughs are identical
2. **Grid-based movement with terrain costs** creates strategic travel decisions
3. **Monthly AI ticks** make the world feel alive — NPCs cultivate, move, interact independently
4. **Bullet-hell combat** in a cultivation setting — satisfying visual feedback
5. **Deep character system** — personality, charm, reputation affect interactions
6. **Sect hierarchy** creates social progression alongside personal power
7. **Art style as identity** — the ink-wash painting aesthetic is inseparable from the game's appeal
8. **Mod ecosystem depth** — Fatury framework enables near-total customization of visuals, audio, and UI

### Relevant Systems for NT-WORLD Crawl Module

- **Random map generation** with terrain costs and fog of war
- **NPC autonomous behavior** (monthly AI simulation)
- **Event-driven world events** that cluster NPCs
- **Tiered progression gates** (province unlocking)
- **Social relationship web** (master, disciple, sworn sibling, rival, dual cultivation partner)
- **Skill qualification system** (12 martial arts + 6 elements + 4 craft skills)
- **Artifact management** (3 slot system with distinct combat functions)

---

*Report compiled from 10 web searches across 3DM, Steam, Nexus Mods, GamerSky, Bilibili, TapTap, and GitHub sources.*

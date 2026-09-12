# Stardew Valley — Complete Game Design Research
> Reference document for Consciousness Valley development.
> Source: Wiki, IGN, community guides, and design analysis (2026 research batch).

---

## 1. Core Loop — Farming Cycle, Daily Rhythm, Seasonal Progression

### Daily Cycle
- **20-hour in-game day**: 6am → 2am. Real-time ratio: 0.7 real seconds per in-game minute (~14 minutes per full day).
- **Wake → Plan → Act → Sleep**: Player wakes at 6am, plans activities, executes, and returns to bed before 2am. Passing out costs gold and energy.
- **Time pauses** in single-player during menus, cutscenes, dialogue, fishing reeling. Multiplayer does NOT pause.
- **Energy system**: Every tool use costs energy. Exhaustion at 0 energy forces 50% energy next morning. Sleep restores fully. Spa restores energy when standing in water.

### Seasonal Progression
- **4 seasons × 28 days = 112 days per year**. No year limit.
- **Crops die at season change** unless multi-seasonal (Corn: Summer+Fall). Fertilizer also expires on season change.
- **Farm disrepair**: Minor debris each season change, major debris Winter→Spring.
- **Each season has 2-4 festivals** — community milestones that break monotony and provide unique rewards.
- **Seasonal content rotation**: Different crops, fish, forage, seed shop inventory, and dialogue per season. Creates anticipation and FOMO-free pacing.

### Engagement Loops Created
| Loop | Mechanism | Compulsion Type |
|------|-----------|----------------|
| **Micro-loop** (min) | Water → harvest → sell | Immediate reward |
| **Daily loop** (day) | Morning plan → activity → evening return | Goal-setting + consequence |
| **Seasonal loop** (28d) | New crops, festivals, progression gates | Anticipation + novelty |
| **Yearly loop** (112d) | Community Center, relationships deepen | Long-term mastery |
| **Meta-loop** (infinite) | Perfection tracker, optimization | Completionist drive |

### Design Lesson
The day cycle creates a natural "time budget" that forces prioritization. Players must choose: farm more, mine deeper, socialize, fish, or explore. This constraint is the engine of engagement — every day is a meaningful choice, not a grind.

---

## 2. Skill System — 5 Skills, Leveling, Professions

### Skill Overview
| Skill | How to Level | XP Sources | Tool Proficiency |
|-------|-------------|------------|-----------------|
| **Farming** | Harvest crops, pet/milk/shear animals | Crop quality, animal care | Hoe, Watering Can |
| **Mining** | Break rocks (pickaxe/bombs/monsters) | Rock type determines XP | Pickaxe |
| **Foraging** | Gather forage items, chop trees | Item type, tree species | Axe |
| **Fishing** | Catch fish, use crab pots | Fish difficulty, perfect catches | Fishing Rod |
| **Combat** | Kill monsters | Monster difficulty | N/A (weapons use no energy) |

### Leveling Structure
- **10 levels per skill**. Each level grants:
  - Tool proficiency (+1 per level, reducing energy cost)
  - Crafting/cooking recipes
  - Passive HP bonuses (Combat: +5 HP per level)
- **Profession choices at Level 5 and Level 10** — binary branching specialization

### Complete Profession Tree

#### Farming
| Level 5 | Effect | Level 10 (from L5) | Effect |
|---------|--------|---------------------|--------|
| **Rancher** | Animal products +20% | Coopmaster | Befriend coop animals faster, incubation halved |
| | | Shepherd | Befriend barn animals faster, sheep wool faster |
| **Tiller** | Crops +10% (vegetables, flowers, non-foraged fruit) | Artisan | Artisan goods (wine, cheese, oil) +40% |
| | | Agriculturist | All crops grow 10% faster |

#### Mining
| Level 5 | Effect | Level 10 (from L5) | Effect |
|---------|--------|---------------------|--------|
| **Miner** | +1 ore per vein | Blacksmith | Metal bars +50% sell price |
| | | Prospector | Double coal chance |
| **Geologist** | Gems appear in pairs | Excavator | Double geode drop chance |
| | | Gemologist | Gems/minerals +30% |

#### Foraging
| Level 5 | Effect | Level 10 (from L5) | Effect |
|---------|--------|---------------------|--------|
| **Forester** | Trees yield +25% wood | Lumberjack | Common trees occasionally drop hardwood |
| | | Tapper | Syrups +25% |
| **Gatherer** | 20% chance double forage harvest | Botanist | Foraged items always iridium quality |
| | | Tracker | Forageable locations shown on map |

#### Fishing
| Level 5 | Effect | Level 10 (from L5) | Effect |
|---------|--------|---------------------|--------|
| **Fisher** | Fish +25% sell price | Angler | Fish +50% sell price |
| | | Pirate | Double treasure chance while fishing |
| **Trapper** | Crab pot resource cost reduced | Mariner | Crab pots never catch junk |
| | | Luremaster | Crab pots no longer need bait |

#### Combat
| Level 5 | Effect | Level 10 (from L5) | Effect |
|---------|--------|---------------------|--------|
| **Fighter** | +10% damage, +15 HP | Brute | +15% damage (multiplicative with Fighter) |
| | | Defender | +25 HP |
| **Scout** | +50% crit chance (multiplicative) | Acrobat | Special move cooldown halved |
| | | Desperado | Critical strikes deal 2x damage |

### Mastery System (Post Level 10)
- When any two skills reach level 10, the Mastery Cave unlocks
- Each skill can be "Mastered" for additional bonuses and advanced recipes
- Mastery is the true endgame skill progression

### Design Lessons
- **Binary choices at milestones (5/10)** create meaningful specialization without overwhelming complexity
- **Professions compound**: Artisan + Tiller = 54% more crop value (1.1 × 1.4). This creates build diversity.
- **Each skill has its own economy**: Farming → crops, Mining → ore/bars, Foraging → wood/syrup, Fishing → fish, Combat → monster loot. Players naturally specialize.
- **XP sources are intrinsically tied to activities** — you level by doing, not by grinding a separate meter.

---

## 3. NPC System — 30+ NPCs, Friendship, Hearts, Gifts, Events

### Friendship Point System
- **Each heart = 250 friendship points**
- **10 hearts max** for regular NPCs (14 for spouse)
- Marriage candidates capped at 8 hearts until given a Bouquet

### Ways to Earn Friendship
| Action | Points | Notes |
|--------|--------|-------|
| Daily talk | +20 | Must initiate conversation |
| Loved gift (normal day) | +80 | Quality multiplier applies |
| Liked gift (normal day) | +45 | |
| Neutral gift | +20 | |
| Disliked gift | -20 | |
| Hated gift | -40 | |
| **Birthday loved gift** | **+640** | 8× multiplier — massive spike |
| **Feast of Winter Star** | **+400** | 5× multiplier |
| Festival dance | +1 heart flat | Flower Dance |
| Movie (loved) | +50 | |
| Slingshot hit | -30 | Penalty |
| Garbage can rummage (seen) | -25 | Linus: +5 instead |
| Not talking daily | Decay | Varies by friendship level |

### Gift Limits
- **2 gifts per week** per NPC (resets Monday)
- **1 birthday gift** (8× multiplier — the single biggest friendship action)
- Quality matters: iridium quality loved gift on birthday = maximum possible single gift

### Universal Gifts
| Category | Items | Points |
|----------|-------|--------|
| **Universal Loves** | Prismatic Shard, Pearl, Rabbit's Foot, Golden Pumpkin, Magic Rock Candy | +80 |
| **Universal Likes** | All fruit tree fruit, all gems (except Prismatic), all vegetables (except hops/tea/wheat), Life Elixir, Maple Syrup | +45 |
| **Universal Hates** | All artifacts, all minerals (except gems), newspapers, garbage | -40 |

### Heart Events
- Triggered at specific heart levels (2, 4, 6, 8, 10 hearts)
- Cutscenes reveal backstory, affect dialogue, unlock recipes
- Some choices in events affect friendship positively or negatively
- **Marriage candidates**: 8 hearts → Bouquet available, 10 hearts → Mermaid Pendant proposal

### NPC Count & Types
- **34 total NPCs** (12 marriage candidates + 22 non-dateable)
- Each has: daily schedule, home, workplace, birthday, gift preferences, dialogue tree, heart events
- NPCs move between locations based on time, season, weather, and events
- **Dialogue evolves** with friendship level — same NPC says different things at 0 vs 5 vs 10 hearts

### Design Lessons
- **Birthday gifts create a "burst" moment** — players plan around birthdays for massive friendship gains
- **Gift limits prevent brute-force** — you must spread gifts across the week, forcing route planning
- **Universal gifts provide safety** but optimal play requires learning individual preferences
- **Heart events are the "payoff"** — they make friendship feel rewarding beyond numbers
- **NPC schedules create emergent gameplay** — players learn routines, intercept NPCs on their paths

---

## 4. World Structure — Pelican Town, Farm Types, Mines

### Pelican Town Layout
- **Central hub**: Town square, Community Center, Pierre's Store, Saloon
- **Connected regions**:
  - North → Mountains (Robin's Shop, Mines, Railroad, Quarry)
  - South → Beach (Willy's Fish Shop, fishing spots)
  - West → Cindersap Forest (Marnie's Ranch, Wizard's Tower, Traveling Cart, Secret Woods)
  - East → Bus Stop → Farm
- **Forage spawns** at 1.2/night (Spring/Summer), 0.9 (Fall), 0.7 (Winter)
- **Artifact spots** scattered for archaeological digging

### Farm Types (8 Maps)
| Map | Focus | Key Feature | Trade-off |
|-----|-------|-------------|-----------|
| **Standard** | Farming | Maximum tillable land | No special bonuses |
| **Riverland** | Fishing | Fish from your farm (town+forest fish pools) | Reduced farming space |
| **Forest** | Foraging | Renewable hardwood stumps, seasonal forage, unique weeds | Reduced farming space |
| **Hilltop** | Mining | Ore/geode nodes spawn daily (scales with Mining level) | Reduced farming space |
| **Wilderness** | Combat | Monsters spawn at night (scales with Combat level), Wilderness Golems | Reduced farming space |
| **Four Corners** | Multiplayer | 4 quadrants (forest/standard/lake/hilltop) | Split layout |
| **Beach** | Fishing+Foraging | Ocean fishing, supply crates on shore | Sand can't be watered by sprinklers |
| **Meadowlands** | Animal farming | Starts with Coop+2 chickens, chewy blue grass | Reduced crop space |

### The Mines (120 Floors)
- **Floor progression**: Every 5 floors = elevator checkpoint
- **Resource tiers**: Copper (floors 1-40), Iron (41-80), Gold (81-120)
- **Special floors**: Treasure rooms, monster floors, shrine floors
- **Floor 120**: Skull Key unlocks Skull Cavern
- **Danger in the Deep**: Post-game harder variant with new monster types

### Skull Cavern (Infinite)
- **No elevator** — each run starts from floor 1
- **No practical limit** (integer limit: 2,147,483,647 floors)
- **Iridium ore** increases dramatically with depth
- **Time moves slower**: 0.9 real seconds per in-game minute (vs 0.7 normal)
- **Key mechanics**: Staircases (craft or buy), bombs for rapid clearing, lucky days matter
- **Monsters scale**: Purple Slimes drop Iridium, Serpents are fast, Mummies need bomb finishing

### Design Lessons
- **Farm type selection** at game start creates replayability and specialization identity
- **Mines = risk/reward**: Deeper floors = better loot but more danger and time pressure
- **Skull Cavern = skill ceiling**: Infinite depth with no elevator creates "one more run" compulsion
- **World regions are gated by progression**: Bus repair (Community Center), boat repair (Ginger Island)

---

## 5. Economy — Shipping, Shops, Casino, Traveling Cart

### Shipping Bin
- **Primary sell mechanism**: Place items, receive gold after sleep
- **Delayed gratification**: Must wait until next morning for gold
- **Collection tracking**: Ships count toward Shipping Collection (100% perfection requirement)
- **Mini-Shipping Bin**: Portable 9-slot version from Special Orders

### Shop Ecosystem
| Shop | Owner | Hours | Specialty | Notes |
|------|-------|-------|-----------|-------|
| **Pierre's General Store** | Pierre | 9am-5pm (closed Wed without CC) | Seeds, supplies, buys crops | Primary seed source |
| **JojaMart** | Morris | 9am-11pm | Same items, higher prices (without membership) | Membership = CC alternative |
| **Adventurer's Guild** | Marlon | 2pm-10pm | Weapons, boots, rings | Monster eradication rewards |
| **Carpenter's Shop** | Robin | 9am-8pm | House upgrades, farm buildings | Tool upgrades |
| **Fish Shop** | Willy | Varies | Rods, bait, tackle | Crab pots |
| **Oasis** | Sandy | Desert | Seeds, decor | Access to Casino |
| **Traveling Cart** | Unknown | Fri+Sun in Cindersap | Random 10 items/day from huge pool | Rare items, Pufferfish |
| **Desert Trader** | Unknown | Daily | Item trades (no gold) | Rotating stock |
| **Stardrop Saloon** | Gus | 12pm-12am | Cooking recipes, food | rotates daily selection |

### Casino
- **Currency**: Qi Coins (10g = 1 Qi Coin, no conversion back)
- **Games**: Slots, CalicoJack (Blackscreen)
- **Exclusive items**: Rarecrow #3, Galaxy Soul, other prizes
- **Access**: Complete "The Mysterious Qi" quest chain

### Price Dynamics
- Pierre sells seeds cheaper than Joja (except Sunflower Seeds)
- Joja membership equalizes prices but kills Community Center
- **Artisan goods sell for dramatically more than raw crops**: Starfruit Wine = 2250g base vs Starfruit = 750g
- **Quality multiplier**: Iridium quality = 2× base price

### Design Lessons
- **Multiple sell channels** (shipping bin vs shops) create strategic choice (delayed gold vs immediate)
- **Traveling cart RNG** creates "treasure hunt" excitement — rare items appear randomly
- **Casino creates a gambling loop** — Qi Coins as separate currency prevents gold inflation
- **Pierre vs Joja** is a narrative choice with gameplay consequences (community vs convenience)

---

## 6. Crafting — Recipes, Artisan Machines, Automation

### Crafting Recipe Sources
- Skill level-ups (primary source)
- Community Center bundles
- Bookseller purchases
- Special Orders rewards
- **Total**: ~149 craftable items

### Artisan Equipment (Profit Multipliers)
| Machine | Input → Output | Time | Base Value Change | Unlock |
|---------|---------------|------|-------------------|--------|
| **Mayonnaise Machine** | Egg → Mayo | ~3h | Egg ×3-5 | Farming 2 |
| **Bee House** | Flowers nearby → Honey | 4 days | Flower-dependent | Farming 3 |
| **Preserves Jar** | Fruit → Jelly, Veg → Pickles | 2-3 days | Base ×2 + 50g | Farming 4 |
| **Cheese Press** | Milk → Cheese | ~3h | Milk ×2.3 | Farming 6 |
| **Loom** | Wool → Cloth | 4h | Wool ×1.4 | Farming 7 |
| **Oil Maker** | Truffle → Truffle Oil | 6h | Truffle ×2+ | Farming 8 |
| **Keg** | Fruit → Wine, Veg → Juice | 7-14 days | Base ×3 | Farming 8 |
| **Cask** | Wine/Cheese → Aged | Months | Up to 2× (iridium) | Cellar upgrade |
| **Fish Smoker** | Fish + Coal → Smoked Fish | ~1h | Fish ×2 | Fishing |
| **Dehydrator** | Fruit → Dried Fruit | Varies | Concentrated value | Farming |

### Key Crafting Machines
| Machine | Purpose | Unlock |
|---------|---------|--------|
| **Furnace** | Ore → Bars | Intro quest |
| **Crystalarium** | Gem duplication (infinite) | Mining 9 |
| **Seed Maker** | Crop → Seeds (variable count) | Farming 9 |
| **Recycling Machine** | Trash → Resources | Foraging 4 |
| **Lightning Rod** | Storms → Battery Packs | Foraging 6 |
| **Worm Bin** | Produces bait over time | Fishing 8 |
| **Slime Egg-Press** | Slime → Egg | Combat 6 |
| **Geode Crusher** | Opens geodes automatically | Mining level |

### Scarecrow System
- **Standard Scarecrow**: 8-tile radius (17×17 area minus corners = 249 tiles)
- **Deluxe Scarecrow**: 16-tile radius (888 tiles) — unlocked after collecting all 8 Rarecrows
- **8 Rarecrows**: Festival purchases, casino, museum donations
- Crows eat crops when >15 growing and no scarecrow in range

### Design Lessons
- **Artisan machines create a "processing chain"**: Raw crop → Keg → Cask = 3×→6× value
- **Machine time creates backlog**: You need many machines to process a large farm, driving expansion
- **Crafting is gated by skill progression** — each level unlocks new recipes, tying crafting to the skill loop
- **Scarecrows create spatial puzzles** — optimal placement for coverage drives farm design

---

## 7. Farming — Crops, Regrowth, Quality, Scarecrows

### Crop Statistics
- **60+ crop types** across 4 seasons (plus multi-seasonal: Corn, Ancient Fruit)
- **Quality tiers**: Normal → Silver → Gold → Iridium
- **Quality sources**: Farming skill level + Fertilizer + Seed Maker (random)
- **Giant Crops**: Cauliflower, Melons, Pumpkins in 3×3 grid (1% chance per day after maturity)

### Regrowth Mechanics
| Crop | Growth | Regrowth | Seasons | Gold/Day |
|------|--------|----------|---------|----------|
| Blueberry | 13 days | 4 days | Summer | 20.8g |
| Cranberry | 14 days | 4 days | Fall | ~7.4g |
| Tomato | 11 days | 4 days | Summer | ~1.9-7.4g |
| Ancient Fruit | 28 days | 7 days | All (Greenhouse) | High |
| Strawberry | 8 days | 4 days | Spring | High |

### Crop Processing Chains
```
Crop → [Seed Maker] → Seeds (replanting)
Crop → [Preserves Jar] → Jelly/Pickles (Base ×2 + 50g)
Crop → [Keg] → Wine/Juice (Base ×3)
Wine → [Cask] → Aged Wine (Base ×3 ×2 = ×6 at iridium)
```

### Fertilizer Types
| Fertilizer | Effect | Cost |
|-----------|--------|------|
| Basic Fertilizer | +1 quality tier chance | 2 fiber |
| Quality Fertilizer | +2 quality tier chance | Fish + sap |
| Deluxe Fertilizer | Guarantees gold+, enables iridium | 1 quartz, 1 fire quartz, 1 cinder shard |
| Speed-Gro | -1 day growth time | Honey + sap |
| Deluxe Speed-Gro | -3 days growth time | Oak resin + coral + 3 ancient fossil |
| Retaining Soil | Chance to skip watering | Clay + fiber |

### Sprinkler System
| Sprinkler | Tiles Watered | Unlock |
|-----------|--------------|--------|
| Sprinkler | 4 (cross pattern) | Farming 2 |
| Quality Sprinkler | 8 (3×3 minus center) | Farming 6 |
| Iridium Sprinkler | 24 (5×5 minus center) | Farming 9 |

### Animal Farming
- **Chickens** → Eggs → Mayo (daily)
- **Ducks** → Duck Eggs/Duck Feathers
- **Rabbits** → Wool/Rabbit's Foot
- **Cows** → Milk → Cheese
- **Goats** → Goat Milk → Goat Cheese
- **Pigs** → Truffles (outdoor, weather-dependent) → Truffle Oil (highest profit animal chain)
- **Sheep** → Wool → Cloth
- **Happiness matters**: Pet, feed, and let outside for better products

### Design Lessons
- **Regrowth crops create passive income** — plant once, harvest repeatedly
- **Quality system creates skill expression** — higher skill = better quality = more gold
- **Processing chains multiply value** — the core economic loop is grow → process → sell
- **Sprinklers automate watering** — progression from manual → sprinklers = time liberation
- **Pigs are the ultimate animal** — truffles → truffle oil is the highest-value animal product chain

---

## 8. Combat — Monsters, Weapons, Rings, Professions

### Weapon Types
| Type | Damage | Speed | Special Attack | Playstyle |
|------|--------|-------|----------------|-----------|
| **Sword** | Medium | Medium | Block/Parry (damage reduction) | Balanced |
| **Club** | High | Slow | Ground slam (AoE knockback) | Crowd control |
| **Dagger** | Low per hit | Fast | Rapid 4-hit combo | DPS burst |
| **Slingshot** | Ranged | Varies | Ammo-dependent | Ranged safety |

### Weapon Progression Tiers
1. **Early**: Rusty Sword → Wooden Blade → Bone Sword
2. **Mid**: Steel Falchion → Obsidian Edge → Lava Katana
3. **Late**: Galaxy Sword → Infinity Blade (with 3 Galaxy Souls + Cinder Shards)
4. **Endgame**: Dragontooth weapons (Volcano Dungeon)

### Ring System (2 Equippable Slots)
| Ring | Effect | Source |
|------|--------|--------|
| **Iridium Band** | +10% damage, +5 magnetism, +10% attack (glows) | Combat 9 |
| **Burglar's Ring** | Monsters drop more loot | Kill 1000 Dust Spirits |
| **Napalm Ring** | Enemies explode on death | Kill 100 Rock Crabs |
| **Vampire Ring** | +2 HP on kill | Combat 7 |
| **Warrior Ring** | Chance for "Warrior Energy" buff on kill | Combat 4 |
| **Crabshell Ring** | +5 Defense | Kill 60 Rock Crabs |
| **Phoenix Ring** | Revive once per level at 50% HP | Kill 200 Bats |
| **Lucky Ring** | +1 Luck | Found in mines/Skull Cavern |

### Ring Combinations (2 rings can be combined at Forge)
- **Iridium Band + Burglar's Ring** = Maximum loot + damage
- **Iridium Band + Vampire Ring** = Sustain build
- **Lucky Ring × 2** = Maximum luck for Skull Cavern runs

### Monster Types by Location
| Location | Key Monsters | Drops |
|----------|-------------|-------|
| **Mines 1-39** | Green Slime, Bug, Bat, Cave Bat | Bug Meat, Bat Wing, Slime |
| **Mines 40-79** | Frost Bat, Dust Spirit, Skeleton | Frozen Tear, Coal, Bone Fragment |
| **Mines 80-119** | Lava Bat, Fire Slime, Metal Head | Fire Quartz, Solar Essence |
| **Skull Cavern** | Purple Slime, Serpent, Mummy, Iridium Bat | Iridium Ore, Iridium Bar, Prismatic Shard |
| **Volcano Dungeon** | Hot Head, Magma Sprite, Tiger Slime | Cinder Shard, Dragon Tooth |
| **Farm** | Scales with Combat level | Wilderness Golem, Slimes |

### Monster Eradication Goals (Adventurer's Guild)
| Monster | Count | Reward |
|---------|-------|--------|
| Slimes | 1,000 | Slime Charmer Ring (immune to slime debuff) |
| Bats | 200 | Vampire Ring |
| Skeletons | 50 | Skull Ring |
| Rock Crabs | 60 | Crabshell Ring |
| Dust Spirits | 500 | Burglar's Ring |
| Mummies | 100 | Napalm Ring |
| Void Spirits | 150 | Savage Ring (+2 Speed on kill) |

### Damage Formula
```
Base Damage = random(weapon.minDamage, weapon.maxDamage)
Final Damage = Base × (1 + ruby_forge × 10%) × (1 + ring_bonus%)
Crit Check = weapon_crit × 1.5 (Scout) × (1 + aquamarine_rings × 10%) × (1 + blessing_of_fangs)
Crit Damage = Final × crit_multiplier × (2 if Desperado)
```

### Design Lessons
- **Weapon variety creates build identity** — Sword (balanced), Club (AoE), Dagger (burst), Slingshot (ranged)
- **Ring combinations are the "build"** — 2-ring slots + forge combination = hundreds of viable setups
- **Monster eradication goals** create long-term combat progression beyond simple leveling
- **Bombs in Skull Cavern** create a strategic layer — bomb placement for speed vs safety
- **Iridium is the bottleneck** — Skull Cavern depth = Iridium = endgame progression

---

## 9. Social — Marriage, Children, Special Orders, Qi Beans

### Marriage System
- **12 marriage candidates** (6 bachelors + 6 bachelorettes, 1.7 adds Clint + Sandy = 14)
- **Progression**: 8 hearts → Bouquet → 10 hearts → Mermaid Pendant (5,000g) → Wedding (3 days later)
- **Prerequisites**: First house upgrade + Old Mariner appears on rainy days at beach
- **Spouse benefits**: Farm help (watering, feeding, breakfast), unique dialogue, 14-heart max
- **Jealousy**: Spouse can become jealous if you gift other marriage candidates
- **Roommate option**: Krobus as non-romantic roommate

### Children
- **Up to 2 children** (1 boy, 1 girl) — requires 2nd house upgrade (nursery)
- **1/20 chance** per night spouse asks after sleep
- Children never grow past toddler stage
- **Removal**: Witch's Hut → turn children into doves (costs Prismatic Shard)

### Special Orders System
- **Unlocked Fall 2, Year 1** — Special Orders board in front of Lewis's house
- **More complex** than Help Wanted quests, time-limited, cannot be cancelled
- **Some repeatable**, others one-time
- **Examples**:
  - Crop Order: Ship 100 of a specific crop (28 days) → Gold + Mini Shipping Bin
  - Cave Patrol: Slay 50 bats/dust spirits (7 days) → Gold + friendship
  - Community Cleanup: Fish out 20 trash items (7 days) → Gold + Fiber Seeds recipe

### Qi's Special Orders (Ginger Island)
- **Found in Qi's Walnut Room** — unlocked at 100 Golden Walnuts
- **Rewards**: Qi Gems (currency for exclusive items)
- **Examples**:
  - Qi's Crop: Ship 500 Qi Fruit (28 days) → 100 Qi Gems
  - Let's Play A Game: Score 50,000 in Junimo Kart → Qi Gems
  - Extended Family: Catch 10 legendary fish variants → Qi Gems
  - Predatory: Catch specific fish → Qi Gems

### Qi Bean System
- **Dropped by**: Geodes (7.5%), Artifact Spots, Volcano chests, Fishing, Mining crates
- **Grows into Qi Fruit** → Ship 500 for Qi's Crop quest
- **Seed Maker**: Qi Fruit → more Qi Beans (chain propagation)
- **Despawn**: All Qi Beans/Fruit despawn when quest ends
- **Winter growable**: Can be planted on farm during winter

### Design Lessons
- **Marriage is a major milestone** — requires relationship investment + economic investment (house upgrades)
- **Special Orders create weekly objectives** — new orders on Monday create a "fresh week" feeling
- **Qi's orders are the endgame quest system** — harder, with exclusive currency rewards
- **Qi Beans create a collection/scaling challenge** — find → grow → propagate → ship 500
- **Time-limited quests create urgency** — must plan and execute within the window

---

## 10. Endgame — Perfection, Golden Clock, Obelisks, Ginger Island

### Perfection Tracker (Qi's Walnut Room)
The endgame "100%" checklist visible in Qi's Walnut Room:

| Category | Requirement | Weight |
|----------|-------------|--------|
| **Produce & Forage Shipped** | Ship 1 of every item (154 items) | 15% |
| **Obelisks on Farm** | Build all 4 obelisks | 4% |
| **Golden Clock** | Build the Golden Clock | 10% |
| **Monster Slayer Hero** | Complete all 12 eradication goals | 10% |
| **Great Friends** | Max hearts with ALL villagers (81 hearts total) | 10% |
| **Crafting Recipes** | Craft all 149 items | 10% |
| **Fish Caught** | Catch all 72 fish | 10% |
| **Golden Walnuts** | Find all 130 on Ginger Island | 5% |
| **Farmer Level** | All 5 skills to level 10 | 5% |
| **Stardrops Found** | Find all 7 Stardrops | 10% |
| **Cooking Recipes** | Cook all 81 recipes | 10% |

### Golden Clock
- **Cost**: 10,000,000g
- **Effect**: Prevents debris on farm, fences never decay
- **Unlocks**: After "Goblin Problem" quest (Wizard)
- **Required for Perfection**: 10% of total score

### Obelisks (Fast Travel)
| Obelisk | Cost | Materials | Destination |
|---------|------|-----------|-------------|
| **Earth Obelisk** | 500,000g | 10 Iridium Bars, 10 Earth Crystals | Mountain area |
| **Water Obelisk** | 500,000g | 5 Iridium Bars, 10 Clams, 10 Coral | Beach |
| **Desert Obelisk** | 1,000,000g | 20 Iridium Bars, 10 Coconuts, 10 Cactus Fruit | Calico Desert |
| **Island Obelisk** | 1,000,000g | 10 Iridium Bars, 10 Dragon Teeth, 10 Bananas | Ginger Island |

### Ginger Island
- **Unlock**: Repair boat behind Fish Shop (after Community Center)
- **130 Golden Walnuts** scattered across the island
- **Key areas**:
  - **West**: Farm, Qi's Walnut Room, Island Trader
  - **North**: Dig Site, Leo's house
  - **South**: Beach Resort (NPCs visit), Pirate Cove
  - **Volcano Dungeon**: Endgame combat + Dragon Teeth
- **Island Farm**: Year-round tropical farming (no seasons)
- **Beach Resort**: NPCs visit, unlocked with 20 Golden Walnuts

### Perfection Waivers
- **Fizz** (Joja employee) offers waivers at 500,000g each
- **Buys out** specific perfection requirements (e.g., Golden Clock, Obelisks)
- **Cheaper than building**: 14 waivers = 7,000,000g vs 13,000,000g for buildings

### Summit Cutscene
- **Unlocked at 100% Perfection**
- Special ending cutscene at the Summit
- **Statue of True Perfection**: Unlocked, gives Prismatic Shard daily

### Design Lessons
- **Perfection tracker is the ultimate completionist loop** — 10 categories, each requiring mastery
- **Obelisks solve late-game travel fatigue** — teleportation as a reward for massive investment
- **Golden Clock is the ultimate luxury** — 10M gold = months of optimized farming
- **Ginger Island is the "second game"** — new area, new mechanics, new currency, new challenges
- **Qi's Walnut Room creates endgame quest loops** — repeatable orders with exclusive rewards
- **100% perfection = narrative conclusion** (Summit cutscene) + ongoing play (Statue of True Perfection)

---

## Cross-System Analysis: How Systems Interconnect

### The Economy Web
```
FARMING → Crops → Kegs/Preserves → Wine/Jelly → Shipping Bin → Gold
    ↓                                                              ↓
ANIMALS → Eggs/Milk/Wool/Truffles → Mayo/Cheese/Cloth/Oil → Gold
    ↓                                                              ↓
MINING → Ore → Furnace → Bars → Craft Machines → Gold
    ↓                                                              ↓
FISHING → Fish → Fish Smoker → Smoked Fish → Gold
    ↓                                                              ↓
FORAGING → Wood/Stone → Crafting → Machines → Gold
    ↓                                                              ↓
COMBAT → Monster Loot → Craft Items/Rings → Better Mining → More Gold
```

### Progression Gates
| Gate | Requires | Unlocks |
|------|----------|---------|
| Mines open | Day 5 | Combat, ore, deeper progression |
| Minecart repair | CC bundles or Joja | Fast travel Mountains↔Town |
| Bus repair | Vault bundles (42,500g) or Joja | Desert, Skull Cavern |
| Greenhouse | CC or Joja | Year-round farming |
| Boat repair | CC complete | Ginger Island |
| Qi's Walnut Room | 100 Golden Walnuts | Endgame quests, Perfection tracker |
| Wizard's Shop | Dark Talisman + Goblin Problem | Obelisks, Golden Clock |
| House Upgrade 2 | 50,000g + 150 Hardwood | Marriage, children |

### Skill Interconnections
| Skill | Benefits Other Skills |
|-------|----------------------|
| **Farming** | Provides food for Mining/Combat, Kegs for economy |
| **Mining** | Provides bars for Crafting (all skills), bombs for Mining |
| **Foraging** | Provides wood/stone for buildings, syrups for Kegs |
| **Fishing** | Provides fish for food, gift items, Community Center |
| **Combat** | Provides monster loot for crafting, Iridium for tools |

---

## Key Design Principles (What Makes Stardew Valley Addictive)

### 1. **Meaningful Time Budget**
Every in-game minute has opportunity cost. Players must choose between activities, creating natural prioritization and replayability.

### 2. **Layered Progression**
- Skill levels → recipes → machines → automation
- Money → buildings → more capacity → more money
- Friendship → heart events → marriage → children

### 3. **Delayed Gratification with Immediate Feedback**
- Crops take days to grow (delayed) but each day shows progress (immediate)
- Shipping bin pays next morning (delayed) but gold counter updates (immediate)

### 4. **Multiple Viable Paths**
No "wrong" way to play. Fisher, farmer, miner, fighter, socialite — all viable. This prevents min-max anxiety.

### 5. **Seasonal Content Rotation**
28-day seasons create natural pacing. Players always have something to anticipate while appreciating the current moment.

### 6. **Interconnected Systems**
No system exists in isolation. Farming feeds crafting, mining feeds building, fishing feeds community, combat feeds mining. This creates emergent complexity from simple rules.

### 7. **The Farm as Canvas**
The farm is both a gameplay space and an expression of identity. Decorative elements, layout choices, and optimization all make the farm "yours."

### 8. **Narrative Through Mechanics**
The Community Center vs Joja choice, the valley's mysteries, NPC backstories — all delivered through gameplay, not cutscenes.

### 9. **The "One More Day" Compulsion**
End-of-day routine (sleep → save → morning) creates a natural save point that invites "just one more day." This is the most powerful retention mechanic in the game.

### 10. **Perfection as Optional North Star**
100% perfection exists for completionists but is never required. The game respects player time while rewarding mastery.

---

## Summary Stats

| Metric | Value |
|--------|-------|
| Total crops | 60+ types |
| Total fish | 72+ (plus legendary variants) |
| Total NPCs | 34 (14 marriage candidates in 1.7) |
| Total crafting recipes | 149 |
| Total cooking recipes | 81 |
| Total monster types | 40+ |
| Total weapon types | 60+ (swords, clubs, daggers, slingshots) |
| Total ring types | 25+ |
| Total farm maps | 8 |
| Total seasons | 4 × 28 days = 112 days/year |
| Perfection categories | 11 |
| Golden Walnuts | 130 |
| Mine floors | 120 + infinite Skull Cavern |
| Game length to "complete" | 100-200+ hours |
| Game length to 100% perfection | 400-800+ hours |

---

*Document compiled for Consciousness Valley reference. All data sourced from Stardew Valley Wiki, IGN, community guides, and design analysis (September 2026).*

# RPG Game Systems: Dialogue, Quest, NPC, Event & Cutscene

Comprehensive reference for implementing RPG game subsystems in NeoTrix.

---

## 1. Dialogue System

### 1.1 Dialogue Tree Structure

A dialogue tree is a directed graph where:

- **Nodes** = A single unit of dialogue (speaker line + text)
- **Edges** = Player choices or automatic transitions connecting nodes
- **Entry Node** = Root/start of conversation (usually NPC greeting)
- **Exit Node** = Terminal node (conversation ends)

**Node anatomy:**
```
DialogueNode {
    id: string             // unique identifier
    speaker: string        // NPC name or "player"
    text: string           // dialogue content
    emotion: string        // expression/sprite variant (idle, happy, angry, sad)
    portrait: string       // portrait asset path
    choices: Choice[]      // player options (empty = auto-advance)
    next: string | null    // next node ID (null = end conversation)
    conditions: Condition[] // visibility/availability conditions
    actions: Action[]      // side effects on enter/exit
}
```

**Choice anatomy:**
```
Choice {
    id: string
    text: string              // display text
    next_node: string         // target node ID
    conditions: Condition[]   // when this choice is shown
    requires: Requirement[]   // skill checks, item checks
    mood_delta: number        // relationship impact (+/-)
    set_flags: Flag[]         // flags to set on selection
}
```

### 1.2 Dialogue States

```
IDLE ──(interact)──► SPEAKING ──(has choices)──► CHOOSING ──(select)──► BRANCH
  ▲                        │                        │                    │
  │                        ▼                        ▼                    │
  │                   WAITING_FOR_INPUT        WAITING_FOR_INPUT        │
  │                        │                        │                    │
  │                        ▼                        ▼                    │
  └────(end/farewell)──── SPEAKING ◄──────────── BRANCH ◄───────────────┘
```

**State transitions:**
- `IDLE` → `SPEAKING`: Player interacts with NPC
- `SPEAKING` → `CHOOSING`: Node has player choices
- `SPEAKING` → `IDLE`: No choices, `next` is null (conversation ends)
- `CHOOSING` → `BRANCH`: Player selects choice
- `BRANCH` → `SPEAKING`: Follow edge to next node

### 1.3 Portrait & Expression System

**Expression map:**
| Expression | Usage |
|-----------|-------|
| `neutral` | Default, idle, information |
| `happy` | Agreement, reward, favorability high |
| `angry` | Disagreement, threat, favorability low |
| `sad` | Loss, regret, bad news |
| `surprised` | Shock, unexpected events |
| `thinking` | Processing, considering |
| `fear` | Threatened, scared |
| `disgust` | Rejection, revulsion |
| `trust` | Alliance, agreement |
| `anticipation` | Eager, expectant |

**Implementation pattern:**
```
portrait_system:
    npc_id: "merchant_01"
    sprites:
        neutral: "sprites/merchant_neutral.png"
        happy: "sprites/merchant_happy.png"
        angry: "sprites/merchant_angry.png"
    current_expression: "neutral"
    transition_animation: "fade"  # or "snap", "slide"
```

### 1.4 Variable Substitution

Dialogue text supports runtime variable interpolation:

```
Syntax: {variable_name} or ${expression}

Examples:
  "Welcome, {player_name}!"                    → "Welcome, Alice!"
  "You have {inventory.gold} gold."            → "You have 150 gold."
  "Quest status: {quest.main_slay_dragon}"     → "Quest status: Active"
  "Reputation with {faction_name}: {rep}"      → "Reputation with Guards: Friendly"
  "Day {game_time.day}, Hour {game_time.hour}" → "Day 14, Hour 8"
```

**Available variables:**
- `player.*` — name, class, level, stats
- `inventory.*` — gold, items, equipment
- `quest.*` — quest states (active/complete/failed)
- `reputation.*` — faction relationship values
- `game_time.*` — day, hour, minute, season
- `npc.*` — current NPC's state, favorability
- `flags.*` — custom game flags
- `random(min, max)` — random integer function

### 1.5 Conditions

Conditions control whether a dialogue node, choice, or branch is available:

| Condition Type | Syntax | Example |
|---------------|--------|---------|
| Flag check | `flag(name) == true/false` | `flag("knows_secret") == true` |
| Item check | `has(item, count)` | `has("magic_key", 1)` |
| Quest state | `quest(id) == state` | `quest("slay_dragon") == "complete"` |
| Reputation | `rep(faction) >= value` | `rep("thieves_guild") >= 50` |
| Stat check | `stat(name) >= value` | `stat("charisma") >= 10` |
| Level check | `level >= value` | `level >= 5` |
| Time check | `time.hour >= value` | `time.hour >= 18` |
| Day check | `time.day >= value` | `time.day >= 7` |
| AND | `a AND b` | `has("key") AND quest("dungeon") == "active"` |
| OR | `a OR b` | `flag("a") OR flag("b")` |
| NOT | `NOT a` | `NOT quest("betrayed") == "complete"` |

**Scripted dialogue example (Narrat-style):**
```yaml
greeting:
  if $data.player.evil:
    talk shopkeeper angry "Get away from me, monster!"
  else:
    talk shopkeeper neutral "Welcome! Would you like to buy a potion?"
    choice:
      "Buy a health potion (10 gold)":
        if $data.inventory.gold >= 10:
          talk shopkeeper happy "Here you go!"
          set data.inventory.gold -= 10
          set data.inventory.health_potion += 1
        else:
          talk shopkeeper sad "You can't afford that."
      "Never mind":
        talk shopkeeper neutral "Come back anytime."
```

---

## 2. Quest System

### 2.1 Quest Types

| Type | Description | Example |
|------|-------------|---------|
| **Main** | Core story progression, required | "Defeat the Dark Lord" |
| **Side** | Optional, world-building | "Find the missing cat" |
| **Chain** | Multi-part sequential quests | "Bandit Investigation" (3 parts) |
| **Daily** | Reset every 24h, modest rewards | "Gather 10 herbs" |
| **Weekly** | Reset every 7 days, better rewards | "Complete 5 dungeon runs" |
| **Hidden** | Discovered through exploration | "Find the ancient artifact" |
| **Dynamic** | Procedurally generated | "Clear the nearby cave" |
| **Tutorial** | Teaches game mechanics | "Open your inventory" |
| **Faction** | Tied to faction reputation | "Prove your worth to the Mages" |
| **Event** | Time-limited, seasonal | "Halloween: Find 5 pumpkins" |

### 2.2 Quest States

```
                    ┌──────────────┐
                    │   UNLOCKED   │ ← prerequisites met, available to accept
                    └──────┬───────┘
                           │ (player accepts)
                    ┌──────▼───────┐
         ┌────────►│    ACTIVE    │◄────────┐
         │         └──────┬───────┘         │
         │                │                 │
         │    (complete    │    (fail        │
         │     condition)  │    condition)   │
         │                ▼                 │
         │         ┌──────────────┐         │
         │         │   COMPLETE   │         │
         │         └──────────────┘         │
         │                                  │
         │         ┌──────────────┐         │
         └────────►│    FAILED    │─────────┘
                   └──────────────┘

  Also: AVAILABLE (quest giver offers), REJECTED (player declined),
        CANCELED (player abandoned mid-quest)
```

**Full state machine:**
| State | Can transition to | Notes |
|-------|-------------------|-------|
| `UNAVAILABLE` | `AVAILABLE` | Prerequisites not yet met |
| `AVAILABLE` | `ACTIVE`, `REJECTED` | Offered to player |
| `ACTIVE` | `COMPLETE`, `FAILED`, `CANCELED` | In progress |
| `COMPLETE` | (terminal) | Success |
| `FAILED` | `AVAILABLE` (retry) | Failure, may retry |
| `REJECTED` | `AVAILABLE` | Player declined |
| `CANCELED` | `AVAILABLE` | Player abandoned |

### 2.3 Objective Types

| Objective | Description | Tracking |
|-----------|-------------|----------|
| **Kill** | Defeat N enemies of type | `enemies_killed / total` |
| **Collect** | Gather N items | `items_collected / total` |
| **Talk** | Speak to specific NPC | `npc_id` visited |
| **GoTo** | Reach location | `distance_to_target` |
| **Escort** | Guide NPC to destination | NPC alive + at destination |
| **Interact** | Use object/trigger | Object activated |
| **Deliver** | Bring item to NPC | Item delivered to target |
| **Defend** | Protect location/NPC | Timer or health check |
| **Explore** | Discover N locations | `locations_discovered / total` |
| **Craft** | Create N items | `items_crafted / total` |
| **EscortSurvive** | Keep NPC alive during travel | NPC HP > 0 |
| **BossKill** | Defeat named/boss enemy | Boss HP <= 0 |
| **Timed** | Complete within time limit | `time_remaining` |

**Objective definition:**
```json
{
  "objective_id": "kill_wolves",
  "type": "kill",
  "target": "wolf",
  "required": 5,
  "current": 0,
  "description": "Slay 5 wolves attacking the farm",
  "optional": false,
  "hidden": false,
  "auto_track": true,
  "markers": [{"type": "compass", "location": "forest_wolf_den"}]
}
```

### 2.4 Quest Journal / Log

**Journal structure:**
```
QuestJournal {
    active_quests: Quest[]
    completed_quests: Quest[]
    failed_quests: Quest[]
    available_quests: Quest[]
    quest_log: LogEntry[]        // chronological history
    max_active: number           // cap (e.g., 20)
    sort_order: enum             // manual, type, date, priority
    filter: QuestFilter          // by type, status, area
}
```

**LogEntry:**
```
LogEntry {
    quest_id: string
    timestamp: GameTime
    entry_type: "new" | "update" | "complete" | "fail"
    message: string              // human-readable update
    objectives_changed: string[] // which objectives updated
}
```

### 2.5 Reward System

**Reward types:**
| Reward | Description |
|--------|-------------|
| `experience` | XP points |
| `gold` | Currency |
| `item` | Specific item(s) |
| `reputation` | Faction reputation change |
| `unlock` | Unlock new quest/area/feature |
| `skill_point` | Skill/trait point |
| `title` | Player title |
| `ability` | New ability/spell |
| `map_marker` | Reveal location on map |
| `discount` | Shop price reduction |

**Quest reward definition:**
```json
{
  "rewards": {
    "base": {
      "experience": 500,
      "gold": 100,
      "reputation": [{"faction": "village", "delta": 10}]
    },
    "bonus": {
      "condition": "objectives.all_optional_complete",
      "gold": 50,
      "item": {"id": "rare_sword", "quantity": 1}
    },
    "choice": {
      "prompt": "Choose your reward:",
      "options": [
        {"text": "Enchanted Shield", "item": "enchanted_shield"},
        {"text": "Magic Amulet", "item": "magic_amulet"},
        {"text": "200 Gold", "gold": 200}
      ]
    }
  }
}
```

### 2.6 Quest Design Patterns (Smith et al., 2011)

**Arrowhead Questing:** Chain of quests narrowing from broad to specific objectives.

**Contextualizing Quests:** Indirectly introduce player to game systems (e.g., scanning Keepers introduces the fast-travel system in Mass Effect).

**Quest Structure Hierarchy:**
```
Superstructure (game-wide narrative arc)
  └── Structure (quest chain / arc)
        └── Objective (individual task)
              └── Action (player behavior: kill, collect, talk)
```

---

## 3. NPC System

### 3.1 NPC Finite State Machine

```
                    ┌─────────┐
            ┌──────┤  IDLE   ├──────┐
            │      └────┬────┘      │
            │           │           │
    (time到了)    (player    (player
            │      nearby)   spotted)
            │           │           │
     ┌──────▼──────┐   │     ┌─────▼─────┐
     │   PATROL    │   │     │   CHASE   │
     └──────┬──────┘   │     └─────┬─────┘
            │           │           │
     (reach point)      │     (in range)
            │           │           │
     ┌──────▼──────┐   │     ┌─────▼─────┐
     │   WANDER    │   │     │   ATTACK  │
     └──────┬──────┘   │     └─────┬─────┘
            │           │           │
            └──────►IDLE◄──────────┘
                    │
              (low HP)
                    │
              ┌─────▼─────┐
              │    FLEE    │
              └───────────┘
```

**State definitions:**
| State | Behavior | Transition triggers |
|-------|----------|-------------------|
| `Idle` | Stand still, play idle anim | Player detected → Chase; Timer → Patrol |
| `Patrol` | Follow waypoint path | Waypoint reached → Idle/Next; Player → Chase |
| `Wander` | Random movement within radius | Timer → Idle; Player → Chase |
| `Chase` | Pursue player using pathfinding | In attack range → Attack; Lost → Search |
| `Search` | Go to last known player position | Timer/Found → Chase; Timeout → Patrol |
| `Attack` | Deal damage at close range | Player leaves range → Chase; Low HP → Flee |
| `Flee` | Move away from player | Safe distance reached → Idle |
| `Dead` | Death animation, drop loot | Timer → Despawn or Revive |

### 3.2 NPC Behavior Architecture

**Behavior Tree hierarchy (alternative to FSM):**
```
Root (Selector)
├── Sequence: Combat
│   ├── Condition: Player detected
│   ├── Action: Chase player
│   └── Action: Attack
├── Sequence: Patrol
│   ├── Action: Get next waypoint
│   ├── Action: Move to waypoint
│   └── Action: Wait at waypoint
└── Sequence: Idle
    ├── Action: Play idle animation
    └── Action: Look around
```

**Hybrid approach:** FSM for high-level states, Behavior Trees within states for complex sub-behaviors.

### 3.3 NPC Schedule System

```
NPCSchedule {
    npc_id: string
    entries: ScheduleEntry[]
}

ScheduleEntry {
    hour_start: number      // 0-23
    hour_end: number        // 0-23
    location: LocationId    // where to be
    activity: ActivityId    // what to do
    speed: number           // movement speed
    priority: number        // override priority
}
```

**Example schedule:**
```json
{
  "npc_id": "blacksmith",
  "schedule": [
    {"hour_start": 6, "hour_end": 8, "location": "home_bedroom", "activity": "sleep"},
    {"hour_start": 8, "hour_end": 8.5, "location": "home_kitchen", "activity": "eat"},
    {"hour_start": 8.5, "hour_end": 17, "location": "forge", "activity": "work"},
    {"hour_start": 17, "hour_end": 18, "location": "tavern", "activity": "socialize"},
    {"hour_start": 18, "hour_end": 22, "location": "home_living", "activity": "relax"},
    {"hour_start": 22, "hour_end": 6, "location": "home_bedroom", "activity": "sleep"}
  ]
}
```

### 3.4 NPC Relationships

```
Relationship {
    npc_a: string
    npc_b: string           // can be player
    favorability: number    // -100 to +100
    trust: number           // 0 to 100
    familiarity: number     // 0 to 100 (increases with interactions)
    disposition: enum       // hostile, unfriendly, neutral, friendly, allied
    flags: string[]         // "romance_candidate", "quest_giver", etc.
}
```

**Disposition thresholds:**
| Favorability | Disposition |
|-------------|-------------|
| -100 to -50 | Hostile (attacks on sight) |
| -49 to -10 | Unfriendly (won't trade/help) |
| -9 to 9 | Neutral |
| 10 to 49 | Friendly (discounts, info) |
| 50 to 100 | Allied (special quests, romance) |

### 3.5 NPC Dialogue Selection

NPCs select dialogue based on multiple factors:

```
DialogueSelection {
    npc_id: string
    context: {
        time_of_day: number
        location: string
        weather: string
        quest_state: map
        relationship: Relationship
        recent_events: string[]
        npc_mood: string
        player_reputation: number
    }
    selection_priority: [
        "quest_critical",       // quest-related dialogue first
        "time-sensitive",       // time-limited dialogue
        "relationship_reaction", // reaction to player actions
        "greeting",             // standard greeting
        "idle"                  // ambient dialogue
    ]
}
```

**Greeting variants by relationship:**
```
hostile:    "I've got my eye on you." / draws weapon
unfriendly: *nods coldly* / ignores player
neutral:    "Hello." / "Can I help you?"
friendly:   "Hey {player_name}! Good to see you!" / smile
allied:     "My friend! Come, let me show you something!" / embrace
```

---

## 4. Event System

### 4.1 Event Trigger Types

| Trigger | Description | Parameters |
|---------|-------------|------------|
| `onInteract` | Player presses action button on object | object_id |
| `onPlayerEnter` | Player enters trigger zone | zone_id, once_only |
| `onPlayerExit` | Player leaves trigger zone | zone_id |
| `onTime` | Specific game time | hour, minute, day |
| `onItem` | Player gains/uses item | item_id, action |
| `onQuest` | Quest state changes | quest_id, state |
| `onFlag` | Custom flag changes | flag_name, value |
| `onKill` | Enemy defeated | enemy_type, count |
| `onTalk` | NPC conversation ends | npc_id, choice_made |
| `onArea` | Player enters named area | area_name |
| `onDistance` | Player within N tiles of object | object_id, distance |
| `onRepeat` | Timer-based repeat | interval_seconds |
| `onStartup` | Map/scene loads | scene_id |
| `onWeather` | Weather changes | weather_type |
| `onCombat` | Combat starts/ends | state |

### 4.2 Event Actions

| Action | Description | Parameters |
|--------|-------------|------------|
| `showDialogue` | Start dialogue tree | dialogue_id, npc_id |
| `playCutscene` | Trigger cutscene | cutscene_id |
| `spawnEntity` | Spawn NPC/enemy/item | entity_type, location, count |
| `teleport` | Move player | destination_id |
| `giveItem` | Add item to inventory | item_id, quantity |
| `removeItem` | Remove item | item_id, quantity |
| `setFlag` | Set game flag | flag_name, value |
| `modifyQuest` | Advance/modify quest | quest_id, action, params |
| `changeWeather` | Set weather | weather_type, duration |
| `playMusic` | Change background music | track_id, fade_time |
| `playSound` | Play sound effect | sfx_id |
| `showText` | Display floating text | text, location, duration |
| `cameraShake` | Screen shake | intensity, duration |
| `screenFade` | Fade to/from black | duration, direction |
| `lockControls` | Disable player input | duration |
| `moveNPC` | Force NPC movement | npc_id, path, speed |
| `setAnimation` | Play NPC animation | npc_id, anim_id |
| `modifyStats` | Change player stats | stat, delta |
| `unlockArea` | Reveal hidden area | area_id |
| `shopOpen` | Open shop interface | shop_id |

### 4.3 Event Conditions

Events fire only when ALL conditions are true:

```yaml
event_id: "secret_chamber_opens"
triggers:
  - type: "onInteract"
    object: "bookshelf_painting"
conditions:
  - flag("has_torch") == true
  - flag("found_clue") == true
  - quest("library_mystery") == "active"
  - time.hour >= 20 OR time.hour <= 4   # nighttime only
  - NOT flag("chamber_already_opened")
actions:
  - playCutscene: "painting_reveals_passage"
  - setFlag: {"chamber_discovered": true}
  - spawnEntity: {"type": "npc_ghost", "location": "secret_chamber"}
  - playSound: "secret_door_open"
  - showText: {"text": "A hidden passage is revealed!", "duration": 3}
```

### 4.4 Event Chains

Events can chain into sequences:

```
Event Chain: "Dragon Attack"
  Step 1: onTime(hour=6) → showCutscene("dragon_flyover")
  Step 2: flag("dragon_seen") → showDialogue("villager_panic")
  Step 3: onTalk("guard_captain") → modifyQuest("defend_village", "start")
  Step 4: quest("defend_village") == "wave_1_complete" → spawnEnemies("dragon_minions", 10)
  Step 5: quest("defend_village") == "complete" → showCutscene("dragon_retreat")
  Step 6: flag("dragon_defeated") → unlockArea("dragon_lair")
```

**RPG Maker event model (reference):**
| Trigger | Behavior |
|---------|----------|
| Action Button | Player presses action button while facing event |
| Player Touch | Player walks into/on event tile |
| Event Touch | Event walks into player tile |
| Autorun | Runs automatically when conditions met (blocks input) |
| Parallel | Runs automatically but allows player input |

### 4.5 Event Priority

Events have priority layers:
- `Below`: Under player (floor effects, shadows)
- `Same`: Same level as player (NPCs, chests)
- `Above`: Over player (overhead effects, flying objects)

---

## 5. Game Scripting Languages for Dialogue & Quests

### 5.1 ink (inkle)

**Used by:** Heaven's Vault, 80 Days, Sorcery! series

```ink
=== tavern_greeting ===
The tavern is warm and inviting. A grizzled bartender polishes a glass.

+ [Ask about rumors] -> ask_rumors
+ [Buy a drink] -> buy_drink
+ [Leave] -> END

=== ask_rumors ===
"The usual rubbish," he says. "Though there's been talk of wolves near the old mill."

~ knows_wolves = true

+ [Ask about the wolves] -> ask_wolves
+ [Thanks] -> END

=== ask_wolves ===
{knows_wolves:
    "They say three of them, big as ponies. Farmer John lost two sheep last night."
    ~ quest_started = true
}
    + [Offer to help] -> accept_wolves_quest
    + [Not my problem] -> END
```

**Key features:**
- Markup-first syntax (text with logic, not code with text)
- Variables, functions, tunnels, stitches
- Conditional text and choices
- External function calls (C#/JavaScript integration)
- Compiles to JSON

### 5.2 Yarn Spinner

**Used by:** Night in the Woods, A Short Hike, Read Only Memories

```yarn
title: merchant_greeting
tags: merchant, shop
---
Hello there! Welcome to my shop.
-> options

-> options

-> options

title: options
---
<<if $gold >= 10>>
    [Buy a potion (10 gold)]
        <<set $gold -= 10>>
        <<set $inventory.potions += 1>>
        Here you go! Careful now.
<<endif>>
[Ask about quests]
    -> quest_info
[Goodbye]
    Come back soon!
===
```

### 5.3 Twine (Harlowe/Chapbook)

```twine
:: StoryTitle
My RPG

:: StoryData
{
  "startup": "intro"
}

:: intro
You stand at the crossroads.

[[Go north->forest]]
[[Go south->village]]

:: forest
The forest is dark and menacing.

(set: $has_sword to true)
You found a sword!

[[Return to crossroads->intro]]
```

### 5.4 Lua-based (RPG Maker / Defold / LÖVE)

```lua
-- Dialogue script
dialogue:start("merchant", "neutral", {
    text = "Welcome, {player_name}! I have fine wares.",
    choices = {
        {
            text = "Show me your goods",
            condition = function()
                return player.gold >= 10
            end,
            action = function()
                shop:open("general_store")
            end
        },
        {
            text = "Got any quests?",
            action = function()
                dialogue:start("merchant_quest", "excited")
            end
        },
        {
            text = "Goodbye",
            action = function()
                dialogue:end_conversation()
            end
        }
    }
})
```

### 5.5 GDScript-based (Godot DialogueQuest)

```
dialogue:
    npc_name "Merchant"
    emotion "neutral"
    text "Welcome! Would you like to buy a potion?"
    choice "Buy (10 gold)" gold >= 10:
        set gold -= 10
        set potions += 1
        goto "thank_you"
    choice "No thanks":
        goto "farewell"

label "thank_you":
    npc_name "Merchant"
    text "Here you go! Come again!"

label "farewell":
    npc_name "Merchant"
    text "Have a good day!"
    end
```

### 5.6 Custom JSON Format

```json
{
  "dialogue_id": "merchant_greeting",
  "nodes": {
    "start": {
      "speaker": "Merchant",
      "emotion": "neutral",
      "text": "Welcome, {player_name}! What can I do for you?",
      "choices": [
        {
          "text": "Buy a health potion (10 gold)",
          "next": "buy_potion",
          "conditions": [{"type": "stat", "stat": "gold", "op": ">=", "value": 10}]
        },
        {
          "text": "Any quests available?",
          "next": "quest_offer"
        },
        {
          "text": "Goodbye",
          "next": null
        }
      ]
    },
    "buy_potion": {
      "speaker": "Merchant",
      "emotion": "happy",
      "text": "Here you go! Fine quality, I promise.",
      "actions": [{"type": "give_item", "item": "health_potion", "qty": 1}, {"type": "modify_stat", "stat": "gold", "delta": -10}]
    }
  }
}
```

---

## 6. Cutscene System

### 6.1 Cutscene Scripting Format

A cutscene is a time-ordered sequence of commands:

```
Cutscene {
    id: string
    duration: number          // total seconds (0 = until all commands done)
    skippable: bool
    commands: CutsceneCommand[]
}

CutsceneCommand {
    time: number              // seconds from start
    type: string              // command type
    params: map               // command-specific parameters
    duration: number          // how long command takes (for parallel ops)
}
```

### 6.2 Camera Movements

| Command | Description | Parameters |
|---------|-------------|------------|
| `cameraMoveTo` | Move camera to position | target, duration, easing |
| `cameraPan` | Smooth camera movement | from, to, duration |
| `cameraZoom` | Zoom in/out | target_zoom, duration |
| `cameraShake` | Screen shake | intensity, duration, frequency |
| `cameraFollow` | Follow entity | entity_id, offset, smooth |
| `cameraLock` | Lock camera position | position |
| `cameraFree` | Return control to player | — |

**Easing functions:** `linear`, `easeIn`, `easeOut`, `easeInOut`, `bounce`, `elastic`

**Camera path example:**
```yaml
cameraMoveTo:
  target: [10, 5, 0]
  duration: 2.0
  easing: easeInOut
  look_at: [10, 0, 0]
```

### 6.3 Character Animations

| Command | Description | Parameters |
|---------|-------------|------------|
| `moveTo` | Move character to position | entity_id, target, duration, path |
| `faceDirection` | Turn to face direction | entity_id, direction |
| `playAnimation` | Play animation clip | entity_id, anim_id, speed, loop |
| `setEmotion` | Change expression | entity_id, emotion |
| `hide` / `show` | Toggle visibility | entity_id, fade_duration |
| `setPose` | Set static pose | entity_id, pose_id |
| `attachTo` | Parent to another entity | entity_id, parent_id, offset |
| `teleport` | Instant move | entity_id, position |

### 6.4 Text Display

| Command | Description | Parameters |
|---------|-------------|------------|
| `showDialogue` | Show dialogue box | speaker, text, emotion, speed |
| `showNarration` | Show narration text | text, position, style |
| `showSubtitle` | Show subtitle | text, duration, style |
| `showChoice` | Player choice in cutscene | choices[], next_beat |
| `typewriter` | Typewriter text effect | speed, skip_on_input |
| `waitForInput` | Pause until player input | — |
| `wait` | Timed pause | duration_seconds |

### 6.5 Cutscene with Player Choices

```yaml
cutscene_id: "village_attack_intro"
skippable: true
commands:
  - time: 0
    type: screenFade
    params: { from: "black", duration: 2.0 }
  
  - time: 2.0
    type: cameraMoveTo
    params: { target: "village_gate", duration: 3.0 }
  
  - time: 5.0
    type: showNarration
    params: { text: "The village burns. Smoke fills the sky." }
  
  - time: 7.0
    type: showChoice
    params:
      choices:
        - text: "Rush in to save villagers"
          next_beat: "rush_in"
        - text: "Sneak around to scout"
          next_beat: "sneak_scout"
        - text: "Call for reinforcements"
          next_beat: "call_help"

  - beat: "rush_in"
    time: 0
    type: moveTo
    params: { entity: "player", target: "village_center", duration: 2.0 }
  
  - beat: "sneak_scout"
    time: 0
    type: moveTo
    params: { entity: "player", target: "forest_edge", duration: 3.0 }
```

### 6.6 Cutscene Frameworks (Reference)

| Framework | Engine | Features |
|-----------|--------|----------|
| **GDrama** | Godot | Screenplay syntax, beats, choices, animations |
| **ink + Unity** | Unity | Narrative scripting compiled to JSON |
| **Level Sequence** | Unreal | Visual sequencer, keyframe animation |
| **RPG Maker Events** | RPG Maker | Autorun events, show text, move routes |
| **Sequencer** | Godot 4 | Built-in timeline editor, keyframes |
| **Timeline** | Unity | Playable Director, signal track |

---

## 7. Integration Patterns

### 7.1 Dialogue ↔ Quest Integration

```
DialogueNode → triggers → QuestActivate
DialogueNode → checks → QuestState
DialogueChoice → sets → QuestObjective
QuestComplete → unlocks → DialogueNode
QuestFailed → changes → DialogueNode
```

### 7.2 NPC ↔ Event Integration

```
NPC Schedule → triggers → Event (onTime)
NPC Favorability → affects → Dialogue options
NPC Location → affects → Event availability
NPC Death → triggers → Event (onKill)
NPC Interaction → triggers → Event (onTalk)
```

### 7.3 Event ↔ Cutscene Integration

```
Event Trigger → starts → Cutscene
Cutscene Command → sets → Flag/Variable
Cutscene Choice → modifies → Quest State
Cutscene End → triggers → Event Chain
Event Chain → plays → Another Cutscene
```

### 7.4 Unified State Flow

```
Player Action
  → Event System (evaluate triggers)
    → Quest System (check/update objectives)
      → NPC System (update relationships, schedules)
        → Dialogue System (select appropriate dialogue)
          → Cutscene System (if triggered)
            → UI System (update journal, notifications)
```

---

## 8. Data Format Recommendations

### 8.1 Dialogue Format (JSON)

```json
{
  "$schema": "dialogue_v1",
  "dialogue_id": "blacksmith_greeting",
  "npc_id": "blacksmith_01",
  "conditions": [],
  "nodes": {
    "start": {
      "speaker": "blacksmith_01",
      "emotion": "neutral",
      "portrait": "sprites/blacksmith/neutral.png",
      "text": "Morning, {player_name}. Need something forged?",
      "actions": [],
      "choices": [
        {
          "id": "buy_weapon",
          "text": "Show me your weapons",
          "next": "shop_weapon",
          "conditions": []
        },
        {
          "id": "quest_info",
          "text": "I need help with something",
          "next": "quest_offer",
          "conditions": [
            {"type": "quest", "quest": "iron_ore_shortage", "state": "unavailable"}
          ]
        },
        {
          "id": "goodbye",
          "text": "Just passing through",
          "next": null,
          "conditions": []
        }
      ]
    }
  }
}
```

### 8.2 Quest Format (JSON)

```json
{
  "$schema": "quest_v1",
  "quest_id": "iron_ore_shortage",
  "type": "side",
  "title": "Iron Ore Shortage",
  "description": "The blacksmith has run out of iron ore. Help him find a new source.",
  "giver": "blacksmith_01",
  "prerequisites": [],
  "objectives": [
    {
      "id": "find_mine",
      "type": "goto",
      "target_location": "abandoned_mine",
      "description": "Find the abandoned mine",
      "optional": false,
      "hidden": false
    },
    {
      "id": "collect_ore",
      "type": "collect",
      "target_item": "iron_ore",
      "required": 10,
      "description": "Collect 10 iron ore",
      "optional": false,
      "hidden": false,
      "auto_track": true
    },
    {
      "id": "return_to_blacksmith",
      "type": "talk",
      "target_npc": "blacksmith_01",
      "description": "Return the ore to the blacksmith",
      "optional": false,
      "hidden": true
    }
  ],
  "rewards": {
    "base": {
      "experience": 200,
      "gold": 50,
      "reputation": [{"faction": "village", "delta": 5}]
    },
    "choice": {
      "prompt": "Choose your reward:",
      "options": [
        {"text": "Iron Shield", "item": "iron_shield"},
        {"text": "Steel Sword", "item": "steel_sword"},
        {"text": "100 Gold", "gold": 100}
      ]
    }
  },
  "journal_entries": {
    "start": "The blacksmith needs iron ore. He mentioned an abandoned mine to the east.",
    "find_mine": "Found the abandoned mine. It looks like it's been worked before.",
    "collect_ore": "Gathering iron ore from the mine.",
    "return": "Got the ore. Time to return to the blacksmith.",
    "complete": "The blacksmith is grateful. The village will have weapons again."
  }
}
```

### 8.3 NPC Format (JSON)

```json
{
  "$schema": "npc_v1",
  "npc_id": "blacksmith_01",
  "name": "Thorin",
  "class": "blacksmith",
  "level": 5,
  "stats": {"hp": 100, "attack": 8, "defense": 12},
  "schedule": [
    {"hour_start": 6, "hour_end": 8, "location": "home_bedroom", "activity": "sleep"},
    {"hour_start": 8, "hour_end": 17, "location": "forge", "activity": "work"},
    {"hour_start": 17, "hour_end": 22, "location": "tavern", "activity": "socialize"},
    {"hour_start": 22, "hour_end": 6, "location": "home_bedroom", "activity": "sleep"}
  ],
  "dialogue_map": {
    "default": "blacksmith_greeting",
    "quest_active_iron_ore": "blacksmith_quest_active",
    "quest_complete_iron_ore": "blacksmith_quest_complete",
    "relationship_hostile": "blacksmith_hostile",
    "relationship_friendly": "blacksmith_friendly"
  },
  "shop": {
    "shop_id": "blacksmith_shop",
    "inventory": ["iron_sword", "iron_shield", "iron_helmet", "iron_boots"],
    "buy_markup": 1.2,
    "sell_discount": 0.5
  },
  "relationships": {
    "default_favorability": 20,
    "default_disposition": "neutral"
  }
}
```

### 8.4 Event Format (JSON)

```json
{
  "$schema": "event_v1",
  "event_id": "secret_chamber_discovery",
  "triggers": [
    {"type": "onInteract", "object": "bookshelf_painting"},
    {"type": "onFlag", "flag": "has_torch", "value": true}
  ],
  "conditions": [
    {"type": "flag", "flag": "found_clue", "value": true},
    {"type": "quest", "quest": "library_mystery", "state": "active"},
    {"type": "time", "hour_start": 20, "hour_end": 4},
    {"type": "not_flag", "flag": "chamber_already_opened"}
  ],
  "actions": [
    {"type": "playCutscene", "cutscene_id": "painting_reveals_passage"},
    {"type": "setFlag", "flag": "chamber_discovered", "value": true},
    {"type": "spawnEntity", "entity_type": "npc_ghost", "location": "secret_chamber"},
    {"type": "playSound", "sfx_id": "secret_door_open"},
    {"type": "showText", "text": "A hidden passage is revealed!", "duration": 3}
  ],
  "chain": {
    "next_event": "ghost_dialogue",
    "delay": 5.0
  }
}
```

---

## 9. Key Design Principles

1. **Separation of Data and Logic** — Dialogue/quest data in JSON/YAML; game logic in code
2. **Event-Driven Architecture** — Systems communicate through events, not direct calls
3. **State Machine Pattern** — Quest, NPC, and dialogue all use explicit state machines
4. **Condition Evaluation** — All conditions evaluated at runtime, not compile time
5. **Composability** — Small reusable systems (dialogue nodes, quest objectives, event triggers) composed into complex behaviors
6. **Save/Load Support** — All state must be serializable for save games
7. **Modularity** — Each system can be developed and tested independently
8. **Backward Compatibility** — Schema versioning for data formats

---

## 10. References

- Smith, G. et al. (2011). "Situating Quests: Design Patterns for Quest and Level Design in Role-Playing Games." ICIDS 2011.
- inkle. "ink: A narrative scripting language for games." github.com/inkle/ink
- Narrat Documentation. "Branching dialogue and choices." docs.narrat.dev
- DialogueForge. "Visual, node-based dialogue tree editor." github.com/nikatopu/dialogue-forge
- RPG Maker MZ Documentation. "Map Event System." rpgmakerofficial.com
- GDrama. "Framework for writing and animating cutscenes in Godot." github.com/moraguma/GDrama
- Godot Quest Manager. "Powerful quest management system." github.com/Rubonnek/quest-manager
- GameDeveloper. "Branching Conversation Systems and the Working Writer." gamedeveloper.com
- GameAnalytics. "How to write perfect dialogue trees for games." gameanalytics.com
- Barotrauma Wiki. "Events — trigger conditions and branching paths."
- Divinity Engine Wiki. "Character and Item Script Triggers, Calls, and Queries."

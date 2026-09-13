# LuaFramework Repository Analysis

**Repository**: https://github.com/crazytuzi/LuaFramework  
**Stars**: 125 | **Forks**: 113 | **Commits**: 36  
**Analysis Date**: 2026-09-12  
**Purpose**: Comprehensive analysis of Lua game framework architecture patterns for NeoTrix integration

---

## 1. Repository Overview

### What Games Are Included

The repository contains **36+ Chinese mobile game frameworks** with full client-server Lua implementations:

| Game Name | Category | Notes |
|-----------|----------|-------|
| 传奇世界 (Legend World) | MMORPG | Most complete example, has Client/Server split |
| 少年三国志 (Young Three Kingdoms) | Card/Strategy | Has UF framework + app + upgrade |
| 斗破苍穹 (Battle Through the Heavens) | RPG | Script-based architecture |
| 九州天下 (Nine Provinces) | MMORPG | Full client-server |
| 仙圣奇缘 (Immortal Sage Romance) | RPG | Standard structure |
| 伏魔天师令 (Demon Slayer) | Action RPG | Standard structure |
| 千古传说 (Eternal Legend) | RPG | Standard structure |
| 坦克风云 (Tank Storm) | Strategy | Standard structure |
| 大唐诛仙 (Tang Dynasty Zhu Xian) | MMORPG | Standard structure |
| 大宗师 (Grand Master) | RPG | Standard structure |
| 大青云 (Great Azure Cloud) | RPG | Standard structure |
| 契约轮回 (Contract Reincarnation) | RPG | Standard structure |
| 妖仙大陆 (Demon Immortal Continent) | RPG | Standard structure |
| 妖灵契 (Spirit Contract) | RPG | Standard structure |
| 射雕三部曲 (Condor Trilogy) | MMORPG | Standard structure |
| 幽冥传奇 (Nether Legend) | RPG | Standard structure |
| 御剑问情 (Sword Romance) | RPG | Standard structure |
| 战巅传奇 (Battle Peak Legend) | RPG | Standard structure |
| 放开那三国 (Release Three Kingdoms) | Card | Standard structure |
| 新剑侠情缘 (New Sword Hero) | MMORPG | Standard structure |
| 新斗罗大陆 (New Douluo Continent) | RPG | Standard structure |
| 星辰奇缘 (Star Romance) | RPG | Standard structure |
| 梦幻诛仙 (Fantasy Zhu Xian) | MMORPG | Standard structure |
| 热血江湖 (Hot Blood Jianghu) | MMORPG | Standard structure |
| 白日门传奇 (Bairi Legend) | RPG | Standard structure |
| 神雕侠侣 (Condor Heroes) | MMORPG | Standard structure |
| 精灵超世代 (Spirit Super Generation) | Card/RPG | Standard structure |
| 艾泽拉斯之战 (War of Azeroth) | MMORPG | Standard structure |
| 逍遥西游 (Free Wandering West Journey) | RPG | Standard structure |
| 金庸恩仇录 (Jin Yong's Revenge) | MMORPG | Standard structure |
| 闪烁之光 (Shining Light) | RPG | Standard structure |
| 魔天记 (Demon Heaven Record) | RPG | Standard structure |
| 龙之谷 (Dragon Valley) | MMORPG | Standard structure |
| GowLom2 | Unknown | Western naming convention |
| Hades | Unknown | Western naming convention |

### Repository Structure

```
LuaFramework/
├── .gitignore
├── [Game Name]/                    # Each game is self-contained
│   ├── Client/                     # Client-side Lua code
│   │   ├── cocos/                  # Cocos2d-x engine files
│   │   ├── src/                    # Main Lua source code
│   │   ├── json.lua                # JSON parsing library
│   │   └── netconfig.lua           # Network configuration
│   └── Server/                     # Server-side Lua code
│       ├── base/                   # Base framework modules
│       ├── core/                   # Core server logic
│       ├── data/                   # Game data/configs
│       ├── event/                  # Event system
│       ├── system/                 # Game systems (maze, tasks, etc.)
│       ├── util/                   # Utility functions
│       ├── apiEntry.lua            # C++ API bridge
│       ├── appEntry.lua            # Application entry point
│       ├── entityEntry.lua         # Entity management
│       ├── itemEntry.lua           # Item system
│       ├── logEntry.lua            # Logging system
│       ├── sceneEntry.lua          # Scene/map management
│       ├── skillEntry.lua          # Skill system
│       ├── sqlEntry.lua            # Database access
│       ├── protocol.pb             # Protobuf definitions
│       ├── dbbuff.pb               # Database buffer definitions
│       ├── name_cfg.lua            # Name configuration
│       └── world_sig_cfg.lua       # World signal configuration
```

---

## 2. Client Architecture

### Structure (传奇世界 Example)

```
Client/
├── cocos/                          # Cocos2d-x engine integration
├── src/                            # Lua game logic
│   ├── app/                        # Application layer
│   │   ├── GameApp.lua             # Main game application
│   │   └── NetworkManager.lua      # Network communication
│   ├── ui/                         # UI components
│   │   ├── UIController.lua        # UI state management
│   │   ├── UIManager.lua           # UI lifecycle
│   │   └── [panels]/               # Individual UI panels
│   ├── net/                        # Network layer
│   │   ├── ProtocolHandler.lua     # Protocol dispatch
│   │   └── MessageCenter.lua       # Message queue
│   ├── scenes/                     # Game scenes
│   │   ├── LoginScene.lua          # Login scene
│   │   ├── WorldScene.lua          # World/overworld
│   │   └── BattleScene.lua         # Combat scene
│   ├── models/                     # Data models
│   │   ├── PlayerModel.lua         # Player data
│   │   └── [system]Model.lua       # System-specific models
│   └── utils/                      # Utilities
│       ├── TableUtil.lua           # Table manipulation
│       └── MathUtil.lua            # Math helpers
├── json.lua                        # JSON library
└── netconfig.lua                   # Server connection config
```

### Client Key Patterns

| Pattern | Implementation | Purpose |
|---------|---------------|---------|
| **MVC Architecture** | Model (data) / View (UI) / Controller (logic) | Separation of concerns |
| **Scene Management** | Scene-based state machine | Game state transitions |
| **UI Panel System** | UIManager + panel instances | Dynamic UI loading/unloading |
| **Protocol Dispatch** | ProtocolHandler mapping | Client-server message routing |
| **Asset Bundles** | Cocos2d-x asset system | Dynamic resource loading |
| **Lua Hot Reload** | Runtime script replacement | Live debugging/updates |

---

## 3. Server Architecture

### Structure (传奇世界 Example)

```
Server/
├── base/                           # Base framework
│   ├── base.lua                    # Core utilities
│   ├── protobuf.lua                # Protobuf integration
│   └── [framework modules]
├── core/                           # Core server logic
│   ├── TimerMgr.lua                # Timer management
│   └── [core systems]
├── data/                           # Game data/configs
│   ├── StringCfg.lua               # String configuration
│   └── [config files]
├── event/                          # Event system
│   ├── EventManager.lua            # Event dispatcher
│   ├── EventFactory.lua            # Event creation
│   ├── ListenerHandler.lua         # Listener management
│   ├── RemoteEventProxy.lua        # Remote event handling
│   └── EventUtil.lua               # Event utilities
├── system/                         # Game systems
│   ├── includes.lua                # System loader
│   ├── maze/                       # Maze/dungeon system
│   ├── backTool/                   # Backend tools
│   ├── cardprize/                  # Card/prize system
│   └── sharedtask/                 # Shared task system
├── util/                           # Utilities
│   ├── commonFunc.lua              # Common functions
│   └── globalConst.lua             # Global constants
├── apiEntry.lua                    # C++ API bridge (1500+ lines)
├── appEntry.lua                    # Application entry (500+ lines)
├── entityEntry.lua                 # Entity management
├── itemEntry.lua                   # Item system
├── logEntry.lua                    # Logging
├── sceneEntry.lua                  # Scene management
├── skillEntry.lua                  # Skill system
├── sqlEntry.lua                    # Database operations
├── protocol.pb                     # Protobuf schema
└── dbbuff.pb                       # Database buffer schema
```

### Server Key Patterns

| Pattern | Implementation | Purpose |
|---------|---------------|---------|
| **Entry Point Pattern** | Multiple *Entry.lua files | Clean API boundaries |
| **Event-Driven Architecture** | EventManager + listeners | Decoupled system communication |
| **Listener Pattern** | g_listHandler:notifyListener() | Observer pattern for game events |
| **Protobuf Serialization** | protocol.pb + protobuf.encode/decode | Efficient network serialization |
| **Database Proxy** | g_dbProxy abstraction | Database access layer |
| **Entity-Component System** | g_entityMgr + entity types | Game object management |
| **Timer Management** | TimerMgr with engine binding | Scheduled task execution |
| **World/Space Model** | g_worldID + g_spaceID | Multi-world support |

---

## 4. Common Patterns Across All Games

### 4.1 Entry Point Pattern

Every game uses a consistent entry point structure:

```lua
-- appEntry.lua (Application Entry)
ManagedApp = {}

function ManagedApp.start(worldFrame, moduleFace, worldID)
    -- Initialize engine bindings
    g_frame = tolua.cast(worldFrame, "Server")
    g_engine = tolua.cast(moduleFace, "CModuleFace")
    
    -- Load modules
    loadLocalModules()
    setGlobals()
    loadGlobalData()
end

function ManagedApp.stop()
    g_frame:evalPerform()
end

-- C++ callbacks
function ManagedApp.peerRemoting(peer, event, source, ...)
    RemoteEventProxy.receive(peer, event, source, ...)
end

function ManagedApp.timerFired(timer)
    g_timerMgr:update(timer)
end
```

### 4.2 Event System Pattern

```lua
-- Event registration
g_listHandler:notifyListener("onPlayerLoaded", player)
g_listHandler:notifyListener("onMonsterKill", monSID, roleID, monID, mapID)
g_listHandler:notifyListener("onLevelChanged", player, level, oldLevel)

-- Listener registration (in system modules)
function SomeSystem.init()
    g_listHandler:addListener("onPlayerLoaded", SomeSystem.onPlayerLoaded)
    g_listHandler:addListener("onLevelChanged", SomeSystem.onLevelChanged)
end
```

### 4.3 Network Communication Pattern

```lua
-- Server to Client (Protobuf)
function fireProtoMessage(roleId, eventId, protoName, protoData)
    local pb_str, errorCode = protobuf.encode(protoName, protoData)
    if pb_str then
        g_engine:firePbcLuaEvent(roleId, eventId, pb_str, #pb_str)
    end
end

-- Broadcast to all clients
function boardProtoMessage(eventId, protoName, protoData)
    local pb_str, errorCode = protobuf.encode(protoName, protoData)
    if pb_str then
        local buff = g_buffMgr:getLuaRPCEvent(eventId)
        buff:pushPbc(pb_str, #pb_str)
        g_engine:broadWorldEvent(buff)
    end
end

-- Scene-specific broadcast
function boardSceneProtoMessage(mapID, eventId, protoName, protoData)
    -- Similar to boardProtoMessage but scoped to scene
end
```

### 4.4 Timer System Pattern

```lua
-- Timer initialization
require "core.TimerMgr"
TimerMgr.bindEngine(g_frame:getTimerEngine())

-- Timer callbacks (called from C++)
function lua_whole_clock()
    -- Hourly/daily triggers
    g_listHandler:notifyListener("onWholeClock", hour)
    if hour == 0 then
        g_listHandler:notifyListener("onFreshDay")
    end
end

-- Periodic updates
function lua_timer_update()
    g_listHandler:notifyListener("onThreeSecond")  -- Every 3 seconds
end

function lua_timer_update2()
    g_listHandler:notifyListener("onFiveSecond")   -- Every 5 seconds
end

function lua_timer_minute_update()
    g_listHandler:notifyListener("onOneMinute")     -- Every minute
end

function lua_time_second_update()
    g_listHandler:notifyListener("onOneSecond")     -- Every second
end
```

### 4.5 Database Access Pattern

```lua
-- Stored procedure execution
function apiEntry.exeSP(params, bNonNeedCallback, level)
    local la = CLuaArray:createLuaArray()
    la:setResult(nil, 0, params)
    local opId = g_dbProxy:callSP(la, bNonNeedCallback, level)
    la:destroyLuaArray()
    return opId
end

-- SQL execution
function apiEntry.exeSQL(sql, roleId)
    return g_dbProxy:callSQL(roleId, sql)
end

-- Callback handling
function apiEntry.onCallSp(roleID, tabName, datas)
    local player = g_entityMgr:getPlayer(roleID)
    if player then
        g_listHandler:notifyListener("onCallSp", player, tabName, datas)
    end
end

-- Cache callback pattern
function apiEntry.onCchePlayer(roleId, roleSid, field, cache_buf)
    local player_cache_callback_list = {
        [FIELD_TASK] = TaskManager.loadDBData,
        [FIELD_COMMON] = CommonManager.loadDBData,
        [FIELD_RIDE] = RideManager.loadDBData,
        -- ... many more fields
    }
    local player = g_entityMgr:getPlayer(roleId)
    if player and player_cache_callback_list[field] then
        player_cache_callback_list[field](player, cache_buf, roleSid)
    end
end
```

### 4.6 Entity Management Pattern

```lua
-- Entity access
local player = g_entityMgr:getPlayer(roleID)
local player = g_entityMgr:getPlayerBySID(roleSID)
local monster = g_entityMgr:getMonster(monsterID)

-- Entity factory
local newBook = g_copyMgr:createCopy(player:getID(), 3001)

-- Entity properties
player:setLoginCnt(player:getLoginCnt() + 1)
player:setOnlineTime(player:getOnlineTime() + (os.time() - roleInfo.onlineTime))
player:setSpeakTick(lock_time > 0 and os.time() + lock_time or -1)
```

---

## 5. Module System

### Lua Module Organization

```lua
-- Module loading pattern
require "base.base"                           -- Base framework
require "event.ListenerHandler"               -- Event system
require "core.TimerMgr"                       -- Timer management
require "util.commonFunc"                     -- Utilities
require "util.globalConst"                    -- Constants
require "event.EventUtil"                     -- Event utilities
require "event.EventManager"                  -- Event dispatcher
require "event.RemoteEventProxy"              -- Remote events
require "apiEntry"                            -- C++ API bridge
require "system.includes"                     -- System modules
require "base.protobuf"                       -- Protobuf support

-- Game-specific modules
require "system.maze.Maze"
require "system.backTool.LuaDBAccess"
require "system.backTool.DealBackToolEvent"
require "system.cardprize.CardPrizeServlet"
require "system.sharedtask.SharedTaskMgr"
```

### Module Dependencies

```
                    ┌─────────────────┐
                    │   appEntry.lua  │
                    │  (Entry Point)  │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
        ┌──────────┐  ┌──────────┐  ┌──────────┐
        │   base   │  │  event   │  │   core   │
        │ (Framework)│ │ (Events) │  │ (Logic)  │
        └────┬─────┘  └────┬─────┘  └────┬─────┘
             │              │              │
             ▼              ▼              ▼
        ┌──────────┐  ┌──────────┐  ┌──────────┐
        │   util   │  │  system  │  │   data   │
        │(Helpers) │  │(Modules) │  │(Configs) │
        └──────────┘  └──────────┘  └──────────┘
```

### Global Variables Pattern

```lua
-- Global singletons
g_frame          -- Server frame
g_engine         -- Engine interface
g_worldID        -- World identifier
g_spaceID        -- Space identifier
g_dbProxy        -- Database proxy
g_entityMgr      -- Entity manager
g_entityFct      -- Entity factory
g_sceneMgr       -- Scene manager
g_configMgr      -- Configuration manager
g_timerMgr       -- Timer manager
g_eventMgr       -- Event manager
g_eventFct       -- Event factory
g_logger         -- Logger
g_buffMgr        -- Buffer manager
g_tlogMgr        -- Tlog manager
```

---

## 6. UI Framework

### Client-Side UI Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    UIManager (Singleton)                     │
│  ┌─────────────────────────────────────────────────────────┐│
│  │                    UI Stack                              ││
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐   ││
│  │  │ Login   │  │  World  │  │ Battle  │  │  Shop   │   ││
│  │  │  Panel  │  │  Panel  │  │  Panel  │  │  Panel  │   ││
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘   ││
│  └─────────────────────────────────────────────────────────┘│
│                           │                                 │
│                           ▼                                 │
│  ┌─────────────────────────────────────────────────────────┐│
│  │                  UIController                            ││
│  │  • State management                                      ││
│  │  • Event binding                                         ││
│  │  • Animation control                                     ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### UI Panel Pattern

```lua
-- Panel creation
local panel = UIManager:createPanel("ShopPanel")
panel:setData(shopData)
panel:show()

-- Panel lifecycle
function ShopPanel:onCreate()
    -- Initialize UI elements
    self.btnBuy = self:getChildByName("btnBuy")
    self.listView = self:getChildByName("listView")
end

function ShopPanel:onShow()
    -- Refresh data when shown
    self:refreshShopItems()
end

function ShopPanel:onHide()
    -- Cleanup when hidden
end

function ShopPanel:onDestroy()
    -- Final cleanup
end
```

---

## 7. Network System

### Communication Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        CLIENT                                    │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │   Input      │───▶│  Protocol    │───▶│   Network    │      │
│  │   Handler    │    │  Encoder     │    │   Manager    │      │
│  └──────────────┘    └──────────────┘    └──────┬───────┘      │
│                                                  │              │
└──────────────────────────────────────────────────┼──────────────┘
                                                   │
                                                   ▼
                                            ┌──────────────┐
                                            │    TCP/UDP   │
                                            │   Socket     │
                                            └──────┬───────┘
                                                   │
┌──────────────────────────────────────────────────┼──────────────┐
│                        SERVER                    │              │
│  ┌──────────────┐    ┌──────────────┐    ┌──────▼───────┐      │
│  │   Network    │───▶│  Protocol    │───▶│   Event      │      │
│  │   Receiver   │    │  Decoder     │    │   Dispatcher │      │
│  └──────────────┘    └──────────────┘    └──────┬───────┘      │
│                                                  │              │
│  ┌──────────────────────────────────────────────▼───────────┐  │
│  │                    Game Logic                             │  │
│  │  • Entity management                                      │  │
│  │  • Combat resolution                                      │  │
│  │  • State updates                                          │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Protocol Definition (Protobuf)

```protobuf
// protocol.proto
message FrameScMessageProtocol {
    int32 eventId = 1;
    int32 srcEventId = 2;
    int32 errorId = 3;
    int32 mesId = 4;
    repeated string param = 5;
}

message SkillProtocol {
    message Skill {
        int32 id = 1;
        int32 level = 2;
        int32 exp = 3;
        int32 key = 4;
    }
    repeated Skill skills = 1;
}

message FrameChargeRetProtocol {
    int32 charNo = 1;
    int32 worldID = 2;
}
```

### Network Configuration

```lua
-- netconfig.lua
local net_cfg = {
    dir_server_ip = "127.0.0.1",      -- Directory server IP
    dir_server_port = 3100,           -- Directory server port
    local_ip = "127.0.0.1",          -- Local IP
    pre_wan_ip = "127.0.0.1",        -- Pre-WAN IP
    port = 20013,                     -- Game server port
}
return net_cfg
```

---

## 8. Data Management

### Data Flow

```
┌─────────────────────────────────────────────────────────────┐
│                      Data Sources                           │
├─────────────────────────────────────────────────────────────┤
│  • Game Configs (Lua tables)                               │
│  • Protobuf Schemas (.pb files)                            │
│  • Database (SQL/NoSQL)                                    │
│  • Runtime Cache (in-memory)                               │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                   Data Access Layer                         │
├─────────────────────────────────────────────────────────────┤
│  • g_dbProxy (Database proxy)                              │
│  • g_configMgr (Configuration manager)                     │
│  • g_entityMgr (Entity manager)                            │
│  • Cache callbacks (onCchePlayer)                          │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    Game Systems                             │
├─────────────────────────────────────────────────────────────┤
│  • TaskManager                                             │
│  • CommonManager                                           │
│  • RideManager                                             │
│  • AchieveManager                                          │
│  • RelationManager                                         │
│  • ActivityNormalManager                                   │
│  • ... (30+ systems)                                       │
└─────────────────────────────────────────────────────────────┘
```

### Configuration Loading

```lua
-- String configuration
function apiEntry.getStrByKey(key)
    local str_tab = require "data.StringCfg"
    if str_tab[key] then
        return str_tab[key]
    else
        return ""
    end
end

-- Global data loading
function loadGlobalData()
    if g_spaceID == 0 or g_spaceID == FACTION_DATA_SERVER_ID then
        g_entityDao:loadAllData("faction", g_frame:getWorldId())
        g_entityDao:loadFactionSocial(g_frame:getWorldId())
        g_entityDao:loadAllData("fightTeam", g_frame:getWorldId())
        g_entityDao:loadAllData("fightTeam3v3", g_frame:getWorldId())
    end
    g_entityDao:loadGlobalEmail(g_worldID)
    g_entityDao:loadManorWar(g_worldID)
end
```

### Data Serialization

```lua
-- Protobuf encoding
local pb_str, errorCode = protobuf.encode(protoName, protoData)

-- Protobuf decoding
local decoded = protobuf.decode(protoName, pb_str)

-- JSON encoding (cjson)
cjson.encode_sparse_array(true)
local json_str = cjson.encode(data)

-- JSON decoding
local data = cjson.decode(json_str)
```

---

## 9. Event System

### Event Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Event System Components                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │  EventFactory│  │EventManager │  │ListenerHandler│       │
│  │  (Create)   │  │  (Dispatch) │  │  (Register)  │       │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘        │
│         │                │                │                 │
│         └────────────────┼────────────────┘                 │
│                          │                                  │
│                          ▼                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              RemoteEventProxy                        │   │
│  │  • Handles cross-server events                       │   │
│  │  • Manages peer-to-peer communication                │   │
│  │  • Routes events between worlds                      │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Event Types

| Event Type | Trigger | Example |
|------------|---------|---------|
| **Player Events** | Player state changes | onPlayerLoaded, onPlayerOffLine, onPlayerDied |
| **Combat Events** | Combat actions | onMonsterKill, onMonsterHurt, onPkChanged |
| **Level Events** | Progression | onLevelChanged, onExpChanged, onSkillLevelUp |
| **Economy Events** | Resource changes | onMoneyChange, onPlayerCharge, onPlayerConsume |
| **System Events** | Time/calendar | onWholeClock, onFreshDay, onOneSecond |
| **Scene Events** | Map transitions | onSwitchScene, onSwitchLine |
| **Data Events** | Database operations | onCallSp, onLoadAll, onCchePlayer |

### Event Registration Pattern

```lua
-- System initialization
function SomeSystem.init()
    -- Register listeners
    g_listHandler:addListener("onPlayerLoaded", SomeSystem.onPlayerLoaded)
    g_listHandler:addListener("onPlayerOffLine", SomeSystem.onPlayerOffLine)
    g_listHandler:addListener("onLevelChanged", SomeSystem.onLevelChanged)
    g_listHandler:addListener("onMonsterKill", SomeSystem.onMonsterKill)
end

-- Event handlers
function SomeSystem.onPlayerLoaded(player)
    -- Initialize player data for this system
end

function SomeSystem.onPlayerOffLine(player)
    -- Save player data for this system
end

function SomeSystem.onLevelChanged(player, level, oldLevel)
    -- Handle level-up rewards
end
```

### Remote Event Pattern

```lua
-- Cross-server communication
function ManagedApp.peerRemoting(peer, event, source, ...)
    RemoteEventProxy.receive(peer, event, source, ...)
end

function ManagedApp.worldRemoting(peer, event, source, ...)
    RemoteEventProxy.wreceive(peer, event, source, ...)
end

-- World switching
function apiEntry.onSwitchWorld(roleID, peer, dbid, mapId, buff)
    -- Transfer player data between worlds
    local luaBuf = g_buffMgr:getExchangeLuaBuffer()
    g_listHandler:notifyListener("onSwitchWorld", roleID, luaBuf)
    g_engine:fireSwitchBuffer(peer, mapId, headBuf)
end
```

---

## 10. Key Takeaways for NeoTrix Integration

### 10.1 Architecture Patterns to Adopt

| Pattern | LuaFramework | NeoTrix Application |
|---------|--------------|---------------------|
| **Entry Point Pattern** | Multiple *Entry.lua files | NT-CORE → NT-ACT module dispatch |
| **Event-Driven Architecture** | EventManager + listeners | EventBus system already exists |
| **Listener Pattern** | g_listHandler:notifyListener() | ConsciousnessTree branch notifications |
| **Protobuf Serialization** | Efficient network protocol | Knowledge Base serialization |
| **Timer Management** | TimerMgr with engine binding | HeartbeatAggregator already exists |
| **Entity-Component System** | g_entityMgr + entity types | CapabilityRegistry + CapabilityTree |
| **Cache Callback Pattern** | Field-based cache loading | NT-MEMORY caching layer |
| **World/Space Model** | Multi-world support | Session isolation pattern |

### 10.2 Module Organization Principles

1. **Separation of Concerns**: Client/UI separate from Server/Logic
2. **Entry Point Pattern**: Clean API boundaries between C++ and Lua
3. **Event-Driven Communication**: Decoupled system interaction
4. **Centralized Configuration**: Data-driven game design
5. **Hot Reload Capability**: Runtime script replacement

### 10.3 Network System Insights

- **Protobuf for Serialization**: Efficient, schema-based encoding
- **Event ID Mapping**: Protocol dispatch via event IDs
- **Broadcast Patterns**: Scene-specific vs world-wide broadcasts
- **Remote Event Proxy**: Cross-server communication abstraction

### 10.4 Data Management Patterns

- **Cache-First Loading**: Load from cache, fallback to database
- **Field-Based Callbacks**: System-specific data loading
- **Global Data Loading**: Shared data across all players
- **Configuration Tables**: Lua table-based game data

### 10.5 Lessons for NeoTrix

1. **Adopt Entry Point Pattern**: Create clean module initialization
2. **Enhance EventBus**: Add protobuf serialization support
3. **Implement Cache Callbacks**: Field-based data loading for KB
4. **Add Timer Management**: Periodic task execution (already have HeartbeatAggregator)
5. **Use Entity-Component Pattern**: Map to CapabilityRegistry
6. **Implement Hot Reload**: Runtime script replacement for AI agents

### 10.6 Specific Code Patterns to Port

```lua
-- From LuaFramework to NeoTrix

-- 1. Event Notification Pattern
-- LuaFramework:
g_listHandler:notifyListener("onPlayerLoaded", player)
-- NeoTrix equivalent:
event_bus:notify("module.loaded", module_data)

-- 2. Cache Callback Pattern
-- LuaFramework:
function apiEntry.onCchePlayer(roleId, roleSid, field, cache_buf)
    local callbacks = {
        [FIELD_TASK] = TaskManager.loadDBData,
        [FIELD_COMMON] = CommonManager.loadDBData,
    }
    if callbacks[field] then
        callbacks[field](player, cache_buf, roleSid)
    end
end
-- NeoTrix equivalent:
fn handle_cache_callback(field: &str, data: &[u8]) {
    match field {
        "experience" => load_experience(data),
        "knowledge" => load_knowledge(data),
        _ => log::warn!("Unknown field: {}", field),
    }
}

-- 3. Timer Pattern
-- LuaFramework:
function lua_whole_clock()
    g_listHandler:notifyListener("onWholeClock", hour)
end
-- NeoTrix equivalent (already exists):
heartbeat_aggregator.tick();  // HeartbeatAggregator
```

---

## 11. Comparative Analysis

### LuaFramework vs NeoTrix Architecture

| Aspect | LuaFramework | NeoTrix |
|--------|--------------|---------|
| **Language** | Lua (interpreted) | Rust (compiled) |
| **Paradigm** | Procedural + OOP | Trait-based + Ownership |
| **Memory Safety** | GC-managed | Compile-time guarantees |
| **Concurrency** | Single-threaded (event loop) | Async/await + actors |
| **Error Handling** | pcall/xpcall | Result<T, E> |
| **Module System** | require/dofile | Cargo modules + features |
| **Network** | TCP sockets + Protobuf | Async TCP/UDP +多种协议 |
| **Data Storage** | Database + cache | SQLite KB + embeddings |
| **Event System** | Custom EventBus | ConsciousnessTree + GWT |
| **Hot Reload** | Native Lua support | Script compilation pipeline |

### Strengths of LuaFramework for NeoTrix

1. **Mature Event System**: 30+ years of game development patterns
2. **Efficient Serialization**: Protobuf for network communication
3. **Proven Entity Management**: Entity-Component pattern
4. **Configuration-Driven**: Data tables for game balance
5. **Hot Reload**: Runtime code replacement

### Weaknesses to Avoid

1. **Global State Pollution**: Too many global variables
2. **No Type Safety**: Dynamic typing leads to runtime errors
3. **Memory Leaks**: Manual memory management required
4. **Single-Threaded**: Limited concurrency model
5. **Security Concerns**: Lua code injection risks

---

## 12. Implementation Recommendations

### Phase 1: Adopt Core Patterns (Priority: High)

- [ ] Implement Entry Point Pattern for NT-* modules
- [ ] Enhance EventBus with protobuf serialization
- [ ] Add Cache Callback Pattern for KB data loading
- [ ] Implement Timer Management for periodic tasks

### Phase 2: Network System (Priority: Medium)

- [ ] Add Protobuf schema for Knowledge Base operations
- [ ] Implement broadcast patterns (scene/world scoped)
- [ ] Create Remote Event Proxy for cross-session communication
- [ ] Add network configuration management

### Phase 3: Data Management (Priority: Medium)

- [ ] Implement field-based cache callbacks
- [ ] Add global data loading pattern
- [ ] Create configuration table system
- [ ] Add data serialization layer

### Phase 4: Entity Management (Priority: Low)

- [ ] Map LuaFramework entities to NeoTrix capabilities
- [ ] Implement Entity-Component pattern
- [ ] Add entity factory pattern
- [ ] Create entity lifecycle management

---

## 13. References

### Repository
- **Main**: https://github.com/crazytuzi/LuaFramework
- **Stars**: 125 | **Forks**: 113
- **Last Updated**: Active development

### Key Files Analyzed
- `appEntry.lua` (500+ lines) - Application entry point
- `apiEntry.lua` (1500+ lines) - C++ API bridge
- `netconfig.lua` - Network configuration
- `event/` directory - Event system implementation
- `system/` directory - Game systems

### Related Resources
- Cocos2d-x Lua Framework
- Protobuf for Lua
- LuaSocket library
- Lua event-driven architecture patterns

---

*Analysis completed: 2026-09-12*  
*Next steps: Implement recommended patterns in NeoTrix NT-ACT and NT-MEMORY modules*

# Legend of Mir 2 — Source Code Landscape Report

> Generated 2026-09-13 from GitHub and web search across 10+ queries.

---

## 1. Repository List (Ranked by Stars / Importance)

### Tier 1 — High Stars, Production-Grade

| # | Repo | Stars | Language | License | Description |
|---|------|-------|----------|---------|-------------|
| 1 | [etorth/mir2x](https://github.com/etorth/mir2x) | **523** | C++ (C++23) | — | Cross-platform MMORPG client+server. Actor-model parallelism via C++ coroutines. SDL3/FLTK/asio/lua. Builds on Linux & Windows. |
| 2 | [mirbeta/OpenMir2](https://github.com/mirbeta/OpenMir2) | **389** | C# (.NET) | MIT | Legend of Mir 2 game server. 3001 commits. Production gateway architecture (LoginGate→LoginSvr→DBSvr, GameGate→GameSvr). |
| 3 | [c-zhuo/Mir2](https://github.com/c-zhuo/Mir2) | **195** | JavaScript (Canvas) + Node.js | — | **HTML5 Canvas recreation** of Mir2. Uses Easycanvas library. Client-server in JS. Supports multiplayer. |
| 4 | [lzxsz/MIR2](https://github.com/lzxsz/MIR2) | **103** | Delphi (Pascal) | GPL-3.0 | Classic GameOfMir engine. Server + Client for v1.5/1.7.6. Requires Delphi 6. Original DelphiX/JSocket/TWMImage components. |

### Tier 2 — Reference Implementations (Crystal Source)

| # | Repo | Stars | Language | Description |
|---|------|-------|----------|-------------|
| 5 | [Suprcode/mir2](https://github.com/Suprcode/mir2) (canonical) | ~2K+ forks | C# | **The Crystal Source** — most widely used open-source Mir2 server+client. |
| 6 | [choby/mir2](https://github.com/choby/mir2) | fork | C# | Latest Crystal fork (2026-01-14), 2149 commits. GPL v2. |
| 7 | [MirageCode/mir2](https://github.com/MirageCode/mir2) | 6 | C# | Crystal fork with SDL rendering backend (branch: sdl). |
| 8 | [ApocalypseMir/mir2](https://github.com/ApocalypseMir/mir2) | — | C# | Crystal fork. |

### Tier 3 — Web/HTML5 Implementations

| # | Repo | Stars | Language | Description |
|---|------|-------|----------|-------------|
| 9 | [shellohunter/mir2-web](https://github.com/shellohunter/mir2-web) | 0 (fork of Zombieliu/mir2) | Rust + Next.js + Bevy WASM | **Modern web reimplementation** — WebGPU/WebGL2, Next.js frontend, Rust Gateway + Simulation backend. 468 commits. |
| 10 | [jootm2/client](https://github.com/jootm2/client) | 3 | JavaScript (PixiJS) | HTML5 port of "小火炬" (XiaoHuoJu) Mir2 engine client. Uses PixiJS for rendering. Apache-2.0. |
| 11 | [chenzhuo1992/Mir2](https://github.com/chenzhuo1992/Mir2) | — | JavaScript | Same as c-zhuo/Mir2 (original author's repo). |

### Tier 4 — Mir3 and Other Versions

| # | Repo | Stars | Language | Description |
|---|------|-------|----------|-------------|
| 12 | [dongzheqi/mir3-zircon](https://github.com/dongzheqi/mir3-zircon) | — | C# | Legend of Mir 3 Zircon Source. Client, Server, RenderingCore, PluginCore, etc. |
| 13 | [ufaith/mir3](https://github.com/ufaith/mir3) | 4 | Delphi | Mir3 client only. Works with existing Mir3 server files. |
| 14 | [wpzwdz/mir-3d](https://github.com/wpzwdz/mir-3d) | 0 | C# | Mir Eternal (Mir 3D) — AccountServer, GameServer, Launcher, Library. |
| 15 | [jayandradeph/Mir4](https://github.com/jayandradeph/Mir4) | 0 | C++/Unreal? | Legend of Mir 4 Topaz Source. |

### Tier 5 — Server Infrastructure

| # | Repo | Stars | Language | Description |
|---|------|-------|----------|-------------|
| 16 | [mirbeta/MirServer](https://github.com/mirbeta/MirServer) | 49 | Delphi | Mir2 server files: CloudGate, DBServer, LoginGate, LoginSrv, Mir200, RunGate, SelGate. |
| 17 | [pangliang/mirserver-go](https://github.com/pangliang/mirserver-go) | — | Go | Mir server in Go (传奇服务器Go语言实现). |

### Tier 6 — Tools / Utilities

| # | Repo | Stars | Language | Description |
|---|------|-------|----------|-------------|
| 18 | [XadillaX/wil-viewer](https://github.com/XadillaX/wil-viewer) | 11 | TypeScript/Electron | Cross-platform *.wil file viewer (sprite resource format). |
| 19 | [gitcode: GameOfMirGOM](https://gitcode.com/open-source-toolkit/f8ddb) | 9 | Delphi | GameOfMir GOM engine full source code. |

---

## 2. Architecture Analysis

### Classic Mir2 Server Architecture (GameOfMir / OpenMir2)

The original Mir2 uses a **multi-gateway layered architecture**:

```
Client
  ↕ TCP
LoginGate (port 7000) — login connection forwarder
  ↕
LoginSvr — account auth, server selection
  ↕
DBSvr — database persistence layer
  ↕
SelGate (port 7100) — character selection gateway
  ↕
GameGate (port 7200) — game session gateway
  ↕
GameSvr — game logic engine (combat, movement, items, spells)
```

**Key architectural patterns:**
- **Gateway isolation**: Each gateway handles a specific phase (login → character select → gameplay)
- **DBSvr separation**: Database operations isolated from game logic
- **Packet-based protocol**: Binary packets with opcode-based dispatch
- **State machine**: Player states (login → character select → playing → trading → dead)
- **Map instance**: Each map is a separate game area with spawn/respawn logic

### Crystal Source (C#) Architecture

```
Client/
  ├── Scenes/         # Game scenes (Login, Character, Game, Trade, etc.)
  ├── Controls/       # UI controls
  ├── Network/        # TCP client, packet handling
  ├── Objects/        # Player, Monster, NPC, Item objects
  └── Libraries/      # Sprite, sound, map loading
Server/
  ├── Database/       # DB layer (MySQL/SQLite)
  ├── Network/        # TCP server, packet routing
  ├── Objects/        # Server-side entities
  ├── PVP/            # Combat system
  ├── Guilds/         # Guild system
  └── Conquest/       # Territory system
Shared/
  ├── PacketDefinitions.cs  # All packet opcodes (ServerPackets, ClientPackets)
  └── Common.cs             # Shared enums, constants
```

### mir2x (C++) Architecture

```
client/
  ├── gameclib/       # Game client library
  └── sdlmain/        # SDL3 entry point + UI
server/
  ├── cored/          # Server core daemon
  └── monoserver/     # Standalone server
common/
  ├── netio/          # asio-based networking
  ├── fs/             # File system abstraction
  └── lib/            # Shared libraries
tools/
  ├── pkgviewer/      # Resource package viewer
  ├── animaker/       # Animation tool
  └── mapeditor/      # Map editor
```

### mir2-web (Modern Web) Architecture

```
apps/
  ├── web/                    # Next.js player frontend
  ├── game-client/runtime/    # Bevy WASM (WebGPU/WebGL2) renderer
  ├── gateway/                # Rust TCP/HTTP/WebSocket server
  ├── simulation/             # Authoritative gameplay engine
  ├── admin-web/              # Operations UI
  └── admin-api/              # Management API
packages/
  ├── protocol/               # Packet definitions
  ├── game-data/              # Game data processing
  └── conversion/             # Asset conversion tools
Crystal/                      # Reference client submodule (parity testing)
```

---

## 3. Rendering System

### Classic Mir2 (Delphi / GameOfMir)
- **Engine**: DelphiX (DirectX 7 wrapper for Delphi)
- **Rendering**: Hardware-accelerated 2D sprites via DirectX
- **Sprite format**: `.wil` / `.wzl` files — packed sprite sheets
- **Layer order**: Ground tiles → Objects → Characters → Effects → UI
- **Coordinate system**: Isometric (diamond tiles, 48x24 pixel tile size at 2x)
- **Camera**: Fixed viewport centered on player character

### Crystal Source (C#)
- **Engine**: Custom C# renderer using DirectX/OpenGL (via managed wrappers)
- **Sprite loading**: Library files (.lib) containing packed sprite indices
- **Drawing**: Sprite-based with animation frames, direction-based animation
- **Effects**: Particle system for magic, weather, ambient effects

### mir2x (C++)
- **Engine**: SDL3 with OpenGL/SDL_gpu
- **Rendering**: Hardware-accelerated 2D sprite batching
- **Map**: Isometric tile rendering with off-screen culling
- **Character sprites**: Multi-part (body, weapon, hair, armor, wings) composited
- **Resolution**: Configurable, supports fullscreen/windowed

### HTML5 Canvas (c-zhuo/Mir2)
- **Engine**: [Easycanvas](https://github.com/chenzhuo1992/easycanvas) — 2D Canvas animation library
- **Rendering**: HTML5 Canvas 2D context
- **Sprites**: Cut from sprite sheets, individual PNG per animation frame
- **Map tiles**: Isometric diamond tile rendering on canvas
- **Layer order**: Tiles → items → characters → effects → UI overlay
- **Performance**: 30fps target, uses requestAnimationFrame

### mir2-web (Bevy WASM)
- **Engine**: Bevy ECS compiled to WASM
- **Backends**: WebGPU (primary) + WebGL2 (fallback)
- **Rendering**: GPU-accelerated sprite batching
- **Resource pipeline**: Crystal asset conversion tools
- **Visual parity**: Automated pixel comparison against native Crystal client

### jootm2 (PixiJS)
- **Engine**: PixiJS (WebGL 2D renderer)
- **Rendering**: WebGL sprite batching
- **Map**: Isometric tile loading from .map files
- **Sprites**: Loaded from .wil resources decoded in browser

---

## 4. Game Loop

### Classic Mir2 / Crystal
```
while (running) {
    // 1. Process network messages (async recv)
    ProcessNetworkPackets();
    
    // 2. Update game state
    UpdatePlayerPosition();    // movement interpolation
    UpdateMonsters();          // AI + position sync
    UpdateNPCs();              // dialogue state
    UpdateEffects();           // particle/animation ticks
    UpdateInventory();         // item management
    
    // 3. Handle input
    ProcessKeyboardInput();    // WASD/arrow movement, hotkeys
    ProcessMouseInput();       // click-to-move, target selection
    
    // 4. Render frame
    ClearScreen();
    DrawMapTiles();            // ground layer
    DrawObjects();             // static objects (trees, rocks)
    DrawCharacters();          // players + monsters
    DrawEffects();             // magic/weather/particles
    DrawUI();                  // HP/MP bars, inventory, chat
    FlipBuffer();              // present frame
    
    // 5. Sync frame timing
    Sleep(30);                 // ~33ms frame time (30fps)
}
```

### mir2x (C++)
Uses **actor-model** with C++23 coroutines:
- Server: Each player connection is an actor with its own coroutine
- Client: SDL event loop + render thread
- Game logic runs in server actors, client is a thin renderer
- Network messages are actor messages

### HTML5 (c-zhuo/Mir2)
```javascript
// Easycanvas game loop
const scene = new ejs.Scene(canvas);
scene.on('update', (step) => {
    // Process input
    handleKeyboard();
    handleMouse();
    
    // Update game state
    updatePlayerPosition();
    updateMonsters();
    updateProjectiles();
    
    // Update UI
    updateUI();
});

scene.on('draw', () => {
    // Clear
    ctx.clearRect(0, 0, W, H);
    
    // Draw layers
    drawGroundTiles();
    drawItems();
    drawCharacters();
    drawEffects();
    drawUI();
});

scene.start();  // Runs requestAnimationFrame loop
```

---

## 5. Map System

### Map Format
- **Tile-based**: Maps are grids of diamond-shaped tiles (isometric projection)
- **Standard size**: Maps range from 100x100 to 500x500 tiles
- **Tile size**: 48x24 pixels (at 2x zoom) or 96x48 pixels (at 1x)
- **Tile types**: Walkable (grass, dirt, stone) vs non-walkable (water, walls)
- **Cell data**: Each cell stores: tile index, object index, NPC index, movement flags

### Map File Structure (.map)
```
Header:
  - Map width (u16)
  - Map height (u16)
  
Cell array (width × height):
  - Lower tile index (u16)
  - Upper tile index (u16)  
  - Object index (u16)  // 0 = no object
  - Door index (u8)
  - Movement flags (u8) // walkable, blocked, water, etc.
```

### Map Rendering
1. Calculate visible tile range from camera position
2. Draw bottom tiles first (ground layer)
3. Draw top tiles (overlay layer, e.g., water surface)
4. Draw map objects (trees, buildings, signs) sorted by Y position
5. Cull tiles outside viewport

### mir2x Map System
- Maps stored in `mir2x_res` git repository
- Map editor tool included (`tools/mapeditor`)
- Supports map objects, NPCs, spawn points, warps

---

## 6. Sprite System

### Sprite Format (.wil / .wzl)
- **Packed sprite sheets**: Multiple animation frames in one file
- **Structure**: Each `.wil` file contains an index table + image data
- **Index entry**: offset, width, height, compressed size
- **Image data**: Raw bitmap or compressed (RLE/Zlib)
- **Animation**: Sequential frames in the sprite sheet

### Character Sprite Anatomy
```
Character sprite = Composite of multiple layers:
  ├── Body base (direction × animation × frame)
  ├── Armor / Robe overlay
  ├── Weapon (direction-specific)
  ├── Hair style
  ├── Wings / Cape
  ├── Mount
  └── Effect overlay (buff/debuff glow)
```

### Animation System
- **8 directions**: Up, Down, Left, Right, and 4 diagonals
- **Animation states**: Idle, Walk, Attack, Cast, Hit, Die, Sit
- **Frame rate**: ~8-12 FPS for character animations
- **Direction rotation**: Sprites for all 8 directions are stored sequentially

### mir2x Sprite System
- Client loads sprites from `.pkg` resource packages
- `pkgviewer` tool to inspect packages
- Sprites composited at render time (body + equipment + effects)

### HTML5 Sprite System (c-zhuo)
- Sprites pre-cut from sprite sheets into individual PNGs
- Naming convention: `char_type_direction_frame.png`
- Loaded as HTML Image objects, drawn to Canvas via `drawImage()`
- Script-based cutting from original sprite sheets

---

## 7. UI System

### Classic Mir2 UI
- **Layout**: Fixed-position panels
  - Bottom: Hotbar (F1-F8), HP/MP/Experience bars
  - Left: Minimap
  - Right: Chat window
  - Overlay panels: Inventory (F9), Character (F10), Skills (F11), Quest (F12)
- **Controls**: Custom drawn buttons, text fields, list views
- **Drag & Drop**: Items can be dragged between inventory slots
- **Chat**: Scrollable text list with command parsing (/say, /trade, etc.)

### Crystal Source UI
- **WinForms-based**: C# Windows Forms controls
- **Custom painting**: Owner-drawn controls for game-specific visuals
- **Modal dialogs**: Trade, shop, NPC dialogue windows

### mir2x UI
- **FLTK toolkit**: Cross-platform GUI widgets
- **Custom widgets**: Game-specific controls (inventory grid, skill bar)
- **IME support**: Chinese text input in fullscreen mode

### HTML5 UI (c-zhuo)
- **Canvas overlay**: UI elements drawn on Canvas
- **DOM elements**: Some UI (login, inventory) may use HTML elements
- **Touch support**: Basic click/tap handling

---

## 8. Input System

### Classic Mir2 Input
- **Keyboard**:
  - WASD / Arrow keys: Movement
  - F1-F8: Hotbar skill/item slots
  - F9-F12: Panel toggles (inventory, character, skills, quest)
  - Space: Pick up items
  - Tab: Target nearest monster
  - Enter: Open chat
  - Shift+Click: Trade/inspect player
- **Mouse**:
  - Left click: Move to position / Attack target / Interact
  - Right click: Context menu / Use skill
  - Click on minimap: Fast travel
  - Drag items: Inventory management

### HTML5 Input (c-zhuo)
- **Keyboard events**: `keydown` / `keyup` listeners
- **Mouse events**: `click`, `mousemove`, `mousedown`, `mouseup`
- **Click-to-move**: Pathfinding to clicked position
- **Auto-attack**: Click on monster to attack

### mir2x Input
- SDL3 event system
- Configurable key bindings
- IME integration for Chinese input

---

## 9. Key Code Patterns (Extractable for NeoTrix)

### Pattern 1: Packet Opcode Dispatch
```csharp
// Crystal Source pattern
switch (p.Opcode) {
    case (ushort)ServerPacketIds.MapData:
        HandleMapData(p);
        break;
    case (ushort)ServerPacketIds.ObjectPlayer:
        HandlePlayerUpdate(p);
        break;
    case (ushort)ServerPacketIds.ObjectMonster:
        HandleMonsterUpdate(p);
        break;
}
```

### Pattern 2: Sprite Compositing
```javascript
// HTML5 pattern — layer compositing
function drawCharacter(ctx, character, x, y) {
    // 1. Draw body
    drawSprite(ctx, character.body, character.direction, frame, x, y);
    // 2. Draw weapon overlay
    if (character.weapon) drawSprite(ctx, character.weapon, ...);
    // 3. Draw armor overlay  
    if (character.armor) drawSprite(ctx, character.armor, ...);
    // 4. Draw name/HP bar
    drawText(ctx, character.name, x, y - 20);
}
```

### Pattern 3: Isometric Coordinate Conversion
```javascript
// Screen → Map tile
function screenToTile(sx, sy, offsetX, offsetY) {
    const x = (sx + offsetX) / TILE_WIDTH;
    const y = (sy + offsetY) / TILE_HEIGHT;
    return { tx: Math.floor(x + y), ty: Math.floor(y - x) };
}

// Map tile → Screen position
function tileToScreen(tx, ty, offsetX, offsetY) {
    return {
        sx: (tx - ty) * TILE_WIDTH / 2 - offsetX,
        sy: (tx + ty) * TILE_HEIGHT / 2 - offsetY
    };
}
```

### Pattern 4: Gateway Architecture
```
Client → LoginGate (auth) → GameGate (gameplay) → GameSvr (logic)
                                ↕
                           SelGate (character) → DBSvr (persistence)
```
- Each gateway is a separate process/service
- Gateways forward packets, optionally filter/validate
- GameSvr is the authoritative game logic

### Pattern 5: Isometric Tile Rendering with Y-Sort
```
for (let y = minY; y <= maxY; y++) {
    for (let x = minX; x <= maxX; x++) {
        drawTile(map.getBottomTile(x, y));
        drawTile(map.getTopTile(x, y));
        drawObject(map.getObject(x, y));
    }
}
// Then draw all characters/monsters sorted by Y position
entities.sort((a, b) => a.y - b.y);
entities.forEach(e => drawEntity(e));
```

### Pattern 6: Resource Package Loading
```
.pkg / .wil file:
  Header: [magic][version][index_count]
  Index:  [offset_0][offset_1]...[offset_n]
  Data:   [sprite_0][sprite_1]...[sprite_n]
  
Loading:
  1. Open file, read header
  2. Read index table into memory
  3. On-demand: seek to offset, decompress, decode bitmap
  4. Cache decoded sprites in memory
```

---

## 10. Resource Files

### Standard Mir2 Resource Files

| File | Contents | Size |
|------|----------|------|
| `Objects1.wil` / `Objects2.wil` | Static map objects (trees, rocks, buildings) | 50-200MB |
| `Objects_rc.wil` | Additional objects | 20-50MB |
| `Mon1.wil` - `Mon20.wil` | Monster sprites (each monster = 1 file) | 10-50MB each |
| `Hum.wil` | Player character sprites (body, hair) | 100-300MB |
| `Weapon1.wil` - `Weapon30.wil` | Weapon sprites | 5-20MB each |
| `Magic.wil` / `Magic2.wil` | Spell effect animations | 20-100MB |
| `Effect.wil` | General effects (buffs, debuffs, ambient) | 10-50MB |
| `Tile.wil` | Ground tile sprites | 20-50MB |
| `Building.wil` | Building/structure sprites | 20-50MB |
| `DN.wil` | Underground/dungeon tiles | 10-30MB |
| `Item.wil` | Item icons | 10-30MB |
| `UI.wil` | UI elements (buttons, panels, icons) | 5-20MB |
| `Sound.wil` / `cQSound` | Sound effects and music | 50-200MB |
| `.map` files | Map data (one per map) | 10KB-100KB each |
| `Monster.txt` | Monster definitions | ~100KB |
| `ItemDB.txt` | Item database | ~200KB |
| `MagicDB.txt` | Spell definitions | ~100KB |
| `Setup.txt` | Server configuration | ~10KB |

### Total Asset Size
- **Full game assets**: ~2-8 GB (varies by version)
- **Minimum playable**: ~500MB (core sprites + maps)
- **mir2-web Starter pack**: Small subset for quick start (full pack ~18GB)

### Asset Resource Locations
- Original game data: From installed Mir2 client (version 1.5 or 1.7.6)
- mir2x resources: `github.com/etorth/mir2x_res` (separate repo)
- Crystal resources: Downloaded via auto-patcher from `mirfiles.co.uk`
- c-zhuo/Mir2: Assets not in repo (too large), separate download

---

## 11. Community & Resources

| Resource | URL |
|----------|-----|
| LOMCN (Legend of Mir Community Network) | https://www.lomcn.org/ |
| LOMCN Development Forum | https://www.lomcn.net/forum/categories/legend-of-mir-development.430 |
| LOMCN Wiki | https://www.lomcn.org/wiki/index.php/Main_Page |
| Crystal Source (canonical) | https://github.com/Suprcode/mir2 |
| mir2 GitHub Topic | https://github.com/topics/mir2 |
| mir2x Wiki | https://github.com/etorth/mir2x/wiki |

---

## 12. Summary: Which Repo to Use for What

| Goal | Best Repo | Why |
|------|-----------|-----|
| **Web/browser game** | c-zhuo/Mir2 or shellohunter/mir2-web | HTML5 Canvas or WebGPU/WASM |
| **C++ performance** | etorth/mir2x | Modern C++23, actor-model, cross-platform |
| **C# server** | mirbeta/OpenMir2 | MIT license, production-ready gateway architecture |
| **Classic Delphi source** | lzxsz/MIR2 | Original GameOfMir engine, full client+server |
| **Reference implementation** | Suprcode/mir2 (Crystal) | Most widely used, best documented |
| **Mir3** | dongzheqi/mir3-zircon | Mir3 Zircon source |
| **Sprite tools** | XadillaX/wil-viewer | .wil file viewer |
| **Map editing** | etorth/mir2x tools | Built-in map editor |

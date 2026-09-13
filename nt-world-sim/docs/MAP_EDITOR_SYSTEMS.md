# Map Editor Systems & Tile-Based Game Maps

Comprehensive reference for tile-based map formats, auto-tile systems, map features, and procedural generation algorithms.

---

## 1. Tiled Map Editor (TMX/JSON)

Tiled is the de facto standard for 2D tile-based map editing. Maps export as XML (TMX) or JSON.

### 1.1 Map Format Structure (JSON)

```json
{
  "width": 100,              // tile columns
  "height": 75,              // tile rows
  "tilewidth": 32,           // pixel width per tile
  "tileheight": 32,          // pixel height per tile
  "orientation": "orthogonal",
  "renderorder": "right-down",
  "infinite": false,
  "layers": [],
  "tilesets": [],
  "nextobjectid": 12,
  "tiledversion": "1.11.0"
}
```

**Key fields:**
- `orientation`: `"orthogonal"`, `"isometric"`, `"staggered"`, or `"hexagonal"`
- `renderorder`: `"right-down"` (default), `"right-up"`, `"left-down"`, `"left-up"`
- `infinite`: When `true`, data is stored in chunks, not a single flat array
- `hexsidelength`: Required for hexagonal maps
- `staggeraxis` / `staggerindex`: For staggered/hex layouts

### 1.2 Layer Types

| Type | Purpose | Key Fields |
|------|---------|------------|
| `tilelayer` | Grid of tile references (GIDs) | `data`, `compression`, `encoding` |
| `objectgroup` | Freely positioned objects | `objects[]`, `draworder` |
| `imagelayer` | Single background/foreground image | `image`, `transparentcolor` |
| `group` | Nested layer organization | `layers[]` (recursive) |

**Tile layer encoding:**
- `encoding`: `"csv"` (default) or `"base64"`
- `compression`: `"zlib"`, `"gzip"`, `"zstd"`, or empty
- Data is an array of Global Tile IDs (GIDs) or base64-encoded binary

**Object group draw order:**
- `"topdown"`: Objects sorted by Y-coordinate (default)
- `"index"`: Manual stacking order

### 1.3 Layer Properties

```
id              int     Unique across all layers
name            string  User-assigned name
visible         bool    Show/hide in editor and game
opacity         float   0.0 to 1.0
offsetx/y       double  Pixel offset for parallax scrolling
tintcolor       string  Hex color multiplied with layer graphics
locked          bool    Lock in editor (since 1.8.2)
mode            string  Blend mode (since 1.12)
class           string  Custom class (since 1.9)
```

### 1.4 Object Types

| Shape | Description | Common Use |
|-------|-------------|------------|
| Rectangle | Axis-aligned rectangle | Triggers, zones, spawn areas |
| Ellipse | Ellipse or circle | Area-of-effect, circular regions |
| Point | Single point (since 1.1) | Spawn points, waypoints |
| Polygon | Closed shape | Complex collision boundaries |
| Polyline | Open path | Patrol routes, dialogue paths |
| Tile | Freely placed tile graphic | Decorations, dynamic objects |
| Text | Text label (since 1.0) | Notes, in-game signs |

**Object fields:**
```
id          int     Unique across all objects
name        string  Editor name
type        string  Custom type/class
x, y        int     Position in pixels
width       int     Width in pixels (ignored if gid set)
height      int     Height in pixels
rotation    float   Degrees clockwise
gid         int     Tile ID (if object is a tile)
visible     bool
properties  array   Custom properties
polyline    array   Points for polyline objects
polygon     array   Points for polygon objects
```

### 1.5 Tile Properties

Tile properties are stored per-tileset and inherited by individual tiles:

```
objectgroup    Layer     Collision shapes for this tile
probability    double    Weight for random replacement
terrain        array[4]  Terrain index for each corner (47-tile system)
properties     array     Custom properties (walkable, speed, damage, etc.)
animation      array     Frame sequence for animated tiles
```

**Common custom properties for tiles:**
- `walkable` (bool) — Can the player walk on this tile?
- `speed` (float) — Movement speed multiplier
- `damage` (int) — Damage per turn on this tile
- `swim` (bool) — Requires swimming to traverse
- `climb` (bool) — Requires climbing ability
- `encounter_rate` (float) — Random encounter probability

### 1.6 Map Example (Minimal)

```json
{
  "width": 4,
  "height": 4,
  "tilewidth": 32,
  "tileheight": 32,
  "orientation": "orthogonal",
  "renderorder": "right-down",
  "layers": [
    {
      "data": [1, 2, 1, 2, 3, 1, 3, 1, 2, 2, 3, 3, 4, 4, 4, 1],
      "height": 4,
      "name": "ground",
      "opacity": 1,
      "type": "tilelayer",
      "visible": true,
      "width": 4
    },
    {
      "draworder": "topdown",
      "name": "entities",
      "objects": [
        {
          "id": 1,
          "name": "player_spawn",
          "type": "spawn_point",
          "x": 64,
          "y": 64,
          "width": 32,
          "height": 32
        }
      ],
      "type": "objectgroup",
      "visible": true
    }
  ],
  "tilesets": [
    {
      "firstgid": 1,
      "image": "tileset.png",
      "imagewidth": 512,
      "imageheight": 512,
      "tilecount": 256,
      "columns": 16,
      "tilewidth": 32,
      "tileheight": 32
    }
  ]
}
```

---

## 2. Tile Properties & Collision System

### 2.1 Property Storage Pattern

In Tiled, properties attach to tiles via the tileset definition:

```json
{
  "tilesets": [{
    "tiles": {
      "5": {
        "properties": [
          { "name": "walkable", "type": "bool", "value": false },
          { "name": "damage", "type": "int", "value": 10 }
        ],
        "objectgroup": {
          "objects": [{
            "x": 0, "y": 0, "width": 32, "height": 32,
            "polygon": [{"x":0,"y":0},{"x":32,"y":0},{"x":32,"y":32},{"x":0,"y":32}]
          }]
        }
      }
    }
  }]
}
```

### 2.2 Collision Shapes

Each tile can have collision polygons defined in its `objectgroup`:

- **Full tile**: Rectangle covering the entire 32x32 area
- **Partial**: Smaller rectangle, polygon, or ellipse for precise collision
- **Multiple shapes**: Array of collision objects for complex geometry

**Godot 4.x collision approach:**
```
collision_layer   int     Physics layer bitmask
collision_mask    int     Physics mask bitmask
collision_animatable bool  Sync to physics tick (for moving platforms)
```

**Unity Tilemap Collider 2D:**
- `TilemapCollider2D` generates shapes per tile
- `CompositeCollider2D` merges neighboring colliders for performance
- `ColliderType`: Sprite shape, Grid shape, or None
- Static rigidbody reduces physics calculations

### 2.3 RPG Maker Tile Flags

RPG Maker MZ stores collision as bitflags per tile in the `flags` array:

| Bit | Flag | Effect |
|-----|------|--------|
| 0x0001 | Impassable (down) | Block downward movement |
| 0x0002 | Impassable (left) | Block leftward movement |
| 0x0004 | Impassable (right) | Block rightward movement |
| 0x0008 | Impassable (up) | Block upward movement |
| 0x0010 | Impassable (down-left) | Block diagonal |
| 0x0020 | Impassable (down-right) | Block diagonal |
| 0x0040 | Impassable (up-left) | Block diagonal |
| 0x0080 | Impassable (up-right) | Block diagonal |
| 0x0100 | Ladder | Can climb up/down |
| 0x0200 | Bush | Passable with bush graphic |
| 0x0400 | Counter | Can interact over |
| 0x0800 | Platform | Always passable from below |

---

## 3. Auto-Tile System

### 3.1 Bitmask Methods

#### 4-Bit Autotiling (Cardinal Neighbors Only)

Checks 4 cardinal directions → 4-bit mask → 16 unique tiles.

```
Bit layout (N=8, E=4, S=2, W=1):
  N
W   E
  S

Example: Tile surrounded on N, E, S → mask = 8+4+2 = 14
```

**Limitation**: No inner corner handling. Concave turns look wrong.

#### 8-Bit Autotiling (Blob/47-Tile Set)

Checks all 8 neighbors (4 cardinal + 4 diagonal) → 8-bit mask → 256 raw values, reducible to **47 unique tiles**.

```
Bit layout:
7 | 0 | 1
--+---+--
6 |   | 2
--+---+--
5 | 4 | 3

NW=128, N=1, NE=2, E=4, SE=8, S=16, SW=32, W=64
```

**Why 47 tiles from 256?** The blob reduction eliminates physically impossible combinations (e.g., a tile that has a corner but no supporting orthogonal neighbors).

### 3.2 Terrain Transition System

Tiled uses a terrain system with per-corner assignment:

```
Each tile has 4 corner terrain indices:
[TL, TR, BL, BR]

When painting, the terrain brush checks corners and
automatically selects the correct transition tile.
```

**Advantages:**
- Artist only paints terrain type, editor picks the correct tile
- Smooth transitions between different terrain types
- Works with 16-tile (4-bit) or 47-tile (8-bit) sets

### 3.3 Wang Tiles

Wang tiles assign colors/labels to each edge. Tiles can only be placed adjacent if shared edges match.

**2-edge 2-corner Wang set** = Blob tileset (47 tiles):
- Each edge can be one of 2 colors
- Each corner can be one of 2 colors
- Produces smooth inner and outer corners

**Wang set in Tiled:**
```
terrains: [
  { "name": "grass", "tile": 0 },
  { "name": "water", "tile": 16 }
]
// Each tile gets: [NW_edge, NE_edge, SE_edge, SW_edge] color indices
```

### 3.4 Engine Implementations

| Engine | Method | Tiles | Notes |
|--------|--------|-------|-------|
| Godot TileMap | Bitmask (3x3 min or full) | 16 or 47 | Configurable per terrain |
| Tiled editor | Terrain/Wang sets | 16-47 | Editor-assisted placement |
| RPG Maker MZ | 6x8 mini-tile layout | 48 (47+1 dup) | Quarter-based assembly |
| GameMaker | 47-tile sprite | 47 | Hash table lookup |

### 3.5 Comparison

| Technique | Tiles Needed | Corners? | Art Complexity | Best For |
|-----------|-------------|----------|----------------|----------|
| 4-bit bitmask | 16 | No | Low | Prototyping, simple styles |
| 8-bit blob | 47 | Yes | Medium | Polished 2D games |
| Dual grid | 16 | Yes | Low | Smooth terrain transitions |
| Mini-tile (2x2) | 5 shapes | Yes | Low | RPG Maker style |
| Wang set | 16-47 | Yes | Medium | Organic terrain variety |

---

## 4. Map Features

### 4.1 Fog of War

#### Three States
1. **Unexplored/Hidden** (0) — Completely black, no information
2. **Explored/Fogged** (1) — Previously seen, now dimmed (50% opacity)
3. **Visible/Clear** (2) — Currently in line of sight

#### Implementation Approaches

**Tile-Based FoW:**
```
visibility_map[w][h] = { HIDDEN, FOGGED, VISIBLE }

For each entity with vision:
  1. Compute line-of-sight tiles (raycasting/shadowcasting)
  2. Mark visible tiles as VISIBLE
  3. After update: VISIBLE→FOGGED, keep HIDDEN as HIDDEN
```

**FOV Algorithms (from BenMakesGames/FoV):**
- **Diamond Walls**: Fast, reveals more tiles, good default
- **Milazzo's Beveled**: Intuitive LOS for single-tile walls
- **Raycasting**: Slowest, occasionally unintuitive
- **Shadowcasting**: Recursive, very fast for large radii

**Rendering techniques:**
- Separate fog layer (semi-transparent texture over map)
- Render-to-texture at 1 pixel per tile, scale with bilinear filtering for smooth edges
- Per-tile alpha: VISIBLE=0%, FOGGED=50%, HIDDEN=100%

**Performance optimization:**
- Only update tiles within entity vision radius
- Cache fog state in a separate 2D array
- Use GPU shaders for smooth fog edges (transparent black mask + LOS circles as mask)

#### Dota 2 Constraints (Valve reference)
- Terrain must be flat outside of stairs/ramps
- Max 32 elevation levels for fog LOS calculation
- No traversable terrain below Z=0
- Max map size: 16384x16384 units

### 4.2 Minimap Rendering

#### Architecture
```
Minimap:
  - Pre-rendered background texture (full map at reduced scale)
  - Viewport indicator rectangle (white/red outline)
  - Entity markers (dots for NPCs, enemies, objectives)
  - Click-to-navigate interaction
```

**Performance critical**: Pre-render the full map as a single texture at connection/load time. Never draw 65,536 individual dots per frame.

**Coordinate mapping:**
```
WorldToMinimap(worldX, worldY):
  miniX = (worldX / worldWidth) * minimapWidth
  miniY = (worldY / worldHeight) * minimapHeight

MinimapToWorld(miniX, miniY):
  worldX = (miniX / minimapWidth) * worldWidth
  worldY = (miniY / minimapHeight) * worldHeight
```

**Viewport indicator:**
```
rect.x = (camera.x / worldWidth) * minimapWidth
rect.y = (camera.y / worldHeight) * minimapHeight
rect.w = (camera.width / worldWidth) * minimapWidth
rect.h = (camera.height / worldHeight) * minimapHeight
```

**Integration with FoW:**
- Minimap respects fog state: HIDDEN tiles = black, FOGGED = dim, VISIBLE = bright
- Pre-render only the visible/explored portion

### 4.3 Camera Bounds

#### Standard Camera System
```
Camera:
  x, y         current position (top-left corner)
  width, height viewport size
  maxX, maxY    clamping limits

maxX = mapWidth * tileWidth - cameraWidth
maxY = mapHeight * tileHeight - cameraHeight

// Clamp
camera.x = clamp(camera.x, 0, maxX)
camera.y = clamp(camera.y, 0, maxY)
```

#### Scrollable Tilemap Rendering
```
startCol = floor(camera.x / tileWidth)
endCol = startCol + ceil(camera.width / tileWidth) + 1
startRow = floor(camera.y / tileHeight)
endRow = startRow + ceil(camera.height / tileHeight) + 1

offsetX = -camera.x + startCol * tileWidth
offsetY = -camera.y + startRow * tileHeight

for c in startCol..endCol:
  for r in startRow..endRow:
    tile = map.getTile(c, r)
    drawTile(tile, (c - startCol) * tileWidth + offsetX,
                    (r - startRow) * tileHeight + offsetY)
```

#### Non-Rectangular Bounds
For circular or polygon-shaped camera limits:
- Use polygon containment test (point-in-polygon)
- Or distance check for circular stages
- Godot Camera2DLimit plugin reads TileMapLayer bounds automatically

### 4.4 Region Transitions

Region transitions connect separate map sections:

```json
{
  "name": "to_forest",
  "type": "region_exit",
  "x": 0, "y": 128,
  "width": 32, "height": 64,
  "properties": [
    { "name": "target_map", "type": "string", "value": "forest.json" },
    { "name": "target_x", "type": "int", "value": 480 },
    { "name": "target_y", "type": "int", "value": 32 }
  ]
}
```

**Implementation pattern:**
1. Player overlaps trigger rectangle
2. Load target map data
3. Transfer entity state (position, inventory, HP)
4. Render new map, despawn old entities
5. Fade transition for polish

---

## 5. RPG Maker Map Format

### 5.1 Map Data Structure (MZ/MV)

RPG Maker stores maps as JSON with these key structures:

```
Map:
  width, height        tile dimensions (max 256x256)
  tilesetId            reference to tileset
  scrollType           0=loop, 1=scroll horizontal, 2=scroll vertical, 3=both
  battleback1Name      battle background (ground)
  battleback2Name      battle background (sky)
  encounterList        array of troop encounter data
  encounterStep        average steps between encounters
  data                 flat array of tile IDs (5 layers × width × height)
  events               array of event objects
```

**Data array layout:**
```
data = [layer0_tiles..., layer1_tiles..., layer2_tiles..., layer3_tiles..., layer4_tiles...]

Index for tile at (x, y, layer):
  index = layer * (width * height) + y * width + x
```

**Layer order (bottom to top):**
- Layer 0: Ground tiles (A5/A2 autotiles)
- Layer 1: Ground detail (A1 animated water, A3 walls)
- Layer 2: Structure (B/C/D/E tiles)
- Layer 3: Upper overlay
- Layer 4: Touch/trigger layer

### 5.2 Tileset Sheets (A-E)

| Sheet | Content | Autotile? |
|-------|---------|-----------|
| A1 | Animated water/waterfall | Yes (animated) |
| A2 | Ground autotiles | Yes (47-tile) |
| A3 | Wall autotiles | Yes (wall-top/side) |
| A4 | Wall-top and wall-side | Yes (47-tile) |
| A5 | Normal tiles (no autotile) | No |
| B | Background/terrain tiles | No |
| C | Character/event tiles | No |
| D | Damage floor tiles | No |
| E | Special/overlay tiles | No |

### 5.3 RPG Maker MZ Tilemap Rendering

RPG Maker MZ uses PIXI.js with a custom `Tilemap` renderer:
- WebGL texture atlas approach
- Internal textures packed into 1024x1024 atlases
- 4 texture slots per layer for autotile compositing
- `setData(width, height, data)` — flat array of tile IDs
- `setBitmaps(bitmaps)` — tileset images
- `animationCount` — drives autotile frame updates

### 5.4 Events (NPCs, Triggers)

Events are stored separately from tile data:

```
Event:
  id              unique ID
  name            editor name
  x, y            tile position
  pages[]         array of event pages (conditions → commands)

EventPage:
  conditions:     switch, variable, actor, timer checks
  image:          character sprite
  trigger:        0=action button, 1=touch, 2=player touch,
                  3=event touch, 4=autorun, 5=parallel
  list[]:         array of event commands (dialogue, transfer, etc.)
```

---

## 6. Procedural Map Generation

### 6.1 Perlin Noise Terrain

**Use case**: Heightmaps, biomes, organic terrain, infinite worlds.

```
1. Generate 2D Perlin/Simplex noise at grid resolution
2. Sample noise at each tile position
3. Apply thresholds for biome assignment:
   - value < 0.3  → water
   - 0.3-0.45     → sand/beach
   - 0.45-0.7     → grass
   - 0.7-0.85     → forest
   - > 0.85       → mountain/snow
4. Smooth edges between biomes
```

**Key parameters:**
- `octaves`: Detail layers (3-6 typical)
- `persistence`: Amplitude decay per octave (0.5 typical)
- `lacunarity`: Frequency multiplier per octave (2.0 typical)
- `scale`: Zoom level (lower = more zoomed out)
- `seed`: Deterministic reproduction

**Godot FastNoiseLite:**
```gdscript
var noise = FastNoiseLite.new()
noise.seed = 12345
noise.noise_type = FastNoiseLite.TYPE_SIMPLEX_SMOOTH
noise.frequency = 0.02
noise.fractal_octaves = 4
var value = noise.get_noise_2d(x, y)  # returns -1.0 to 1.0
```

**Performance**: Sample noise once into an array. Never sample per-frame. Generate into Image or Array, then read from memory.

### 6.2 Cellular Automata Caves

**Use case**: Organic cave systems, dungeon vegetation, natural-looking terrain.

**Algorithm:**
```
1. Initialize grid randomly (40-50% floor fill)
2. Repeat N times (typically 4-6):
   For each cell:
     count = count adjacent floor tiles (8 neighbors)
     if count > 4: cell = FLOOR
     if count < 4: cell = WALL
     else: cell unchanged
3. Run flood-fill to detect connected regions
4. Remove small disconnected regions (< threshold)
5. Place rooms in largest open areas
```

**Rules variant (B5/S45):**
- Birth: Cell becomes floor if exactly 5 neighbors are floor
- Survival: Floor cell stays floor if 4 or 5 neighbors are floor

**Post-processing:**
- Flood fill to find connected components
- Room splitter to enforce min/max room sizes
- Door placement between rooms
- Staircase/exit placement

### 6.3 BSP Dungeon Generation

**Use case**: Structured indoor levels, room-corridor dungeons, mansions.

**Algorithm:**
```
1. Start with full map rectangle
2. Recursively split:
   - Choose split direction (horizontal/vertical)
   - Choose split point (random within bounds)
   - Split into two sub-regions
   - Stop when region < min_size
3. Place rooms inside each leaf node:
   - Random rectangle within region
   - Enforce min/max room size
   - Ensure no overlap
4. Connect rooms:
   - Build BSP tree adjacency graph
   - Connect sibling rooms via corridors
   - Use L-shaped corridors (horizontal + vertical)
5. Post-process:
   - Add doors at corridor-room junctions
   - Place stairs, loot, spawn points
```

**Parameters:**
- `min_room_size`: Minimum room width/height (e.g., 6)
- `max_room_size`: Maximum room width/height
- `split_offset`: Random variation in split point
- `corridor_width`: Width of connecting corridors

### 6.4 Wave Function Collapse (WFC)

**Use case**: Constraint-based tile placement, preserving local structure from examples.

**Algorithm:**
```
1. Initialize: Every cell has ALL possible tiles (superposition)
2. Observe: Find cell with minimum entropy (fewest possibilities)
3. Collapse: Randomly select a tile (weighted by probability)
4. Propagate: For each neighbor:
   - Get compatible tiles (matching edge constraints)
   - Intersect with neighbor's possibilities
   - If changed, add neighbor to propagation queue
5. Repeat from step 2 until all cells collapsed
6. If contradiction (no valid tiles): restart or backtrack
```

**Tile constraints (sockets):**
```
Each tile has 4 sockets: [N, E, S, W]
Socket defines what can neighbor on that side.

Example:
  Grass-tile: N=grass, E=grass, S=grass, W=grass
  Path-tile:  N=grass, E=path,  S=grass, W=path
  
  → Path can only connect to grass on N/S, path on E/W
```

**Entropy calculation:**
```
entropy = -Σ (w_i / W_total) * log(w_i / W_total)

where w_i = weight of tile i, W_total = sum of all weights
Lower entropy = fewer possibilities = collapse first
```

**Complexity:**
- Initialize: O(w × h × t) where t = tile count
- Find min entropy: O(w × h)
- Propagate: O(w × h × t) worst case
- Total: O(w × h × t × i) where i = iterations

**Features:**
- Socket-based adjacency rules (like matching puzzle pieces)
- Weighted random selection for natural variation
- Contradiction detection with retry/backtrack
- Deterministic with seed

### 6.5 Algorithm Comparison

| Algorithm | Best For | Structure | Speed | Controllability |
|-----------|----------|-----------|-------|-----------------|
| Perlin Noise | Terrain, biomes | Organic, continuous | Fast | Medium (thresholds) |
| Cellular Automata | Caves, vegetation | Organic, clustered | Fast | Low-Medium |
| BSP | Indoor dungeons | Rectangular rooms | Fast | High |
| Drunkard's Walk | Tunnels, rivers | Winding paths | Medium | Low |
| WFC | Constraint-based tiles | Matches examples | Slow | High (via constraints) |
| L-System | Trees, branching | Fractal, natural | Fast | Medium |

### 6.6 Hybrid Approaches

Combine algorithms for better results:

```
1. BSP for room layout → define regions
2. Cellular Automata within BSP rooms → organic detail
3. WFC for tile decoration → constraint-based polish
4. Perlin noise for terrain overlay → elevation/biome

Pipeline example:
  BSP rooms → CA caves in unused space → WFC tile decoration → Perlin heightmap overlay
```

---

## 7. Camera & Rendering Systems

### 7.1 Scrolling Tilemap Rendering

**Core loop:**
```javascript
const startCol = Math.floor(camera.x / tileWidth);
const endCol = Math.ceil((camera.x + camera.width) / tileWidth);
const startRow = Math.floor(camera.y / tileHeight);
const endRow = Math.ceil((camera.y + camera.height) / tileHeight);

const offsetX = -camera.x + startCol * tileWidth;
const offsetY = -camera.y + startRow * tileHeight;

for (let c = startCol; c <= endCol; c++) {
    for (let r = startRow; r <= endRow; r++) {
        const tile = getTile(c, r);
        if (tile !== 0) {
            drawTile(tile,
                (c - startCol) * tileWidth + offsetX,
                (r - startRow) * tileHeight + offsetY
            );
        }
    }
}
```

### 7.2 Isometric Rendering

**Coordinate conversion (orthogonal → isometric):**
```
worldX = (tileX - tileY) * (tileWidth / 2)
worldY = (tileX + tileY) * (tileHeight / 2)
```

**Isometric → orthogonal:**
```
tileX = floor((worldX / halfW + worldY / halfH) / 2)
tileY = floor((worldY / halfH - worldX / halfW) / 2)
```

### 7.3 Parallax Scrolling

Layers render at different scroll speeds for depth illusion:

```
layer1.parallax = { x: 0.5, y: 0.5 }  // background (slow)
layer2.parallax = { x: 1.0, y: 1.0 }  // main gameplay
layer3.parallax = { x: 1.5, y: 1.5 }  // foreground (fast)
```

**Rendering offset:**
```
offsetX = (camera.x - parallaxOrigin.x) * parallax.x + layerOffset.x
```

---

## 8. Engine-Specific Implementations

### 8.1 Godot 4.x TileMap

**Node hierarchy:**
```
TileMap (root)
  ├── TileMapLayer (ground)
  ├── TileMapLayer (details)
  ├── TileMapLayer (collision)
  └── TileMapLayer (overlay)
```

**Key APIs:**
```gdscript
tile_map.set_cell(layer, coords, source_id, atlas_coords, alternative_tile)
tile_map.get_cell_tile_data(layer, coords)
tile_map.get_used_cells(layer)
tile_map.get_used_rect()
tile_map.map_to_world(map_position)
tile_map.world_to_map(world_position)
```

**Collision setup:**
- TileSet physics layers with per-tile collision shapes
- `collision_animatable` for moving platforms
- Quadrant-based rendering optimization (default 16 tiles)

### 8.2 Unity Tilemap

**Components:**
- `Tilemap` — Data container
- `TilemapRenderer` — Rendering (chunk culling for performance)
- `TilemapCollider2D` — Physics collision per tile
- `CompositeCollider2D` — Merges colliders for optimization
- `TilemapAnimator` — Tile animation

**Optimization:**
- Chunk culling bounds (Auto or Manual)
- Composite colliders reduce physics body count
- Static rigidbody on tilemap for batch optimization

### 8.3 RPG Maker MZ

**Rendering pipeline:**
1. Tilemap data as flat array (5 layers)
2. PIXI.js WebGL renderer
3. Custom `rpgtilemap` plugin for tile rendering
4. Autotile compositing via 4 texture slots per layer
5. Sprite children for events/characters layered on top

---

## 9. Map Data Transfer Patterns

### 9.1 Serialization

```
Save:  MapData → JSON/binary → file
Load:  file → JSON/binary → MapData → Render

MapData struct:
  width, height
  tileWidth, tileHeight
  layers[]: {
    type: TileLayer | ObjectLayer
    data/objects
    properties
  }
  tilesets[]: { image, tiles, properties }
  events[]: { x, y, pages[] }
```

### 9.2 Chunked Loading (Infinite Maps)

For large/infinite maps:
```
divide map into chunks (e.g., 16x16 tiles)
load chunks near camera position
unload chunks beyond distance threshold
store chunk data in memory cache or disk
```

### 9.3 Versioning

Track format versions for backward compatibility:
```
tiledversion: "1.11.0"   // editor version
version: "1.10"          // format version
// Always check version before parsing fields
```

---

## 10. Reference Links

| Resource | URL |
|----------|-----|
| Tiled JSON Format | https://doc.mapeditor.org/en/stable/reference/json-map-format |
| Tiled Manual | https://doc.mapeditor.org/en/stable/manual/ |
| Tiled Layers | https://doc.mapeditor.org/en/stable/manual/layers |
| Godot TileMap | https://docs.godotengine.org/en/4.7/classes/class_tilemap.html |
| Godot TileSet | https://docs.godotengine.org/en/4.7/classes/class_tileset.html |
| Unity Tilemap | https://docs.unity3d.com/6000.0/Manual/tilemaps/work-with-tilemaps/ |
| RPG Maker MZ Tilemap | https://rpgmakerofficial.com/product/mz/rmmz_api/Tilemap.html |
| RPG Maker MZ Map Props | https://rpgmakerofficial.com/product/MZ_help-en/01_07_03.html |
| Red Blob Games Autotile | https://www.redblobgames.com/articles/autotile/ |
| Blob Tileset (BorisTheBrave) | http://www.boristhebrave.com/permanent/24/06/cr31/stagecast/wang/blob.html |
| FoV Algorithms | https://github.com/BenMakesGames/FoV |
| RPG Maker Autotile Spec | https://github.com/EllyeFS/tileset-format-specs-fork |
| WFC Algorithm | https://gridbugs.org/wave-function-collapse/ |
| MDN Tilemap Scrolling | https://developer.mozilla.org/en-US/docs/games/techniques/tilemaps/square_tilemaps_implementation_colon__scrolling_maps |

---

*Document created: 2026-09-12*
*Sources: Tiled docs, Godot/Unity/RPG Maker documentation, Red Blob Games, GitHub implementations*

# Working HTML5/JavaScript RPG Games — Source Code Study

> Comprehensive analysis of actual working HTML5 RPG games found in the wild.
> Focus: architecture patterns, rendering, input, state, collision, UI.

---

## Games Found & Studied

### 1. Shattered Vale RPG

- **URL**: https://github.com/goldipl/Shattered-Vale-RPG
- **Demo**: https://goldipl.github.io/Shattered-Vale-RPG/
- **Tech**: Vanilla JavaScript, HTML5 Canvas, no engine, no build step, no dependencies
- **Size**: 27 JS files, ~109x236 tile map
- **Status**: Complete, playable, free

**Why it works well**: Zero dependencies. Every sprite is procedurally drawn in code. The architecture is clean: `core/` (game loop, input, combat), `entities/` (player, enemy, NPC), `systems/` (camera, particles, dialogue, inventory), `world/` (tilemap), `ui/` (HUD, screens), `config/` (pure data), `sprites/` (procedural art). The game state object is passed explicitly through the entire call chain — no module closes over game.js locals.

**Key patterns**:
- State object passed explicitly (no singletons, no globals except constants)
- Depth-sorted rendering (all drawables sorted by Y before drawing)
- Offscreen canvas baking for static tiles
- DOM HUD + Canvas HUD hybrid approach
- Procedural pixel art (no external image assets)
- Versioned save system with forward migrations

### 2. Claudicus

- **URL**: https://github.com/brian-benzinger/claudicus
- **Demo**: https://claudicus.vercel.app
- **Tech**: TypeScript, HTML5 Canvas, esbuild, Vitest
- **Size**: ~15 source files, 960x640 canvas
- **Status**: Complete, playable, MIT license

**Why it works well**: Everything is code-drawn — sprites, tiles, music (Web Audio API synthesizer). TypeScript catches game logic bugs at compile time. State machine in `main.ts` drives all transitions. 95% test coverage enforced. Versioned saves with forward migrations.

**Key patterns**:
- 120fps frame cap with delta-time check
- Typed GameState enum for all transitions
- Smooth tile-to-tile movement with ease-out interpolation
- InputManager with separate held/justPressed buffers
- MusicEngine with Web Audio API oscillator synthesis

### 3. PocketMockster

- **URL**: https://github.com/KeigoShimadaCC/PocketMockster
- **Tech**: TypeScript, HTML5 Canvas, Vite, Playwright E2E
- **Size**: 20 creatures, 10-type effectiveness, full battle system
- **Status**: Complete, playable with E2E tests

**Why it works well**: Data-driven content pipeline (creatures, moves, maps, tiles all in separate data files). AI-agent debugger via MCP. Deterministic seed RNG for reproducible testing. Classic RPG damage formula with IVs, EXP curves, evolutions.

### 4. Starfall Chronicles

- **URL**: https://github.com/mtaylor-create/teamsDemo-RPG2
- **Tech**: TypeScript, HTML5 Canvas, Vite, no game engine
- **Size**: 640x480 canvas, turn-based combat, multiple screens
- **Status**: Demo v0.3 (Act 1 playable)

**Why it works well**: Clean separation between engine (`engine/`), screens (`screens/`), world (`world/`), UI (`ui/`), and data (`data/`). Screen registry pattern. JSON data files for characters, enemies, items, dialogue. Procedural tile rendering.

### 5. PocketJS

- **URL**: https://github.com/realjyce/PocketJS
- **Tech**: Vanilla JavaScript (ES6+), HTML5 Canvas
- **Features**: Grid-based movement, turn-based combat, NPC dialogue, state persistence

**Why it works well**: Demonstrates core RPG architecture without any framework. Per-pixel interpolation for smooth grid-locked traversal. Sprite animation pipeline with direction-based frame management.

### 6. Crystal Shards

- **URL**: https://devpost.com/software/crystal-shards-a-browser-rpg-adventure
- **Tech**: Vanilla HTML5 Canvas, JavaScript, no frameworks
- **Features**: Complete beginning-middle-end, touch D-pad, particle effects

**Why it works well**: Touch controls via DOM overlay (more reliable than canvas hit detection). Typewriter dialogue with callbacks. Full game loop with scene transitions.

### 7. Age of Awakening (觉醒纪元)

- **URL**: https://github.com/huqingjie0118-droid/gdd-legends
- **Demo**: https://h5-game-nine.vercel.app
- **Tech**: Vanilla JavaScript (ES6+), HTML5 Canvas, ECS architecture
- **Features**: 8 classes, AI teammates, guild, seasons, cross-server

**Why it works well**: ECS (Entity-Component-System) + server-authoritative architecture. Client prediction + server rollback + state interpolation. Deterministic seed RNG (Mulberry32). Particle LOD, render batching, memory optimization. 60fps on high-end, 45+ on low-end.

### 8. Dungeon Depths

- **URL**: https://github.com/sdeveer/DungeonDepths
- **Tech**: Vanilla JS + HTML5 Canvas, isometric 2:1 diamond tiles
- **Features**: Procedural dungeons, A*, line of sight, skill trees, traps

**Why it works well**: Diablo-style isometric rendering with depth-sorted occlusion. Procedural canvas drawing as fallback when image assets are missing. PostgreSQL-backed save system. A* pathfinding for enemies.

---

## Universal Patterns Extracted

### 1. Canvas Setup

Every working game initializes canvas the same way:

```javascript
// HTML
<canvas id="game" width="640" height="480"></canvas>

// JavaScript
const canvas = document.getElementById('game');
const ctx = canvas.getContext('2d');

// Pixel-art crisp rendering (CSS)
// canvas { image-rendering: pixelated; image-rendering: crisp-edges; }
```

**Key detail**: Use `image-rendering: pixelated` on the canvas CSS for crisp pixel art. Scale with CSS transforms, not canvas scaling.

**Resolution choices from real games**:
- 640×480 (Starfall Chronicles)
- 960×640 (Claudicus)
- 800×600 (Phaser RPG Tutorial)
- 640×360 (Pixel Quest Deluxe — then CSS-scaled 2x)

### 2. Game Loop

Every game uses `requestAnimationFrame`. The two dominant patterns:

**Pattern A — Simple loop (Shattered Vale)**:
```javascript
function loop() {
  update();
  draw();
  requestAnimationFrame(loop);
}
loop();
```

**Pattern B — Frame-capped loop (Claudicus)**:
```javascript
private lastTimestamp: number = 0;
private readonly FRAME_MS: number = 1000 / 120; // 120fps cap

private gameLoop = (timestamp: number): void => {
  requestAnimationFrame(this.gameLoop);
  const delta = timestamp - this.lastTimestamp;
  if (delta < this.FRAME_MS) return;
  this.lastTimestamp = timestamp - (delta % this.FRAME_MS);
  this.update();
  this.render();
};
```

**Pattern C — Delta-time loop (advanced)**:
```javascript
let lastTime = 0;
function loop(timestamp) {
  const dt = (timestamp - lastTime) / 1000; // seconds
  lastTime = timestamp;
  update(dt);
  draw();
  requestAnimationFrame(loop);
}
```

**Recommendation**: Pattern A for simplicity, Pattern B for consistent speed across refresh rates.

### 3. Input System

Every game tracks two states: **held** (for movement) and **justPressed** (for menus/actions).

**Shattered Vale pattern** (minimal, proven):
```javascript
function createInputState() {
  const keys = {};
  const justPressed = {};

  window.addEventListener('keydown', (e) => {
    const k = e.key.toLowerCase();
    if (!keys[k]) justPressed[k] = true;
    keys[k] = true;
    if (['arrowup','arrowdown','arrowleft','arrowright',' '].includes(k))
      e.preventDefault();
  });
  window.addEventListener('keyup', (e) => {
    keys[e.key.toLowerCase()] = false;
  });

  return {
    keys,
    justPressed,
    clearJustPressed() {
      for (const k in justPressed) delete justPressed[k];
    },
  };
}

// In game loop:
input.clearJustPressed(); // at END of frame
```

**Claudicus pattern** (TypeScript, cleaner):
```typescript
class InputManager {
  private held: Set<string> = new Set();
  private justPressed: Set<string> = new Set();
  private justPressedBuffer: Set<string> = new Set();

  constructor() {
    window.addEventListener('keydown', (e) => this.onKeyDown(e));
    window.addEventListener('keyup', (e) => this.onKeyUp(e));
  }

  private onKeyDown(e: KeyboardEvent): void {
    const key = e.key.toLowerCase();
    if (!this.held.has(key)) {
      this.held.add(key);
      this.justPressedBuffer.add(key);
    }
  }

  private onKeyUp(e: KeyboardEvent): void {
    this.held.delete(e.key.toLowerCase());
  }

  flushFrame(): void {
    this.justPressed = new Set(this.justPressedBuffer);
    this.justPressedBuffer.clear();
  }

  isHeld(key: string): boolean { return this.held.has(key); }
  wasJustPressed(key: string): boolean { return this.justPressed.has(key); }
}
```

**Key insight**: The double-buffer pattern (justPressedBuffer → justPressed) prevents input loss when key events fire between frames.

### 4. State Machine

Every game uses a state machine for screen transitions. The cleanest pattern:

```javascript
// State enum
const GameState = {
  TITLE: 'title',
  PLAYING: 'playing',
  DIALOG: 'dialog',
  INVENTORY: 'inventory',
  COMBAT: 'combat',
  PAUSED: 'paused',
  GAMEOVER: 'gameover',
};

// In update:
function update() {
  switch (state.gameState) {
    case GameState.TITLE:    updateTitle(); break;
    case GameState.PLAYING:  updatePlaying(); break;
    case GameState.DIALOG:   updateDialog(); break;
    case GameState.COMBAT:   updateCombat(); break;
    // ...
  }
}

// In draw:
function draw() {
  ctx.clearRect(0, 0, VIEW_W, VIEW_H);
  switch (state.gameState) {
    case GameState.TITLE:    drawTitle(); break;
    case GameState.PLAYING:  drawGame(); break;
    case GameState.DIALOG:   drawGame(); drawDialog(); break; // overlay!
    // ...
  }
}
```

**Critical pattern**: Some states render as overlays on top of the game (dialog, inventory, pause). The Claudicus pattern handles this cleanly:

```typescript
case GameState.OVERWORLD:
case GameState.DIALOG:
case GameState.SHOP:
case GameState.PAUSE:
case GameState.INVENTORY:
  this.renderOverworld();           // always draw the world first
  if (this.state === GameState.DIALOG) this.renderDialog();
  else if (this.state === GameState.SHOP) this.renderShop();
  else if (this.state === GameState.PAUSE) this.ui.drawPauseMenu();
  break;
```

### 5. Rendering — Sprites & Tiles

**Sprite sheet animation** (universal pattern):
```javascript
class AnimatedSprite {
  constructor(sheet, frameW, frameH, dirRows = true) {
    this.sheet = sheet;
    this.fw = frameW;
    this.fh = frameH;
    this.dirRows = dirRows;
    this.frame = 0;
    this.frameTimer = 0;
    this.frameSpeed = 8; // frames per animation step
  }

  update(moving) {
    if (!moving) { this.frame = 0; return; }
    this.frameTimer++;
    if (this.frameTimer >= this.frameSpeed) {
      this.frameTimer = 0;
      this.frame = (this.frame + 1) % 4; // 4-frame walk cycle
    }
  }

  draw(ctx, x, y, dir, flashWhite = false) {
    const dirIndex = { down: 0, left: 1, right: 2, up: 3 };
    const row = this.dirRows ? dirIndex[dir] : 0;
    ctx.drawImage(
      this.sheet,
      this.frame * this.fw, row * this.fh, this.fw, this.fh, // source
      x, y, this.fw, this.fh                                   // destination
    );
    if (flashWhite) {
      ctx.save();
      ctx.globalCompositeOperation = 'source-atop';
      ctx.fillStyle = 'rgba(255,255,255,0.65)';
      ctx.fillRect(x, y, this.fw, this.fh);
      ctx.restore();
    }
  }
}
```

**Tilemap rendering** (bake static, animate dynamic):
```javascript
// Bake static tiles to offscreen canvas ONCE (or on map change)
_bakeStatic() {
  const off = makeCanvas(this.cols * TILE, this.rows * TILE);
  const ctx = off.getContext('2d');
  for (let y = 0; y < this.rows; y++) {
    for (let x = 0; x < this.cols; x++) {
      drawGroundTile(ctx, this, x, y);
    }
  }
  this._offscreen = off;
}

// Draw static layer (single blit per frame)
drawGround(ctx, camX, camY, viewW, viewH) {
  ctx.drawImage(this._offscreen, camX, camY, viewW, viewH, 0, 0, viewW, viewH);
}

// Draw animated tiles separately (water shimmer, lava glow)
drawAnimated(ctx, camX, camY, viewW, viewH, t) {
  // Only iterate visible tiles
  const startX = Math.max(0, Math.floor(camX / TILE));
  const endX = Math.min(this.cols, Math.ceil((camX + viewW) / TILE));
  // ...
}
```

**Depth-sorted rendering** (critical for top-down RPGs):
```javascript
const drawables = [];
npcs.forEach((n) => drawables.push({ y: n.y + n.h, draw: () => n.draw(ctx, camX, camY) }));
enemies.forEach((en) => drawables.push({ y: en.y + en.h, draw: () => en.draw(ctx, camX, camY) }));
drawables.push({ y: player.y + player.h, draw: () => player.draw(ctx, camX, camY) });
drawables.sort((a, b) => a.y - b.y);
drawables.forEach((d) => d.draw());
```

### 6. Camera System

```javascript
class Camera {
  constructor(viewW, viewH) {
    this.x = 0; this.y = 0;
    this.viewW = viewW; this.viewH = viewH;
    this.shakeTime = 0; this.shakeMag = 0;
  }

  follow(targetX, targetY, mapW, mapH) {
    const desiredX = targetX - this.viewW / 2;
    const desiredY = targetY - this.viewH / 2;
    this.x = lerp(this.x, desiredX, 0.12); // smooth follow
    this.y = lerp(this.y, desiredY, 0.12);
    this.x = clamp(this.x, 0, Math.max(0, mapW - this.viewW));
    this.y = clamp(this.y, 0, Math.max(0, mapH - this.viewH));
  }

  shake(mag, time) { this.shakeMag = mag; this.shakeTime = time; }

  getOffset() {
    if (this.shakeTime > 0) {
      this.shakeTime--;
      return {
        x: this.x + randRange(-this.shakeMag, this.shakeMag),
        y: this.y + randRange(-this.shakeMag, this.shakeMag)
      };
    }
    return { x: this.x, y: this.y };
  }
}
```

### 7. Collision Detection

**Tile-based collision** (universal):
```javascript
// Tile type → solid mapping
const SOLID_TILES = new Set([TileType.WALL, TileType.WATER, TileType.TREE]);

isSolid(x, y) {
  return !this.inBounds(x, y) || SOLID_TILES.has(this.get(x, y));
}

// Player collision check (4-corner probe)
tryMove(dx, dy, map) {
  if (dx !== 0) {
    const nx = this.x + dx;
    const corners = [
      [nx + 4, this.y + 10],
      [nx + this.w - 4, this.y + 10],
      [nx + 4, this.y + this.h],
      [nx + this.w - 4, this.y + this.h],
    ];
    if (!corners.some(([cx, cy]) =>
      map.isSolid(Math.floor(cx / TILE), Math.floor(cy / TILE))
    )) {
      this.x = nx;
    }
  }
  // Same for dy...
}
```

**AABB entity collision** (rectangles overlap):
```javascript
function rectsOverlap(a, b) {
  return a.x < b.x + b.w && a.x + a.w > b.x &&
         a.y < b.y + b.h && a.y + a.h > b.y;
}

// Usage: player attack hitbox vs enemy
const hb = player.attackHitbox();
enemies.forEach((en) => {
  if (!en.alive || !rectsOverlap(hb, en)) return;
  en.takeDamage(player.attackDamage, particles);
});
```

**Axis-separated sliding** (prevents wall-sticking):
```javascript
// If full move blocked, try each axis independently
const fullX = this.x + mx, fullY = this.y + my;
if (canMoveTo(fullX, fullY)) {
  this.x = fullX; this.y = fullY;
} else {
  if (mx !== 0 && canMoveTo(this.x + mx, this.y)) this.x += mx;
  if (my !== 0 && canMoveTo(this.x, this.y + my)) this.y += my;
}
```

### 8. UI Rendering

**Health/mana bars** (canvas-drawn):
```javascript
// Enemy HP bar
if (this.hp < this.maxHp) {
  const barW = this.w + 4;
  const bx = this.centerX - camX - barW / 2;
  const by = drawY - 7;
  ctx.fillStyle = 'rgba(0,0,0,0.5)';
  ctx.fillRect(bx, by, barW, 4);
  ctx.fillStyle = '#97c459';
  ctx.fillRect(bx, by, barW * (this.hp / this.maxHp), 4);
}
```

**Boss health banner** (centered, with name):
```javascript
function drawBossBanner(ctx, boss, viewW) {
  const barW = Math.min(340, viewW - 80);
  const x = (viewW - barW) / 2;
  const pct = clamp(boss.hp / boss.maxHp, 0, 1);
  const low = pct <= 0.25;

  ctx.save();
  ctx.textAlign = 'center';
  ctx.font = 'bold 22px sans-serif';
  ctx.fillStyle = boss.color;
  ctx.fillText(boss.name, viewW / 2, 22);

  // Bar background
  ctx.fillStyle = 'rgba(10,12,8,0.75)';
  roundRect(ctx, x - 3, 30, barW + 6, 22, 6);
  ctx.fill();

  // Bar fill
  ctx.fillStyle = low ? '#da1f1f' : boss.color;
  roundRect(ctx, x, 33, barW * pct, 16, 4);
  ctx.fill();

  // HP text
  ctx.font = 'bold 11px sans-serif';
  ctx.fillStyle = '#f1efe8';
  ctx.fillText(`${boss.hp} / ${boss.maxHp}`, viewW / 2, 44);
  ctx.textAlign = 'left';
  ctx.restore();
}
```

**Dialogue box** (typewriter effect):
```javascript
function drawDialogue(ctx, npc, line, charIndex, canvasW, canvasH) {
  const margin = 24;
  const boxW = canvasW - margin * 2;
  const shown = line.substring(0, Math.floor(charIndex));
  const boxH = 92;
  const boxY = canvasH - boxH - 20;

  ctx.save();
  // Background
  ctx.fillStyle = 'rgba(12,14,10,0.88)';
  roundRect(ctx, margin, boxY, boxW, boxH, 10);
  ctx.fill();
  ctx.strokeStyle = 'rgba(232,228,216,0.35)';
  ctx.lineWidth = 1.5;
  roundRect(ctx, margin, boxY, boxW, boxH, 10);
  ctx.stroke();

  // Name tag
  ctx.fillStyle = '#e8c93c';
  ctx.font = 'bold 13px sans-serif';
  const nameW = ctx.measureText(npc.name).width + 28;
  roundRect(ctx, margin + 14, boxY - 14, nameW, 26, 6);
  ctx.fill();
  ctx.fillStyle = '#2c2418';
  ctx.fillText(npc.name, margin + 28, boxY - 1);

  // Text (word-wrapped)
  ctx.fillStyle = '#f1efe8';
  ctx.font = '14px sans-serif';
  wrapText(ctx, shown, margin + 20, boxY + 28, boxW - 40, 20);

  // Continue indicator (bouncing)
  if (charIndex >= line.length) {
    const bounce = Math.sin(Date.now() / 200) * 2;
    ctx.fillStyle = 'rgba(232,228,216,0.6)';
    ctx.font = '11px sans-serif';
    ctx.fillText('▼ space to continue', margin + boxW - 150, boxY + boxH - 12 + bounce);
  }
  ctx.restore();
}
```

**Toast notifications**:
```javascript
function drawToast(ctx, msg, timer, viewW) {
  if (!msg) return;
  ctx.save();
  const w = ctx.measureText(msg).width + 40;
  const x = (viewW - w) / 2;
  ctx.globalAlpha = clamp(timer / 30, 0, 1);
  ctx.fillStyle = 'rgba(10,12,8,0.85)';
  roundRect(ctx, x, 16, w, 32, 8);
  ctx.fill();
  ctx.strokeStyle = 'rgba(232,201,60,0.5)';
  ctx.lineWidth = 1;
  roundRect(ctx, x, 16, w, 32, 8);
  ctx.stroke();
  ctx.fillStyle = '#f1efe8';
  ctx.font = '13px sans-serif';
  ctx.textAlign = 'center';
  ctx.fillText(msg, viewW / 2, 37);
  ctx.textAlign = 'left';
  ctx.restore();
}
```

### 9. Particle System

Every game uses a lightweight particle system for juice:

```javascript
class ParticleSystem {
  constructor() {
    this.particles = [];
    this.texts = [];
  }

  burst(x, y, color, count = 6) {
    for (let i = 0; i < count; i++) {
      const angle = Math.random() * Math.PI * 2;
      const speed = 0.6 + Math.random() * 1.6;
      this.particles.push({
        x, y,
        vx: Math.cos(angle) * speed,
        vy: Math.sin(angle) * speed - 0.5,
        life: 16 + Math.random() * 12,
        maxLife: 28,
        size: 2 + Math.random() * 2,
        color,
        gravity: 0.08,
      });
    }
  }

  floatText(x, y, text, color, size = 13) {
    this.texts.push({ x, y, text, color, life: 40, maxLife: 40, size });
  }

  update() {
    this.particles.forEach(p => {
      p.x += p.vx; p.y += p.vy;
      p.vy += p.gravity;
      p.life--;
    });
    this.particles = this.particles.filter(p => p.life > 0);
    this.texts.forEach(t => { t.y -= 0.6; t.life--; });
    this.texts = this.texts.filter(t => t.life > 0);
  }

  draw(ctx, camX, camY) {
    this.particles.forEach(p => {
      ctx.globalAlpha = p.life / p.maxLife;
      ctx.fillStyle = p.color;
      ctx.fillRect(p.x - camX - p.size/2, p.y - camY - p.size/2, p.size, p.size);
    });
    ctx.globalAlpha = 1;
    this.texts.forEach(t => {
      ctx.globalAlpha = clamp(t.life / t.maxLife, 0, 1);
      ctx.font = `bold ${t.size}px sans-serif`;
      ctx.textAlign = 'center';
      ctx.fillStyle = 'rgba(0,0,0,0.6)';
      ctx.fillText(t.text, t.x - camX + 1, t.y - camY + 1);
      ctx.fillStyle = t.color;
      ctx.fillText(t.text, t.x - camX, t.y - camY);
    });
    ctx.globalAlpha = 1;
    ctx.textAlign = 'left';
  }
}
```

### 10. Save/Load System

```javascript
const SAVE_KEY = 'game_save_v6';

function saveGame(state) {
  const data = {
    version: 6,
    player: { ... },
    quests: { ... },
    world: { ... },
    timestamp: Date.now(),
  };
  localStorage.setItem(SAVE_KEY, JSON.stringify(data));
}

function loadGame() {
  const raw = localStorage.getItem(SAVE_KEY);
  if (!raw) return null;
  try {
    const data = JSON.parse(raw);
    // Forward migrations by version number
    if (data.version < 6) migrateV5toV6(data);
    return data;
  } catch { return null; }
}
```

### 11. Utility Functions

Every game needs these:

```javascript
const TILE = 32;

function clamp(v, lo, hi) { return Math.max(lo, Math.min(hi, v)); }
function lerp(a, b, t) { return a + (b - a) * t; }
function dist(x1, y1, x2, y2) { return Math.hypot(x2 - x1, y2 - y1); }
function rectsOverlap(a, b) {
  return a.x < b.x + b.w && a.x + a.w > b.x &&
         a.y < b.y + b.h && a.y + a.h > b.y;
}
function randRange(min, max) { return min + Math.random() * (max - min); }

function roundRect(ctx, x, y, w, h, r) {
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.arcTo(x + w, y, x + w, y + h, r);
  ctx.arcTo(x + w, y + h, x, y + h, r);
  ctx.arcTo(x, y + h, x, y, r);
  ctx.arcTo(x, y, x + w, y, r);
  ctx.closePath();
}

function makeCanvas(w, h) {
  const c = document.createElement('canvas');
  c.width = w; c.height = h;
  return c;
}

function wrapText(ctx, text, maxWidth) {
  const words = text.split(' ');
  const lines = [];
  let line = '';
  for (const w of words) {
    const test = line ? line + ' ' + w : w;
    if (ctx.measureText(test).width > maxWidth && line) {
      lines.push(line); line = w;
    } else { line = test; }
  }
  if (line) lines.push(line);
  return lines;
}

function hashTile(x, y) {
  let h = (x * 374761393 + y * 668265263) % 2147483647;
  h = (h ^ (h >> 13)) * 1274126177 % 2147483647;
  return Math.abs(h) / 2147483647;
}
```

---

## Architecture Summary

The universal architecture for a working HTML5 RPG:

```
index.html          → <canvas> + optional DOM HUD
js/
  utils.js          → clamp, lerp, dist, rectsOverlap, roundRect, makeCanvas
  config/           → pure data (balance numbers, map layouts, enemy stats)
  sprites/          → procedural sprite generation or sprite sheet loading
  world/
    tilemap.js      → TileMap class (grid, gates, bake static, draw)
    tilemap-builder.js → terrain generation
    tilemap-renderer.js → per-tile drawing
  entities/
    animated-sprite.js → shared frame-walking animation
    player.js       → movement, combat, leveling
    enemy.js        → AI (wander/aggro), damage, death
    npc.js          → dialogue, shops
  systems/
    camera.js       → smooth follow, screen shake, clamp to bounds
    particles.js    → burst, sparkle, float text
    dialogue.js     → typewriter text, word wrap
    inventory.js    → item slots, equip/unequip
  ui/
    hud.js          → HP/XP/mana bars, boss banner, toast, quest tracker
    screens.js      → start menu, game over, how-to-play
  core/
    input.js        → keyboard state (held + justPressed)
    combat.js       → hit resolution, rewards, loot
    game.js         → bootstrap, main loop, state machine, restart
```

**Data flow**: `game.js` owns the state object → passes it to every system → systems never close over game.js locals → restart resets the state object.

**Rendering order**: Clear → map ground → map animated tiles → depth-sorted entities → particles → vignette → HUD → dialogue → inventory → screen overlays.

**Update order**: Input clear → dialogue update → player update → enemy updates → combat resolution → particle update → camera follow → timer ticks → input clear (end of frame).

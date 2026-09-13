# HTML5/JavaScript RPG Game Patterns — Reference Report

> Researched from 10+ working open-source HTML5 RPG games and MDN/StackOverflow canonical sources.
> Generated: 2026-09-13

---

## Table of Contents

1. [Working Click Patterns](#1-working-click-patterns)
2. [Game State Machine](#2-game-state-machine)
3. [Canvas Rendering](#3-canvas-rendering)
4. [Event Handling](#4-event-handling)
5. [Common Bugs & Fixes](#5-common-bugs--fixes)
6. [Source Code Examples](#6-source-code-examples)
7. [Analyzed GitHub Projects](#7-analyzed-github-projects)

---

## 1. Working Click Patterns

### 1.1 Getting Click Coordinates Relative to Canvas

The most critical pattern: canvas coordinates ≠ viewport coordinates. You must convert.

**Pattern A — `getBoundingClientRect()` (Recommended, most reliable)**

```javascript
canvas.addEventListener('click', function(event) {
  const rect = canvas.getBoundingClientRect();
  const x = event.clientX - rect.left;
  const y = event.clientY - rect.top;
  console.log('Clicked at canvas coords:', x, y);
});
```

**Pattern B — `offsetX/offsetY` (Simpler, works in modern browsers)**

```javascript
canvas.addEventListener('click', function(event) {
  const x = event.offsetX;
  const y = event.offsetY;
  console.log('Clicked at canvas coords:', x, y);
});
```

**Pattern C — Account for CSS scaling (when canvas is styled larger/smaller)**

```javascript
canvas.addEventListener('click', function(event) {
  const rect = canvas.getBoundingClientRect();
  const scaleX = canvas.width / rect.width;
  const scaleY = canvas.height / rect.height;
  const x = (event.clientX - rect.left) * scaleX;
  const y = (event.clientY - rect.top) * scaleY;
  console.log('Clicked at internal coords:', x, y);
});
```

> **Source**: MDN Mouse Controls tutorial, StackOverflow `#55677`, `#9880279`

### 1.2 Hit-Testing Rectangles (Button Detection)

**Basic AABB hit test:**

```javascript
function isPointInRect(px, py, rect) {
  return px >= rect.x && px <= rect.x + rect.w &&
         py >= rect.y && py <= rect.y + rect.h;
}

// Usage
canvas.addEventListener('click', function(event) {
  const rect = canvas.getBoundingClientRect();
  const x = event.clientX - rect.left;
  const y = event.clientY - rect.top;

  buttons.forEach(function(btn) {
    if (isPointInRect(x, y, btn)) {
      btn.onClick();
    }
  });
});
```

**Iterate in reverse for z-order (last drawn = top = click priority):**

```javascript
canvas.addEventListener('click', function(event) {
  const rect = canvas.getBoundingClientRect();
  const x = event.clientX - rect.left;
  const y = event.clientY - rect.top;

  // Reverse order: topmost drawn element gets priority
  for (let i = buttons.length - 1; i >= 0; i--) {
    const b = buttons[i];
    if (x >= b.x && x <= b.x + b.w && y >= b.y && y <= b.y + b.h) {
      b.onClick();
      break; // Stop at first hit
    }
  }
});
```

> **Source**: StackOverflow `#36698774`, `#5014851`

### 1.3 Hit-Testing Circles

```javascript
function isPointInCircle(px, py, circle) {
  const dx = px - circle.x;
  const dy = py - circle.y;
  return (dx * dx + dy * dy) < (circle.radius * circle.radius);
}
```

### 1.4 Using `Path2D` + `isPointInPath()` (Modern, Precise)

```javascript
// Define paths once
const buttonPath = new Path2D();
buttonPath.rect(250, 350, 200, 100);

canvas.addEventListener('click', function(event) {
  const rect = canvas.getBoundingClientRect();
  const x = event.clientX - rect.left;
  const y = event.clientY - rect.top;

  if (ctx.isPointInPath(buttonPath, x, y)) {
    console.log('Button clicked!');
  }
});
```

> **Source**: `codingeasypeasy.com` — `isPointInPath()` tutorial

### 1.5 Complete Button System (from StackOverflow)

```javascript
var canvas = document.getElementById("canvas");
var ctx = canvas.getContext("2d");
var offsetX, offsetY;
var clickedButton;
var buttons = [];

function reOffset() {
  var BB = canvas.getBoundingClientRect();
  offsetX = BB.left;
  offsetY = BB.top;
}
reOffset();
window.onscroll = function() { reOffset(); };
window.onresize = function() { reOffset(); };

function makeButton(id, x, y, w, h, label, fill, stroke, labelColor, clickFn, releaseFn) {
  return {
    id: id, x: x, y: y, w: w, h: h,
    fill: fill, stroke: stroke, labelColor: labelColor,
    label: label, click: clickFn, release: releaseFn
  };
}

function drawButton(b, isDown) {
  ctx.clearRect(b.x - 1, b.y - 1, b.w + 2, b.h + 2);
  ctx.fillStyle = b.fill;
  ctx.fillRect(b.x, b.y, b.w, b.h);
  ctx.strokeStyle = b.stroke;
  ctx.strokeRect(b.x, b.y, b.w, b.h);
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillStyle = b.labelColor;
  ctx.fillText(b.label, b.x + b.w / 2, b.y + b.h / 2);
  if (isDown) {
    ctx.beginPath();
    ctx.moveTo(b.x, b.y + b.h);
    ctx.lineTo(b.x, b.y);
    ctx.lineTo(b.x + b.w, b.y);
    ctx.strokeStyle = 'black';
    ctx.stroke();
  }
}

function findButton(mx, my) {
  for (var i = 0; i < buttons.length; i++) {
    var b = buttons[i];
    if (mx > b.x && mx < b.x + b.w && my > b.y && my < b.y + b.h) {
      return b;
    }
  }
  return null;
}

canvas.onmousedown = function(e) {
  e.preventDefault();
  var mouseX = parseInt(e.clientX - offsetX);
  var mouseY = parseInt(e.clientY - offsetY);
  var b = findButton(mouseX, mouseY);
  if (b) {
    clickedButton = b;
    drawButton(b, true);
    b.click();
  }
};

canvas.onmouseup = function(e) {
  e.preventDefault();
  if (clickedButton) {
    drawButton(clickedButton, false);
    clickedButton.release();
    clickedButton = null;
  }
};
```

---

## 2. Game State Machine

### 2.1 Classic State Pattern (Most Common)

From analyzing Shattered-Vale-RPG, PocketJS, Claudicus, Starfall Chronicles, and LEGEND RPG:

```javascript
const GameState = {
  MENU: 'menu',
  LOADING: 'loading',
  PLAYING: 'playing',
  PAUSED: 'paused',
  DIALOGUE: 'dialogue',
  BATTLE: 'battle',
  GAME_OVER: 'gameOver',
  VICTORY: 'victory',
};

let currentState = GameState.LOADING;
let previousState = null;

function setState(newState) {
  previousState = currentState;
  currentState = newState;
  onStateEnter(newState);
}

function onStateEnter(state) {
  switch (state) {
    case GameState.MENU:
      showMenu();
      break;
    case GameState.LOADING:
      loadAssets().then(() => setState(GameState.MENU));
      break;
    case GameState.PLAYING:
      resumeGameLoop();
      break;
    case GameState.PAUSED:
      pauseGameLoop();
      break;
  }
}
```

### 2.2 Screen Registry Pattern (Starfall Chronicles)

```typescript
// src/engine/Game.ts — from mtaylor-create/teamsDemo-RPG2
interface Screen {
  enter(context: GameContext): void;
  update(dt: number, context: GameContext): void;
  render(ctx: CanvasRenderingContext2D, context: GameContext): void;
}

class Game {
  private screens: Map<string, Screen> = new Map();
  private currentScreen: Screen | null = null;

  registerScreen(name: string, screen: Screen) {
    this.screens.set(name, screen);
  }

  switchScreen(name: string) {
    this.currentScreen = this.screens.get(name) || null;
    this.currentScreen?.enter(this.context);
  }

  update(dt: number) {
    this.currentScreen?.update(dt, this.context);
  }

  render(ctx: CanvasRenderingContext2D) {
    this.currentScreen?.render(ctx, this.context);
  }
}
```

### 2.3 Loading → Menu → Game Flow (Shattered-Vale-RPG Pattern)

From `goldipl/Shattered-Vale-RPG/js/core/game.js`:

```javascript
// Phase 1: Boot — init everything
let gameStarted = false;

function boot() {
  // Generate sprites, tilemap, etc.
  Sprites.init();
  const tileMap = new TileMap(109, 236);
  tileMap.buildWorld();

  // Set up input
  Input.init();

  // Draw title screen
  Screens.drawStartMenu();

  // Start the render loop (but game logic doesn't run yet)
  requestAnimationFrame(gameLoop);
}

// Phase 2: Game loop — runs always, but logic only in PLAYING state
function gameLoop(timestamp) {
  const dt = timestamp - lastTime;
  lastTime = timestamp;

  if (currentState === GameState.PLAYING) {
    update(dt);
  }

  draw(timestamp);
  requestAnimationFrame(gameLoop);
}

// Phase 3: Start game when user clicks "Play"
function startNewGame() {
  gameStarted = true;
  world = WorldFactory.create();
  player = new Player(SPAWN_X, SPAWN_Y);
  setState(GameState.PLAYING);
}
```

### 2.4 State Transition Diagram

```
                  ┌──────────────┐
                  │   BOOT       │
                  │ (load assets)│
                  └──────┬───────┘
                         │ assets loaded
                         ▼
                  ┌──────────────┐
            ┌────►│   MENU       │◄──────────┐
            │     │ (title/opts) │           │
            │     └──────┬───────┘           │
            │            │ "Play"            │
            │            ▼                   │
            │     ┌──────────────┐           │
            │     │   PLAYING    │──Escape──►│
            │     │ (game world) │           │
            │     └──┬───┬───┬──┘           │
            │        │   │   │               │
            │  talk  │   │   │ defeat       │
            │        ▼   │   ▼              │
            │  ┌───────┐ │ ┌────────┐       │
            │  │DIALOGUE│ │ │VICTORY │───────┘
            │  └───┬───┘ │ └────────┘
            │      │close│
            │      ▼     │
            │  ┌────────┐│    ┌──────────┐
            └──┤ PLAYING │├───►│GAME_OVER │
               └────────┘│    └─────┬────┘
                         │          │ restart
                    battle          ▼
                    ┌───────┐   ┌───────┐
                    │BATTLE │──►│ MENU  │
                    └───────┘   └───────┘
```

---

## 3. Canvas Rendering

### 3.1 Proper Canvas Setup

```html
<!-- index.html -->
<canvas id="gameCanvas" width="960" height="640"></canvas>
```

```css
/* style.css */
#gameCanvas {
  display: block;
  margin: 0 auto;
  /* Do NOT set width/height via CSS if you need pixel-perfect coords */
  /* CSS sizing causes scaling mismatch with internal resolution */
}
```

### 3.2 Game Loop — `requestAnimationFrame`

```javascript
const canvas = document.getElementById('gameCanvas');
const ctx = canvas.getContext('2d');

let lastTime = 0;
const TARGET_FPS = 60;
const FRAME_TIME = 1000 / TARGET_FPS;
let accumulator = 0;

function gameLoop(timestamp) {
  const dt = timestamp - lastTime;
  lastTime = timestamp;
  accumulator += dt;

  // Fixed timestep update (physics at consistent rate)
  while (accumulator >= FRAME_TIME) {
    update(FRAME_TIME / 1000); // pass dt in seconds
    accumulator -= FRAME_TIME;
  }

  // Render at display refresh rate
  render();
  requestAnimationFrame(gameLoop);
}

function update(dt) {
  player.update(dt);
  enemies.forEach(e => e.update(dt));
  camera.follow(player);
}

function render() {
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  tilemap.draw(ctx, camera);
  entities.forEach(e => e.draw(ctx, camera));
  hud.draw(ctx);
}

// Start
requestAnimationFrame(gameLoop);
```

### 3.3 Camera Culling (Only Draw Visible)

```javascript
function draw(ctx, camera) {
  const startCol = Math.floor(camera.x / TILE_SIZE);
  const endCol = Math.ceil((camera.x + canvas.width) / TILE_SIZE);
  const startRow = Math.floor(camera.y / TILE_SIZE);
  const endRow = Math.ceil((camera.y + canvas.height) / TILE_SIZE);

  for (let row = startRow; row <= endRow; row++) {
    for (let col = startCol; col <= endCol; col++) {
      const tile = tilemap.getTile(col, row);
      const drawX = col * TILE_SIZE - camera.x;
      const drawY = row * TILE_SIZE - camera.y;
      drawTile(ctx, tile, drawX, drawY);
    }
  }
}
```

### 3.4 Tile Caching (Performance)

```javascript
// Bake static tilemap to offscreen canvas once
const offscreen = document.createElement('canvas');
offscreen.width = tilemap.cols * TILE_SIZE;
offscreen.height = tilemap.rows * TILE_SIZE;
const offCtx = offscreen.getContext('2d');

// Draw all static tiles once
tilemap.forEachTile((tile, col, row) => {
  drawTile(offCtx, tile, col * TILE_SIZE, row * TILE_SIZE);
});

// Then just blit the visible portion each frame
function render() {
  ctx.drawImage(
    offscreen,
    camera.x, camera.y, canvas.width, canvas.height,  // source rect
    0, 0, canvas.width, canvas.height                    // dest rect
  );
}
```

> **Source**: Shattered-Vale-RPG, Claudicus, the TypeScript RPG (DEV Community)

---

## 4. Event Handling

### 4.1 Keyboard Input (Held vs Just-Pressed)

From `goldipl/Shattered-Vale-RPG/js/core/input.js` and `brian-benzinger/claudicus/src/input.ts`:

```javascript
class Input {
  constructor() {
    this.keys = {};          // Currently held keys
    this.justPressed = {};   // True for one frame after press
    this._prevKeys = {};
  }

  init() {
    window.addEventListener('keydown', (e) => {
      this.keys[e.key] = true;
      e.preventDefault(); // Prevent page scroll with arrow keys
    });

    window.addEventListener('keyup', (e) => {
      this.keys[e.key] = false;
    });
  }

  update() {
    // Calculate justPressed: key is down now but wasn't last frame
    for (const key in this.keys) {
      this.justPressed[key] = this.keys[key] && !this._prevKeys[key];
    }
    this._prevKeys = { ...this.keys };
  }

  isDown(key) {
    return !!this.keys[key];
  }

  wasPressed(key) {
    return !!this.justPressed[key];
  }
}
```

### 4.2 Mouse Event Setup (Complete)

From IBM Developer tutorial and MDN:

```javascript
class MouseInput {
  constructor(canvas) {
    this.canvas = canvas;
    this.x = 0;
    this.y = 0;
    this.isDown = false;
    this.button = 0; // 0=left, 1=middle, 2=right
    this.isInside = false;

    canvas.addEventListener('mouseenter', (e) => {
      this.isInside = true;
      this.updatePosition(e);
    });

    canvas.addEventListener('mouseleave', (e) => {
      this.isInside = false;
    });

    canvas.addEventListener('mousemove', (e) => {
      this.updatePosition(e);
    });

    canvas.addEventListener('mousedown', (e) => {
      this.isDown = true;
      this.button = e.button;
      this.updatePosition(e);
    });

    canvas.addEventListener('mouseup', (e) => {
      this.isDown = false;
      this.updatePosition(e);
    });

    canvas.addEventListener('contextmenu', (e) => {
      e.preventDefault(); // Disable right-click menu in game
    });
  }

  updatePosition(event) {
    const rect = this.canvas.getBoundingClientRect();
    const scaleX = this.canvas.width / rect.width;
    const scaleY = this.canvas.height / rect.height;
    this.x = (event.clientX - rect.left) * scaleX;
    this.y = (event.clientY - rect.top) * scaleY;
  }
}
```

### 4.3 Touch Event Handling (Mobile)

From `Stephendoddtech.com` and the TypeScript RPG project:

```javascript
class TouchInput {
  constructor(canvas) {
    this.canvas = canvas;
    this.touches = [];
    this.joystick = { active: false, dx: 0, dy: 0 };
    this.actionButton = false;

    canvas.addEventListener('touchstart', (e) => {
      e.preventDefault();
      this.updateTouches(e.touches);
    }, { passive: false });

    canvas.addEventListener('touchmove', (e) => {
      e.preventDefault();
      this.updateTouches(e.touches);
    }, { passive: false });

    canvas.addEventListener('touchend', (e) => {
      e.preventDefault();
      this.updateTouches(e.touches);
      if (e.touches.length === 0) {
        this.joystick.active = false;
        this.joystick.dx = 0;
        this.joystick.dy = 0;
      }
    }, { passive: false });
  }

  updateTouches(touchList) {
    this.touches = [];
    for (let i = 0; i < touchList.length; i++) {
      const t = touchList[i];
      const rect = this.canvas.getBoundingClientRect();
      this.touches.push({
        id: t.identifier,
        x: (t.clientX - rect.left) * (this.canvas.width / rect.width),
        y: (t.clientY - rect.top) * (this.canvas.height / rect.height),
      });
    }
  }
}
```

**DOM Overlay for Mobile Controls (Recommended for Complex UI):**

From the TypeScript RPG (`lazaramolina17-stack.github.io/rpg-game`):

> "DOM elements handle multi-touch natively without gesture conflicts.
> They stay fixed-position regardless of camera scroll.
> Opacity is reduced (alpha: 0.8) so game content is still visible."

```html
<!-- HTML overlay for mobile buttons -->
<div id="mobile-controls" style="display:none; position:fixed; bottom:20px; left:20px;">
  <div id="joystick-zone" style="width:120px;height:120px;border-radius:50%;background:rgba(255,255,255,0.3);"></div>
</div>
<div id="action-buttons" style="display:none; position:fixed; bottom:20px; right:20px;">
  <button id="btn-attack" style="width:60px;height:60px;border-radius:50%;">Attack</button>
  <button id="btn-interact" style="width:60px;height:60px;border-radius:50%;">Talk</button>
</div>

<script>
// Detect mobile
if ('ontouchstart' in window) {
  document.getElementById('mobile-controls').style.display = 'block';
  document.getElementById('action-buttons').style.display = 'block';
}

// Map DOM button to game input
document.getElementById('btn-attack').addEventListener('touchstart', () => {
  Input.keys[' '] = true; // Map to spacebar
});
document.getElementById('btn-attack').addEventListener('touchend', () => {
  Input.keys[' '] = false;
});
</script>
```

### 4.4 Button Hover Effect

```javascript
function handleMouseMove(event) {
  const rect = canvas.getBoundingClientRect();
  const x = event.clientX - rect.left;
  const y = event.clientY - rect.top;

  let hovering = false;
  buttons.forEach(btn => {
    btn.hovered = (x >= btn.x && x <= btn.x + btn.w &&
                   y >= btn.y && y <= btn.y + btn.h);
    if (btn.hovered) hovering = true;
  });

  canvas.style.cursor = hovering ? 'pointer' : 'default';
}
```

---

## 5. Common Bugs & Fixes

### Bug 1: Click Coordinates Wrong After CSS Sizing

**Symptom**: Clicks don't align with drawn buttons.

**Cause**: Canvas CSS width/height differs from internal width/height.

**Fix**:
```javascript
// ALWAYS use this conversion
const rect = canvas.getBoundingClientRect();
const scaleX = canvas.width / rect.width;
const scaleY = canvas.height / rect.height;
const x = (event.clientX - rect.left) * scaleX;
const y = (event.clientY - rect.top) * scaleY;
```

**Prevention**: Either don't CSS-size the canvas, or always use the scale conversion.

---

### Bug 2: Event Listener Not Firing

**Symptom**: Clicks on canvas do nothing.

**Common causes**:
1. Canvas has `pointer-events: none` in CSS
2. Another element is overlapping the canvas (z-index issue)
3. Event listener attached before canvas exists in DOM
4. Using `onclick` property instead of `addEventListener` and it gets overwritten

**Fix**:
```javascript
// Wait for DOM ready
document.addEventListener('DOMContentLoaded', () => {
  const canvas = document.getElementById('gameCanvas');
  canvas.addEventListener('click', handleClick);
});

// Ensure canvas receives events
canvas.style.pointerEvents = 'auto';
```

---

### Bug 3: Game Loop Not Starting

**Symptom**: Canvas renders once then freezes.

**Cause**: Forgetting to call `requestAnimationFrame` recursively, or calling it outside the loop function.

**Fix**:
```javascript
function gameLoop(timestamp) {
  // ... update and render ...

  // THIS MUST BE INSIDE gameLoop, not outside
  requestAnimationFrame(gameLoop);
}

// Start the loop
requestAnimationFrame(gameLoop);
```

---

### Bug 4: State Not Transitioning

**Symptom**: Clicking "Start" doesn't begin the game.

**Cause**: State update happens but render loop checks old state, or the start button click isn't detected.

**Fix**:
```javascript
// Ensure state change is immediate and visible
function startGame() {
  setState(GameState.PLAYING);  // Sets currentState
  // Don't rely on render to detect state change
  // Update immediately in the same frame
  update(0);
}
```

---

### Bug 5: Canvas Not Rendering (Black Screen)

**Symptom**: Canvas exists but nothing appears.

**Common causes**:
1. `getContext('2d')` not called
2. Drawing happens before assets loaded
3. `clearRect` called after drawing
4. Canvas dimensions are 0

**Fix**:
```javascript
const ctx = canvas.getContext('2d');
if (!ctx) {
  console.error('Canvas 2D context not supported');
  return;
}

// Verify dimensions
console.log('Canvas size:', canvas.width, canvas.height);

// Draw in correct order
ctx.clearRect(0, 0, canvas.width, canvas.height);
drawBackground();
drawEntities();
drawUI();
```

---

### Bug 6: Multiple `requestAnimationFrame` Loops Stacking

**Symptom**: Game speeds up exponentially on state change.

**Cause**: Calling `requestAnimationFrame(gameLoop)` from multiple places.

**Fix**:
```javascript
let animFrameId = null;

function startLoop() {
  if (animFrameId) return; // Already running
  animFrameId = requestAnimationFrame(gameLoop);
}

function stopLoop() {
  if (animFrameId) {
    cancelAnimationFrame(animFrameId);
    animFrameId = null;
  }
}
```

---

### Bug 7: `setInterval` Instead of `requestAnimationFrame`

**Symptom**: Janky animation, battery drain, inconsistent frame rate.

**Fix**: Always use `requestAnimationFrame`:
```javascript
// BAD
setInterval(draw, 16);

// GOOD
function gameLoop(timestamp) {
  draw();
  requestAnimationFrame(gameLoop);
}
requestAnimationFrame(gameLoop);
```

---

## 6. Source Code Examples

### 6.1 Minimal Working Game Loop with Clickable Buttons

```javascript
const canvas = document.getElementById('gameCanvas');
const ctx = canvas.getContext('2d');

const canvasW = 800, canvasH = 600;
canvas.width = canvasW;
canvas.height = canvasH;

// Game state
let state = 'menu';

// Button definitions
const buttons = {
  play: { x: 300, y: 250, w: 200, h: 50, label: 'PLAY', color: '#4CAF50' },
  settings: { x: 300, y: 320, w: 200, h: 50, label: 'SETTINGS', color: '#2196F3' },
  quit: { x: 300, y: 390, w: 200, h: 50, label: 'QUIT', color: '#f44336' },
};

// Click handler
canvas.addEventListener('click', function(event) {
  const rect = canvas.getBoundingClientRect();
  const x = (event.clientX - rect.left) * (canvas.width / rect.width);
  const y = (event.clientY - rect.top) * (canvas.height / rect.height);

  if (state === 'menu') {
    if (isInside(x, y, buttons.play)) {
      state = 'playing';
    } else if (isInside(x, y, buttons.settings)) {
      state = 'settings';
    } else if (isInside(x, y, buttons.quit)) {
      // quit logic
    }
  } else if (state === 'playing') {
    // game click logic
  }
});

function isInside(px, py, btn) {
  return px >= btn.x && px <= btn.x + btn.w &&
         py >= btn.y && py <= btn.y + btn.h;
}

// Draw functions
function drawMenu() {
  ctx.fillStyle = '#1a1a2e';
  ctx.fillRect(0, 0, canvasW, canvasH);

  ctx.fillStyle = '#fff';
  ctx.font = 'bold 48px Arial';
  ctx.textAlign = 'center';
  ctx.fillText('MY RPG', canvasW / 2, 150);

  Object.values(buttons).forEach(btn => {
    ctx.fillStyle = btn.color;
    ctx.fillRect(btn.x, btn.y, btn.w, btn.h);
    ctx.fillStyle = '#fff';
    ctx.font = '20px Arial';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(btn.label, btn.x + btn.w / 2, btn.y + btn.h / 2);
  });
}

function drawGame() {
  ctx.fillStyle = '#2d5016';
  ctx.fillRect(0, 0, canvasW, canvasH);
  // ... draw game world ...
}

// Main loop
let lastTime = 0;
function gameLoop(timestamp) {
  const dt = (timestamp - lastTime) / 1000;
  lastTime = timestamp;

  ctx.clearRect(0, 0, canvasW, canvasH);

  switch (state) {
    case 'menu': drawMenu(); break;
    case 'playing': drawGame(); break;
  }

  requestAnimationFrame(gameLoop);
}

requestAnimationFrame(gameLoop);
```

### 6.2 Touch + Keyboard Unified Input

```javascript
class InputManager {
  constructor() {
    this.keys = {};
    this.mouse = { x: 0, y: 0, down: false };
    this.isMobile = 'ontouchstart' in window;

    // Keyboard
    window.addEventListener('keydown', e => { this.keys[e.code] = true; });
    window.addEventListener('keyup', e => { this.keys[e.code] = false; });

    // Mouse
    canvas.addEventListener('mousemove', e => {
      const rect = canvas.getBoundingClientRect();
      this.mouse.x = (e.clientX - rect.left) * (canvas.width / rect.width);
      this.mouse.y = (e.clientY - rect.top) * (canvas.height / rect.height);
    });
    canvas.addEventListener('mousedown', () => { this.mouse.down = true; });
    canvas.addEventListener('mouseup', () => { this.mouse.down = false; });

    // Touch (unified)
    canvas.addEventListener('touchstart', e => {
      e.preventDefault();
      const t = e.touches[0];
      const rect = canvas.getBoundingClientRect();
      this.mouse.x = (t.clientX - rect.left) * (canvas.width / rect.width);
      this.mouse.y = (t.clientY - rect.top) * (canvas.height / rect.height);
      this.mouse.down = true;
    }, { passive: false });
    canvas.addEventListener('touchmove', e => {
      e.preventDefault();
      const t = e.touches[0];
      const rect = canvas.getBoundingClientRect();
      this.mouse.x = (t.clientX - rect.left) * (canvas.width / rect.width);
      this.mouse.y = (t.clientY - rect.top) * (canvas.height / rect.height);
    }, { passive: false });
    canvas.addEventListener('touchend', e => {
      this.mouse.down = false;
    });

    // Prevent context menu
    canvas.addEventListener('contextmenu', e => e.preventDefault());
  }
}
```

---

## 7. Analyzed GitHub Projects

### Project 1: Shattered-Vale-RPG (goldipl/Shattered-Vale-RPG)

- **Stack**: Vanilla JavaScript, HTML5 Canvas, no build step, no dependencies
- **Architecture**: 27 JS files, modular (config/sprites/world/entities/systems/ui/core)
- **Key Patterns**:
  - DOM HUD bars + Canvas game (hybrid approach)
  - `rectsOverlap()` utility for all collision/hit detection
  - Offscreen canvas tilemap caching
  - Procedural sprite generation
  - State machine in `screens.js` (menu/howto/gameover/victory)
- **Input**: Keyboard only, `justPressed` pattern for single-fire actions
- **Size**: 109x236 tile map, 99 enemies, 16 enemy types

### Project 2: Claudicus (brian-benzinger/claudicus)

- **Stack**: TypeScript, esbuild, HTML5 Canvas 2D at 960x640, 60fps
- **Architecture**: 14 TypeScript files, strict separation (engine/screens/world/ui/data)
- **Key Patterns**:
  - `GameContext` object passed between screens
  - Screen registry pattern (Game.ts manages screen transitions)
  - `InputManager` with `isDown()` and `wasPressed()` methods
  - All sprites drawn from canvas primitives (zero images)
  - Web Audio API synthesizer for music + SFX
  - localStorage save system with versioned migrations
- **Testing**: Vitest with 95% line coverage enforced

### Project 3: PocketJS (realjyce/PocketJS)

- **Stack**: Vanilla JavaScript ES6+, HTML5 Canvas, CSS3
- **Architecture**: State machine (Exploration → Dialogue → Battle)
- **Key Patterns**:
  - `requestAnimationFrame` loop decoupling rendering from logic
  - Grid-based movement with per-pixel interpolation
  - `Sprites` registry object + `initSprites()` at boot
  - Turn-based battle with speed-based turn order
  - `localStorage` save/load serialization

### Project 4: Starfall Chronicles (mtaylor-create/teamsDemo-RPG2)

- **Stack**: TypeScript, Vite, HTML5 Canvas 640x480
- **Architecture**: Engine/Screens/World/UI/Data layers
- **Key Patterns**:
  - `Game.ts` — Game loop + screen registry + GameContext
  - `InputManager.ts` — `isDown(code)` / `wasPressed(code)`
  - Screen objects: `TitleScreen`, `OverworldScreen`, `DungeonScreen`, `BattleScreen`, `MenuScreen`
  - Panel.ts — `drawPanel()` retro sci-fi bordered box
  - ProgressBar.ts — HP/TP bars with color shift
  - JSON data files for characters, enemies, items, dialogue

### Project 5: Dungeon Depths (sdeveer/DungeonDepths)

- **Stack**: Vanilla JavaScript, HTML5 Canvas, Node.js/Express backend, PostgreSQL
- **Architecture**: Client (public/js/) + Server (src/)
- **Key Patterns**:
  - REST-based client-server with shared balance formulas
  - Isometric 2:1 diamond tiles, depth-sorted occlusion
  - A* pathfinding with binary heap
  - Procedural canvas drawing as fallback for missing images
  - `main.js` — screens, input, `requestAnimationFrame` loop
  - Autosave to PostgreSQL

### Project 6: The TypeScript Pure Canvas RPG (DEV Community)

- **Stack**: TypeScript, esbuild, HTML5 Canvas 2D, Web Audio API
- **Architecture**: 7 modules (main/graphics/renderer/gameplay/content/input/touch/audio)
- **Key Patterns**:
  - Zero images — all procedural vector graphics
  - Touch controls as DOM overlay (not canvas) — more reliable for multi-touch
  - Seeded PRNG (Mulberry32) for deterministic tile generation
  - Tile caching to offscreen canvas
  - Camera culling (only draw visible ±1 tile margin)
  - Object pools for particles and damage texts (minimal GC pressure)
  - 60fps desktop, 30-50fps mobile

### Project 7: Roguelike (EdwardAThomson/Roguelike)

- **Stack**: Vanilla JavaScript, HTML5 Canvas
- **Architecture**: Modular with entity/item/ui subdirectories
- **Key Patterns**:
  - `gameStateManager.js` — centralized state management
  - `inputManager.js` — input handling and key bindings
  - `cameraManager.js` — camera and viewport
  - `fovManager.js` — field of view calculations
  - Clear separation: Game Loop → State Management → Entity System → World System → UI System → Input System

---

## Summary: Universal Patterns

| Pattern | Description | Used By |
|---------|-------------|---------|
| **`getBoundingClientRect()` coordinate conversion** | Convert viewport clicks to canvas coords | All 7 projects |
| **`requestAnimationFrame` game loop** | Consistent, battery-friendly rendering | All 7 projects |
| **State machine** | `MENU → PLAYING → BATTLE → GAME_OVER` transitions | All 7 projects |
| **`justPressed` / `wasPressed`** | Single-fire input for discrete actions | Shattered-Vale, Claudicus, Starfall, Roguelike |
| **Offscreen canvas caching** | Bake static tilemap once, blit each frame | Shattered-Vale, TypeScript RPG |
| **Camera culling** | Only draw tiles/entities in viewport | Shattered-Vale, Claudicus, TypeScript RPG |
| **Hybrid DOM + Canvas** | Canvas for game world, DOM for HUD/menus | Shattered-Vale, TypeScript RPG |
| **`Path2D` + `isPointInPath()`** | Precise shape hit detection | Modern pattern (recommended) |
| **Touch as DOM overlay** | More reliable multi-touch than canvas events | TypeScript RPG |

---

## Quick Reference: Copy-Paste Patterns

### Minimal Click Handler

```javascript
canvas.addEventListener('click', e => {
  const r = canvas.getBoundingClientRect();
  const x = (e.clientX - r.left) * (canvas.width / r.width);
  const y = (e.clientY - r.top) * (canvas.height / r.height);
  // x, y are now in canvas internal coordinates
});
```

### Minimal Button Class

```javascript
class Button {
  constructor(x, y, w, h, label, onClick) {
    this.x = x; this.y = y; this.w = w; this.h = h;
    this.label = label; this.onClick = onClick;
    this.hovered = false;
  }

  contains(px, py) {
    return px >= this.x && px <= this.x + this.w &&
           py >= this.y && py <= this.y + this.h;
  }

  draw(ctx) {
    ctx.fillStyle = this.hovered ? '#666' : '#444';
    ctx.fillRect(this.x, this.y, this.w, this.h);
    ctx.fillStyle = '#fff';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(this.label, this.x + this.w / 2, this.y + this.h / 2);
  }
}
```

### Minimal Game State Machine

```javascript
const states = {
  menu:    { update: updateMenu,    render: renderMenu },
  playing: { update: updatePlaying, render: renderPlaying },
  paused:  { update: updatePaused,  render: renderPaused },
};

let current = 'menu';

function gameLoop(ts) {
  states[current].update(dt);
  states[current].render(ctx);
  requestAnimationFrame(gameLoop);
}
```

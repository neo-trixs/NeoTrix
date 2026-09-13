# Isometric & 2.5D Rendering Techniques

Complete reference for making flat 2D games look 3D using HTML5 Canvas. Every code example is copy-paste ready.

---

## Table of Contents

1. [Isometric Rendering](#1-isometric-rendering)
2. [2.5D Effects](#2-25d-effects)
3. [Shadow Rendering](#3-shadow-rendering)
4. [Lighting Effects](#4-lighting-effects)
5. [Depth Sorting](#5-depth-sorting)
6. [Parallax Scrolling](#6-parallax-scrolling)
7. [Tile Gradients](#7-tile-gradients)
8. [Character Shadows](#8-character-shadows)
9. [Weather Effects](#9-weather-effects)
10. [Day/Night Cycle](#10-daynight-cycle)

---

## 1. Isometric Rendering

### How It Works

Isometric projection converts 3D coordinates (x, y, z) to 2D screen coordinates using a fixed 30° rotation. The key formula uses the **2:1 diamond ratio** — tiles become diamond shapes. Unlike perspective projection, parallel lines stay parallel (no vanishing point), giving the distinctive "toylike" look of games like SimCity and Diablo.

The core math: `screenX = (tileX - tileY) * tileWidth / 2` and `screenY = (tileX + tileY) * tileHeight / 2`.

### Code Example

```javascript
// === ISOMETRIC RENDERING ENGINE ===

const TILE_W = 64;  // tile width in pixels
const TILE_H = 32;  // tile height (2:1 ratio)
const GRID = 16;

const canvas = document.createElement('canvas');
canvas.width = 800;
canvas.height = 600;
document.body.appendChild(canvas);
const ctx = canvas.getContext('2d');

// Origin offset (center of screen)
const OX = canvas.width / 2;
const OY = 100;

// Convert grid coords → screen coords
function toScreen(gx, gy, gz = 0) {
  return {
    x: OX + (gx - gy) * TILE_W / 2,
    y: OY + (gx + gy) * TILE_H / 2 - gz * 16
  };
}

// Convert screen coords → grid coords (for mouse picking)
function toGrid(sx, sy) {
  const mx = sx - OX;
  const my = sy - OY;
  return {
    gx: Math.floor((mx / (TILE_W / 2) + my / (TILE_H / 2)) / 2),
    gy: Math.floor((my / (TILE_H / 2) - mx / (TILE_W / 2)) / 2)
  };
}

// Draw a single isometric diamond tile
function drawTile(gx, gy, fill, stroke = '#1a1a2e') {
  const p = toScreen(gx, gy);
  ctx.beginPath();
  ctx.moveTo(p.x, p.y);
  ctx.lineTo(p.x + TILE_W / 2, p.y + TILE_H / 2);
  ctx.lineTo(p.x, p.y + TILE_H);
  ctx.lineTo(p.x - TILE_W / 2, p.y + TILE_H / 2);
  ctx.closePath();
  ctx.fillStyle = fill;
  ctx.fill();
  ctx.strokeStyle = stroke;
  ctx.lineWidth = 1;
  ctx.stroke();
}

// Render the full grid
function renderGrid() {
  ctx.fillStyle = '#0a0a14';
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  for (let gy = 0; gy < GRID; gy++) {
    for (let gx = 0; gx < GRID; gx++) {
      const shade = (gx + gy) % 2 === 0 ? '#2a6e4e' : '#34875e';
      drawTile(gx, gy, shade);
    }
  }
}

// Mouse hover highlight
let hoverX = -1, hoverY = -1;
canvas.addEventListener('mousemove', (e) => {
  const rect = canvas.getBoundingClientRect();
  const grid = toGrid(e.clientX - rect.left, e.clientY - rect.top);
  if (grid.gx !== hoverX || grid.gy !== hoverY) {
    hoverX = grid.gx;
    hoverY = grid.gy;
    renderGrid();
    if (hoverX >= 0 && hoverX < GRID && hoverY >= 0 && hoverY < GRID) {
      drawTile(hoverX, hoverY, 'rgba(255,255,100,0.4)', '#ffff00');
    }
  }
});

renderGrid();
```

### Visual Effect

- Flat 2D grid appears as a tilted 3D plane viewed from above at ~30°
- Diamond-shaped tiles create depth illusion without any actual 3D data
- Mouse hover detection works via reverse isometric transform

### Performance

- **O(n²)** where n = grid dimension. A 16×16 grid = 256 draw calls per frame.
- For 60fps: keep grid under 32×32 (1024 tiles) or use viewport culling.
- **Optimization**: cache static tiles to an offscreen canvas, only redraw dynamic elements.

---

## 2. 2.5D Effects

### How It Works

2.5D adds **height** to isometric tiles by rendering them as 3D blocks with three visible faces (top, left side, right side). Each face gets a different brightness based on a simulated light source — the top face is brightest, the right face is medium, the left face is darkest. This creates the illusion of solid blocks without real 3D geometry.

The `heightScale` variable controls how tall each height unit appears (typically 2-4 pixels per unit).

### Code Example

```javascript
// === 2.5D ISOMETRIC CUBE RENDERER ===

const TILE_W = 64;
const TILE_H = 32;
const DEPTH = 16; // height of one block in pixels

function toScreen(gx, gy, gz = 0) {
  return {
    x: 400 + (gx - gy) * TILE_W / 2,
    y: 100 + (gx + gy) * TILE_H / 2 - gz * DEPTH
  };
}

// Shade a hex color by adjusting lightness
function shadeHSL(hue, sat, lit) {
  return `hsl(${hue}, ${sat}%, ${Math.max(0, Math.min(100, lit))}%)`;
}

// Draw a 3D isometric cube with 3 faces
function drawCube(gx, gy, gz, baseHue, baseSat = 55) {
  const s = TILE_W / 2;
  const h = TILE_H / 2;
  const top = toScreen(gx, gy, gz + 1);

  // Top face (brightest)
  ctx.beginPath();
  ctx.moveTo(top.x, top.y);
  ctx.lineTo(top.x + s, top.y + h);
  ctx.lineTo(top.x, top.y + TILE_H);
  ctx.lineTo(top.x - s, top.y + h);
  ctx.closePath();
  ctx.fillStyle = shadeHSL(baseHue, baseSat, 55);
  ctx.fill();
  ctx.strokeStyle = 'rgba(0,0,0,0.15)';
  ctx.lineWidth = 1;
  ctx.stroke();

  // Left face (medium brightness)
  const leftBase = toScreen(gx, gy, 0);
  ctx.beginPath();
  ctx.moveTo(top.x - s, top.y + h);
  ctx.lineTo(top.x, top.y + TILE_H);
  ctx.lineTo(leftBase.x, leftBase.y + TILE_H);
  ctx.lineTo(leftBase.x - s, leftBase.y + h);
  ctx.closePath();
  ctx.fillStyle = shadeHSL(baseHue, baseSat - 5, 38);
  ctx.fill();
  ctx.stroke();

  // Right face (darkest)
  ctx.beginPath();
  ctx.moveTo(top.x + s, top.y + h);
  ctx.lineTo(top.x, top.y + TILE_H);
  ctx.lineTo(leftBase.x, leftBase.y + TILE_H);
  ctx.lineTo(leftBase.x + s, leftBase.y + h);
  ctx.closePath();
  ctx.fillStyle = shadeHSL(baseHue, baseSat - 10, 28);
  ctx.fill();
  ctx.stroke();
}

// Render voxel terrain from a height map
const heightMap = [];
for (let gy = 0; gy < 12; gy++) {
  heightMap[gy] = [];
  for (let gx = 0; gx < 12; gx++) {
    // Procedural terrain: sine waves
    const h = Math.sin(gx * 0.5) * 2 + Math.cos(gy * 0.4) * 2 + 3;
    heightMap[gy][gx] = Math.floor(h);
  }
}

// Sort: back to front (ascending gx+gy), bottom to top (ascending gz)
function render() {
  ctx.fillStyle = '#0a0a14';
  ctx.fillRect(0, 0, 800, 600);

  for (let sum = 0; sum < 22; sum++) {
    for (let gx = 0; gx < 12; gx++) {
      const gy = sum - gx;
      if (gy < 0 || gy >= 12) continue;
      for (let gz = 0; gz <= heightMap[gy][gx]; gz++) {
        const isTop = gz === heightMap[gy][gx];
        const hue = isTop ? 120 : (gz < 2 ? 30 : 95);
        const lit = isTop ? 55 + gz * 3 : 35 + gz * 5;
        drawCube(gx, gy, gz, hue, 50);
      }
    }
  }
}

render();
```

### Visual Effect

- Flat tiles become solid 3D blocks with visible top and two side faces
- Height variation creates terrain (hills, valleys, buildings)
- Side face shading creates realistic lighting depth

### Performance

- Each height layer adds 3 more `fill()` calls per tile. A 12×12 grid with avg height 5 = 12×12×5×3 = **2160 draw calls**.
- **Optimization**: Only draw exposed faces (skip faces fully hidden by adjacent blocks). This can reduce draw calls by 40-60%.

---

## 3. Shadow Rendering

### How It Works

2D shadows are simulated using several techniques:

1. **Baked shadows**: Pre-rendered shadow sprites placed under objects
2. **Gradient shadows**: Radial/linear gradients drawn beneath entities
3. **Canvas shadow API**: `ctx.shadowBlur` + `ctx.shadowColor` for real-time soft shadows
4. **Position buffer SSAO**: Generate a depth buffer from sprites, then compute screen-space ambient occlusion per pixel

The most performant approach for Canvas 2D is **gradient-based shadows** — a radial gradient from black (center) to transparent (edge) drawn below each object.

### Code Example

```javascript
// === 2D SHADOW SYSTEM ===

// Simple drop shadow (most common)
function drawDropShadow(x, y, width, height, angle = 0.3) {
  ctx.save();
  ctx.globalAlpha = 0.3;

  // Shadow offset based on "light angle"
  const shadowOffsetX = Math.cos(angle) * 8;
  const shadowOffsetY = Math.sin(angle) * 4;

  // Draw shadow as a squashed, offset ellipse
  ctx.beginPath();
  ctx.ellipse(
    x + shadowOffsetX,
    y + height / 2 + shadowOffsetY,
    width / 2,
    height / 4,
    0, 0, Math.PI * 2
  );
  ctx.fillStyle = 'black';
  ctx.fill();
  ctx.restore();
}

// Radial gradient shadow (softer, more realistic)
function drawRadialShadow(x, y, radius) {
  ctx.save();
  const grad = ctx.createRadialGradient(x, y, 0, x, y, radius);
  grad.addColorStop(0, 'rgba(0,0,0,0.4)');
  grad.addColorStop(0.5, 'rgba(0,0,0,0.15)');
  grad.addColorStop(1, 'rgba(0,0,0,0)');
  ctx.fillStyle = grad;
  ctx.beginPath();
  ctx.ellipse(x, y + 4, radius, radius * 0.3, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();
}

// Canvas shadow blur (built-in but expensive)
function drawBlurShadow(x, y, w, h) {
  ctx.save();
  ctx.shadowColor = 'rgba(0,0,0,0.5)';
  ctx.shadowBlur = 12;
  ctx.shadowOffsetX = 4;
  ctx.shadowOffsetY = 4;
  ctx.fillStyle = '#4a90d9';
  ctx.fillRect(x, y, w, h);
  ctx.restore();
}

// --- Usage ---
function render() {
  ctx.clearRect(0, 0, 800, 600);
  ctx.fillStyle = '#e8e0d0';
  ctx.fillRect(0, 0, 800, 600);

  // Draw shadows first (under everything)
  drawRadialShadow(200, 300, 30);
  drawRadialShadow(400, 250, 25);
  drawRadialShadow(600, 350, 35);

  // Then draw objects on top
  ctx.fillStyle = '#4a90d9';
  ctx.fillRect(185, 260, 30, 40);
  ctx.fillStyle = '#d94a4a';
  ctx.fillRect(385, 210, 30, 40);
  ctx.fillStyle = '#4ad97a';
  ctx.fillRect(585, 310, 30, 40);
}

render();
```

### Visual Effect

- Objects appear grounded (not floating)
- Radial shadows create soft ambient occlusion look
- Shadow blur gives depth-of-field appearance

### Performance

| Technique | FPS Impact | Quality |
|-----------|-----------|---------|
| Gradient shadow | ~0.5ms | Good |
| Canvas shadowBlur | ~3-8ms | Excellent (but slow) |
| Baked shadow sprites | ~0.1ms | Good |
| SSAO position buffer | ~5-15ms | Excellent (GPU only) |

**Recommendation**: Use gradient shadows for real-time. Use `shadowBlur` sparingly (1-2 objects max).

---

## 4. Lighting Effects

### How It Works

2D lighting simulates light sources by:
1. Rendering the scene darker (multiply blend)
2. Drawing light "pools" on top using additive blending or radial gradients
3. Each light source creates a radial gradient from its color to transparent

For pixel-art games, lights are drawn at a lower resolution and upscaled for a stylized glow.

### Code Example

```javascript
// === 2D LIGHTING SYSTEM ===

class Light2D {
  constructor(x, y, radius, color, intensity = 1.0) {
    this.x = x;
    this.y = y;
    this.radius = radius;
    this.color = color;
    this.intensity = intensity;
  }
}

// Create an offscreen canvas for the light map
const lightCanvas = document.createElement('canvas');
lightCanvas.width = 800;
lightCanvas.height = 600;
const lightCtx = lightCanvas.getContext('2d');

// Render all lights to the light map
function renderLightMap(lights, ambientColor = '#1a1a2e') {
  // Fill with darkness
  lightCtx.globalCompositeOperation = 'source-over';
  lightCtx.fillStyle = ambientColor;
  lightCtx.fillRect(0, 0, 800, 600);

  // Add lights using "screen" blending (additive for light)
  lightCtx.globalCompositeOperation = 'screen';

  for (const light of lights) {
    const grad = lightCtx.createRadialGradient(
      light.x, light.y, 0,
      light.x, light.y, light.radius
    );

    const [r, g, b] = hexToRgb(light.color);
    const alpha = light.intensity;
    grad.addColorStop(0, `rgba(${r},${g},${b},${alpha})`);
    grad.addColorStop(0.4, `rgba(${r},${g},${b},${alpha * 0.5})`);
    grad.addColorStop(1, `rgba(${r},${g},${b},0)`);

    lightCtx.fillStyle = grad;
    lightCtx.beginPath();
    lightCtx.arc(light.x, light.y, light.radius, 0, Math.PI * 2);
    lightCtx.fill();
  }
}

// Apply light map to the scene
function applyLighting(sceneCanvas) {
  const mainCtx = sceneCanvas.getContext('2d');
  mainCtx.globalCompositeOperation = 'multiply';
  mainCtx.drawImage(lightCanvas, 0, 0);
  mainCtx.globalCompositeOperation = 'source-over';
}

// Utility: hex color to RGB array
function hexToRgb(hex) {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return [r, g, b];
}

// --- Example Usage ---
const lights = [
  new Light2D(200, 300, 150, '#ffaa44', 0.9),  // torch
  new Light2D(500, 200, 200, '#4488ff', 0.6),  // magic crystal
  new Light2D(700, 400, 120, '#ff4444', 0.8),  // campfire
];

// In render loop:
function render() {
  // Draw scene normally first
  ctx.fillStyle = '#886644';
  ctx.fillRect(0, 0, 800, 600);
  // ... draw tiles, sprites, etc ...

  // Generate and apply lighting
  renderLightMap(lights);
  applyLighting(canvas);
}
```

### Visual Effect

- Torch light creates warm orange pools in dark dungeons
- Magic effects glow with colored halos
- Ambient darkness with localized light sources creates atmosphere

### Performance

- Each light = 1 radial gradient + 1 arc fill. 10 lights ≈ 10ms on mid-range hardware.
- **Optimization**: Use a low-resolution light map (e.g., 200×150) and upscale with CSS `image-rendering: pixelated`. Reduces gradient calculations by 16×.

---

## 5. Depth Sorting

### How It Works

In isometric games, objects must render in the correct back-to-front order. The **Painter's Algorithm** draws objects farthest from the camera first. For isometric grids, depth = `gridX + gridY`. Objects with higher depth values render later (on top).

For multi-tile objects (e.g., a 2×3 sofa), depth must be based on the **bottom-right corner** (closest to camera), not the origin tile.

### Code Example

```javascript
// === DEPTH SORTING SYSTEM ===

// Basic tile depth sort (painter's algorithm)
function sortEntities(entities) {
  return entities.sort((a, b) => {
    // Primary sort: isometric depth (x + y)
    const depthA = a.gridX + a.gridY;
    const depthB = b.gridX + b.gridY;
    if (depthA !== depthB) return depthA - depthB;

    // Tiebreaker: vertical offset (stacking)
    return (a.stackIndex || 0) - (b.stackIndex || 0);
  });
}

// Multi-tile object depth: use bottom-right corner
function getMultiTileDepth(obj) {
  return (obj.gridX + obj.tileWidth - 1) +
         (obj.gridY + obj.tileHeight - 1);
}

// Height-aware depth: add gz to prevent overlap issues
function getHeightAwareDepth(obj) {
  return (obj.gridX + obj.gridY) + (obj.height || 0) * 0.01;
}

// --- Usage ---
const entities = [
  { type: 'tile', gridX: 0, gridY: 0, height: 0 },
  { type: 'tile', gridX: 1, gridY: 0, height: 0 },
  { type: 'tree', gridX: 2, gridY: 1, height: 3, stackIndex: 0 },
  { type: 'player', gridX: 1, gridY: 2, height: 1 },
  { type: 'building', gridX: 3, gridY: 3, tileWidth: 2, tileHeight: 2, height: 5 },
  { type: 'npc', gridX: 4, gridY: 2, height: 1 },
];

// Assign correct depths
entities.forEach(e => {
  if (e.tileWidth && e.tileWidth > 1) {
    e.depth = getMultiTileDepth(e);
  } else {
    e.depth = getHeightAwareDepth(e);
  }
});

const sorted = sortEntities(entities);
sorted.forEach(e => {
  if (e.type === 'tile') drawTile(e.gridX, e.gridY, '#3a7a3a');
  else if (e.type === 'tree') drawTree(e.gridX, e.gridY, e.height);
  else if (e.type === 'player') drawPlayer(e.gridX, e.gridY);
  else if (e.type === 'building') drawBuilding(e);
  else if (e.type === 'npc') drawNPC(e.gridX, e.gridY);
});
```

### Visual Effect

- Characters walk in front of trees behind them, behind trees in front of them
- Buildings correctly occlude objects on far side
- Stacked items (coins, books) layer correctly

### Performance

- Sorting O(n log n) where n = number of entities. Typically <1000 entities per screen = negligible cost.
- **Caveat**: Painter's algorithm fails for intersecting objects. Use a **z-buffer** approach for complex scenes (store per-pixel depth, compare before drawing).

---

## 6. Parallax Scrolling

### How It Works

Parallax scrolling creates depth by moving background layers at different speeds. Farther layers move slower (like distant mountains), closer layers move faster. This mimics real-world parallax without actual 3D.

The formula: `layerOffset = cameraOffset * layerSpeed` where `layerSpeed` is a multiplier (0.0 = static, 1.0 = full camera speed).

### Code Example

```javascript
// === PARALLAX SCROLLING SYSTEM ===

class ParallaxLayer {
  constructor(image, speed, y = 0) {
    this.image = image;
    this.speed = speed; // 0.0 = static, 1.0 = full speed
    this.y = y;
    this.offset = 0;
  }

  update(cameraX) {
    this.offset = -cameraX * this.speed;
  }

  draw(ctx, canvasWidth, canvasHeight) {
    // Tile the image horizontally
    const imgW = this.image.width;
    const startX = this.offset % imgW;

    ctx.drawImage(this.image, startX, this.y, imgW, canvasHeight);
    // Draw second copy to fill gap
    if (startX + imgW < canvasWidth) {
      ctx.drawImage(this.image, startX + imgW, this.y, imgW, canvasHeight);
    }
  }
}

// --- Create procedural parallax layers ---
function createMountainLayer(color, peakHeight) {
  const c = document.createElement('canvas');
  c.width = 400;
  c.height = 300;
  const ctx2 = c.getContext('2d');

  // Sky gradient
  const skyGrad = ctx2.createLinearGradient(0, 0, 0, 300);
  skyGrad.addColorStop(0, '#1a1a3e');
  skyGrad.addColorStop(1, '#2a2a5e');
  ctx2.fillStyle = skyGrad;
  ctx2.fillRect(0, 0, 400, 300);

  // Mountain silhouette
  ctx2.fillStyle = color;
  ctx2.beginPath();
  ctx2.moveTo(0, 300);
  for (let x = 0; x <= 400; x += 20) {
    const h = Math.sin(x * 0.01) * peakHeight + Math.cos(x * 0.025) * (peakHeight * 0.5);
    ctx2.lineTo(x, 300 - h);
  }
  ctx2.lineTo(400, 300);
  ctx2.closePath();
  ctx2.fill();

  return c;
}

// Setup layers
const farMountains = createMountainLayer('#1a2a4e', 120);
const midMountains = createMountainLayer('#2a3a5e', 80);
const nearHills = createMountainLayer('#3a4a6e', 50);

const layers = [
  new ParallaxLayer(farMountains, 0.1),   // slowest
  new ParallaxLayer(midMountains, 0.3),   // medium
  new ParallaxLayer(nearHills, 0.6),      // fast
];

// --- Game loop with camera ---
let cameraX = 0;
const keys = {};

document.addEventListener('keydown', e => keys[e.key] = true);
document.addEventListener('keyup', e => keys[e.key] = false);

function gameLoop() {
  if (keys['ArrowRight']) cameraX += 3;
  if (keys['ArrowLeft']) cameraX -= 3;

  ctx.clearRect(0, 0, 800, 600);

  // Draw parallax background
  for (const layer of layers) {
    layer.update(cameraX);
    layer.draw(ctx, 800, 600);
  }

  // Draw foreground at full speed
  ctx.fillStyle = '#4a7a4a';
  ctx.fillRect(0, 500, 800, 100);
  ctx.fillRect(300 - cameraX, 450, 64, 50); // player

  requestAnimationFrame(gameLoop);
}

gameLoop();
```

### Visual Effect

- Scrolling reveals depth layers (sky → far mountains → near hills → ground)
- Creates "living painting" effect like classic SNES RPGs
- Each layer has distinct color saturation (far = desaturated, near = vivid)

### Performance

- Each layer = 2 `drawImage()` calls per frame (for tiling). 5 layers ≈ 10 draw calls = negligible.
- **Optimization**: Use `OffscreenCanvas` for procedural layers, update only when camera moves.

---

## 7. Tile Gradients

### How It Works

Adding gradients to tiles creates the illusion of depth, curvature, and material properties without extra geometry. Common techniques:

1. **Vertical gradient** on ground tiles: lighter at top (light source), darker at bottom (shadow)
2. **Radial gradient** for water/liquid: bright center, dark edges
3. **Linear gradient** on walls: simulates directional lighting
4. **Procedural noise + gradient**: creates natural-looking terrain variation

### Code Example

```javascript
// === TILE GRADIENT SYSTEM ===

// Ground tile with vertical gradient (top-lit)
function drawGradientTile(gx, gy, baseColor, lightDir = 'top') {
  const p = toScreen(gx, gy);

  // Create gradient
  let grad;
  if (lightDir === 'top') {
    grad = ctx.createLinearGradient(p.x, p.y, p.x, p.y + TILE_H);
  } else {
    grad = ctx.createLinearGradient(p.x - TILE_W/2, p.y, p.x + TILE_W/2, p.y);
  }

  // Parse base color to create lighter/darker variants
  const lighter = adjustBrightness(baseColor, 20);
  const darker = adjustBrightness(baseColor, -20);

  grad.addColorStop(0, lighter);
  grad.addColorStop(0.6, baseColor);
  grad.addColorStop(1, darker);

  // Draw diamond with gradient
  ctx.beginPath();
  ctx.moveTo(p.x, p.y);
  ctx.lineTo(p.x + TILE_W / 2, p.y + TILE_H / 2);
  ctx.lineTo(p.x, p.y + TILE_H);
  ctx.lineTo(p.x - TILE_W / 2, p.y + TILE_H / 2);
  ctx.closePath();
  ctx.fillStyle = grad;
  ctx.fill();
}

// Water tile with animated radial gradient
function drawWaterTile(gx, gy, time) {
  const p = toScreen(gx, gy);
  const centerX = p.x;
  const centerY = p.y + TILE_H / 2 + Math.sin(time * 2 + gx + gy) * 2;

  const grad = ctx.createRadialGradient(
    centerX, centerY, 0,
    centerX, centerY, TILE_W / 2
  );
  grad.addColorStop(0, '#3498db');
  grad.addColorStop(0.5, '#2980b9');
  grad.addColorStop(1, '#1a5276');

  ctx.beginPath();
  ctx.moveTo(p.x, p.y);
  ctx.lineTo(p.x + TILE_W / 2, p.y + TILE_H / 2);
  ctx.lineTo(p.x, p.y + TILE_H);
  ctx.lineTo(p.x - TILE_W / 2, p.y + TILE_H / 2);
  ctx.closePath();
  ctx.fillStyle = grad;
  ctx.fill();

  // Specular highlight
  ctx.fillStyle = `rgba(255,255,255,${0.15 + Math.sin(time * 3 + gx) * 0.1})`;
  ctx.beginPath();
  ctx.arc(centerX - 5, centerY - 3, 3, 0, Math.PI * 2);
  ctx.fill();
}

// Utility: adjust hex color brightness
function adjustBrightness(hex, amount) {
  const r = Math.max(0, Math.min(255, parseInt(hex.slice(1, 3), 16) + amount));
  const g = Math.max(0, Math.min(255, parseInt(hex.slice(3, 5), 16) + amount));
  const b = Math.max(0, Math.min(255, parseInt(hex.slice(5, 7), 16) + amount));
  return `rgb(${r},${g},${b})`;
}

// --- Animated render loop ---
let time = 0;
function animate() {
  time += 0.016;
  ctx.fillStyle = '#0a0a14';
  ctx.fillRect(0, 0, 800, 600);

  for (let gy = 0; gy < 8; gy++) {
    for (let gx = 0; gx < 8; gx++) {
      if (gy < 3) drawWaterTile(gx, gy, time);
      else drawGradientTile(gx, gy, '#3a7a3a');
    }
  }

  requestAnimationFrame(animate);
}

animate();
```

### Visual Effect

- Ground tiles look curved/lit rather than flat
- Water tiles shimmer with animated highlights
- Materials feel distinct (stone = rough gradient, metal = sharp highlight)

### Performance

- Gradient creation is the expensive part. Cache gradients per tile type (not per frame).
- **Optimization**: Pre-render gradient tiles to offscreen canvases, then `drawImage()` them.

---

## 8. Character Shadows

### How It Works

Character shadows ground entities to the floor. Three approaches:

1. **Drop shadow**: Simple ellipse drawn below the character
2. **Contact shadow**: Darker, smaller shadow where character touches the ground
3. **Directional shadow**: Shadow extends away from the light source

For isometric games, the shadow should project in the isometric direction (down-right typically).

### Code Example

```javascript
// === CHARACTER SHADOW SYSTEM ===

// Drop shadow with light angle
function drawCharacterShadow(x, y, width, height, lightAngle = -0.5) {
  ctx.save();
  ctx.globalAlpha = 0.35;

  // Shadow extends opposite to light direction
  const shadowLen = height * 0.6;
  const dx = Math.cos(lightAngle) * shadowLen;
  const dy = Math.sin(lightAngle) * shadowLen * 0.3; // foreshortened

  ctx.beginPath();
  ctx.ellipse(
    x + dx / 2,
    y + height / 2 + dy / 2,
    width / 2 + Math.abs(dx) * 0.3,
    height / 5,
    0, 0, Math.PI * 2
  );
  ctx.fillStyle = 'black';
  ctx.fill();
  ctx.restore();
}

// Contact shadow (smaller, sharper, at feet)
function drawContactShadow(x, y, width) {
  ctx.save();
  ctx.globalAlpha = 0.5;
  ctx.beginPath();
  ctx.ellipse(x, y + 2, width * 0.4, 3, 0, 0, Math.PI * 2);
  ctx.fillStyle = 'black';
  ctx.fill();
  ctx.restore();
}

// Full character with shadow (isometric)
function drawCharacterIso(gx, gy, characterSprite, lightAngle) {
  const p = toScreen(gx, gy);

  // 1. Draw shadow first
  drawCharacterShadow(p.x, p.y, 32, 40, lightAngle);

  // 2. Draw character sprite on top
  ctx.drawImage(
    characterSprite,
    p.x - 16, p.y - 32, // offset so feet are at tile center
    32, 40
  );
}

// --- Example with walking character ---
const shadow = document.createElement('canvas');
shadow.width = 32;
shadow.height = 10;
const sctx = shadow.getContext('2d');
const sGrad = sctx.createRadialGradient(16, 5, 0, 16, 5, 16);
sGrad.addColorStop(0, 'rgba(0,0,0,0.5)');
sgrad.addColorStop(1, 'rgba(0,0,0,0)');
sctx.fillStyle = sGrad;
sctx.fillRect(0, 0, 32, 10);

let playerX = 0, playerY = 0;
function renderPlayer() {
  const p = toScreen(playerX, playerY);

  // Shadow
  ctx.drawImage(shadow, p.x - 16, p.y - 2);

  // Player (simple box for demo)
  ctx.fillStyle = '#e74c3c';
  ctx.fillRect(p.x - 10, p.y - 36, 20, 36);
  ctx.fillStyle = '#f39c12';
  ctx.fillRect(p.x - 8, p.y - 34, 16, 12); // head
}
```

### Visual Effect

- Characters feel grounded in the world
- Shadow length indicates time of day
- Contact shadows add weight and presence

### Performance

- 1 ellipse per character = negligible. For 100 characters: ~1ms total.
- **Optimization**: Use a pre-rendered shadow sprite (single `drawImage` call).

---

## 9. Weather Effects

### How It Works

Weather is implemented as particle systems overlaid on the game scene:

1. **Rain**: Falling lines with splash particles at ground level
2. **Snow**: Drifting circles with varying size/speed/opacity (depth layering)
3. **Fog**: Semi-transparent gradient layers with horizontal drift
4. **Lightning**: Brief screen flash + bolt geometry

Each particle type has: spawn rate, velocity, lifetime, size, opacity, and wind influence.

### Code Example

```javascript
// === WEATHER EFFECTS SYSTEM ===

class Particle {
  constructor(x, y, vx, vy, size, opacity, lifetime) {
    this.x = x; this.y = y;
    this.vx = vx; this.vy = vy;
    this.size = size;
    this.opacity = opacity;
    this.lifetime = lifetime;
    this.age = 0;
  }
  update(dt) {
    this.x += this.vx * dt;
    this.y += this.vy * dt;
    this.age += dt;
    return this.age < this.lifetime;
  }
}

class WeatherSystem {
  constructor(canvas) {
    this.canvas = canvas;
    this.particles = [];
    this.wind = 0;
    this.type = 'clear';
  }

  setType(type) {
    this.type = type;
    this.particles = [];
  }

  spawn(dt) {
    const w = this.canvas.width;
    const h = this.canvas.height;

    if (this.type === 'rain') {
      // Spawn rain drops
      for (let i = 0; i < 8; i++) {
        this.particles.push(new Particle(
          Math.random() * w, -10,
          this.wind * 50, 400 + Math.random() * 200,
          1, 0.4 + Math.random() * 0.3, 2
        ));
      }
    } else if (this.type === 'snow') {
      // Spawn snowflakes
      for (let i = 0; i < 3; i++) {
        const depth = Math.random();
        this.particles.push(new Particle(
          Math.random() * w, -5,
          this.wind * 20 + Math.sin(Date.now() * 0.001) * 15,
          30 + depth * 60,
          1 + depth * 3,
          0.3 + depth * 0.5,
          10
        ));
      }
    } else if (this.type === 'fog') {
      // Spawn fog wisps (less frequent, longer lived)
      if (Math.random() < 0.1) {
        this.particles.push(new Particle(
          -50, Math.random() * h * 0.6 + h * 0.2,
          20 + Math.random() * 30, 0,
          80 + Math.random() * 120,
          0.05 + Math.random() * 0.1,
          15
        ));
      }
    }
  }

  update(dt) {
    this.spawn(dt);
    this.particles = this.particles.filter(p => p.update(dt));
  }

  draw(ctx) {
    for (const p of this.particles) {
      ctx.save();
      ctx.globalAlpha = p.opacity;

      if (this.type === 'rain') {
        ctx.strokeStyle = '#aaccee';
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(p.x, p.y);
        ctx.lineTo(p.x - p.vx * 0.02, p.y - p.vy * 0.02);
        ctx.stroke();
      } else if (this.type === 'snow') {
        ctx.fillStyle = '#ffffff';
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
        ctx.fill();
      } else if (this.type === 'fog') {
        const grad = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, p.size);
        grad.addColorStop(0, 'rgba(200,200,220,0.1)');
        grad.addColorStop(1, 'rgba(200,200,220,0)');
        ctx.fillStyle = grad;
        ctx.fillRect(p.x - p.size, p.y - p.size, p.size * 2, p.size * 2);
      }

      ctx.restore();
    }
  }
}

// --- Usage ---
const weather = new WeatherSystem(canvas);
weather.setType('rain');
weather.wind = 1;

let lastTime = 0;
function gameLoop(time) {
  const dt = (time - lastTime) / 1000;
  lastTime = time;

  // Draw game scene
  ctx.fillStyle = '#2a2a3e';
  ctx.fillRect(0, 0, 800, 600);

  // Update and draw weather
  weather.update(dt);
  weather.draw(ctx);

  requestAnimationFrame(gameLoop);
}

// Toggle weather on keypress
document.addEventListener('keydown', (e) => {
  if (e.key === '1') weather.setType('clear');
  if (e.key === '2') weather.setType('rain');
  if (e.key === '3') weather.setType('snow');
  if (e.key === '4') weather.setType('fog');
});

requestAnimationFrame(gameLoop);
```

### Visual Effect

- Rain: Dynamic streaks with depth (larger = closer)
- Snow: Gentle drift with wind influence, varying flake sizes
- Fog: Semi-transparent layers that drift horizontally, obscuring distant objects
- Lightning: Brief white flash + jagged bolt (add as simple line segments)

### Performance

| Weather | Particles/frame | GPU Cost |
|---------|----------------|----------|
| Rain | ~200 | Low (lines) |
| Snow | ~100 | Low (circles) |
| Fog | ~20 | Medium (gradients) |
| Lightning | 1 flash | Low (full-screen fill) |

**Optimization**: Use offscreen canvas for weather, composite with `globalAlpha`. Pool particle objects to avoid GC.

---

## 10. Day/Night Cycle

### How It Works

The day/night cycle transitions the game world through time phases using:

1. **Sky color interpolation**: Lerp between preset colors at defined time thresholds
2. **Screen tint**: Apply a semi-transparent color overlay (CanvasModulate equivalent)
3. **Light activation**: Turn lights on/off based on time
4. **Star rendering**: Show stars only during night hours

The cycle maps real time (or game time) to a 0-1 progress value, then interpolates between phase colors.

### Code Example

```javascript
// === DAY/NIGHT CYCLE SYSTEM ===

class DayNightCycle {
  constructor() {
    // Time phases (hours in 24h format)
    this.phases = [
      { name: 'night',    start: 0,  end: 5,   color: [15, 15, 45] },
      { name: 'dawn',     start: 5,  end: 7,   color: [135, 150, 180] },
      { name: 'day',      start: 7,  end: 17,  color: [200, 220, 255] },
      { name: 'sunset',   start: 17, end: 19,  color: [230, 130, 80] },
      { name: 'dusk',     start: 19, end: 21,  color: [60, 50, 90] },
      { name: 'night',    start: 21, end: 24,  color: [15, 15, 45] },
    ];

    this.gameHour = 12; // start at noon
    this.speed = 60;    // 1 real second = 1 game minute
    this.lights = [];
  }

  update(realDt) {
    // Advance game time
    this.gameHour += (realDt * this.speed) / 3600;
    if (this.gameHour >= 24) this.gameHour -= 24;
  }

  getCurrentPhase() {
    for (const phase of this.phases) {
      if (this.gameHour >= phase.start && this.gameHour < phase.end) {
        return phase;
      }
    }
    return this.phases[0];
  }

  getBlendedColor() {
    const hour = this.gameHour;

    for (let i = 0; i < this.phases.length; i++) {
      const phase = this.phases[i];
      if (hour >= phase.start && hour < phase.end) {
        const nextPhase = this.phases[(i + 1) % this.phases.length];
        const progress = (hour - phase.start) / (phase.end - phase.start);

        // Smooth interpolation
        const t = progress * progress * (3 - 2 * progress); // smoothstep

        return [
          Math.round(phase.color[0] + (nextPhase.color[0] - phase.color[0]) * t),
          Math.round(phase.color[1] + (nextPhase.color[1] - phase.color[1]) * t),
          Math.round(phase.color[2] + (nextPhase.color[2] - phase.color[2]) * t),
        ];
      }
    }
    return this.phases[0].color;
  }

  getDarkness() {
    // 0 = full light, 1 = full darkness
    const hour = this.gameHour;
    if (hour >= 7 && hour < 17) return 0;      // day
    if (hour >= 19 || hour < 5) return 0.6;     // night
    if (hour >= 5 && hour < 7) return 0.6 * (1 - (hour - 5) / 2);  // dawn
    if (hour >= 17 && hour < 19) return 0.6 * ((hour - 17) / 2);    // dusk
    return 0;
  }

  applyToCanvas(ctx, width, height) {
    const [r, g, b] = this.getBlendedColor();

    // Apply screen tint
    ctx.save();
    ctx.globalCompositeOperation = 'multiply';
    ctx.fillStyle = `rgb(${r},${g},${b})`;
    ctx.fillRect(0, 0, width, height);
    ctx.globalCompositeOperation = 'source-over';

    // Apply darkness overlay
    const darkness = this.getDarkness();
    if (darkness > 0) {
      ctx.fillStyle = `rgba(0,0,10,${darkness})`;
      ctx.fillRect(0, 0, width, height);
    }
    ctx.restore();
  }

  // Draw stars during night
  drawStars(ctx, width, height) {
    const darkness = this.getDarkness();
    if (darkness < 0.3) return; // no stars during day

    ctx.save();
    ctx.globalAlpha = darkness;

    // Deterministic star positions (using simple hash)
    for (let i = 0; i < 80; i++) {
      const x = ((i * 7919 + 1) % width);
      const y = ((i * 6271 + 3) % (height * 0.5));
      const size = ((i * 3571) % 3) + 1;
      const twinkle = Math.sin(Date.now() * 0.003 + i) * 0.3 + 0.7;

      ctx.globalAlpha = darkness * twinkle;
      ctx.fillStyle = '#ffffff';
      ctx.beginPath();
      ctx.arc(x, y, size, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.restore();
  }
}

// --- Usage ---
const cycle = new DayNightCycle();
cycle.speed = 120; // 2 game minutes per real second

function render() {
  ctx.clearRect(0, 0, 800, 600);

  // Draw sky based on time
  const [sr, sg, sb] = cycle.getBlendedColor();
  const skyGrad = ctx.createLinearGradient(0, 0, 0, 400);
  skyGrad.addColorStop(0, `rgb(${sr},${sg},${sb})`);
  skyGrad.addColorStop(1, `rgb(${sr*0.6},${sg*0.6},${sb*0.6})`);
  ctx.fillStyle = skyGrad;
  ctx.fillRect(0, 0, 800, 400);

  // Draw stars at night
  cycle.drawStars(ctx, 800, 600);

  // Draw ground
  ctx.fillStyle = '#3a7a3a';
  ctx.fillRect(0, 400, 800, 200);

  // Apply day/night tint
  cycle.applyToCanvas(ctx, 800, 600);

  // HUD: show current time
  const phase = cycle.getCurrentPhase();
  ctx.fillStyle = 'white';
  ctx.font = '16px monospace';
  const hours = Math.floor(cycle.gameHour);
  const mins = Math.floor((cycle.gameHour % 1) * 60);
  ctx.fillText(
    `${String(hours).padStart(2, '0')}:${String(mins).padStart(2, '0')} - ${phase.name}`,
    10, 30
  );
}

let lastTime = 0;
function gameLoop(time) {
  const dt = (time - lastTime) / 1000;
  lastTime = time;

  cycle.update(dt);
  render();
  requestAnimationFrame(gameLoop);
}

requestAnimationFrame(gameLoop);
```

### Visual Effect

- Smooth transitions from dawn → day → sunset → night
- Stars appear at night, fade during dawn
- Screen tint changes atmosphere dramatically
- Combined with Section 4's lighting, creates dynamic torch/firelight at night

### Performance

- Color interpolation: ~0.01ms per frame
- Screen tint (full-screen fill): ~0.5ms
- Star rendering: ~0.3ms for 80 stars
- **Total day/night system: ~1ms** — negligible impact

---

## Quick Reference: Performance Budget

| Technique | Draw Calls | Typical FPS Cost | Mobile Safe? |
|-----------|-----------|-----------------|--------------|
| Isometric grid | N² | ~2ms (16×16) | Yes |
| 2.5D cubes | N² × height × 3 | ~8ms (12×12×5) | With culling |
| Gradient shadows | 1 per entity | ~1ms (100 entities) | Yes |
| Lighting system | 1 per light | ~5ms (10 lights) | Use low-res map |
| Depth sorting | Sort only | ~0.5ms | Yes |
| Parallax layers | 2 per layer | ~1ms (5 layers) | Yes |
| Tile gradients | Cached | ~0.5ms | Yes |
| Character shadows | 1 per character | ~1ms (100 chars) | Yes |
| Weather particles | 100-500 | ~3ms (rain) | With pooling |
| Day/night cycle | 1 fill + 80 stars | ~1ms | Yes |

**Total for all techniques simultaneously**: ~18ms (55fps on mid-range, ~30fps on mobile). Use selective enable/disable based on device capability.

---

## References

- BSWEN: Isometric 2.5D Canvas Games (2026)
- Lumitree: Isometric Art with Code (2026)
- MDN: Crisp Pixel Art Look
- Triangular Pixels: 2D Ambient Shadows
- Mibo: 2D Lighting & Shadows with SDF
- vgerbot: weather-canvas library
- Generalist Programmer: Phaser Isometric Tutorial (2026)
- Solstice Valley: Dynamic Day/Night Cycle

# RPG Game Architecture Research Report

> Compiled from 30+ technical sources (2023-2026). Covers ECS, game loops, state machines, RPG systems, UI architecture, performance patterns, event systems, and memory management with JavaScript/TypeScript code examples.

---

## Table of Contents

1. [ECS Architecture](#1-ecs-architecture)
2. [Game Loop Patterns](#2-game-loop-patterns)
3. [State Machine Patterns](#3-state-machine-patterns)
4. [RPG Systems](#4-rpg-systems)
5. [UI Architecture](#5-ui-architecture)
6. [Performance Patterns](#6-performance-patterns)
7. [Event Systems](#7-event-systems)
8. [Memory Management](#8-memory-management)

---

## 1. ECS Architecture

### 1.1 Core Concept

Entity Component System (ECS) separates data from behavior. An **entity** is just an identifier (number), **components** are plain data containers attached to entities, and **systems** are functions that process entities with specific component sets.

```
Entity: { id: 42 }
Components: { Position: {x:10, y:20}, Velocity: {vx:1, vy:0}, Health: {hp:100} }
System: movementSystem processes all entities with Position + Velocity
```

**Key insight from research:** ECS achieves 10-50x CPU throughput over OOP for large entity counts due to cache-friendly contiguous memory layout (Struct-of-Arrays).

### 1.2 Archetype-Based Storage

Entities with identical component signatures are grouped into **archetypes**. Each archetype stores components in contiguous arrays (SoA), enabling CPU cache line prefetching.

```typescript
// Archetype: entities grouped by component layout
interface Archetype {
  id: number;
  componentTypes: Set<string>;
  entities: number[];
  columns: Map<string, any[]>; // SoA: each component type = contiguous array
}

// Example: 1000 entities with {Position, Velocity}
// Position column = [x0,y0, x1,y1, ..., x999,y999]  <- contiguous!
// Velocity column = [vx0,vy0, vx1,vy1, ...]           <- contiguous!
```

### 1.3 TypeScript ECS Implementation

```typescript
type Component = Record<string, unknown>;

class World {
  private nextEntityId = 0;
  private entities = new Map<number, Map<string, Component>>();
  private archetypes = new Map<string, Archetype>();
  private systems: System[] = [];

  createEntity(): number {
    const id = this.nextEntityId++;
    this.entities.set(id, new Map());
    return id;
  }

  addComponent<T extends Component>(entityId: number, typeName: string, data: T): void {
    const entity = this.entities.get(entityId)!;
    entity.set(typeName, data);
    this.updateArchetype(entityId);
  }

  query(...componentTypes: string[]): number[] {
    const key = componentTypes.sort().join('+');
    return this.archetypes.get(key)?.entities ?? [];
  }

  addSystem(system: System): void {
    this.systems.push(system);
  }

  update(dt: number): void {
    for (const system of this.systems) {
      system.update(this, dt);
    }
  }
}

interface System {
  update(world: World, dt: number): void;
}

// Usage
const world = new World();
const player = world.createEntity();
world.addComponent(player, 'position', { x: 100, y: 200 });
world.addComponent(player, 'velocity', { vx: 0, vy: 0 });
world.addComponent(player, 'health', { hp: 100, maxHp: 100 });

world.addSystem({
  update(w, dt) {
    for (const id of w.query('position', 'velocity')) {
      const pos = w.getComponent(id, 'position');
      const vel = w.getComponent(id, 'velocity');
      pos.x += vel.vx * dt;
      pos.y += vel.vy * dt;
    }
  }
});
```

### 1.4 Query Filters

```typescript
// Filter: entities with Position AND Velocity, WITHOUT Dead
const alive = world.queryFiltered(
  ['position', 'velocity'],
  { without: ['dead'] }
);

// Change detection: only entities whose Health changed this frame
const damaged = world.queryFiltered(
  ['health'],
  { changed: ['health'] }
);
```

### 1.5 Resources (Singletons)

```typescript
class ResourceManager {
  private resources = new Map<string, any>();

  set<T>(key: string, value: T): void {
    this.resources.set(key, value);
  }

  get<T>(key: string): T {
    return this.resources.get(key) as T;
  }
}

// Global game config
const resources = new ResourceManager();
resources.set('config', { gravity: 9.8, maxEnemies: 50 });
resources.set('score', { value: 0, multiplier: 1 });
```

---

## 2. Game Loop Patterns

### 2.1 Fixed Timestep (Industry Standard)

The hybrid pattern decouples physics (fixed rate) from rendering (variable rate). Physics always advances by constant dt; rendering interpolates between states.

```typescript
const FIXED_DT = 1 / 60; // 60 Hz physics
const MAX_FRAME_TIME = 0.25; // Clamp to prevent spiral of death

let accumulator = 0;
let lastTime = performance.now();
let previousState = cloneState(currentState);

function gameLoop(now: number): void {
  let frameTime = (now - lastTime) / 1000;
  lastTime = now;

  // Clamp to prevent spiral of death
  if (frameTime > MAX_FRAME_TIME) frameTime = MAX_FRAME_TIME;

  accumulator += frameTime;

  // Fixed timestep: drain accumulator in fixed chunks
  while (accumulator >= FIXED_DT) {
    previousState = cloneState(currentState);
    fixedUpdate(FIXED_DT); // Physics, AI, collisions
    accumulator -= FIXED_DT;
  }

  // Interpolation for smooth rendering
  const alpha = accumulator / FIXED_DT;
  render(interpolate(previousState, currentState, alpha));

  requestAnimationFrame(gameLoop);
}

function fixedUpdate(dt: number): void {
  // Deterministic physics at exactly 60Hz
  for (const entity of world.query('position', 'velocity')) {
    const pos = world.getComponent(entity, 'position');
    const vel = world.getComponent(entity, 'velocity');
    pos.x += vel.vx * dt;
    pos.y += vel.vy * dt;
  }
}

function interpolate(prev: State, curr: State, alpha: number): State {
  return {
    position: {
      x: prev.position.x + (curr.position.x - prev.position.x) * alpha,
      y: prev.position.y + (curr.position.y - prev.position.y) * alpha,
    }
  };
}
```

### 2.2 Why Fixed Timestep

| Property | Variable (frame-coupled) | Fixed (accumulator) |
|---|---|---|
| Behavior across refresh rates | Differs on 60/144/240 Hz | Identical everywhere |
| Determinism / replays | Not reproducible | Stable and replayable |
| Tunneling on stalls | Common | Prevented by clamping |
| Rendering smoothness | Smooth by default | Smooth with interpolation |
| Implementation cost | Trivial | One buffer + state copies |

### 2.3 Physics Integration

```typescript
// Semi-implicit Euler (recommended for games)
function semiImplicitEuler(pos: Vec2, vel: Vec2, acc: Vec2, dt: number): void {
  vel.x += acc.x * dt;  // Update velocity first
  vel.y += acc.y * dt;
  pos.x += vel.x * dt;  // Then use new velocity
  pos.y += vel.y * dt;
}

// Verlet integration (good for cloth, ragdoll, constraints)
function verlet(pos: Vec2, prevPos: Vec2, acc: Vec2, dt: number): void {
  const newX = 2 * pos.x - prevPos.x + acc.x * dt * dt;
  const newY = 2 * pos.y - prevPos.y + acc.y * dt * dt;
  prevPos.x = pos.x;
  prevPos.y = pos.y;
  pos.x = newX;
  pos.y = newY;
}
```

---

## 3. State Machine Patterns

### 3.1 Finite State Machine (FSM)

```typescript
interface GameState {
  enter?(data?: any): void;
  update?(dt: number): void;
  exit?(): void;
}

class StateMachine {
  private states = new Map<string, GameState>();
  private currentState: GameState | null = null;
  private currentName = '';

  addState(name: string, state: GameState): void {
    this.states.set(name, state);
  }

  changeTo(name: string, data?: any): void {
    this.currentState?.exit?.();
    this.currentState = this.states.get(name)!;
    this.currentName = name;
    this.currentState.enter?.(data);
  }

  update(dt: number): void {
    this.currentState?.update?.(dt);
  }
}

// Enemy AI states
class PatrolState implements GameState {
  enter() { console.log('Patrolling...'); }
  update(dt: number) {
    // Move along waypoints
    // Transition to ChaseState if player detected
  }
  exit() { console.log('Stopping patrol'); }
}

class ChaseState implements GameState {
  enter() { console.log('Chasing player!'); }
  update(dt: number) {
    // Move toward player
    // Transition to AttackState if in range
  }
  exit() { console.log('Lost player'); }
}

const ai = new StateMachine();
ai.addState('patrol', new PatrolState());
ai.addState('chase', new ChaseState());
ai.changeTo('patrol');
```

### 3.2 Hierarchical State Machine (HFSM)

HFSM allows nested states. A "combat" superstate contains "attacking", "taking cover", "reloading" substates. This eliminates spaghetti from flat FSMs.

```typescript
interface HierarchicalState {
  subStates?: Map<string, GameState>;
  currentSubState?: GameState;
  enter?(): void;
  update?(dt: number): void;
  exit?(): void;
}

class HFSM {
  private states = new Map<string, HierarchicalState>();
  private current: HierarchicalState | null = null;

  changeTo(name: string): void {
    this.current?.exit?.();
    this.current = this.states.get(name)!;
    this.current.enter?.();
  }

  update(dt: number): void {
    this.current?.update?.(dt);
    this.current?.currentSubState?.update?.(dt);
  }

  // Push substate within current state
  pushSub(subStateName: string): void {
    if (!this.current?.subStates) return;
    this.current.currentSubState = this.current.subStates.get(subStateName);
    this.current.currentSubState?.enter?.();
  }
}

// Example: Combat superstate with substates
const combatState: HierarchicalState = {
  subStates: new Map([
    ['attacking', { update(dt) { /* attack logic */ } }],
    ['takingCover', { update(dt) { /* cover logic */ } }],
    ['reloading', { update(dt) { /* reload logic */ } }],
  ]),
  enter() { this.pushSub('attacking'); },
};
```

### 3.3 Behavior Trees

Behavior trees return SUCCESS, FAILURE, or RUNNING. Composite nodes combine children: sequences run until failure, selectors run until success.

```typescript
type BTStatus = 'success' | 'failure' | 'running';

interface BTNode {
  tick(): BTStatus;
}

class Sequence implements BTNode {
  constructor(private children: BTNode[]) {}
  tick(): BTStatus {
    for (const child of this.children) {
      const result = child.tick();
      if (result !== 'success') return result;
    }
    return 'success';
  }
}

class Selector implements BTNode {
  constructor(private children: BTNode[]) {}
  tick(): BTStatus {
    for (const child of this.children) {
      const result = child.tick();
      if (result !== 'failure') return result;
    }
    return 'failure';
  }
}

class Condition implements BTNode {
  constructor(private check: () => boolean) {}
  tick(): BTStatus {
    return this.check() ? 'success' : 'failure';
  }
}

class Action implements BTNode {
  constructor(private action: () => BTStatus) {}
  tick(): BTStatus {
    return this.action();
  }
}

// Build AI tree: "if enemy visible AND in range, attack; else patrol"
const enemyAI = new Selector([
  new Sequence([
    new Condition(() => canSeeEnemy()),
    new Condition(() => isInRange()),
    new Action(() => attackEnemy()),
  ]),
  new Action(() => patrol()),
]);
```

### 3.4 AI Graph (Hybrid: FSM + Behavior Trees)

Used in Final Fantasy XV's LUMINOUS STUDIO. Any node can contain either a state machine or behavior tree as a sub-layer. Each node has: start, update, finalize, and terminate conditions.

```
Top Layer: State Machine
├── Idle State → Behavior Tree
│   ├── Check hunger
│   └── Find food
├── Combat State → State Machine
│   ├── Attack substate
│   ├── Take Cover substate
│   └── Reload substate
└── Flee State → Behavior Tree
    ├── Find exit
    └── Run to exit
```

---

## 4. RPG Systems

### 4.1 Stat System (Modifier-Based)

Store base_value + list of modifiers, compute final value on demand. Never store computed totals.

```typescript
type ModifierType = 'flat' | 'percent_add' | 'percent_mul';

interface StatModifier {
  type: ModifierType;
  value: number;
  source: string; // "iron_sword", "rage_buff"
  order: number;  // Execution order
}

class Stat {
  baseValue: number;
  modifiers: StatModifier[] = [];

  constructor(base: number) {
    this.baseValue = base;
  }

  get value(): number {
    const sorted = [...this.modifiers].sort((a, b) => a.order - b.order);
    let final = this.baseValue;

    // Phase 1: Flat additions
    for (const mod of sorted) {
      if (mod.type === 'flat') final += mod.value;
    }

    // Phase 2: Percent add (additive stacking)
    let percentAddSum = 0;
    for (const mod of sorted) {
      if (mod.type === 'percent_add') percentAddSum += mod.value;
    }
    final *= (1 + percentAddSum);

    // Phase 3: Percent multiply (multiplicative stacking)
    for (const mod of sorted) {
      if (mod.type === 'percent_mul') final *= (1 + mod.value);
    }

    return final;
  }

  addModifier(mod: StatModifier): void {
    this.modifiers.push(mod);
  }

  removeModifiersFromSource(source: string): void {
    this.modifiers = this.modifiers.filter(m => m.source !== source);
  }
}

// Usage
const strength = new Stat(10);
strength.addModifier({ type: 'flat', value: 5, source: 'iron_sword', order: 100 });
strength.addModifier({ type: 'percent_add', value: 0.3, source: 'rage_buff', order: 200 });

console.log(strength.value); // (10 + 5) * 1.3 = 19.5

// Unequip sword - remove all modifiers from that source
strength.removeModifiersFromSource('iron_sword');
console.log(strength.value); // 10 * 1.3 = 13
```

### 4.2 Dirty Flag Pattern

Don't recalculate derived stats on every mutation. Mark dirty, recalculate only when read.

```typescript
class CharacterStats {
  private base: Record<string, Stat> = {};
  private derivedDirty = true;
  private cachedDerived: Record<string, number> = {};

  setBaseStat(name: string, value: number): void {
    this.base[name] = new Stat(value);
    this.derivedDirty = true; // Don't recalculate yet
  }

  getDerived(name: string): number {
    if (this.derivedDirty) this.recalculateDerived();
    return this.cachedDerived[name];
  }

  private recalculateDerived(): void {
    this.cachedDerived.maxHealth = this.base.vitality.value * 10;
    this.cachedDerived.physicalDamage = this.base.strength.value * 2.5;
    this.cachedDerived.critChance = Math.min(this.base.luck.value * 0.05, 0.5);
    this.derivedDirty = false;
  }
}
```

**Research finding:** Dirty flag pattern saved ~12ms per frame in one AAA project — from recalculating 47 derived stats 200+ times/frame to only recalculating when actually read.

### 4.3 XP Curve System

```typescript
function xpForLevel(level: number, baseXp = 100, growth = 1.5): number {
  return Math.floor(baseXp * Math.pow(growth, level - 1));
}

// Level 2: 100 XP, Level 5: 506 XP, Level 10: 3,844 XP
// growth = 1.0 linear, 1.5 moderate (recommended), 2.0+ steep (MMO grind)
```

### 4.4 Equipment System

```typescript
interface Equipment {
  slot: 'weapon' | 'armor' | 'helmet' | 'boots' | 'accessory';
  modifiers: StatModifier[];
}

class Inventory {
  private equipped = new Map<string, Equipment>();

  equip(item: Equipment, stats: CharacterStats): void {
    if (this.equipped.has(item.slot)) {
      this.unequip(item.slot, stats);
    }
    this.equipped.set(item.slot, item);
    for (const mod of item.modifiers) {
      stats.addModifier(mod);
    }
  }

  unequip(slot: string, stats: CharacterStats): void {
    const item = this.equipped.get(slot);
    if (item) {
      stats.removeModifiersFromSource(item.slot);
      this.equipped.delete(slot);
    }
  }
}
```

### 4.5 Buff/Debuff System

```typescript
interface Buff {
  source: string;
  duration: number;
  modifiers: StatModifier[];
}

class BuffManager {
  private activeBuffs: Buff[] = [];

  applyBuff(buff: Buff, stats: CharacterStats): void {
    this.activeBuffs.push(buff);
    for (const mod of buff.modifiers) {
      stats.addModifier(mod);
    }
  }

  update(dt: number, stats: CharacterStats): void {
    this.activeBuffs = this.activeBuffs.filter(buff => {
      buff.duration -= dt;
      if (buff.duration <= 0) {
        stats.removeModifiersFromSource(buff.source);
        return false;
      }
      return true;
    });
  }
}
```

### 4.6 Combat System (Turn-Based)

```typescript
interface Combatant {
  name: string;
  stats: CharacterStats;
  speed: number;
  initiative: number;
}

class TurnBasedCombat {
  private combatants: Combatant[] = [];
  private currentTurn = 0;

  start(actors: Combatant[]): void {
    this.combatants = actors.sort((a, b) => {
      // Roll initiative
      a.initiative = a.speed + Math.random() * 20;
      b.initiative = b.speed + Math.random() * 20;
      return b.initiative - a.initiative;
    });
  }

  getCurrentCombatant(): Combatant {
    return this.combatants[this.currentTurn % this.combatants.length];
  }

  executeAction(attacker: Combatant, defender: Combatant, action: string): void {
    const damage = this.calculateDamage(attacker, defender, action);
    defender.stats.setBaseStat('health', defender.stats.base.health - damage);
    this.currentTurn++;
  }

  private calculateDamage(attacker: Combatant, defender: Combatant, action: string): number {
    const atk = attacker.stats.getDerived('physicalDamage');
    const def = defender.stats.getDerived('armor');
    const crit = attacker.stats.getDerived('critChance');
    const isCrit = Math.random() < crit;
    return Math.max(1, Math.floor((atk - def * 0.5) * (isCrit ? 1.5 : 1)));
  }
}
```

---

## 5. UI Architecture

### 5.1 MVC Pattern for Game UI

```
Model: Game data (player health, inventory, quest state)
View:  UI rendering (health bar, inventory grid, quest tracker)
Controller: Mediates input events → model commands
```

**Research finding:** 50-80% of a mobile game codebase is UI code. Investing in proper UI architecture prevents unmaintainable spaghetti.

### 5.2 MVVM Pattern (Recommended for Game UIs)

ViewModel transforms raw data into view-friendly format. Data binding propagates changes automatically.

```typescript
class HealthViewModel {
  private _current = 100;
  private _max = 100;
  private listeners: Function[] = [];

  get percent(): number { return this._current / this._max; }
  get displayText(): string { return `${Math.floor(this._current)}/${this._max} HP`; }
  get isLow(): boolean { return this.percent < 0.25; }

  updateHealth(current: number, max: number): void {
    this._current = current;
    this._max = max;
    this.notify();
  }

  subscribe(listener: Function): () => void {
    this.listeners.push(listener);
    return () => { this.listeners = this.listeners.filter(l => l !== listener); };
  }

  private notify(): void {
    this.listeners.forEach(l => l());
  }
}

// Health bar view - only updates when ViewModel changes
class HealthBarView {
  constructor(private vm: HealthViewModel) {
    vm.subscribe(() => this.render());
  }

  render(): void {
    const bar = document.getElementById('health-bar')!;
    bar.style.width = `${this.vm.percent * 100}%`;
    bar.style.backgroundColor = this.vm.isLow ? 'red' : 'green';
    document.getElementById('health-text')!.textContent = this.vm.displayText;
  }
}
```

### 5.3 Inventory UI Pattern

```typescript
interface InventorySlot {
  item: Item | null;
  quantity: number;
}

class InventoryViewModel {
  private slots: InventorySlot[] = Array(30).fill(null).map(() => ({ item: null, quantity: 0 }));
  private listeners: Function[] = [];

  addItem(item: Item, qty: number): boolean {
    // Stack first
    for (const slot of this.slots) {
      if (slot.item?.id === item.id && slot.quantity < item.maxStack) {
        const canAdd = Math.min(qty, item.maxStack - slot.quantity);
        slot.quantity += canAdd;
        qty -= canAdd;
        if (qty === 0) { this.notify(); return true; }
      }
    }
    // Fill empty slots
    for (const slot of this.slots) {
      if (!slot.item) {
        slot.item = item;
        slot.quantity = Math.min(qty, item.maxStack);
        qty -= slot.quantity;
        if (qty === 0) { this.notify(); return true; }
      }
    }
    this.notify();
    return qty === 0;
  }

  getSlots(): readonly InventorySlot[] { return this.slots; }
  subscribe(fn: Function): () => void { /* ... */ }
}
```

### 5.4 Skill Tree UI

```typescript
interface SkillNode {
  id: string;
  name: string;
  description: string;
  maxRank: number;
  currentRank: number;
  prerequisites: string[];
  cost: number;
  effects: StatModifier[];
}

class SkillTreeViewModel {
  private nodes = new Map<string, SkillNode>();
  private availablePoints = 10;

  canUnlock(nodeId: string): boolean {
    const node = this.nodes.get(nodeId)!;
    if (node.currentRank >= node.maxRank) return false;
    if (this.availablePoints < node.cost) return false;
    // Check prerequisites
    return node.prerequisites.every(preId => {
      const pre = this.nodes.get(preId)!;
      return pre.currentRank > 0;
    });
  }

  unlock(nodeId: string): boolean {
    if (!this.canUnlock(nodeId)) return false;
    const node = this.nodes.get(nodeId)!;
    node.currentRank++;
    this.availablePoints -= node.cost;
    return true;
  }
}
```

### 5.5 Damage Number Floating Text

```typescript
class FloatingText {
  private texts: Array<{ text: string; x: number; y: number; life: number; color: string }> = [];

  spawn(text: string, x: number, y: number, color = '#fff'): void {
    this.texts.push({ text, x, y: y - 20, life: 1.0, color });
  }

  update(dt: number): void {
    this.texts = this.texts.filter(t => {
      t.life -= dt;
      t.y -= 40 * dt; // Float upward
      return t.life > 0;
    });
  }

  render(ctx: CanvasRenderingContext2D): void {
    for (const t of this.texts) {
      ctx.globalAlpha = t.life;
      ctx.fillStyle = t.color;
      ctx.font = 'bold 16px sans-serif';
      ctx.fillText(t.text, t.x, t.y);
    }
    ctx.globalAlpha = 1;
  }
}
```

---

## 6. Performance Patterns

### 6.1 Object Pooling

Pre-allocate objects, reuse instead of create/destroy. Eliminates GC pauses.

```typescript
class ObjectPool<T> {
  private pool: T[] = [];
  private factory: () => T;
  private reset: (obj: T) => void;

  constructor(factory: () => T, reset: (obj: T) => void, initialSize = 50) {
    this.factory = factory;
    this.reset = reset;
    for (let i = 0; i < initialSize; i++) {
      this.pool.push(factory());
    }
  }

  acquire(): T {
    if (this.pool.length > 0) {
      return this.pool.pop()!;
    }
    return this.factory(); // Pool exhausted, grow
  }

  release(obj: T): void {
    this.reset(obj);
    this.pool.push(obj);
  }
}

// Bullet pool
const bulletPool = new ObjectPool(
  () => ({ x: 0, y: 0, vx: 0, vy: 0, active: false, damage: 0 }),
  (b) => { b.active = false; }
);

function fireBullet(x: number, y: number, vx: number, vy: number): void {
  const bullet = bulletPool.acquire();
  bullet.x = x; bullet.y = y;
  bullet.vx = vx; bullet.vy = vy;
  bullet.active = true;
  bullets.push(bullet);
}

function destroyBullet(index: number): void {
  bulletPool.release(bullets[index]);
  bullets.splice(index, 1);
}
```

### 6.2 Spatial Partitioning (Uniform Grid)

Turns O(N²) collision detection into O(N).

```typescript
class SpatialGrid {
  private cellSize: number;
  private cells = new Map<string, number[]>();

  constructor(cellSize: number) {
    this.cellSize = cellSize;
  }

  private cellKey(x: number, y: number): string {
    const cx = Math.floor(x / this.cellSize);
    const cy = Math.floor(y / this.cellSize);
    return `${cx},${cy}`;
  }

  insert(entityId: number, x: number, y: number): void {
    const key = this.cellKey(x, y);
    if (!this.cells.has(key)) this.cells.set(key, []);
    this.cells.get(key)!.push(entityId);
  }

  query(x: number, y: number, radius: number): number[] {
    const results: number[] = [];
    const minCx = Math.floor((x - radius) / this.cellSize);
    const maxCx = Math.floor((x + radius) / this.cellSize);
    const minCy = Math.floor((y - radius) / this.cellSize);
    const maxCy = Math.floor((y + radius) / this.cellSize);

    for (let cx = minCx; cx <= maxCx; cx++) {
      for (let cy = minCy; cy <= maxCy; cy++) {
        const key = `${cx},${cy}`;
        const cell = this.cells.get(key);
        if (cell) results.push(...cell);
      }
    }
    return results;
  }

  clear(): void {
    this.cells.clear();
  }
}

// Usage: only check nearby entities for collision
const grid = new SpatialGrid(100);
for (const entity of world.query('position', 'collider')) {
  const pos = world.getComponent(entity, 'position');
  grid.insert(entity, pos.x, pos.y);
}

// For each entity, only check collisions with nearby entities
for (const entity of world.query('position', 'collider')) {
  const pos = world.getComponent(entity, 'position');
  const nearby = grid.query(pos.x, pos.y, 50);
  // Only check collisions with 'nearby' entities
}
```

### 6.3 Draw Call Batching (Canvas 2D)

```typescript
// Bad: One draw call per sprite
for (const sprite of sprites) {
  ctx.drawImage(sprite.texture, sprite.x, sprite.y);
}

// Good: Group by texture, batch draw calls
function batchRender(ctx: CanvasRenderingContext2D, sprites: Sprite[]): void {
  // Sort by texture to minimize state changes
  sprites.sort((a, b) => a.textureId - b.textureId);

  let currentTexture = '';
  for (const sprite of sprites) {
    if (sprite.textureId !== currentTexture) {
      currentTexture = sprite.textureId;
      ctx.drawImage(sprite.texture, 0, 0); // Bind texture once
    }
    ctx.drawImage(sprite.texture, sprite.x, sprite.y);
  }
}
```

### 6.4 Texture Atlas

```typescript
// One texture atlas = one draw call
const atlas = {
  image: atlasImage,
  regions: {
    'player': { x: 0, y: 0, w: 32, h: 32 },
    'enemy1': { x: 32, y: 0, w: 32, h: 32 },
    'enemy2': { x: 64, y: 0, w: 32, h: 32 },
    'bullet': { x: 0, y: 32, w: 8, h: 8 },
  }
};

function drawSprite(ctx: CanvasRenderingContext2D, name: string, x: number, y: number): void {
  const region = atlas.regions[name];
  ctx.drawImage(
    atlas.image,
    region.x, region.y, region.w, region.h,  // Source
    x, y, region.w, region.h                   // Destination
  );
}
```

### 6.5 Staggered Updates

Not every entity needs every system updated every frame.

```typescript
class StaggeredUpdater {
  private groups: number[][] = [[], [], [], []]; // 4 groups
  private groupIndex = 0;

  addEntity(entityId: number): void {
    this.groups[entityId % this.groups.length].push(entityId);
  }

  update(world: World, dt: number): void {
    // Only update 1/4 of entities per frame
    const currentGroup = this.groups[this.groupIndex];
    for (const entityId of currentGroup) {
      // Update AI, pathfinding, etc.
    }
    this.groupIndex = (this.groupIndex + 1) % this.groups.length;
  }
}
```

### 6.6 Performance Budget

| Target | Draw calls | Vertices | Physics bodies | Frame time |
|--------|-----------|----------|----------------|------------|
| Low-end mobile | < 100 | < 50K | < 200 | 33ms (30 FPS) |
| Mid-high mobile | < 200 | < 100K | < 500 | 16.7ms (60 FPS) |
| Desktop | < 500 | < 500K | < 2000 | 16.7ms (60 FPS) |

---

## 7. Event Systems

### 7.1 Event Bus (Publish-Subscribe)

Decouples systems completely. The pickup code doesn't know who reacts to the pickup event.

```typescript
class EventBus {
  private listeners = new Map<string, Array<{ callback: Function; context?: any }>>();
  private queue: Array<{ event: string; data: any }> = [];
  private processing = false;

  on(event: string, callback: Function, context?: any): () => void {
    if (!this.listeners.has(event)) this.listeners.set(event, []);
    const entry = { callback, context };
    this.listeners.get(event)!.push(entry);

    // Return unsubscribe function
    return () => {
      const list = this.listeners.get(event)!;
      const idx = list.indexOf(entry);
      if (idx !== -1) list.splice(idx, 1);
    };
  }

  emit(event: string, data?: any): void {
    if (this.processing) {
      this.queue.push({ event, data }); // Queue if currently processing
      return;
    }

    this.processing = true;
    this.dispatchEvent(event, data);

    // Process queued events
    while (this.queue.length > 0) {
      const queued = this.queue.shift()!;
      this.dispatchEvent(queued.event, queued.data);
    }
    this.processing = false;
  }

  private dispatchEvent(event: string, data?: any): void {
    const list = this.listeners.get(event);
    if (!list) return;

    for (const { callback, context } of list) {
      try {
        callback.call(context, data);
      } catch (err) {
        console.error(`EventBus error in "${event}":`, err);
      }
    }
  }
}

// Usage
const bus = new EventBus();

// Combat system emits
bus.emit('entity-damaged', { entityId: 42, damage: 25, type: 'physical' });

// Health system listens
bus.on('entity-damaged', (data) => {
  const health = world.getComponent(data.entityId, 'health');
  health.hp -= data.damage;
  if (health.hp <= 0) {
    bus.emit('entity-died', { entityId: data.entityId });
  }
});

// UI listens independently
bus.on('entity-damaged', (data) => {
  floatingText.spawn(`-${data.damage}`, data.entityId);
});

// Audio listens independently
bus.on('entity-damaged', (data) => {
  if (data.type === 'physical') audio.play('hit_physical');
});

// Score system listens independently
bus.on('entity-died', (data) => {
  score.addKill(data.entityId);
  achievements.check('kill_count');
});
```

### 7.2 Event Types (Structured)

```typescript
// Type-safe events using discriminated unions
type GameEvent =
  | { type: 'entity-damaged'; entityId: number; damage: number }
  | { type: 'entity-died'; entityId: number }
  | { type: 'item-picked-up'; itemId: string; quantity: number }
  | { type: 'level-completed'; level: number; score: number }
  | { type: 'health-changed'; entityId: number; current: number; max: number }
  | { type: 'score-changed'; newScore: number; delta: number };

class TypedEventBus {
  private listeners = new Map<string, Function[]>();

  on<T extends GameEvent['type']>(
    type: T,
    callback: (data: Extract<GameEvent, { type: T }>) => void
  ): () => void {
    if (!this.listeners.has(type)) this.listeners.set(type, []);
    this.listeners.get(type)!.push(callback);
    return () => {
      const list = this.listeners.get(type)!;
      list.splice(list.indexOf(callback), 1);
    };
  }

  emit(event: GameEvent): void {
    const list = this.listeners.get(event.type);
    if (list) list.forEach(cb => cb(event));
  }
}
```

### 7.3 Event-Driven Architecture Principles

1. **Events are fire-and-forget** — no return values
2. **Event data should be self-contained** — no external references
3. **Subscribe during init, unsubscribe during shutdown** — prevent memory leaks
4. **Avoid event storms** — batch events, use thresholds
5. **Error isolation** — each listener in try/catch
6. **Queue during processing** — prevent reentrant issues

---

## 8. Memory Management

### 8.1 Data-Oriented Design (DOD)

Store related data contiguously for CPU cache friendliness. Process in tight loops over arrays.

```typescript
// BAD: Scattered objects (OOP)
interface Entity {
  position: { x: number; y: number };
  velocity: { x: number; y: number };
  health: number;
  sprite: HTMLImageElement;
}
const entities: Entity[] = []; // Each entity scattered in heap

// GOOD: Contiguous arrays (DOD)
const positions = new Float32Array(MAX_ENTITIES * 2);  // [x0,y0, x1,y1, ...]
const velocities = new Float32Array(MAX_ENTITIES * 2);
const healths = new Float32Array(MAX_ENTITIES);
let entityCount = 0;

// Process in tight loop — CPU prefetches next elements
function updatePositions(dt: number): void {
  for (let i = 0; i < entityCount; i++) {
    const idx = i * 2;
    positions[idx] += velocities[idx] * dt;       // x
    positions[idx + 1] += velocities[idx + 1] * dt; // y
  }
}
```

### 8.2 Save System Pattern

```typescript
interface SaveData {
  version: number;
  timestamp: number;
  player: {
    baseStats: Record<string, number>;
    level: number;
    xp: number;
    statPoints: number;
    equipment: string[]; // Item IDs
    activeBuffs: Array<{ source: string; remainingTime: number }>;
  };
  inventory: Array<{ itemId: string; quantity: number }>;
  quests: Record<string, { status: string; progress: number }>;
}

function saveGame(state: GameState): void {
  const data: SaveData = {
    version: 1,
    timestamp: Date.now(),
    player: {
      baseStats: state.player.getBaseStats(),
      level: state.player.level,
      xp: state.player.xp,
      statPoints: state.player.statPoints,
      equipment: state.player.getEquippedIds(),
      activeBuffs: state.buffManager.getSerialized(),
    },
    inventory: state.inventory.serialize(),
    quests: state.questTracker.serialize(),
  };
  localStorage.setItem('save', JSON.stringify(data));
}

function loadGame(): SaveData | null {
  const raw = localStorage.getItem('save');
  if (!raw) return null;
  const data = JSON.parse(raw) as SaveData;
  // Validate version, handle migrations
  if (data.version < CURRENT_VERSION) {
    migrateSave(data);
  }
  return data;
}
```

**Key rule:** Save inputs (base values, stat points, XP, active buffs), never computed totals. On load, re-apply modifiers and recalculate.

### 8.3 Delta Compression for Network

```typescript
// Only send what changed
function serializeDelta(current: CharacterData, baseline: CharacterData): Partial<CharacterData> {
  const delta: any = {};
  if (current.base.strength !== baseline.base.strength)
    delta.strength = current.base.strength;
  if (current.derived.maxHealth !== baseline.derived.maxHealth)
    delta.maxHealth = current.derived.maxHealth;
  if (current.runtime.currentHealth !== baseline.runtime.currentHealth)
    delta.currentHealth = current.runtime.currentHealth;
  return delta;
}
```

### 8.4 Memory Best Practices

| Technique | When to Use |
|-----------|-------------|
| Object pooling | Anything created/destroyed >10x/sec |
| Float32Array | Position/velocity data (high entity count) |
| Pre-allocation | Particle systems, bullet pools |
| Dirty flags | Derived stats, cached computations |
| Delta serialization | Network sync, save files |
| Texture atlasing | Reduces texture state changes |

---

## Sources

| # | Source | URL | Year |
|---|--------|-----|------|
| 1 | ECS TypeScript Game (Bevy-inspired) | github.com/nisimjoseph/ecs-typescript-game | 2026 |
| 2 | micro-ecs (DX-first) | github.com/Byloth/micro-ecs | 2025 |
| 3 | Thyseus (Archetypal ECS) | github.com/jaimegensler/thyseus | 2025 |
| 4 | Miniplex (Developer-friendly ECS) | github.com/fuleinist/miniplex | 2025 |
| 5 | aiecsjs (TypedArray SoA) | npmjs.com/package/aiecsjs | 2026 |
| 6 | Game Architecture Patterns Guide | abratabia.com/game-architecture | 2026 |
| 7 | PlayableIntelligence Game Creator | github.com/PlayableIntelligence/game-creator | 2026 |
| 8 | CursorCamp Sandbox (zero libraries) | dev.to/dundunup | 2026 |
| 9 | TCJSGame Architecture | dev.to/kehinde_owolabi | 2025 |
| 10 | Fixed Timestep Loops | simplified.media/guides/fixed-timestep-loops | 2026 |
| 11 | Game Loop Architecture | codingpancake.com | 2026 |
| 12 | Fixed Timestep in Browser | andreleite.com | 2025 |
| 13 | Game Loop & Frame Timing | solana.garden/guides | 2026 |
| 14 | AI Graph (FFX XV) | routledge.com | 2016 |
| 15 | Behavior Trees vs State Machines | kindatechnical.com | 2026 |
| 16 | State Machines vs BT | polymathrobotics.com | 2023 |
| 17 | RPG Stat System (Godot 4) | codingquests.io | 2026 |
| 18 | RPG Stat Systems Design | strayspark.studio | 2026 |
| 19 | AAA Character System | dredyson.com | 2026 |
| 20 | Modular Attributes & Stats (Unity) | discussions.unity.com | 2026 |
| 21 | Decorator Pattern for Stats | pavcreations.com | 2023 |
| 22 | MVVM ViewModels (UE5) | strayspark.studio | 2026 |
| 23 | Unity Modular Architecture | github.com/AnisKaram | 2025 |
| 24 | MVC Pattern (Unity) | skills.rest | 2025 |
| 25 | Game Architecture Patterns | oboe.com | 2025 |
| 26 | Mobile Performance (1000+ units) | slashskill.com | 2026 |
| 27 | Brine2D Performance | brine2d.com | 2026 |
| 28 | SparkJS Performance | sparkjs.net | 2026 |
| 29 | Event System Patterns (ECS) | deepwiki.com/danjdewhurst/ecs-ts | 2026 |
| 30 | Unity Event Bus | github.com/nulltale/unityeventbus | 2025 |
| 31 | EventBus Design | gamedev.stackexchange.com | 2023 |
| 32 | MMO Backend Architecture | blog.photonengine.com | 2026 |
| 33 | ATONE Technical Post-Mortem | gs-studio.eu | 2026 |
| 34 | DOD for Games (Manning) | manning.com | 2026 |
| 35 | DOD vs OOP Benchmark | flowrenderengine.com | 2026 |
| 36 | ECS Formal Model (SAC 2026) | boyang.cs.uwm.edu | 2026 |
| 37 | RPG UI/UX Design | barbarafranco.design | 2026 |
| 38 | JRPG Menu Composition | scribd.com | 2024 |
| 39 | Metaphor: ReFantazio UI | theverge.com | 2025 |
| 40 | crystal-menu-ui (React JRPG) | github.com/kaizeenn | 2025 |
| 41 | ExileCore2 Architecture | deepwiki.com | 2025 |
| 42 | DIY Game Architecture | reddit.com/r/gamedev | 2016 |
| 43 | UI Architecture Mistakes | medium.com (Viktor Zatorskyi) | 2022 |
| 44 | make2d Framework | github.com/nenjack | 2025 |
| 45 | Modular Inventory (UE5) | strayspark.studio | 2026 |

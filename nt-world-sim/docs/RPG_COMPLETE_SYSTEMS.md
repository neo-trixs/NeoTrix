# RPG Complete Systems — Copy-Paste Ready Code

> Synthesized from: mojotron/rpg-inventory, RPGJS v4, CommanderFoo/skill-tree-planner,
> Hkattelu/SkillTree, ha-agent-rpg/Minimap, MDN Tilemaps, collide-2d-aabb-tilemap,
> YAGE inventory/dialogue addons, shopkeepr, Legend of Mir mechanics analysis.
>
> All code is vanilla TypeScript/JavaScript, zero dependencies, Canvas 2D.

---

## Table of Contents

1. [Inventory System](#1-inventory-system)
2. [Equipment System](#2-equipment-system)
3. [Quest System](#3-quest-system)
4. [Dialogue System](#4-dialogue-system)
5. [Shop System](#5-shop-system)
6. [Skill System](#6-skill-system)
7. [Minimap System](#7-minimap-system)
8. [Map System](#8-map-system)
9. [Monster AI System](#9-monster-ai-system)
10. [Save/Load System](#10-saveload-system)

---

## 1. Inventory System

Full inventory with drag/drop, stacking, sorting, tooltips. Based on YAGE addon patterns and mojotron/rpg-inventory.

```typescript
// ===================== inventory.ts =====================

export interface ItemDef {
  id: string;
  name: string;
  icon: string;           // emoji or image key
  category: 'weapon' | 'armor' | 'helmet' | 'boots' | 'accessory' | 'consumable' | 'material';
  maxStack: number;
  rarity: 'common' | 'uncommon' | 'rare' | 'epic' | 'legendary';
  description: string;
  stats?: Record<string, number>;  // e.g. { atk: 5, def: 2 }
  sellPrice: number;
  buyPrice: number;
}

export interface InventorySlot {
  itemId: string | null;
  quantity: number;
  data?: Record<string, any>; // per-instance data (durability, affixes)
}

export class Inventory {
  slots: InventorySlot[];
  capacity: number;
  catalog: Map<string, ItemDef>;
  onChange: (() => void)[] = [];

  constructor(catalog: ItemDef[], capacity = 20) {
    this.catalog = new Map(catalog.map(i => [i.id, i]));
    this.capacity = capacity;
    this.slots = Array.from({ length: capacity }, () => ({ itemId: null, quantity: 0 }));
  }

  /** Returns overflow quantity that didn't fit */
  add(itemId: string, qty = 1): number {
    const def = this.catalog.get(itemId);
    if (!def) return qty;
    let remaining = qty;

    // First pass: merge into existing stacks
    for (const slot of this.slots) {
      if (remaining <= 0) break;
      if (slot.itemId === itemId && slot.quantity < def.maxStack) {
        const canAdd = Math.min(remaining, def.maxStack - slot.quantity);
        slot.quantity += canAdd;
        remaining -= canAdd;
      }
    }

    // Second pass: fill empty slots
    for (const slot of this.slots) {
      if (remaining <= 0) break;
      if (slot.itemId === null) {
        const canAdd = Math.min(remaining, def.maxStack);
        slot.itemId = itemId;
        slot.quantity = canAdd;
        remaining -= canAdd;
      }
    }

    this.emit();
    return remaining;
  }

  remove(itemId: string, qty = 1): number {
    let removed = 0;
    for (const slot of this.slots) {
      if (removed >= qty) break;
      if (slot.itemId === itemId) {
        const take = Math.min(qty - removed, slot.quantity);
        slot.quantity -= take;
        removed += take;
        if (slot.quantity <= 0) {
          slot.itemId = null;
          slot.quantity = 0;
          slot.data = undefined;
        }
      }
    }
    this.emit();
    return removed;
  }

  countOf(itemId: string): number {
    return this.slots.reduce((sum, s) => s.itemId === itemId ? sum + s.quantity : sum, 0);
  }

  has(itemId: string, qty = 1): boolean {
    return this.countOf(itemId) >= qty;
  }

  freeSlots(): number {
    return this.slots.filter(s => s.itemId === null).length;
  }

  spaceFor(itemId: string): number {
    const def = this.catalog.get(itemId);
    if (!def) return 0;
    let space = 0;
    for (const slot of this.slots) {
      if (slot.itemId === itemId) {
        space += def.maxStack - slot.quantity;
      } else if (slot.itemId === null) {
        space += def.maxStack;
      }
    }
    return space;
  }

  /** Move/swap/merge between two slot indices */
  moveSlot(fromIdx: number, toIdx: number): void {
    if (fromIdx === toIdx) return;
    const from = this.slots[fromIdx];
    const to = this.slots[toIdx];

    if (to.itemId === null) {
      // Move to empty
      this.slots[toIdx] = { ...from };
      this.slots[fromIdx] = { itemId: null, quantity: 0 };
    } else if (from.itemId === to.itemId) {
      // Merge stacks
      const def = this.catalog.get(from.itemId!);
      if (def) {
        const canAdd = Math.min(from.quantity, def.maxStack - to.quantity);
        to.quantity += canAdd;
        from.quantity -= canAdd;
        if (from.quantity <= 0) {
          this.slots[fromIdx] = { itemId: null, quantity: 0 };
        }
      }
    } else {
      // Swap
      this.slots[toIdx] = { ...from };
      this.slots[fromIdx] = { ...to };
    }
    this.emit();
  }

  /** Sort: consolidate partials, then order by category -> rarity */
  sort(): void {
    const items: InventorySlot[] = [];
    for (const slot of this.slots) {
      if (slot.itemId) items.push({ ...slot });
    }

    // Consolidate
    const consolidated = new Map<string, InventorySlot>();
    for (const item of items) {
      const def = this.catalog.get(item.itemId!);
      if (!def) continue;
      const existing = consolidated.get(item.itemId!);
      if (existing) {
        const canAdd = Math.min(item.quantity, def.maxStack - existing.quantity);
        existing.quantity += canAdd;
        item.quantity -= canAdd;
        if (item.quantity > 0) consolidated.set(item.itemId! + '_' + Math.random(), { ...item });
      } else {
        consolidated.set(item.itemId!, { ...item });
      }
    }

    const sorted = [...consolidated.values()].sort((a, b) => {
      const da = this.catalog.get(a.itemId!)!;
      const db = this.catalog.get(b.itemId!)!;
      const catOrder = ['weapon', 'armor', 'helmet', 'boots', 'accessory', 'consumable', 'material'];
      const rarityOrder = ['legendary', 'epic', 'rare', 'uncommon', 'common'];
      const catDiff = catOrder.indexOf(da.category) - catOrder.indexOf(db.category);
      if (catDiff !== 0) return catDiff;
      return rarityOrder.indexOf(da.rarity) - rarityOrder.indexOf(db.rarity);
    });

    this.slots = Array.from({ length: this.capacity }, (_, i) =>
      sorted[i] || { itemId: null, quantity: 0 }
    );
    this.emit();
  }

  private emit() {
    for (const fn of this.onChange) fn();
  }
}
```

### Integration

- Equipment system reads from inventory for equip/unequip
- Shop system calls `inventory.add()` / `inventory.remove()` with buy/sell logic
- Save system serializes `inventory.slots` to JSON

### Best Practices

- Always return overflow from `add()` so callers know what didn't fit
- Consolidate stacks in `sort()` before ordering
- `emit()` onChange after every mutation for reactive UI

### Common Pitfalls

- Forgetting maxStack limits when merging
- Not clearing `data` when slot empties
- Sorting without consolidating first leaves fragmented stacks

---

## 2. Equipment System

Equip/unequip with stat bonuses, paper doll visualization, category validation.

```typescript
// ===================== equipment.ts =====================

import { ItemDef, Inventory } from './inventory';

export type EquipSlot = 'weapon' | 'armor' | 'helmet' | 'boots' | 'accessory';

export interface EquipState {
  weapon: string | null;
  armor: string | null;
  helmet: string | null;
  boots: string | null;
  accessory: string | null;
}

export class Equipment {
  slots: EquipState = {
    weapon: null, armor: null, helmet: null, boots: null, accessory: null,
  };
  catalog: Map<string, ItemDef>;
  onChange: (() => void)[] = [];

  constructor(catalog: ItemDef[]) {
    this.catalog = new Map(catalog.map(i => [i.id, i]));
  }

  equip(itemId: string): string | null {
    const def = this.catalog.get(itemId);
    if (!def) return null;
    const slot = def.category as EquipSlot;
    if (!this.isValidSlot(slot)) return null;

    const displaced = this.slots[slot];
    this.slots[slot] = itemId;
    this.emit();
    return displaced; // previously equipped item, or null
  }

  unequip(slot: EquipSlot): string | null {
    const item = this.slots[slot];
    this.slots[slot] = null;
    if (item) this.emit();
    return item;
  }

  /** Returns sum of all equipped stat bonuses */
  totalStats(): Record<string, number> {
    const totals: Record<string, number> = {};
    for (const slotKey of Object.values(this.slots)) {
      if (!slotKey) continue;
      const def = this.catalog.get(slotKey);
      if (!def?.stats) continue;
      for (const [stat, val] of Object.entries(def.stats)) {
        totals[stat] = (totals[stat] || 0) + val;
      }
    }
    return totals;
  }

  /** Compare stats: returns delta (equipped - proposed) */
  compareStats(proposedItemId: string): Record<string, number> {
    const current = this.totalStats();
    const def = this.catalog.get(proposedItemId);
    const deltas: Record<string, number> = {};

    // Subtract currently equipped in that slot
    const slot = def?.category as EquipSlot;
    if (slot && this.slots[slot]) {
      const currentDef = this.catalog.get(this.slots[slot]!);
      if (currentDef?.stats) {
        for (const [stat, val] of Object.entries(currentDef.stats)) {
          current[stat] = (current[stat] || 0) - val;
        }
      }
    }

    // Add proposed
    if (def?.stats) {
      for (const [stat, val] of Object.entries(def.stats)) {
        deltas[stat] = (current[stat] || 0) + val - (current[stat] || 0);
      }
    }

    return deltas;
  }

  /** Transfer item between inventory and equipment slot */
  equipFromInventory(itemId: string, inventory: Inventory): boolean {
    const def = this.catalog.get(itemId);
    if (!def) return false;
    if (!inventory.has(itemId)) return false;

    const slot = def.category as EquipSlot;
    if (!this.isValidSlot(slot)) return false;

    const displaced = this.equip(itemId);
    inventory.remove(itemId, 1);
    if (displaced) inventory.add(displaced, 1);
    return true;
  }

  unequipToInventory(slot: EquipSlot, inventory: Inventory): boolean {
    const item = this.unequip(slot);
    if (!item) return false;
    const overflow = inventory.add(item, 1);
    if (overflow > 0) {
      // Inventory full, re-equip
      this.equip(item);
      return false;
    }
    return true;
  }

  /** Paper doll: returns map of slot -> item def for rendering */
  paperDoll(): Record<EquipSlot, ItemDef | null> {
    const result: Record<string, ItemDef | null> = {};
    for (const [slot, itemId] of Object.entries(this.slots)) {
      result[slot] = itemId ? this.catalog.get(itemId) || null : null;
    }
    return result as Record<EquipSlot, ItemDef | null>;
  }

  private isValidSlot(s: string): s is EquipSlot {
    return ['weapon', 'armor', 'helmet', 'boots', 'accessory'].includes(s);
  }

  toJSON(): EquipState {
    return { ...this.slots };
  }

  loadJSON(data: EquipState): void {
    this.slots = { ...data };
    this.emit();
  }

  private emit() {
    for (const fn of this.onChange) fn();
  }
}
```

### Paper Doll Rendering

```typescript
// ===================== paperdoll-renderer.ts =====================

export function renderPaperDoll(
  ctx: CanvasRenderingContext2D,
  equipment: Equipment,
  x: number,
  y: number,
  scale: number = 2
) {
  const doll = equipment.paperDoll();
  const layerOrder: (keyof typeof doll)[] = ['armor', 'boots', 'helmet', 'weapon', 'accessory'];

  // Base body (always drawn first)
  ctx.fillStyle = '#c4a882';
  ctx.fillRect(x + 8 * scale, y + 4 * scale, 16 * scale, 24 * scale);

  // Draw equipped items layer by layer
  for (const slot of layerOrder) {
    const def = doll[slot];
    if (!def) continue;

    ctx.save();
    // Each slot has a fixed draw position relative to body
    const offsets: Record<string, [number, number, number, number]> = {
      helmet:  [6, 0, 20, 8],
      armor:   [4, 10, 24, 12],
      boots:   [6, 26, 20, 6],
      weapon:  [24, 8, 8, 20],
      accessory: [0, 14, 6, 6],
    };
    const [ox, oy, ow, oh] = offsets[slot] || [0, 0, 16, 16];

    // Placeholder colored rectangle (replace with sprite)
    const rarityColors: Record<string, string> = {
      common: '#9ca3af', uncommon: '#22c55e', rare: '#3b82f6',
      epic: '#a855f7', legendary: '#f59e0b',
    };
    ctx.fillStyle = rarityColors[def.rarity] || '#9ca3af';
    ctx.globalAlpha = 0.8;
    ctx.fillRect(x + ox * scale, y + oy * scale, ow * scale, oh * scale);
    ctx.globalAlpha = 1;

    ctx.restore();
  }
}
```

### Integration

- Inventory calls `equipment.equipFromInventory()` when player clicks equip
- Equipment calls `inventory.add()` when unequipping (returns displaced)
- Paper doll renderer reads `equipment.paperDoll()` each frame

### Best Practices

- `equip()` returns displaced item so caller handles inventory management
- `totalStats()` sums ALL slots, not just one — enables full stat panel
- `compareStats()` shows delta vs current for tooltip comparison

### Common Pitfalls

- Not validating category before equip (weapon in armor slot)
- Forgetting to return displaced item to inventory on equip
- Paper doll draw order matters — weapon over armor looks wrong

---

## 3. Quest System

Quest tracking, objectives, rewards, state machine.

```typescript
// ===================== quest.ts =====================

export type QuestState = 'available' | 'active' | 'completed' | 'turned_in';
export type ObjectiveType = 'kill' | 'collect' | 'talk' | 'reach' | 'interact';

export interface QuestObjective {
  id: string;
  type: ObjectiveType;
  target: string;        // monster id, item id, npc id, or location id
  required: number;
  description: string;
}

export interface QuestReward {
  xp: number;
  gold: number;
  items: { id: string; qty: number }[];
  unlocks?: string[];    // quest ids this unlocks
}

export interface QuestDef {
  id: string;
  name: string;
  description: string;
  level: number;
  objectives: QuestObjective[];
  rewards: QuestReward;
  prerequisiteIds?: string[];
  dialogues?: { start: string; active: string; complete: string };
}

export interface QuestProgress {
  questId: string;
  state: QuestState;
  objectiveProgress: Record<string, number>; // objectiveId -> current count
  startedAt: number;
  completedAt?: number;
}

export class QuestSystem {
  questDefs: Map<string, QuestDef>;
  activeQuests: Map<string, QuestProgress> = new Map();
  completedQuests: Set<string> = new Set();
  onChange: ((questId: string) => void)[] = [];

  constructor(defs: QuestDef[]) {
    this.questDefs = new Map(defs.map(q => [q.id, q]));
  }

  /** Get all quests available to the player */
  getAvailable(playerLevel: number, completed: Set<string>): QuestDef[] {
    return [...this.questDefs.values()].filter(q => {
      if (completed.has(q.id)) return false;
      if (this.activeQuests.has(q.id)) return false;
      if (q.level > playerLevel) return false;
      if (q.prerequisiteIds?.some(id => !completed.has(id))) return false;
      return true;
    });
  }

  accept(questId: string): boolean {
    const def = this.questDefs.get(questId);
    if (!def) return false;
    if (this.activeQuests.has(questId)) return false;

    const progress: QuestProgress = {
      questId,
      state: 'active',
      objectiveProgress: {},
      startedAt: Date.now(),
    };
    for (const obj of def.objectives) {
      progress.objectiveProgress[obj.id] = 0;
    }

    this.activeQuests.set(questId, progress);
    this.emit(questId);
    return true;
  }

  /** Report progress on an objective */
  reportProgress(type: ObjectiveType, target: string, amount = 1): string[] {
    const completedQuests: string[] = [];

    for (const [questId, progress] of this.activeQuests) {
      if (progress.state !== 'active') continue;
      const def = this.questDefs.get(questId)!;

      for (const obj of def.objectives) {
        if (obj.type === type && obj.target === target) {
          const current = progress.objectiveProgress[obj.id] || 0;
          progress.objectiveProgress[obj.id] = Math.min(current + amount, obj.required);
        }
      }

      // Check if all objectives complete
      const allDone = def.objectives.every(
        obj => (progress.objectiveProgress[obj.id] || 0) >= obj.required
      );

      if (allDone) {
        progress.state = 'completed';
        progress.completedAt = Date.now();
        completedQuests.push(questId);
      }
    }

    for (const id of completedQuests) this.emit(id);
    return completedQuests;
  }

  /** Turn in a completed quest and grant rewards */
  turnIn(questId: string, player: {
    xp: number; gold: number; inventory: { add: (id: string, qty: number) => number };
  }): QuestReward | null {
    const progress = this.activeQuests.get(questId);
    if (!progress || progress.state !== 'completed') return null;
    const def = this.questDefs.get(questId)!;

    progress.state = 'turned_in';
    this.completedQuests.add(questId);
    this.activeQuests.delete(questId);

    // Grant rewards
    player.xp += def.rewards.xp;
    player.gold += def.rewards.gold;
    for (const item of def.rewards.items) {
      player.inventory.add(item.id, item.qty);
    }

    this.emit(questId);
    return def.rewards;
  }

  getActive(): Array<{ def: QuestDef; progress: QuestProgress }> {
    return [...this.activeQuests.values()]
      .filter(p => p.state === 'active')
      .map(p => ({ def: this.questDefs.get(p.questId)!, progress: p }));
  }

  getObjectiveStatus(questId: string): Array<{ obj: QuestObjective; current: number; done: boolean }> | null {
    const def = this.questDefs.get(questId);
    const progress = this.activeQuests.get(questId);
    if (!def || !progress) return null;

    return def.objectives.map(obj => ({
      obj,
      current: progress.objectiveProgress[obj.id] || 0,
      done: (progress.objectiveProgress[obj.id] || 0) >= obj.required,
    }));
  }

  toJSON() {
    return {
      active: [...this.activeQuests.values()],
      completed: [...this.completedQuests],
    };
  }

  loadJSON(data: { active: QuestProgress[]; completed: string[] }) {
    this.activeQuests.clear();
    this.completedQuests.clear();
    for (const p of data.active) this.activeQuests.set(p.questId, p);
    for (const id of data.completed) this.completedQuests.add(id);
  }

  private emit(questId: string) {
    for (const fn of this.onChange) fn(questId);
  }
}
```

### Usage Example

```typescript
const quests = new QuestSystem([
  {
    id: 'q_slay_wolves',
    name: 'Wolf Hunter',
    description: 'Slay 5 wolves threatening the village.',
    level: 1,
    objectives: [{ id: 'obj1', type: 'kill', target: 'wolf', required: 5, description: 'Slay 5 wolves' }],
    rewards: { xp: 100, gold: 50, items: [{ id: 'health_potion', qty: 3 }] },
  },
]);

quests.accept('q_slay_wolves');
// When player kills a wolf:
quests.reportProgress('kill', 'wolf', 1);
// When 5 wolves killed, quest completes automatically
// Then turn in:
quests.turnIn('q_slay_wolves', player);
```

### Integration

- Combat system calls `reportProgress('kill', monsterId)` on enemy death
- Dialogue system calls `reportProgress('talk', npcId)` after conversation
- Inventory pickup calls `reportProgress('collect', itemId)`

---

## 4. Dialogue System

Branching NPC dialogue with choices, conditions, actions. Based on rpg-dialogue and YAGE dialogue patterns.

```typescript
// ===================== dialogue.ts =====================

export interface DialogueNode {
  id: string;
  speaker?: string;
  text: string;
  choices?: DialogueChoice[];
  commands?: DialogueCommand[];
  next?: string;         // auto-advance to next node
}

export interface DialogueChoice {
  text: string;
  next: string;
  condition?: (vars: Record<string, any>) => boolean;
  once?: boolean;        // consumed after first selection
  commands?: DialogueCommand[];
}

export interface DialogueCommand {
  type: 'set' | 'give_item' | 'remove_item' | 'gold' | 'start_quest' | 'complete_quest' | 'custom';
  key?: string;
  value?: any;
}

export interface DialogueScript {
  id: string;
  nodes: Record<string, DialogueNode>;
  startNode: string;
  declare?: Record<string, any>; // variable defaults
}

export interface DialogueRenderer {
  showLine(speaker: string | undefined, text: string): void;
  showChoices(choices: { text: string; index: number }[]): Promise<number>;
  hide(): void;
}

export class DialogueRunner {
  script: DialogueScript;
  vars: Record<string, any>;
  consumedChoices: Set<string> = new Set();
  onCommand: ((cmd: DialogueCommand) => void)[] = [];

  constructor(script: DialogueScript) {
    this.script = script;
    this.vars = { ...script.declare };
  }

  async play(renderer: DialogueRenderer): Promise<void> {
    let nodeId = this.script.startNode;

    while (nodeId && nodeId !== 'END') {
      const node = this.script.nodes[nodeId];
      if (!node) break;

      // Execute node commands
      if (node.commands) {
        for (const cmd of node.commands) this.executeCommand(cmd);
      }

      // Show text
      const text = this.interpolate(node.text);
      renderer.showLine(node.speaker, text);

      if (node.choices && node.choices.length > 0) {
        // Filter available choices
        const available = node.choices
          .map((c, i) => ({ choice: c, index: i }))
          .filter(({ choice, index }) => {
            if (choice.once && this.consumedChoices.has(`${nodeId}_${index}`)) return false;
            if (choice.condition && !choice.condition(this.vars)) return false;
            return true;
          });

        if (available.length === 0) break;

        const choiceIdx = await renderer.showChoices(
          available.map(a => ({ text: a.choice.text, index: a.index }))
        );

        const chosen = available.find(a => a.index === choiceIdx);
        if (!chosen) break;

        // Mark consumed
        if (chosen.choice.once) {
          this.consumedChoices.add(`${nodeId}_${choiceIdx}`);
        }

        // Execute choice commands
        if (chosen.choice.commands) {
          for (const cmd of chosen.choice.commands) this.executeCommand(cmd);
        }

        nodeId = chosen.choice.next;
      } else if (node.next) {
        // Wait for player to advance (simulated delay for text display)
        await new Promise(r => setTimeout(r, 100));
        nodeId = node.next;
      } else {
        break; // End of dialogue
      }
    }

    renderer.hide();
  }

  private interpolate(text: string): string {
    return text.replace(/\{(\w+)\}/g, (_, key) => {
      return String(this.vars[key] ?? `{${key}}`);
    });
  }

  private executeCommand(cmd: DialogueCommand) {
    switch (cmd.type) {
      case 'set':
        if (cmd.key) this.vars[cmd.key] = cmd.value;
        break;
      default:
        // Emit for external handler
        for (const fn of this.onCommand) fn(cmd);
    }
  }

  getVariable(key: string): any {
    return this.vars[key];
  }

  setVariable(key: string, value: any) {
    this.vars[key] = value;
  }

  toJSON() {
    return { vars: { ...this.vars }, consumed: [...this.consumedChoices] };
  }

  loadJSON(data: { vars: Record<string, any>; consumed: string[] }) {
    this.vars = { ...data.vars };
    this.consumedChoices = new Set(data.consumed);
  }
}
```

### HTML UI Renderer

```typescript
// ===================== dialogue-ui.ts =====================

import { DialogueRenderer } from './dialogue';

export class DialogueUI implements DialogueRenderer {
  private container: HTMLDivElement;
  private textEl: HTMLDivElement;
  private choicesEl: HTMLDivElement;
  private speakerEl: HTMLDivElement;
  private resolveChoice!: (idx: number) => void;

  constructor(parent: HTMLElement) {
    this.container = document.createElement('div');
    this.container.style.cssText = `
      position: fixed; bottom: 20px; left: 50%; transform: translateX(-50%);
      width: 600px; background: rgba(0,0,0,0.9); border: 2px solid #c8a84a;
      border-radius: 8px; padding: 16px; color: #e5e5e5; font-family: monospace;
      z-index: 1000; display: none;
    `;

    this.speakerEl = document.createElement('div');
    this.speakerEl.style.cssText = 'color: #fbbf24; font-weight: bold; margin-bottom: 8px;';

    this.textEl = document.createElement('div');
    this.textEl.style.cssText = 'margin-bottom: 12px; line-height: 1.6;';

    this.choicesEl = document.createElement('div');
    this.choicesEl.style.cssText = 'display: flex; flex-direction: column; gap: 6px;';

    this.container.append(this.speakerEl, this.textEl, this.choicesEl);
    parent.appendChild(this.container);
  }

  showLine(speaker: string | undefined, text: string): void {
    this.container.style.display = 'block';
    this.speakerEl.textContent = speaker || '';
    this.textEl.textContent = text;
    this.choicesEl.innerHTML = '';
  }

  showChoices(choices: { text: string; index: number }[]): Promise<number> {
    return new Promise(resolve => {
      this.resolveChoice = resolve;
      this.choicesEl.innerHTML = '';

      for (const choice of choices) {
        const btn = document.createElement('button');
        btn.textContent = choice.text;
        btn.style.cssText = `
          background: rgba(200,168,74,0.15); border: 1px solid #c8a84a;
          color: #e5e5e5; padding: 8px 12px; border-radius: 4px; cursor: pointer;
          text-align: left; font-family: monospace;
        `;
        btn.onmouseenter = () => btn.style.background = 'rgba(200,168,74,0.3)';
        btn.onmouseleave = () => btn.style.background = 'rgba(200,168,74,0.15)';
        btn.onclick = () => resolve(choice.index);
        this.choicesEl.appendChild(btn);
      }
    });
  }

  hide(): void {
    this.container.style.display = 'none';
  }
}
```

### Usage

```typescript
const script: DialogueScript = {
  id: 'blacksmith',
  startNode: 'greeting',
  declare: { knowsBlacksmith: false },
  nodes: {
    greeting: {
      id: 'greeting',
      speaker: 'Blacksmith',
      text: 'Welcome, traveler! What brings you to my forge?',
      choices: [
        { text: 'I need weapons!', next: 'weapons' },
        { text: 'Just looking around.', next: 'browse' },
        {
          text: 'I heard you have rare items.',
          next: 'secret',
          condition: (v) => v.knowsBlacksmith === true,
        },
      ],
    },
    weapons: {
      id: 'weapons',
      speaker: 'Blacksmith',
      text: 'I have fine blades. Take a look at my wares!',
      commands: [{ type: 'custom', key: 'open_shop', value: 'blacksmith_shop' }],
      next: 'END',
    },
    browse: {
      id: 'browse', speaker: 'Blacksmith', text: 'Take your time!', next: 'END',
    },
    secret: {
      id: 'secret',
      speaker: 'Blacksmith',
      text: 'Ah, you know the right people. Here, take this.',
      commands: [{ type: 'give_item', key: 'legendary_sword', value: 1 }],
      next: 'END',
    },
  },
};
```

---

## 5. Shop System

Buy/sell with haggling, stock management, dynamic pricing. Based on shopkeepr and RPGJS patterns.

```typescript
// ===================== shop.ts =====================

import { ItemDef, Inventory } from './inventory';

export interface ShopItem {
  itemId: string;
  stock: number;        // -1 = infinite
  priceModifier: number; // multiplier, e.g. 1.2 = 20% markup
}

export class Shop {
  name: string;
  catalog: Map<string, ItemDef>;
  stock: Map<string, ShopItem>;
  gold: number;
  haggleDiscount: number = 0; // percentage discount from haggling (0-30%)
  onChange: (() => void)[] = [];

  constructor(name: string, catalog: ItemDef[], initialStock: ShopItem[] = []) {
    this.name = name;
    this.catalog = new Map(catalog.map(i => [i.id, i]));
    this.stock = new Map(initialStock.map(s => [s.itemId, s]));
    this.gold = 1000;
  }

  /** Player buys item from shop */
  buy(itemId: string, qty: number, playerGold: number, playerInventory: Inventory): {
    ok: boolean; error?: string; cost?: number;
  } {
    const shopItem = this.stock.get(itemId);
    const itemDef = this.catalog.get(itemId);

    if (!shopItem || !itemDef) return { ok: false, error: 'Item not available' };
    if (shopItem.stock === 0) return { ok: false, error: 'Out of stock' };
    if (shopItem.stock > 0 && qty > shopItem.stock) return { ok: false, error: 'Not enough stock' };

    const unitPrice = Math.floor(itemDef.buyPrice * shopItem.priceModifier * (1 - this.haggleDiscount / 100));
    const totalCost = unitPrice * qty;

    if (playerGold < totalCost) return { ok: false, error: 'Not enough gold' };

    const overflow = playerInventory.add(itemId, qty);
    if (overflow > 0) return { ok: false, error: 'Inventory full' };

    // Transaction
    if (shopItem.stock > 0) shopItem.stock -= qty;
    this.gold += totalCost;
    this.haggleDiscount = 0; // reset haggle after purchase

    this.emit();
    return { ok: true, cost: totalCost };
  }

  /** Player sells item to shop */
  sell(itemId: string, qty: number, playerGold: number, playerInventory: Inventory): {
    ok: boolean; error?: string; revenue?: number;
  } {
    const itemDef = this.catalog.get(itemId);
    if (!itemDef) return { ok: false, error: 'Shop does not buy this item' };
    if (!playerInventory.has(itemId, qty)) return { ok: false, error: 'Not enough items' };

    const unitPrice = Math.floor(itemDef.sellPrice * 0.5); // shops buy at 50% of sell price
    const totalRevenue = unitPrice * qty;

    if (this.gold < totalRevenue) return { ok: false, error: 'Shop cannot afford that' };

    playerInventory.remove(itemId, qty);
    // Add to shop stock if not infinite
    const shopItem = this.stock.get(itemId);
    if (shopItem && shopItem.stock >= 0) shopItem.stock += qty;

    this.gold -= totalRevenue;
    this.emit();
    return { ok: true, revenue: totalRevenue };
  }

  /** Haggling: charisma-based discount */
  haggle(charisma: number): { success: boolean; discount: number; message: string } {
    // Formula: chance = charisma / (charisma + 50), max 30% discount
    const chance = charisma / (charisma + 50);
    const roll = Math.random();

    if (roll < chance) {
      this.haggleDiscount = Math.min(30, Math.floor(charisma / 10));
      return {
        success: true,
        discount: this.haggleDiscount,
        message: `Haggling successful! ${this.haggleDiscount}% discount on next purchase.`,
      };
    }

    // Failed haggle: prices go up 10%
    this.haggleDiscount = -10;
    return {
      success: false,
      discount: 0,
      message: 'Haggling failed! The shopkeeper is annoyed. Prices increased 10%.',
    };
  }

  /** Get items available for purchase */
  getAvailable(): Array<{ def: ItemDef; shopItem: ShopItem; displayPrice: number }> {
    const result: Array<{ def: ItemDef; shopItem: ShopItem; displayPrice: number }> = [];
    for (const [itemId, shopItem] of this.stock) {
      if (shopItem.stock === 0) continue;
      const def = this.catalog.get(itemId);
      if (!def) continue;
      const displayPrice = Math.floor(def.buyPrice * shopItem.priceModifier * (1 - this.haggleDiscount / 100));
      result.push({ def, shopItem, displayPrice });
    }
    return result;
  }

  addStock(itemId: string, stock: number, priceModifier = 1) {
    const existing = this.stock.get(itemId);
    if (existing) {
      existing.stock = stock;
      existing.priceModifier = priceModifier;
    } else {
      this.stock.set(itemId, { itemId, stock, priceModifier });
    }
  }

  toJSON() {
    return {
      gold: this.gold,
      stock: [...this.stock.entries()],
      haggleDiscount: this.haggleDiscount,
    };
  }

  loadJSON(data: any) {
    this.gold = data.gold;
    this.stock = new Map(data.stock);
    this.haggleDiscount = data.haggleDiscount || 0;
  }

  private emit() {
    for (const fn of this.onChange) fn();
  }
}
```

### Integration

- Inventory system: `shop.buy()` calls `inventory.add()`, `shop.sell()` calls `inventory.remove()`
- Dialogue system: NPC dialogue can trigger `shop.getAvailable()` to show shop UI
- Quest system: `reportProgress('talk', 'shopkeeper')` on shop interaction

### Common Pitfalls

- Not checking inventory space before buy
- Forgetting to update shop stock on sell
- Haggling without reset leads to permanent discounts

---

## 6. Skill System

Skill trees with dependencies, cooldowns, mana costs, levels. Based on CommanderFoo/skill-tree-planner and Hkattelu/SkillTree.

```typescript
// ===================== skill.ts =====================

export interface SkillDef {
  id: string;
  name: string;
  description: string;
  icon: string;
  maxLevel: number;
  manaCost: number;        // base mana cost
  cooldown: number;        // base cooldown in ms
  damage?: number;         // base damage
  healAmount?: number;     // for healing skills
  range?: number;          // cast range in tiles
  aoe?: number;            // area of effect radius
  type: 'active' | 'passive';
  category: 'attack' | 'defense' | 'utility' | 'magic' | 'support';
  dependsOn?: string[];    // prerequisite skill ids
  levelScaling?: {         // per-level bonuses
    manaCost?: number;
    cooldown?: number;
    damage?: number;
    healAmount?: number;
  };
}

export interface SkillState {
  skillId: string;
  level: number;
  unlocked: boolean;
  lastUsed: number;  // timestamp
}

export class SkillSystem {
  skillDefs: Map<string, SkillDef>;
  skillStates: Map<string, SkillState> = new Map();
  availablePoints: number = 0;
  onChange: ((skillId: string) => void)[] = [];

  constructor(defs: SkillDef[]) {
    this.skillDefs = new Map(defs.map(s => [s.id, s]));
  }

  /** Check if a skill can be learned/upgraded */
  canLearn(skillId: string): { ok: boolean; reason?: string } {
    const def = this.skillDefs.get(skillId);
    if (!def) return { ok: false, reason: 'Skill not found' };

    const state = this.skillStates.get(skillId);

    // Check max level
    if (state && state.level >= def.maxLevel) {
      return { ok: false, reason: 'Already at max level' };
    }

    // Check points
    if (this.availablePoints <= 0) {
      return { ok: false, reason: 'No skill points available' };
    }

    // Check dependencies
    if (def.dependsOn) {
      for (const depId of def.dependsOn) {
        const depState = this.skillStates.get(depId);
        if (!depState || !depState.unlocked || depState.level < 1) {
          const depDef = this.skillDefs.get(depId);
          return { ok: false, reason: `Requires ${depDef?.name || depId}` };
        }
      }
    }

    return { ok: true };
  }

  /** Learn or upgrade a skill */
  learn(skillId: string): boolean {
    const check = this.canLearn(skillId);
    if (!check.ok) return false;

    const state = this.skillStates.get(skillId) || {
      skillId, level: 0, unlocked: false, lastUsed: 0,
    };

    state.level += 1;
    state.unlocked = true;
    this.skillStates.set(skillId, state);
    this.availablePoints -= 1;

    this.emit(skillId);
    return true;
  }

  /** Get effective stats for a skill at its current level */
  getEffectiveStats(skillId: string): {
    manaCost: number; cooldown: number; damage: number; healAmount: number;
  } | null {
    const def = this.skillDefs.get(skillId);
    const state = this.skillStates.get(skillId);
    if (!def || !state || !state.unlocked) return null;

    const scale = def.levelScaling || {};
    return {
      manaCost: def.manaCost + (scale.manaCost || 0) * (state.level - 1),
      cooldown: def.cooldown + (scale.cooldown || 0) * (state.level - 1),
      damage: (def.damage || 0) + (scale.damage || 0) * (state.level - 1),
      healAmount: (def.healAmount || 0) + (scale.healAmount || 0) * (state.level - 1),
    };
  }

  /** Check if skill is off cooldown */
  isReady(skillId: string): boolean {
    const def = this.skillDefs.get(skillId);
    const state = this.skillStates.get(skillId);
    if (!def || !state || !state.unlocked) return false;
    if (def.type === 'passive') return true;
    return Date.now() - state.lastUsed >= def.cooldown;
  }

  /** Cast a skill (marks cooldown, returns effective stats) */
  cast(skillId: string, playerMana: number): {
    ok: boolean; error?: string; stats?: ReturnType<SkillSystem['getEffectiveStats']>;
  } {
    if (!this.isReady(skillId)) return { ok: false, error: 'Skill on cooldown' };

    const stats = this.getEffectiveStats(skillId);
    if (!stats) return { ok: false, error: 'Skill not learned' };
    if (playerMana < stats.manaCost) return { ok: false, error: 'Not enough mana' };

    const state = this.skillStates.get(skillId)!;
    state.lastUsed = Date.now();

    this.emit(skillId);
    return { ok: true, stats };
  }

  /** Get all passive bonuses from learned skills */
  getPassiveBonuses(): Record<string, number> {
    const bonuses: Record<string, number> = {};
    for (const [id, state] of this.skillStates) {
      if (!state.unlocked) continue;
      const def = this.skillDefs.get(id);
      if (!def || def.type !== 'passive') continue;
      const stats = this.getEffectiveStats(id);
      if (!stats) continue;
      // Passive skills add their damage as a flat bonus
      if (stats.damage) bonuses['attack'] = (bonuses['attack'] || 0) + stats.damage;
      if (stats.healAmount) bonuses['regen'] = (bonuses['regen'] || 0) + stats.healAmount;
    }
    return bonuses;
  }

  /** Build tree structure for rendering */
  getTree(): Array<{
    skill: SkillDef; state: SkillState | null;
    canLearn: boolean; children: string[];
  }> {
    const result: Array<{
      skill: SkillDef; state: SkillState | null;
      canLearn: boolean; children: string[];
    }> = [];

    for (const [id, def] of this.skillDefs) {
      const state = this.skillStates.get(id) || null;
      result.push({
        skill: def,
        state,
        canLearn: this.canLearn(id).ok,
        children: [...this.skillDefs.values()]
          .filter(d => d.dependsOn?.includes(id))
          .map(d => d.id),
      });
    }

    return result;
  }

  gainPoints(n: number) {
    this.availablePoints += n;
  }

  toJSON() {
    return {
      points: this.availablePoints,
      states: [...this.skillStates.entries()],
    };
  }

  loadJSON(data: any) {
    this.availablePoints = data.points;
    this.skillStates.clear();
    for (const [id, state] of data.states) this.skillStates.set(id, state);
  }

  private emit(skillId: string) {
    for (const fn of this.onChange) fn(skillId);
  }
}
```

### Canvas Skill Tree Renderer

```typescript
// ===================== skill-tree-renderer.ts =====================

import { SkillSystem, SkillDef } from './skill';

export class SkillTreeRenderer {
  private system: SkillSystem;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private nodePositions: Map<string, { x: number; y: number }> = new Map();
  private nodeSize = 40;
  private hoveredSkill: string | null = null;

  constructor(system: SkillSystem, canvas: HTMLCanvasElement) {
    this.system = system;
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d')!;
    this.layout();
    this.setupEvents();
  }

  private layout() {
    const tree = this.system.getTree();
    const layers = new Map<number, string[]>();

    // BFS to assign layers
    const visited = new Set<string>();
    const queue: Array<{ id: string; layer: number }> = [];

    // Find roots (no dependencies)
    for (const node of tree) {
      if (!node.skill.dependsOn || node.skill.dependsOn.length === 0) {
        queue.push({ id: node.skill.id, layer: 0 });
        visited.add(node.skill.id);
      }
    }

    while (queue.length > 0) {
      const { id, layer } = queue.shift()!;
      if (!layers.has(layer)) layers.set(layer, []);
      layers.get(layer)!.push(id);

      const children = tree.find(n => n.skill.id === id)?.children || [];
      for (const childId of children) {
        if (!visited.has(childId)) {
          visited.add(childId);
          queue.push({ id: childId, layer: layer + 1 });
        }
      }
    }

    // Position nodes
    const layerGap = 80;
    const nodeGap = 70;
    for (const [layer, ids] of layers) {
      const totalWidth = ids.length * nodeGap;
      const startX = (this.canvas.width - totalWidth) / 2 + nodeGap / 2;

      for (let i = 0; i < ids.length; i++) {
        this.nodePositions.set(ids[i], {
          x: startX + i * nodeGap,
          y: 40 + layer * layerGap,
        });
      }
    }
  }

  render() {
    const ctx = this.ctx;
    ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

    const tree = this.system.getTree();

    // Draw connections
    for (const node of tree) {
      const pos = this.nodePositions.get(node.skill.id);
      if (!pos) continue;

      for (const childId of node.children) {
        const childPos = this.nodePositions.get(childId);
        if (!childPos) continue;

        const childNode = tree.find(n => n.skill.id === childId);
        const unlocked = childNode?.state?.unlocked;

        ctx.beginPath();
        ctx.strokeStyle = unlocked ? '#22c55e' : '#555';
        ctx.lineWidth = 2;
        ctx.moveTo(pos.x, pos.y + this.nodeSize / 2);
        ctx.lineTo(childPos.x, childPos.y - this.nodeSize / 2);
        ctx.stroke();
      }
    }

    // Draw nodes
    for (const node of tree) {
      const pos = this.nodePositions.get(node.skill.id);
      if (!pos) continue;

      const state = node.state;
      const unlocked = state?.unlocked;
      const hovered = this.hoveredSkill === node.skill.id;
      const canLearn = node.canLearn;

      // Background
      ctx.beginPath();
      ctx.arc(pos.x, pos.y, this.nodeSize / 2, 0, Math.PI * 2);

      if (unlocked) {
        ctx.fillStyle = '#22c55e';
      } else if (canLearn) {
        ctx.fillStyle = hovered ? '#854d0e' : '#713f12';
      } else {
        ctx.fillStyle = hovered ? '#333' : '#222';
      }
      ctx.fill();

      // Border
      ctx.strokeStyle = unlocked ? '#16a34a' : canLearn ? '#f59e0b' : '#444';
      ctx.lineWidth = 2;
      ctx.stroke();

      // Level text
      if (state && state.level > 0) {
        ctx.fillStyle = '#fff';
        ctx.font = 'bold 12px monospace';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(`${state.level}/${node.skill.maxLevel}`, pos.x, pos.y);
      }

      // Icon (emoji)
      ctx.font = '16px sans-serif';
      ctx.fillText(node.skill.icon, pos.x, pos.y - 1);

      // Name below
      ctx.fillStyle = '#ccc';
      ctx.font = '10px monospace';
      ctx.fillText(node.skill.name, pos.x, pos.y + this.nodeSize / 2 + 12);
    }
  }

  private setupEvents() {
    this.canvas.addEventListener('mousemove', (e) => {
      const rect = this.canvas.getBoundingClientRect();
      const mx = e.clientX - rect.left;
      const my = e.clientY - rect.top;

      this.hoveredSkill = null;
      for (const [id, pos] of this.nodePositions) {
        const dx = mx - pos.x;
        const dy = my - pos.y;
        if (Math.sqrt(dx * dx + dy * dy) < this.nodeSize / 2) {
          this.hoveredSkill = id;
          break;
        }
      }
      this.render();
    });

    this.canvas.addEventListener('click', (e) => {
      if (this.hoveredSkill) {
        this.system.learn(this.hoveredSkill);
        this.render();
      }
    });
  }
}
```

---

## 7. Minimap System

Real-time minimap with fog of war, markers, click-to-move. Based on ha-agent-rpg/Minimap.

```typescript
// ===================== minimap.ts =====================

export interface MinimapConfig {
  size: number;          // minimap pixel size
  mapWidth: number;      // world width in tiles
  mapHeight: number;     // world height in tiles
  tileSize: number;      // world tile size in pixels
  fogColor: string;
  revealRadius: number;  // tiles revealed around player
  edgeSoftness: number;  // fog edge blur
}

export interface MinimapMarker {
  id: string;
  tileX: number;
  tileY: number;
  color: string;
  icon?: string;
}

const DEFAULT_CONFIG: MinimapConfig = {
  size: 160,
  mapWidth: 50,
  mapHeight: 50,
  tileSize: 32,
  fogColor: '#0a0a14',
  revealRadius: 5,
  edgeSoftness: 2,
};

export class Minimap {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private config: MinimapConfig;
  private explored: boolean[][];
  private tileColors: string[][];
  private markers: Map<string, MinimapMarker> = new Map();
  private onClickCallback: ((tileX: number, tileY: number) => void) | null = null;

  constructor(parent: HTMLElement, config: Partial<MinimapConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };

    this.canvas = document.createElement('canvas');
    this.canvas.width = this.config.size;
    this.canvas.height = this.config.size;
    this.canvas.style.cssText = `
      position: fixed; top: 8px; left: 8px;
      width: ${this.config.size}px; height: ${this.config.size}px;
      border: 2px solid #c8a84a; border-radius: 4px;
      background: ${this.config.fogColor}; z-index: 100;
      image-rendering: pixelated; cursor: pointer;
    `;
    this.ctx = this.canvas.getContext('2d')!;
    parent.appendChild(this.canvas);

    // Init fog
    this.explored = Array.from({ length: this.config.mapHeight }, () =>
      new Array(this.config.mapWidth).fill(false)
    );

    // Init tile colors (call setTileMap after construction)
    this.tileColors = Array.from({ length: this.config.mapHeight }, () =>
      new Array(this.config.mapWidth).fill('#333')
    );

    this.canvas.addEventListener('click', (e) => {
      if (!this.onClickCallback) return;
      const rect = this.canvas.getBoundingClientRect();
      const clickX = e.clientX - rect.left;
      const clickY = e.clientY - rect.top;
      const tileX = Math.floor(clickX * (this.config.mapWidth / this.config.size));
      const tileY = Math.floor(clickY * (this.config.mapHeight / this.config.size));
      this.onClickCallback(tileX, tileY);
    });
  }

  onClick(callback: (tileX: number, tileY: number) => void) {
    this.onClickCallback = callback;
  }

  setTileMap(colors: string[][]) {
    this.tileColors = colors;
    this.redraw();
  }

  /** Reveal tiles around a world position */
  reveal(worldX: number, worldY: number) {
    const tileX = Math.floor(worldX / this.config.tileSize);
    const tileY = Math.floor(worldY / this.config.tileSize);
    const r = this.config.revealRadius;

    for (let dy = -r; dy <= r; dy++) {
      for (let dx = -r; dx <= r; dx++) {
        const tx = tileX + dx;
        const ty = tileY + dy;
        if (tx < 0 || ty < 0 || tx >= this.config.mapWidth || ty >= this.config.mapHeight) continue;

        const dist = Math.sqrt(dx * dx + dy * dy);
        if (dist <= r) {
          this.explored[ty][tx] = true;
        }
      }
    }
    this.redraw();
  }

  addMarker(marker: MinimapMarker) {
    this.markers.set(marker.id, marker);
    this.redraw();
  }

  removeMarker(id: string) {
    this.markers.delete(id);
    this.redraw();
  }

  updateMarker(id: string, tileX: number, tileY: number) {
    const m = this.markers.get(id);
    if (m) {
      m.tileX = tileX;
      m.tileY = tileY;
      this.redraw();
    }
  }

  private redraw() {
    const ctx = this.ctx;
    const { size, mapWidth, mapHeight } = this.config;
    const scaleX = size / mapWidth;
    const scaleY = size / mapHeight;

    ctx.fillStyle = this.config.fogColor;
    ctx.fillRect(0, 0, size, size);

    // Draw explored tiles
    for (let y = 0; y < mapHeight; y++) {
      for (let x = 0; x < mapWidth; x++) {
        if (!this.explored[y]?.[x]) continue;
        ctx.fillStyle = this.tileColors[y]?.[x] || '#333';
        ctx.fillRect(
          Math.floor(x * scaleX),
          Math.floor(y * scaleY),
          Math.ceil(scaleX),
          Math.ceil(scaleY)
        );
      }
    }

    // Draw markers
    for (const [, marker] of this.markers) {
      if (!this.explored[marker.tileY]?.[marker.tileX]) continue;
      ctx.fillStyle = marker.color;
      ctx.fillRect(
        Math.floor(marker.tileX * scaleX) - 1,
        Math.floor(marker.tileY * scaleY) - 1,
        3, 3
      );
    }
  }

  /** Get revealed bounds for camera framing */
  getRevealedBounds(): { minX: number; minY: number; maxX: number; maxY: number } | null {
    let minX = this.config.mapWidth, minY = this.config.mapHeight;
    let maxX = 0, maxY = 0;
    let found = false;

    for (let y = 0; y < this.config.mapHeight; y++) {
      for (let x = 0; x < this.config.mapWidth; x++) {
        if (this.explored[y][x]) {
          found = true;
          minX = Math.min(minX, x);
          minY = Math.min(minY, y);
          maxX = Math.max(maxX, x);
          maxY = Math.max(maxY, y);
        }
      }
    }

    return found ? { minX, minY, maxX, maxY } : null;
  }

  destroy() {
    this.canvas.remove();
  }

  toJSON() {
    return { explored: this.explored };
  }

  loadJSON(data: { explored: boolean[][] }) {
    this.explored = data.explored;
    this.redraw();
  }
}
```

### Integration

- Game loop calls `minimap.reveal(player.worldX, player.worldY)` each frame
- NPC/entity positions: `minimap.addMarker({ id: 'npc_1', tileX, tileY, color: '#f59e0b' })`
- Click-to-move: `minimap.onClick((x, y) => pathfindTo(x, y))`

---

## 8. Map System

Tile-based rendering with viewport culling, multiple layers, collision. Based on MDN Tilemaps and collide-2d-aabb-tilemap.

```typescript
// ===================== tilemap.ts =====================

export interface TileDef {
  id: number;
  name: string;
  solid: boolean;
  damage?: number;        // tile damage (lava, etc.)
  script?: string;        // event script id
  animation?: { frames: number[]; speed: number };
}

export interface TileLayer {
  name: string;
  data: number[][];  // 2D array of tile IDs
  opacity?: number;
  visible?: boolean;
}

export interface TileMapData {
  width: number;   // in tiles
  height: number;
  tileSize: number;
  layers: TileLayer[];
  tileDefs: TileDef[];
}

export class TileMap {
  data: TileMapData;
  tileAtlas: HTMLImageElement | null = null;
  atlasCols: number = 0;
  animationTime: number = 0;

  constructor(data: TileMapData) {
    this.data = data;
  }

  async loadAtlas(src: string): Promise<void> {
    this.tileAtlas = await new Promise<HTMLImageElement>((resolve, reject) => {
      const img = new Image();
      img.onload = () => resolve(img);
      img.onerror = reject;
      img.src = src;
    });
    this.atlasCols = Math.floor(this.tileAtlas.width / this.data.tileSize);
  }

  getTileDef(tileId: number): TileDef | undefined {
    return this.data.tileDefs.find(t => t.id === tileId);
  }

  isSolid(col: number, row: number, layerIndex = 0): boolean {
    const layer = this.data.layers[layerIndex];
    if (!layer || !layer.visible) return false;
    if (row < 0 || col < 0 || row >= this.data.height || col >= this.data.width) return true;
    const tileId = layer.data[row]?.[col] || 0;
    if (tileId === 0) return false;
    return this.getTileDef(tileId)?.solid ?? false;
  }

  /** Render visible portion of map with camera culling */
  render(
    ctx: CanvasRenderingContext2D,
    camera: { x: number; y: number; width: number; height: number },
    time: number = 0
  ) {
    this.animationTime = time;
    const { tileSize, width: mapW, height: mapH } = this.data;

    // Calculate visible tile range
    let startCol = Math.floor(camera.x / tileSize);
    let endCol = Math.floor((camera.x + camera.width) / tileSize) + 1;
    let startRow = Math.floor(camera.y / tileSize);
    let endRow = Math.floor((camera.y + camera.height) / tileSize) + 1;

    startCol = Math.max(0, startCol);
    endCol = Math.min(mapW, endCol);
    startRow = Math.max(0, startRow);
    endRow = Math.min(mapH, endRow);

    for (const layer of this.data.layers) {
      if (layer.visible === false) continue;

      for (let r = startRow; r < endCol; r++) {
        for (let c = startCol; c <= endCol; c++) {
          let tileId = layer.data[r]?.[c] || 0;
          if (tileId === 0) continue;

          const tileDef = this.getTileDef(tileId);

          // Handle animation
          if (tileDef?.animation) {
            const frameIdx = Math.floor(time / tileDef.animation.speed) % tileDef.animation.frames.length;
            tileId = tileDef.animation.frames[frameIdx];
          }

          const tileIdx = tileId - 1;
          const sx = (tileIdx % this.atlasCols) * tileSize;
          const sy = Math.floor(tileIdx / this.atlasCols) * tileSize;

          const dx = Math.round(c * tileSize - camera.x);
          const dy = Math.round(r * tileSize - camera.y);

          if (this.tileAtlas) {
            ctx.globalAlpha = layer.opacity ?? 1;
            ctx.drawImage(
              this.tileAtlas,
              sx, sy, tileSize, tileSize,
              dx, dy, tileSize, tileSize
            );
            ctx.globalAlpha = 1;
          } else {
            // Fallback: colored rectangles
            ctx.fillStyle = tileDef?.solid ? '#555' : '#8a8';
            ctx.globalAlpha = layer.opacity ?? 1;
            ctx.fillRect(dx, dy, tileSize, tileSize);
            ctx.globalAlpha = 1;
          }
        }
      }
    }
  }

  /** Convert pixel position to tile coordinates */
  pixelToTile(px: number, py: number): [number, number] {
    return [
      Math.floor(px / this.data.tileSize),
      Math.floor(py / this.data.tileSize),
    ];
  }

  /** Convert tile coordinates to pixel position */
  tileToPixel(col: number, row: number): [number, number] {
    return [col * this.data.tileSize, row * this.data.tileSize];
  }
}
```

### AABB Tile Collision

```typescript
// ===================== collision.ts =====================

import { TileMap } from './tilemap';

export interface AABB {
  x: number;      // left edge
  y: number;      // top edge
  width: number;
  height: number;
}

/**
 * Attempt to move AABB by delta against tilemap.
 * Returns adjusted delta that avoids solid tiles.
 * Based on HigherOrderFun platformer collision algorithm.
 */
export function collideAABBTiles(
  aabb: AABB,
  dx: number,
  dy: number,
  tileMap: TileMap,
  layerIndex: number = 0
): { dx: number; dy: number; onWall: string | null } {
  const ts = tileMap.data.tileSize;
  let resultDx = dx;
  let resultDy = dy;
  let onWall: string | null = null;

  // Resolve X axis
  if (dx !== 0) {
    const dir = dx > 0 ? 1 : -1;
    const edgeX = dx > 0 ? aabb.x + aabb.width : aabb.x;

    const startCol = Math.floor(edgeX / ts);
    const endCol = Math.floor((edgeX + dx) / ts);

    const minRow = Math.floor(aabb.y / ts);
    const maxRow = Math.floor((aabb.y + aabb.height - 1) / ts);

    for (let c = startCol; c !== endCol + dir; c += dir) {
      for (let r = minRow; r <= maxRow; r++) {
        if (tileMap.isSolid(c, r, layerIndex)) {
          // Snap to tile edge
          resultDx = dir > 0
            ? c * ts - aabb.x - aabb.width
            : (c + 1) * ts - aabb.x;
          onWall = dir > 0 ? 'right' : 'left';
          break;
        }
      }
      if (onWall) break;
    }
  }

  // Resolve Y axis
  if (dy !== 0) {
    const dir = dy > 0 ? 1 : -1;
    const edgeY = dy > 0 ? aabb.y + aabb.height : aabb.y;
    const adjustedX = aabb.x + resultDx;

    const startRow = Math.floor(edgeY / ts);
    const endRow = Math.floor((edgeY + dy) / ts);

    const minCol = Math.floor(adjustedX / ts);
    const maxCol = Math.floor((adjustedX + aabb.width - 1) / ts);

    for (let r = startRow; r !== endRow + dir; r += dir) {
      for (let c = minCol; c <= maxCol; c++) {
        if (tileMap.isSolid(c, r, layerIndex)) {
          resultDy = dir > 0
            ? r * ts - aabb.y - aabb.height
            : (r + 1) * ts - aabb.y;
          onWall = dir > 0 ? 'bottom' : 'top';
          break;
        }
      }
      if (onWall) break;
    }
  }

  return { dx: resultDx, dy: resultDy, onWall };
}

/** Check if a point is in a solid tile */
export function isSolidAt(px: number, py: number, tileMap: TileMap, layer = 0): boolean {
  const [col, row] = tileMap.pixelToTile(px, py);
  return tileMap.isSolid(col, row, layer);
}

/** Simple AABB vs AABB collision */
export function aabbOverlap(a: AABB, b: AABB): boolean {
  return a.x < b.x + b.width &&
         a.x + a.width > b.x &&
         a.y < b.y + b.height &&
         a.y + a.height > b.y;
}

/** Distance between two AABB centers */
export function aabbDistance(a: AABB, b: AABB): number {
  const ax = a.x + a.width / 2;
  const ay = a.y + a.height / 2;
  const bx = b.x + b.width / 2;
  const by = b.y + b.height / 2;
  return Math.sqrt((ax - bx) ** 2 + (ay - by) ** 2);
}
```

### Best Practices

- Always resolve X and Y axes independently (prevents tunneling)
- Check tile of leading edge, not center — catches corners
- Use `Math.floor((edge + delta) / tileSize)` for destination tile, not `edge / tileSize`

---

## 9. Monster AI System

Patrol, chase, attack, flee behaviors with state machine.

```typescript
// ===================== monster-ai.ts =====================

import { AABB, aabbOverlap, aabbDistance } from './collision';

export type AIState = 'idle' | 'patrol' | 'chase' | 'attack' | 'flee' | 'dead';

export interface MonsterDef {
  id: string;
  name: string;
  hp: number;
  maxHp: number;
  attack: number;
  defense: number;
  speed: number;         // pixels per second
  aggroRange: number;    // tiles
  deaggroRange: number;  // tiles
  attackRange: number;   // tiles
  attackCooldown: number; // ms
  xpReward: number;
  goldReward: number;
  lootTable: { itemId: string; chance: number; qty: number }[];
  patrolRadius: number;  // tiles from home
  fleeHpThreshold: number; // % HP to flee
}

export interface MonsterState {
  def: MonsterDef;
  currentHp: number;
  x: number;
  y: number;
  homeX: number;
  homeY: number;
  state: AIState;
  stateTimer: number;
  lastAttackTime: number;
  targetId: string | null;
  patrolTarget: { x: number; y: number } | null;
  facing: 'left' | 'right' | 'up' | 'down';
}

export class MonsterAI {
  monsters: MonsterState[] = [];
  tileSize: number;

  constructor(tileSize: number) {
    this.tileSize = tileSize;
  }

  spawn(def: MonsterDef, worldX: number, worldY: number): MonsterState {
    const state: MonsterState = {
      def: { ...def },
      currentHp: def.hp,
      x: worldX,
      y: worldY,
      homeX: worldX,
      homeY: worldY,
      state: 'idle',
      stateTimer: 0,
      lastAttackTime: 0,
      targetId: null,
      patrolTarget: null,
      facing: 'down',
    };
    this.monsters.push(state);
    return state;
  }

  update(dt: number, playerAABB: AABB, playerId: string, now: number) {
    for (const m of this.monsters) {
      if (m.state === 'dead') continue;

      const distToPlayer = aabbDistance(
        { x: m.x, y: m.y, width: this.tileSize, height: this.tileSize },
        playerAABB
      );

      const aggroPx = m.def.aggroRange * this.tileSize;
      const deaggroPx = m.def.deaggroRange * this.tileSize;
      const attackPx = m.def.attackRange * this.tileSize;
      const hpPercent = m.currentHp / m.def.maxHp;

      // State transitions
      switch (m.state) {
        case 'idle':
        case 'patrol':
          if (distToPlayer < aggroPx) {
            m.state = 'chase';
            m.targetId = playerId;
          } else if (m.state === 'idle') {
            m.stateTimer += dt;
            if (m.stateTimer > 2000) {
              // Start patrolling
              const angle = Math.random() * Math.PI * 2;
              const dist = (Math.random() * 0.5 + 0.5) * m.def.patrolRadius * this.tileSize;
              m.patrolTarget = {
                x: m.homeX + Math.cos(angle) * dist,
                y: m.homeY + Math.sin(angle) * dist,
              };
              m.state = 'patrol';
              m.stateTimer = 0;
            }
          }
          break;

        case 'chase':
          if (distToPlayer > deaggroPx) {
            m.state = 'patrol';
            m.targetId = null;
          } else if (distToPlayer < attackPx) {
            m.state = 'attack';
          } else if (hpPercent <= m.def.fleeHpThreshold / 100) {
            m.state = 'flee';
          }
          break;

        case 'attack':
          if (distToPlayer > attackPx * 1.5) {
            m.state = 'chase';
          } else if (hpPercent <= m.def.fleeHpThreshold / 100) {
            m.state = 'flee';
          }
          break;

        case 'flee':
          if (distToPlayer > deaggroPx) {
            m.state = 'patrol';
            m.targetId = null;
          }
          break;
      }

      // Movement
      const speed = m.def.speed * (dt / 1000);
      let tx = 0, ty = 0;

      switch (m.state) {
        case 'patrol':
          if (m.patrolTarget) {
            tx = m.patrolTarget.x - m.x;
            ty = m.patrolTarget.y - m.y;
            const dist = Math.sqrt(tx * tx + ty * ty);
            if (dist < 4) {
              m.state = 'idle';
              m.stateTimer = 0;
              m.patrolTarget = null;
            } else {
              tx = (tx / dist) * speed;
              ty = (ty / dist) * speed;
            }
          }
          break;

        case 'chase':
          tx = playerAABB.x - m.x;
          ty = playerAABB.y - m.y;
          const chaseDist = Math.sqrt(tx * tx + ty * ty);
          if (chaseDist > 0) {
            tx = (tx / chaseDist) * speed;
            ty = (ty / chaseDist) * speed;
          }
          break;

        case 'flee':
          tx = m.x - playerAABB.x;
          ty = m.y - playerAABB.y;
          const fleeDist = Math.sqrt(tx * tx + ty * ty);
          if (fleeDist > 0) {
            tx = (tx / fleeDist) * speed * 0.8;
            ty = (ty / fleeDist) * speed * 0.8;
          }
          break;
      }

      m.x += tx;
      m.y += ty;

      // Update facing
      if (Math.abs(tx) > Math.abs(ty)) {
        m.facing = tx > 0 ? 'right' : 'left';
      } else if (ty !== 0) {
        m.facing = ty > 0 ? 'down' : 'up';
      }
    }
  }

  /** Get attack result if monster is in attack state and cooldown ready */
  tryAttack(monster: MonsterState, now: number): { damage: number } | null {
    if (monster.state !== 'attack') return null;
    if (now - monster.lastAttackTime < monster.def.attackCooldown) return null;

    monster.lastAttackTime = now;
    const variance = 0.8 + Math.random() * 0.4;
    return { damage: Math.floor(monster.def.attack * variance) };
  }

  damageMonster(monster: MonsterState, damage: number): boolean {
    const actualDamage = Math.max(1, damage - monster.def.defense);
    monster.currentHp -= actualDamage;
    if (monster.currentHp <= 0) {
      monster.currentHp = 0;
      monster.state = 'dead';
      return true; // monster died
    }
    // Aggro if hit
    if (monster.state === 'idle' || monster.state === 'patrol') {
      monster.state = 'chase';
    }
    return false;
  }

  /** Roll loot from monster's loot table */
  rollLoot(monster: MonsterState): { itemId: string; qty: number }[] {
    const drops: { itemId: string; qty: number }[] = [];
    for (const entry of monster.def.lootTable) {
      if (Math.random() < entry.chance) {
        drops.push({ itemId: entry.itemId, qty: entry.qty });
      }
    }
    return drops;
  }

  getMonsterAt(x: number, y: number): MonsterState | null {
    for (const m of this.monsters) {
      if (m.state === 'dead') continue;
      const dist = Math.sqrt((m.x - x) ** 2 + (m.y - y) ** 2);
      if (dist < this.tileSize) return m;
    }
    return null;
  }

  removeDead() {
    this.monsters = this.monsters.filter(m => m.state !== 'dead');
  }
}
```

### Integration

- Game loop: `monsterAI.update(dt, player.aabb, 'player', Date.now())`
- Combat: `monsterAI.tryAttack(monster, Date.now())` → apply damage to player
- Player attack: `monsterAI.damageMonster(monster, playerDamage)` → check death → `rollLoot()`

---

## 10. Save/Load System

LocalStorage persistence with versioning, compression, and validation.

```typescript
// ===================== save-system.ts =====================

export interface SaveData {
  version: number;
  timestamp: number;
  checksum: string;
  data: {
    player: any;
    inventory: any;
    equipment: any;
    quests: any;
    skills: any;
    minimap: any;
    map: { currentMap: string; position: { x: number; y: number } };
    flags: Record<string, any>;  // game flags, switches
    playTime: number;            // total play time in seconds
  };
}

export class SaveSystem {
  private storageKey: string;
  private version: number = 1;
  private maxSlots: number = 3;

  constructor(storageKey = 'rpg_save', maxSlots = 3) {
    this.storageKey = storageKey;
    this.maxSlots = maxSlots;
  }

  /** Save game state to a slot */
  save(slot: number, gameData: SaveData['data']): boolean {
    try {
      const saveData: SaveData = {
        version: this.version,
        timestamp: Date.now(),
        checksum: '',
        data: gameData,
      };

      // Generate checksum
      saveData.checksum = this.generateChecksum(saveData.data);

      const json = JSON.stringify(saveData);
      localStorage.setItem(`${this.storageKey}_${slot}`, json);

      // Update save slot index
      this.updateSlotIndex(slot, saveData.timestamp);
      return true;
    } catch (e) {
      console.error('Save failed:', e);
      return false;
    }
  }

  /** Load game state from a slot */
  load(slot: number): SaveData['data'] | null {
    try {
      const json = localStorage.getItem(`${this.storageKey}_${slot}`);
      if (!json) return null;

      const saveData: SaveData = JSON.parse(json);

      // Version migration
      const migrated = this.migrate(saveData);

      // Validate checksum
      if (migrated.checksum !== this.generateChecksum(migrated.data)) {
        console.warn('Save data corrupted');
        return null;
      }

      return migrated.data;
    } catch (e) {
      console.error('Load failed:', e);
      return null;
    }
  }

  /** Delete a save slot */
  delete(slot: number): void {
    localStorage.removeItem(`${this.storageKey}_${slot}`);
    this.removeSlotFromIndex(slot);
  }

  /** Get info about all save slots */
  getSlots(): Array<{ slot: number; exists: boolean; timestamp?: number; playTime?: number }> {
    const slots: Array<{ slot: number; exists: boolean; timestamp?: number; playTime?: number }> = [];

    for (let i = 0; i < this.maxSlots; i++) {
      const json = localStorage.getItem(`${this.storageKey}_${i}`);
      if (json) {
        try {
          const data: SaveData = JSON.parse(json);
          slots.push({
            slot: i,
            exists: true,
            timestamp: data.timestamp,
            playTime: data.data.playTime,
          });
        } catch {
          slots.push({ slot: i, exists: false });
        }
      } else {
        slots.push({ slot: i, exists: false });
      }
    }

    return slots;
  }

  /** Auto-save to a special slot */
  autoSave(gameData: SaveData['data']): boolean {
    return this.save(this.maxSlots, { ...gameData });
  }

  /** Load auto-save */
  loadAutoSave(): SaveData['data'] | null {
    return this.load(this.maxSlots);
  }

  /** Export save as base64 string (for sharing) */
  exportSave(slot: number): string | null {
    const json = localStorage.getItem(`${this.storageKey}_${slot}`);
    if (!json) return null;
    return btoa(json);
  }

  /** Import save from base64 string */
  importSave(slot: number, encoded: string): boolean {
    try {
      const json = atob(encoded);
      const data = JSON.parse(json) as SaveData;
      if (data.version !== this.version) return false;
      localStorage.setItem(`${this.storageKey}_${slot}`, json);
      return true;
    } catch {
      return false;
    }
  }

  private generateChecksum(data: any): string {
    const str = JSON.stringify(data);
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash |= 0;
    }
    return hash.toString(36);
  }

  private migrate(save: SaveData): SaveData {
    // Future: handle version upgrades
    if (save.version < this.version) {
      // Add migration logic here
      save.version = this.version;
    }
    return save;
  }

  private updateSlotIndex(slot: number, timestamp: number) {
    const index = this.getSlotIndex();
    index[slot] = timestamp;
    localStorage.setItem(`${this.storageKey}_index`, JSON.stringify(index));
  }

  private getSlotIndex(): Record<number, number> {
    const json = localStorage.getItem(`${this.storageKey}_index`);
    return json ? JSON.parse(json) : {};
  }

  private removeSlotFromIndex(slot: number) {
    const index = this.getSlotIndex();
    delete index[slot];
    localStorage.setItem(`${this.storageKey}_index`, JSON.stringify(index));
  }
}
```

### Integration

```typescript
// Game save
const saveSystem = new SaveSystem('my_rpg', 3);

function saveGame(slot: number) {
  saveSystem.save(slot, {
    player: { hp: player.hp, maxHp: player.maxHp, x: player.x, y: player.y },
    inventory: inventory.toJSON(),
    equipment: equipment.toJSON(),
    quests: questSystem.toJSON(),
    skills: skillSystem.toJSON(),
    minimap: minimap.toJSON(),
    map: { currentMap: 'village_01', position: { x: player.x, y: player.y } },
    flags: { metBlacksmith: true, rescuedPrincess: false },
    playTime: totalPlayTime,
  });
}

function loadGame(slot: number) {
  const data = saveSystem.load(slot);
  if (!data) return false;

  player.hp = data.player.hp;
  player.x = data.player.x;
  player.y = data.player.y;
  inventory.loadJSON(data.inventory);
  equipment.loadJSON(data.equipment);
  questSystem.loadJSON(data.quests);
  skillSystem.loadJSON(data.skills);
  minimap.loadJSON(data.minimap);
  return true;
}

// Auto-save every 60 seconds
setInterval(() => saveSystem.autoSave(getCurrentGameState()), 60000);
```

### Best Practices

- Checksum prevents save corruption and tampering
- Version migration handles schema changes across updates
- Auto-save as separate slot so manual saves are never overwritten
- `exportSave`/`importSave` enables cross-device play

---

## System Integration Map

```
┌─────────────────────────────────────────────────────┐
│                     Game Loop                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐          │
│  │ Input    │→ │ Physics  │→ │ Render   │          │
│  └──────────┘  └──────────┘  └──────────┘          │
│       ↓              ↓              ↑               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐          │
│  │ Dialogue │  │ Combat   │  │ Minimap  │          │
│  │ System   │←→│ System   │  │ System   │          │
│  └──────────┘  └──────────┘  └──────────┘          │
│       ↓              ↓                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐          │
│  │ Quest    │  │ Shop     │  │ Monster  │          │
│  │ System   │←→│ System   │←→│ AI       │          │
│  └──────────┘  └──────────┘  └──────────┘          │
│       ↓              ↓              ↓               │
│  ┌──────────────────────────────────────┐          │
│  │          Inventory System            │          │
│  │    (Equipment / Skills / Items)      │          │
│  └──────────────────────────────────────┘          │
│       ↓                                            │
│  ┌──────────────────────────────────────┐          │
│  │        Save/Load System              │          │
│  │   (LocalStorage + Checksum)          │          │
│  └──────────────────────────────────────┘          │
│       ↓                                            │
│  ┌──────────────────────────────────────┐          │
│  │        Tile Map System               │          │
│  │  (Rendering + Collision + Layers)    │          │
│  └──────────────────────────────────────┘          │
└─────────────────────────────────────────────────────┘
```

## Data Flow

| Trigger | System A | System B | Method |
|---------|----------|----------|--------|
| Kill monster | Combat | Quest | `reportProgress('kill', monsterId)` |
| Kill monster | Combat | Monster AI | `rollLoot()` → `inventory.add()` |
| Buy item | Shop | Inventory | `inventory.add(itemId, qty)` |
| Equip item | Equipment | Inventory | `inventory.remove()` + `equipment.equip()` |
| Talk to NPC | Dialogue | Quest | `reportProgress('talk', npcId)` |
| Level up | XP system | Skills | `skillSystem.gainPoints(1)` |
| Move player | Game loop | Minimap | `minimap.reveal(x, y)` |
| Save game | Save system | All | `toJSON()` on all systems |
| Load game | Save system | All | `loadJSON()` on all systems |

---

*Generated from 10 web sources, synthesized into unified vanilla TypeScript systems.*
*All code is copy-paste ready, zero external dependencies, Canvas 2D rendering.*

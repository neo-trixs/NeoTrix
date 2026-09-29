# NeoTrix 引擎升级研究报告 (Round 2)

> 基于开源项目调研，提炼可落地的引擎能力增强方案
> 日期: 2026-09-21

---

## 1. 状态栈 (State Stack) 设计模式

### 开源参考

| 项目 | 模式 | 可借鉴点 |
|------|------|----------|
| **solstack** | Stack-based state machine | push/pop/set/replace + 生命周期回调 |
| **Bevy State** | Stack-based + 状态集 | on_enter/on_exit/on_pause/on_resume |
| **macroquad GameState** | 枚举匹配 | 简单直接，match 驱动 |
| **Rust Forum** | 5 种状态机实现 | enum vs trait vs 嵌套状态 |

### 推荐设计: solstack 风格

```rust
// 借鉴 solstack 的 4 种操作
pub enum StateCommand {
    None,          // 无操作
    Push(Box<dyn GameState>),   // 推入新状态 (当前暂停)
    Pop,           // 弹出顶部 (恢复下方)
    Replace(Box<dyn GameState>),  // 替换顶部
}

// 借鉴 solstack 的 6 个生命周期回调
trait GameState {
    fn on_start(&mut self, data: &mut GameData) {}   // 进入栈顶
    fn on_stop(&mut self, data: &mut GameData) {}    // 从栈中移除
    fn on_pause(&mut self, data: &mut GameData) {}   // 被新状态覆盖
    fn on_resume(&mut self, data: &mut GameData) {}  // 重新成为栈顶
    fn on_tick(&mut self, data: &mut GameData) -> StateCommand;  // 每帧更新
    fn on_shadow_tick(&mut self, data: &mut GameData) {}  // 下方状态也执行
}
```

**典型用例**:
```
push SPauseMenu  → SGame.on_pause() → SPauseMenu.on_start()
pop              → SPauseMenu.on_stop() → SGame.on_resume()
replace SGameOver → SGame.on_stop() → SGameOver.on_start()
```

---

## 2. ECS (Entity Component System) 架构

### 开源参考

| 项目 | 架构 | 性能数据 |
|------|------|----------|
| **Austin Morlan ECS** | ComponentArray + Signature | 缓存命中率 D1 < 1% |
| **rs-ecs** | hecs 变体，简化 API | 无锁，单线程优化 |
| **archetype_ecs** | Archetype + 并行调度器 | 生产级 ECS |
| **chunkedge** | ArchetypeEntity + TableRow | 类型擦除 archetypes |

### 核心模式: 紧凑数组 (Dense Array)

```
问题: Entity 删除后数组出现空洞 → 缓存失效
解决: 末尾元素填补空洞 + 双向映射

Entity→Index: { 0→0, 1→1, 3→2 }  (Entity 3 占据 Index 2)
Index→Entity: { 0→0, 1→3, 2→2 }
Array: [A, D, C]  (D 从 Index 3 移到 Index 2)
Size: 3
```

**Rust 实现**:
```rust
struct ComponentArray<T> {
    data: Vec<T>,           // 紧凑存储
    entity_to_index: HashMap<u64, usize>,  // Entity → 数组下标
    index_to_entity: HashMap<usize, u64>,  // 数组下标 → Entity
    size: usize,
}

impl<T> ComponentArray<T> {
    fn insert(&mut self, entity: u64, component: T) { ... }
    fn remove(&mut self, entity: u64) { ... }  // 末尾填补
    fn get(&self, entity: u64) -> Option<&T> { ... }
}
```

### Signature (位掩码)

```rust
// 每个组件类型对应一个 bit
type Signature = u64;  // 最多 64 种组件

// System 声明需要的组件
let physics_sig = Signature::from_bits(POSITION | VELOCITY | GRAVITY);

// 检查 Entity 是否匹配
if (entity_signature & system_signature) == system_signature { ... }
```

---

## 3. 背包系统 (Inventory)

### 开源参考

| 项目 | 模式 | 可借鉴点 |
|------|------|----------|
| **game_features** (0.8.3) | `Inventory<K, S, U>` 泛型 | 固定/动态大小、堆叠、转移、耐久 |
| **game_inventory** | trait Item + trait Slot | 抽象物品数据、可插拔 |
| **steel_core** | EquipmentSlot + EntityEquipment | 装备系统、8 槽装备 |
| **LegacyRust** | 40 槽 (30 存储 + 6 腰带 + 4 装备) | 经典生存游戏布局 |

### 推荐设计: game_features 风格

```rust
/// 物品元数据 (永不变化)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub max_stack: u16,     // 0 = 不可堆叠
    pub max_durability: Option<u16>,
    pub item_type: ItemType, // Weapon/Armor/Consumable/Material
}

/// 物品实例 (可变状态)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStack {
    pub def_id: String,
    pub quantity: u16,
    pub durability: Option<u16>,
}

/// 背包槽位
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlot {
    pub item: Option<ItemStack>,
    pub restriction: Option<SlotType>,  // 限制类型
}

/// 背包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    slots: Vec<InventorySlot>,
    sizing: SizingMode,  // Fixed / Dynamic
}
```

**game_features 的 API**:
```rust
inventory.insert(item)          // 自动找空位
inventory.insert_into(idx, item) // 指定位置
inventory.move_item(from, to)    // 移动
inventory.transfer(from, target, to)  // 跨背包转移
inventory.use_item(idx)          // 使用 (消耗耐久)
inventory.consume(idx)           // 消耗 (数量-1)
inventory.has(key)               // 检查拥有
inventory.has_quantity(key, n)   // 检查数量
```

---

## 4. 关键开源资源

| 资源 | URL | 用途 |
|------|-----|------|
| solstack | docs.rs/solstack | 状态栈实现 |
| Austin Morlan ECS | austinmorlan.com/posts/entity_component_system | ECS 完整教程 |
| rs-ecs | docs.rs/rs-ecs | 简化 ECS API |
| archetype_ecs | docs.rs/archetype_ecs | 生产级 ECS |
| game_features | docs.rs/game_features | 泛型背包系统 |
| game_inventory | lib.rs/crates/game_inventory | 抽象物品框架 |
| steel_core | rustdoc.steelmc.dev | 装备系统实现 |
| Macroquad Book | mq.agical.se | 状态管理教程 |

---

## 5. 优先级排序

| 优先级 | 任务 | 工作量 | 收益 |
|--------|------|--------|------|
| 🔴 P0 | 状态栈生命周期回调 (6 个) | 1h | 状态管理完整性 |
| 🔴 P0 | ECS 紧凑数组 (dense storage) | 2h | 缓存性能 |
| 🟡 P1 | 背包系统 (Inventory) | 3h | 核心游戏功能 |
| 🟡 P1 | ECS Signature 位掩码 | 1h | 查询性能 |
| ⚪ P2 | 装备系统 (Equipment) | 2h | 战斗深度 |
| ⚪ P2 | 制作系统 (Crafting) | 3h | 游戏循环 |

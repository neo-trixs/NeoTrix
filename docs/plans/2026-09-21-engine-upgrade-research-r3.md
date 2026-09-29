# NeoTrix 引擎升级研究报告 (Round 3)

> 基于开源项目调研，提炼可落地的引擎能力增强方案
> 日期: 2026-09-21

---

## 1. ECS 物理系统 — 运动/碰撞/重力

### 开源参考

| 项目 | 模式 | 可借鉴点 |
|------|------|----------|
| **Bevy ECS** | `Query<(&mut Position, &Velocity)>` | 系统函数签名驱动数据访问 |
| **Bevy Quickstart** | Avian2d (RigidBody/Collider/LinearVelocity) | ECS 物理引擎集成模式 |
| **freECS** | Archetype-based, Schedule API | 轻量级系统调度 |
| **Legion** | `#[system(for_each)]` 宏 | 简化系统定义 |
| **specs** | `Join` + `Dispatcher` | 并行系统调度 |
| **Rust Roguelike Tutorial** | Velocity+Position+Friction | 简单物理系统 |

### 推荐设计: Bevy 风格 ECS 物理系统

```rust
// 组件 (借鉴 Bevy ECS)
#[derive(Debug, Clone)]
pub struct Position { pub x: f32, pub y: f32 }

#[derive(Debug, Clone)]
pub struct Velocity { pub x: f32, pub y: f32 }

#[derive(Debug, Clone)]
pub struct Gravity { pub scale: f32 }

#[derive(Debug, Clone)]
pub struct Collider { pub w: f32, pub h: f32 }

#[derive(Debug, Clone)]
pub struct Static;

// 系统函数 (借鉴 Bevy: 系统是普通函数)
fn movement_system(ecs: &mut SimpleEcs, dt: f32) {
    for (id, pos, vel) in ecs.query2_mut::<Position, Velocity>() {
        pos.x += vel.x * dt;
        pos.y += vel.y * dt;
    }
}

fn gravity_system(ecs: &mut SimpleEcs, dt: f32) {
    for (id, pos, vel, grav) in ecs.query3_mut::<Position, Velocity, Gravity>() {
        vel.y += 980.0 * grav.scale * dt; // 重力加速度
    }
}

fn friction_system(ecs: &mut SimpleEcs, dt: f32) {
    for (id, vel) in ecs.query1_mut::<Velocity>() {
        let friction = 0.9; // 摩擦系数
        vel.x *= friction;
        vel.y *= friction;
    }
}

fn collision_system(ecs: &mut SimpleEcs) {
    // AABB 碰撞检测 — 借鉴 Rust Roguelike Tutorial
    let entities: Vec<(u64, Position, Collider)> = ecs.query2::<Position, Collider>()
        .into_iter().map(|(id, p, c)| (*id, p.clone(), c.clone())).collect();
    
    for i in 0..entities.len() {
        for j in (i+1)..entities.len() {
            let (id_a, pos_a, col_a) = &entities[i];
            let (id_b, pos_b, col_b) = &entities[j];
            
            if aabb_overlap(pos_a, col_a, pos_b, col_b) {
                // 处理碰撞 — 位置修正
                resolve_collision(ecs, *id_a, *id_b, pos_a, col_a, pos_b, col_b);
            }
        }
    }
}
```

**关键模式**:
- **Bevy 风格**: 系统函数签名自动推导数据访问
- **freECS 风格**: Schedule 管理系统执行顺序
- **Rust Roguelike 风格**: 简单 AABB 碰撞 + 位置修正

---

## 2. 装备槽系统

### 开源参考

| 项目 | 槽位设计 | 可借鉴点 |
|------|----------|----------|
| **steel_core** | `EquipmentSlot` enum (8 slots) | Minecraft 风格装备槽 |
| **physis** | `EquipSlot` enum (14 slots, FFXIV 风格) | 完整装备系统 |
| **game_features** | `Inventory<K, S, U>` 泛型 | 类型安全装备系统 |
| **Rust Wiki** | 双层装备 (L1 服装 + L2 护甲) | 分层装备概念 |

### 推荐设计: steel_core 风格装备系统

```rust
/// 装备槽位 — 借鉴 steel_core 的 EquipmentSlot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipSlot {
    MainHand,    // 主手武器
    OffHand,     // 副手盾牌/武器
    Head,        // 头盔
    Chest,       // 胸甲
    Legs,        // 腿甲
    Feet,        // 靴子
    Ring1,       // 戒指1
    Ring2,       // 戒指2
    Necklace,    // 项链
}

impl EquipSlot {
    pub const ALL: [EquipSlot; 9] = [
        Self::MainHand, Self::OffHand, Self::Head,
        Self::Chest, Self::Legs, Self::Feet,
        Self::Ring1, Self::Ring2, Self::Necklace,
    ];
    
    pub const ARMOR_SLOTS: [EquipSlot; 4] = [
        Self::Head, Self::Chest, Self::Legs, Self::Feet,
    ];
    
    pub fn is_armor(self) -> bool {
        Self::ARMOR_SLOTS.contains(&self)
    }
}

/// 装备系统 — 借鉴 steel_core 的 OwnedEntityEquipment
pub struct Equipment {
    slots: HashMap<EquipSlot, Option<ItemStack>>,
}

impl Equipment {
    pub fn new() -> Self {
        let mut slots = HashMap::new();
        for slot in EquipSlot::ALL {
            slots.insert(slot, None);
        }
        Self { slots }
    }
    
    /// 装备物品
    pub fn equip(&mut self, slot: EquipSlot, item: ItemStack) -> Option<ItemStack> {
        self.slots.insert(slot, Some(item))
    }
    
    /// 卸下装备
    pub fn unequip(&mut self, slot: EquipSlot) -> Option<ItemStack> {
        self.slots.insert(slot, None).flatten()
    }
    
    /// 获取装备
    pub fn get(&self, slot: EquipSlot) -> Option<&ItemStack> {
        self.slots.get(&slot).and_then(|i| i.as_ref())
    }
    
    /// 计算总防御力
    pub fn total_defense(&self, defs: &[ItemDef]) -> f32 {
        self.slots.values()
            .filter_map(|i| i.as_ref())
            .filter_map(|item| defs.iter().find(|d| d.id == item.def_id))
            .map(|def| def.defense_value)
            .sum()
    }
}
```

---

## 3. NPC 商店/交易系统

### 开源参考

| 项目 | 模式 | 可借鉴点 |
|------|------|----------|
| **GUIShop (Rust)** | `ItemData { Buy, Sell, Cooldown }` | 买卖价格分离 |
| **Roaming NPC Vendors** | 漫游NPC + 动态定价 | NPC 行为+经济 |
| **DayZ Trading** | `ShopItem { BuyPrice, SellPrice }` | 服务器验证交易 |
| **Rustya** | Buy/Sell Offers 列表 | NPC 购买/出售列表 |
| **Facepunch Dynamic Pricing** | 滚动窗口定价 | 动态经济系统 |

### 推荐设计: GUIShop 风格商店系统

```rust
/// 商店物品 — 借鉴 GUIShop 的 ItemData
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopItem {
    pub item_id: String,
    pub buy_price: u32,      // 玩家购买价格
    pub sell_price: u32,     // 玩家出售价格
    pub stock: Option<u32>,  // None = 无限库存
    pub level_required: u32, // 等级要求
}

/// 商店 — 借鉴 GUIShop 的 Shop 概念
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shop {
    pub id: String,
    pub name: String,
    pub items: Vec<ShopItem>,
    pub currency: String,     // 货币类型
}

/// 交易结果
pub enum TradeResult {
    Success { item: ItemStack, cost: u32 },
    InsufficientFunds { needed: u32, have: u32 },
    InsufficientStock,
    LevelTooLow { required: u32 },
    InventoryFull,
    ItemNotFound,
}

/// 商店管理器
pub struct ShopManager {
    shops: HashMap<String, Shop>,
    player_gold: u32,
}

impl ShopManager {
    /// 购买物品 — 借鉴 DayZ 的服务器验证模式
    pub fn buy(&mut self, shop_id: &str, item_id: &str, quantity: u32, 
               inventory: &mut Inventory, defs: &[ItemDef]) -> TradeResult {
        let shop = match self.shops.get(shop_id) {
            Some(s) => s,
            None => return TradeResult::ItemNotFound,
        };
        
        let shop_item = match shop.items.iter().find(|i| i.item_id == item_id) {
            Some(i) => i,
            None => return TradeResult::ItemNotFound,
        };
        
        let total_cost = shop_item.buy_price * quantity;
        
        // 检查金币
        if self.player_gold < total_cost {
            return TradeResult::InsufficientFunds { 
                needed: total_cost, 
                have: self.player_gold 
            };
        }
        
        // 检查库存
        if inventory.free() == 0 {
            return TradeResult::InventoryFull;
        }
        
        // 检查库存
        if let Some(stock) = shop_item.stock {
            if stock < quantity {
                return TradeResult::InsufficientStock;
            }
        }
        
        // 执行交易
        self.player_gold -= total_cost;
        let item = ItemStack::new(&shop_item.item_id, quantity);
        inventory.insert(item.clone(), defs);
        
        TradeResult::Success { item, cost: total_cost }
    }
    
    /// 出售物品 — 借鉴 Rustya 的 Sell Offers
    pub fn sell(&mut self, shop_id: &str, slot_idx: u32, quantity: u32,
                inventory: &mut Inventory) -> TradeResult {
        let shop = match self.shops.get(shop_id) {
            Some(s) => s,
            None => return TradeResult::ItemNotFound,
        };
        
        let item = match inventory.get_slot(slot_idx as usize) {
            Some(slot) => match &slot.item {
                Some(i) => i.clone(),
                None => return TradeResult::ItemNotFound,
            },
            None => return TradeResult::ItemNotFound,
        };
        
        let shop_item = shop.items.iter().find(|i| i.item_id == item.def_id);
        let sell_price = shop_item.map_or(0, |i| i.sell_price);
        
        if sell_price == 0 {
            return TradeResult::ItemNotFound; // 商店不收购此物品
        }
        
        let total_earn = sell_price * quantity;
        
        // 执行交易
        self.player_gold += total_earn;
        inventory.remove(slot_idx as usize, quantity);
        
        TradeResult::Success { 
            item: ItemStack::new(&item.def_id, quantity), 
            cost: total_earn 
        }
    }
}
```

---

## 4. ECS 迁移模式

### 开源参考

| 项目 | 迁移策略 | 可借鉴点 |
|------|----------|----------|
| **Bevy ECS** | `world.spawn((Comp1, Comp2, ...))` | 批量创建实体 |
| **Legion** | `world.extend(vec![(...), ...])` | 批量插入 |
| **freECS** | `world.spawn((...)).add_tag::<Player>()` | 标记系统 |
| **Bevy Quickstart** | 组件 → 系统 → 调度 | 渐进迁移 |

### 推荐迁移路径

```
Phase 1: 当前状态 — Vec<GameEntity> + ECS 镜像
Phase 2: ECS 渲染 — render_ecs_entities() 已完成
Phase 3: ECS 物理 — movement/gravity/friction/collision systems
Phase 4: ECS 游戏逻辑 — combat/dialogue/quest 通过 ECS 查询
Phase 5: 移除 Vec<GameEntity> — 完全迁移到 ECS
```

---

## 5. 关键资源

| 资源 | URL | 用途 |
|------|-----|------|
| Bevy ECS | docs.rs/bevy_ecs | ECS 架构参考 |
| Bevy Quickstart | thebevyflock.github.io | Avian2d 物理集成 |
| freECS | docs.rs/freecs | 轻量级 ECS |
| Legion | docs.rs/legion | 并行系统调度 |
| steel_core | rustdoc.steelmc.dev | 装备系统实现 |
| game_features | docs.rs/game_features | 泛型背包/装备 |
| GUIShop | github.com/Nogrod/OxidePlugins | 商店系统设计 |
| Avian2d | docs.rs/avian2d | ECS 物理引擎 |

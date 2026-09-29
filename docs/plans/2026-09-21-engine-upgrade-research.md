# NeoTrix 引擎升级研究报告

> 基于开源项目调研，提炼可落地的引擎能力增强方案
> 日期: 2026-09-21

---

## 1. EventBus 升级方案

### 当前实现
```rust
// 简单枚举 + HashMap<String, Vec<Box<dyn Fn>>>
pub enum GameEvent { ... }
pub struct EventBus { listeners: HashMap<String, Vec<ListenerFn>>, queue: Vec<GameEvent> }
```

### 开源参考

| 项目 | Stars | 核心模式 | 可借鉴点 |
|------|-------|----------|----------|
| **Geese** (DouglasDwyer/geese) | 60 | System + Dependencies + EventHandlers | 系统依赖声明、自动拓扑排序、多线程并行 |
| **eventbus-rs** (olivdx/eventbus-rs) | 1 | #[derive(Event)] + Idempotent Inbox + DLQ | 编译期类型安全、幂等投递、死信队列 |
| **Bevy ECS** | 47.9K | Events<T> + EventReader/Writer | 泛型事件、自动清理、变更检测 |

### 推荐升级: 类型安全泛型 EventBus

```rust
// 升级后: 编译期类型安全，无需字符串分类
pub struct EventBus {
    // 按类型ID索引，而非字符串
    handlers: HashMap<TypeId, Vec<Box<dyn Fn(&dyn Any)>>>,
    queue: Vec<Box<dyn Any>>,
}

impl EventBus {
    /// 注册: 编译期检查事件类型
    pub fn on<T: 'static>(&mut self, handler: impl Fn(&T) + 'static) { ... }

    /// 发射: 编译期检查事件类型
    pub fn emit<T: 'static>(&self, event: T) { ... }

    /// 延迟发射
    pub fn enqueue<T: 'static>(&mut self, event: T) { ... }
}
```

**借鉴 Geese 的系统依赖**:
```rust
// 声明系统间依赖，自动拓扑排序
trait GameSystem {
    const DEPENDENCIES: &'static [TypeId] = &[];
    fn on_event(&mut self, event: &dyn Any, ctx: &SystemContext);
}
```

**借鉴 Bevy 的事件清理**:
```rust
// 自动清理已处理事件，防止内存泄漏
pub fn cleanup(&mut self) {
    self.queue.clear(); // 或保留最近 N 帧
}
```

---

## 2. SaveSystem 升级方案

### 当前实现
```rust
// 简单 JSON 快照
pub struct SaveData { pub meta: SaveMeta, pub player_x: f32, ... }
pub struct SaveManager { save_dir: PathBuf }
```

### 开源参考

| 项目 | 核心模式 | 可借鉴点 |
|------|----------|----------|
| **Rust Roguelike Tutorial** | Specs ECS + #[derive(ConvertSaveload)] + SimpleMarker | ECS 实体序列化、Marker 分配器、宏批量序列化 |
| **LITIENGINE** | SaveData POJO + Binary/JSON + Atomic Write | 原子写入防损坏、世界标志恢复 |
| **specs::saveload** | SerializeComponents + MarkerAllocator | 组件级序列化、实体 ID 稳定化 |
| **Bevy Scene** | SceneBundle + Reflect | 反射驱动序列化、场景嵌套 |

### 推荐升级: 增量存档 + 原子写入

```rust
/// 存档数据 (支持增量)
pub struct SaveData {
    pub meta: SaveMeta,
    pub version: u32,  // 存档版本，支持迁移
    pub full_snapshot: Option<FullSnapshot>,  // 首次全量
    pub deltas: Vec<StateDelta>,  // 后续增量
}

/// 原子写入 (借鉴 LITIENGINE)
pub fn save_atomic(&self, slot: u8, data: &SaveData) -> Result<(), String> {
    let tmp_path = self.save_dir.join(format!("slot_{}.tmp", slot));
    let final_path = self.save_dir.join(format!("slot_{}.json", slot));

    // 1. 写入临时文件
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write(&tmp_path, json)?;

    // 2. 原子重命名 (POSIX 保证)
    std::fs::rename(&tmp_path, &final_path)?;

    Ok(())
}

/// 存档版本迁移
pub fn migrate(data: &mut SaveData, from_version: u32) {
    if from_version < 2 {
        // v1 → v2: 添加新字段默认值
        data.meta.play_time_secs = 0.0;
    }
}
```

**借鉴 Specs 的实体序列化**:
```rust
// 用稳定的 Entity ID 替代内存地址
#[derive(Serialize, Deserialize)]
struct EntityRef {
    stable_id: u64,  // 持久化 ID，非内存地址
    components: Vec<ComponentData>,
}
```

---

## 3. ECS 架构参考

### 开源参考

| 项目 | 架构 | 性能数据 |
|------|------|----------|
| **Bevy ECS** | Archetype-based, SoA | 10K 实体查询 < 1ms |
| **Blubber Engine** (论文) | Hybrid ECS, Component Pools | 缓存命中率 > 95% |
| **helios** (Medium 2026) | Phase/Pass Architecture | DOD vs OOP: 3-5x 提升 |

### 可选演进路径

**Phase 1 (当前)**: Vec<GameEntity> + 手动查询
```rust
// 简单但 O(n) 查询
let player = self.entities.iter().find(|e| e.is_player);
```

**Phase 2 (推荐)**: 简单 ECS 原型
```rust
// 按组件分组存储，缓存友好
struct SimpleEcs {
    positions: Vec<Option<(f32, f32)>>,  // Entity ID → Position
    healths: Vec<Option<(f32, f32)>>,    // Entity ID → (hp, max_hp)
    names: Vec<Option<String>>,
    free_ids: Vec<u64>,
    next_id: u64,
}

impl SimpleEcs {
    pub fn query<A: Component>(&self) -> Vec<(u64, &A)> { ... }
    pub fn query2<A: Component, B: Component>(&self) -> Vec<(u64, &A, &B)> { ... }
}
```

**Phase 3 (远期)**: Archetype ECS
```rust
// 按组件组合分组，相同组合的实体连续存储
struct Archetype {
    component_types: HashSet<TypeId>,
    entities: Vec<u64>,
    columns: HashMap<TypeId, Box<dyn AnyVec>>,
}
```

---

## 4. 事件驱动架构最佳实践

### 来自 Geese 的设计原则

1. **依赖声明**: 系统声明依赖，框架自动拓扑排序
2. **FIFO 保证**: 事件按发射顺序处理
3. **依赖优先**: 父系统的事件处理器先于子系统执行
4. **多线程并行**: 无依赖的系统自动并行

### 来自 eventbus-rs 的可靠性模式

1. **幂等投递**: 同一事件只处理一次 (MessageId 去重)
2. **死信队列**: 失败事件隔离，不阻塞正常流
3. **重试策略**: 可配置退避 (1s → 5s → 30s → 300s)
4. **优雅关闭**: drain() 等待处理中的事件完成

### 来自 Bevy 的便利性

1. **泛型事件**: `Events<T>` 为每种事件类型独立存储
2. **自动清理**: `EventReader` 自动标记已处理
3. **变更检测**: `Changed<T>` 查询只返回变化的组件

---

## 5. 存档系统最佳实践

### 来自 Rust Roguelike Tutorial

1. **Marker 模式**: 用 StableId 替代 Entity 内存地址
2. **Skip 序列化**: `#[serde(skip)]` 排除临时数据 (如 tile_content)
3. **宏批量处理**: 超过 16 个组件类型时用宏展开

### 来自 LITIENGINE

1. **原子写入**: 先写 .tmp 再 rename，防止写入中断损坏
2. **世界标志**: openedChests / defeatedBosses 用 ID 列表
3. **环境恢复**: 加载后重建物理/碰撞状态

### 来自 Bevy Scene

1. **反射驱动**: `Reflect` trait 自动序列化任意结构体
2. **场景嵌套**: 子场景可独立保存/加载
3. **热重载**: 文件变化时自动重新加载

---

## 6. 优先级排序

| 优先级 | 任务 | 工作量 | 收益 |
|--------|------|--------|------|
| 🔴 P0 | EventBus 泛型化 (TypeId 索引) | 2h | 类型安全、重构友好 |
| 🔴 P0 | SaveSystem 原子写入 | 1h | 防存档损坏 |
| 🟡 P1 | 存档版本迁移机制 | 2h | 向前兼容 |
| 🟡 P1 | EventBus 延迟队列优化 | 1h | 防止事件风暴 |
| ⚪ P2 | 简单 ECS 原型 | 4h | 缓存友好、查询性能 |
| ⚪ P2 | 系统依赖声明 (Geese 模式) | 3h | 自动排序、并行 |
| ⚪ P3 | 存档压缩 (zstd) | 2h | 减少磁盘占用 |
| ⚪ P3 | 存档加密 (AES-GCM) | 3h | 防作弊 |

---

## 7. 关键开源资源

| 资源 | URL | 用途 |
|------|-----|------|
| Geese | github.com/DouglasDwyer/geese | 游戏事件系统设计 |
| eventbus-rs | github.com/olivdx/eventbus-rs | 可靠事件投递模式 |
| Bevy | github.com/bevyengine/bevy | ECS + 事件 + 场景系统 |
| Roguelike Tutorial | bfnightly.bracketproductions.com | ECS 存档完整实现 |
| LITIENGINE | docs.litiengine.com/savegames | 存档最佳实践 |
| Blubber Engine | scitepress.org/Papers/2026/150001 | ECS 性能基准 |
| helios | medium.com/@thorstensuckow | Phase/Pass 架构 |

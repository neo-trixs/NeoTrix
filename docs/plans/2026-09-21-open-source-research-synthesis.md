# NeoTrix 开源技术资料综合研究报告

> 生成时间: 2026-09-21 | 涵盖 5 大领域、30+ 开源项目

---

## 一、ECS 战斗系统

### 1.1 多阶段伤害流水线 (bevy_diesel)

**核心思想**: 每个伤害阶段是独立类型，`.chain()` 在一帧内顺序解析。

```rust
#[derive(Message, Clone, Reflect, PropagatedMessage)]
pub struct Attack { defender: Entity, attacker: Entity, ability: Entity, element: String }

#[derive(Message, Clone, Reflect, PropagatedMessage)]
pub struct Hit { defender: Entity, hit_value: f32, element: String }

#[derive(Message, Clone, Reflect, PropagatedMessage)]
pub struct Damage { defender: Entity, amount: f32, element: String }

// 链式解析: resolve_attack → resolve_hit → resolve_damage → on_killed
app.add_systems(Update,
    (resolve_attack, resolve_hit, resolve_damage, on_killed).chain()
);
```

**防御解析**:
```rust
fn resolve_hit(mut reader: MessageReader<Hit>, q_armor: Query<&Armor>, mut writer: MessageWriter<Damage>) {
    for hit in reader.read() {
        let mut remaining = hit.hit_value;
        if let Ok(armor) = q_armor.get(hit.defender) { remaining -= armor.0; }
        if remaining <= 0.0 { continue; } // 完全吸收
        writer.write(Damage { amount: remaining, .. });
    }
}
```

**适用性**: ⭐⭐⭐⭐⭐ — 可直接替代 neotrix-game 当前的简单 `execute()` 方法

### 1.2 状态效果系统 (bevy_alchemy)

**核心思想**: 状态效果就是子实体，支持三种合并模式。

| 合并模式 | 行为 |
|---------|------|
| Stack | 多层叠加 (毒素叠加) |
| Insert | 覆盖写入 (眩晕刷新) |
| Merge | 可配置合并函数 |

**工具组件**: `Lifetime`(自动消失), `Delay`(周期触发), `EffectStacks`(层数)

### 1.3 属性依赖图 (bevy_gauge)

```rust
commands.spawn(attributes! {
    "Strength"  => 20.0,
    "MaxHealth" => "Vitality * 10.0 + 50.0",  // 表达式自动传播
    "Damage.added" [DamageTags::FIRE] => 10.0, // 带标签
});
// 跨实体: 武器伤害随使用者力量缩放
"Damage" => "Strength@Wielder"
```

### 1.4 伤害公式参考 (bevy_battle)

```
基础伤害 = 1d7(6..=12) + 基础加成 + 装备加成
锐利加成 = 3 × 锐利层数
虚弱惩罚 = ×0.7
命中判定 = random(0..1) > accuracy ? Miss
暴击判定 = random() < crit_rate ? ×2
护甲减免 = (伤害 - 护甲).max(1)
吸血 = 伤害 × 吸血百分比
```

### 1.5 开源项目对照表

| 项目 | 模式 | 伤害公式 | 状态效果 | AI |
|------|------|---------|---------|-----|
| bevy_battle | 纯 Rust + Bevy 插件 | 1d7 + 加成 - 护甲, 暴击 ×2 | 职业触发 | 职业制 |
| bevy_turn-based_combat | 阶段制回合 | 技能制 | 回合衰减 | 计划 4 种策略 |
| echos_rl | RON 数据驱动 | 属性制 | Health/Stats 组件 | 行为评分 |
| bevy_diesel | 消息流水线 | Attack→Hit→Dmg→Kill 链 | 通过 gauge 属性 | 模板组合 |
| bevy_alchemy | 效果即实体 | N/A (框架) | Stack/Insert/Merge | N/A |
| bevy_gameplay_effects | UE5 GAS 克隆 | 加成/乘算/边界 | 4 种持续时间 + 叠层 | N/A |

---

## 二、商店/经济/交易系统

### 2.1 三阶段交易状态机 (Veloren)

```
Mutate → Review → Complete
```

- **Mutate**: 双方修改交易内容
- **Review**: 确认阶段，防 bait-and-switch
- **Commit**: 原子提交，物品直到 commit 才转移

**定价**: 材料频率的倒数 + 站点经济因子

### 2.2 多货币 + 懒补货 (Moonforge)

```csharp
ShopDefinition(id, entries, restockIntervalMinutes)
ShopEntryDefinition(itemId, maxStock)  // maxStock=null → 无限

// 多货币选项
buyPriceOptions: [
    PriceOptionDefinition([PriceComponent("gold", 100)]),     // 普通价
    PriceOptionDefinition([PriceComponent("gold", 70)]),      // 折扣价 (声望≥50)
]
```

**关键设计**:
- 懒补货: `(当前分钟 - 上次补货) >= interval` 时触发
- 原子交易: `EconomyTransactionCommand` — 任何失败都回滚

### 2.3 经济健康指标 (AgentE + IVX)

| 指标 | 健康范围 | 说明 |
|------|---------|------|
| Gini 系数 | 0.3–0.6 | 贫富差距 |
| 通胀率 | <5%/月 | 货币贬值速度 |
| 流通速度 | 0.5–2.0/周 | 货币换手频率 |
| 水槽/水源比 | 0.8–1.2 | 消耗/产出平衡 |

**五阶段管道**: Observer → Diagnoser → Simulator → Planner → Executor

### 2.4 物品状态轴 (bevy_advanced_item_system)

```rust
#[variant_of(ItemState)]
enum ItemState {
    OnGround,
    EquippedBy(Entity),
    StoredIn(Entity),
}
// 物品始终处于恰好一个状态，插入新状态自动撤回旧状态
```

---

## 三、数据驱动架构

### 3.1 RON 实体定义 (Echos RL)

```ron
EntityDefinition(
    name: "Player",
    components: EntityComponents(
        turn_actor: Some(TurnActorData(speed: 100)),
        health: Some(HealthData(max_health: 30, current_health: 30)),
        stats: Some(StatsData(strength: 3, defense: 2)),
    ),
)
```

**生成命令**: `commands.spawn(SpawnAICommand { entity_key: "hostile_guard".to_string(), position });`

### 3.2 嵌套资产引用 (ron_asset_manager)

```rust
#[derive(Asset, RonAsset, Deserialize)]
pub struct Wizard {
    pub name: String,
    #[asset] pub sprite: Shandle<Image>,        // 自动加载 PNG
    #[asset] pub sounds: HashMap<String, Shandle<AudioSource>>,
    #[asset] pub spells: Vec<Shandle<Spells>>,   // 递归加载
}
```

### 3.3 对话系统方案对比

| 方案 | 语言 | 特点 | URL |
|------|------|------|-----|
| bubbles | 自定义 .bub | 编译为 Rust 结构体, 变量/条件/本地化 | github.com/viezevingertjes/bubbles |
| dramaturge | YAML | 事实引擎, 角色记忆, 对齐轴 | github.com/sunsided/dramaturge |
| bevy_talks | RON | 动作图, ECS 实体化 | github.com/giusdp/bevy_talks |

**bubbles 示例**:
```
title: Start
Speaker: 你好!
-> 选项1 | 选项2

title: 选项1
Speaker: 你选对了。
-> End
```

### 3.4 任务系统方案

| 方案 | 格式 | 特点 |
|------|------|------|
| bevy_quests | Protobuf → JSON | 完整 schema: Quest/Step/Objective/Reward/Chain |
| game-utils-quest | Serde | 轻量: start → advance → completable → complete |
| TOML + Lua | TOML 定义 + Lua 脚本 | 复杂分支用 Lua |

### 3.5 地图/关卡系统

| 方案 | 工具 | 特点 |
|------|------|------|
| macroquad-tiled | Tiled JSON | 最成熟的 2D 地图编辑器集成 |
| macroquad_ldtk | LDtk | 现代关卡设计工具 |
| layer-proc-gen | 分层生成 | 每层引用更大范围的依赖层, LRU 缓存 |

---

## 四、WASM 优化

### 4.1 体积优化管线

```toml
[profile.release]
opt-level = "s"          # 10–25% 节省
lto = true               # 10–20% 节省
codegen-units = 1        # 5–10% 节省
panic = "abort"          # 5–15% 节省
strip = true             # 5–15% 节省
```

后处理: `wasm-opt -Oz` 再省 15–30%

| 阶段 | 大小 |
|------|------|
| 原始 .wasm | 120 KB |
| gzip | 45 KB |
| Brotli | 38 KB |

### 4.2 边界穿越成本 (关键陷阱)

| 路径 | 成本倍数 |
|------|---------|
| Rust→Rust | 1× |
| JS→Rust (原始类型) | ~10× |
| JS→Rust (字符串/数组复制) | ~50× |
| JS→Rust (serde 往返) | ~100× |
| Rust→DOM (web-sys) | ~200× |

**规则**: 在 Rust 侧批量工作，不要在紧密循环中穿越边界。

### 4.3 浏览器存档方案

| 方案 | 容量 | 阻塞 | 适用 |
|------|------|------|------|
| localStorage | ~5–10 MB | 同步 | 设置/小存档 |
| IndexedDB | 50+ MB | 异步 | 游戏状态 |

**关键陷阱**:
- `beforeunload` 必须同步 — 用 localStorage 做紧急存档
- Safari 可能回收 IndexedDB — 调用 `navigator.storage.persist()`

### 4.4 多人游戏

| 方案 | 特点 | Stars |
|------|------|-------|
| **matchbox** | P2P WebRTC, GGRS 回滚 | 1.5K+ |
| **naia** | 权威服务器, 实体同步 | 1.1K |
| **gloo-net** | 底层 WebSocket | — |

---

## 五、物理/碰撞/寻路

### 5.1 Coyote Time + Jump Buffer (mawida)

```rust
pub struct Jump {
    pub coyote: f32,    // 默认 0.10s (100ms)
    pub buffer: f32,    // 默认 0.12s (120ms)
}

impl Jump {
    pub fn update(&mut self, dt: f32, grounded: bool, pressed: bool) -> bool {
        self.since_grounded = if grounded { 0.0 } else { (self.since_grounded + dt).min(f32::MAX) };
        self.since_pressed = if pressed { 0.0 } else { (self.since_pressed + dt).min(f32::MAX) };
        if self.since_pressed <= self.buffer && self.since_grounded <= self.coyote {
            self.since_pressed = f32::MAX;
            self.since_grounded = f32::MAX;
            true
        } else { false }
    }
}
```

**调优参考**:

| 游戏类型 | Coyote Time | Jump Buffer |
|---------|-------------|-------------|
| 精密平台 | 70–100ms | 70–110ms |
| 动作平台 | 90–140ms | 100–150ms |
| 休闲移动 | 110–170ms | 120–180ms |

### 5.2 碰撞检测模式

**空间哈希 (gravita)**:
- 2D spatial-hash broadphase (圆形 + AABB)
- 半隐式 Euler 或 Verlet 积分
- 恢复系数 + 摩擦

**瓦片碰撞 (CollisionMap)**:
```rust
pub fn sweep_circle(&self, start: Vec2, end: Vec2, radius: f32) -> Vec2 {
    // 步进检测，每步 tile_size * 0.25
    // 失败时分别尝试 X/Y 轴滑动
}
```

### 5.3 寻路方案

| 方案 | 算法 | 特点 |
|------|------|------|
| raasta | A*, JPS, Theta*, D* Lite, Flow Fields | 最完整的 Rust 寻路库 |
| gridkit | A*, Theta*, FlowField | 闭包驱动, 无容器锁定 |
| bevy_pathfinder | BFS Flow Fields | O(1) 查询下一步方向 |
| bevy_flowfield_tiles | 扇区 Flow Fields | LOS 优化, 门户图 |

### 5.4 物理引擎对比

| | Rapier2D | Avian | 自定义 |
|---|---------|-------|--------|
| 性能 | 最佳 (SIMD) | 良好 | 不定 |
| 复杂度 | 中等 | 低 (ECS 原生) | 高 |
| 功能 | 完整 (关节, CCD, 传感器) | 增长中 | 你的责任 |
| 适用 | 通用物理 | Bevy 原生工作流 | 独特机制 |

---

## 六、立即可用的改进方案

### 优先级 1 (本 session 可实施)

| 改进 | 参考项目 | 预估工作量 |
|------|---------|-----------|
| Coyote Time + Jump Buffer | mawida::Jump | 30 行代码 |
| 伤害流水线 Attack→Hit→Kill | bevy_diesel | 中等 |
| 数据驱动敌人定义 (RON) | echos_rl | 中等 |
| 商店补货机制 | Moonforge | 小 |

### 优先级 2 (下一个 session)

| 改进 | 参考项目 | 预估工作量 |
|------|---------|-----------|
| 状态效果系统 (Stack/Insert/Merge) | bevy_alchemy | 中等 |
| 属性表达式系统 | bevy_gauge | 中等 |
| 对话脚本系统 | bubbles/bevy_talks | 大 |
| WASM 体积优化 | LTO + wasm-opt | 配置 |

### 优先级 3 (后续)

| 改进 | 参考项目 | 预估工作量 |
|------|---------|-----------|
| 多人游戏 (matchbox) | matchbox + GGRS | 大 |
| NavMesh 寻路 | raasta | 大 |
| 完整经济模拟 | AgentE 5 阶段管道 | 大 |
| Lua/Rhai 脚本集成 | mlua/bevy_scriptum | 大 |

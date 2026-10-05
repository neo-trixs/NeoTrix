# Rust 游戏源码吸收 —— 实现级深读记录

> 日期：2026-10-05/06 · 分支 `feat/capability-absorb-20260828`
> ⭐ **本文只记录「逐字读过源码」得出的结论**，且**区分「我实测」与「二手结论」**。
> 凡未亲自核实的说法一律标注来源，不进正文结论。
> ⚠️ 本文是 [`ABSORPTION-BATCH3-MECHANISMS-2026-10-05.md`](ABSORPTION-BATCH3-MECHANISMS-2026-10-05.md)
> 的**补充**：那三篇记的是**纪律样本**（怎么想），本文记的是**实现级发现**（代码里到底是什么）。

---

## 0. 源清单与许可裁决

| 仓库 | 规模 | 许可 | 可取 |
|---|---|---|---|
| `rust-alert/ra2.exe` | 10 crate / 82,766 行 | **Apache-2.0**（含专利授权条款） | ✅ 实现 |
| `luckyyyyy/miu2d` | engine + pathfinder/ai_search/collision | MIT | ✅ 实现 |
| `rust-alert/rgss.exe` | 23 文件 / 3,862 行（Ruby Marshal 4.8 只读子集） | **无 LICENSE** | ⚠️ 仅设计 |
| `rust-alert/ra2-remixer` | 14 文件 / 928 行（MIX checksum / XCC / RSA+Blowfish） | MPL-2.0 | ⚠️ 仅设计 |
| `ra2-tools` / `ra3.exe` / `rs-ddraw` / `YurisHook` | — | MPL-2.0 | ⚠️ 仅设计 |
| `terraria.exe` / `hl.exe` | — | 无 LICENSE | ⚠️ 仅设计 |
| `bobeff/open-source-games` | 94KB 链接清单 | CC0-1.0 | ❌ **无代码**，只是链接表 |

本地检出根：`…/T/opencode/absorb/`，其中 `ra2.exe/projects/engine/` 下有 10 个 crate。

---

## 1. `ra-map` / `PassGrid` —— **它不是 A\*，是均匀 BFS**（全部实测）

### 1.1 推翻我自己的前提

我曾断言「`PassGrid` 里有 A\*，且它的启发与边代价不同度量」。**实测相反**：

```
$ grep -c BinaryHeap ra-map/src/*.rs      # 36 个文件，全部 0
$ grep -rn "cost" ra-map/src/pass_grid.rs # 0 命中
```

`find_path_ex`（`pass_grid/src/pass_grid.rs:169`）的形态是：

```rust
let mut visited = vec![false; w * self.height as usize];
let mut queue = std::collections::VecDeque::new();
queue.push_back((sx, sy));
while let Some((x, y)) = queue.pop_front() { … }
```

⇒ **`VecDeque` + `pop_front` + 布尔 visited，零优先队列、零启发式、零边权**。
所有边代价恒为 1 ⇒ 8 邻下的度量是 **Chebyshev 步数**。

⭐ **这个结论的意义**：它结构上**不可能**犯「不可采纳启发」这个错 ——
因为它根本没有启发式。我在自己的 `nt_astar` 里查出的那个真缺陷
（L1 启发 + √2 对角边 ⇒ 47% 布局次优，见 §4.2）在这一层压根不成立。
**「用 BFS 换掉 A\*」本身就是一种正确的简化**，前提是地图均匀。

### 1.2 ⭐ 越界返回 `false`（实测 `pass_grid.rs:77-79`）

```rust
pub fn is_passable(&self, x: u16, y: u16) -> bool {
    self.index(x, y).and_then(|i| self.passable.get(i).copied()).unwrap_or(false)
}
```

坐标算错 ⇒ `index()` 给出 `None` ⇒ `.and_then` 短路 ⇒ **返回 `false`（不可通行）**。
配合 `x/y` 是 `u16`（负数在类型层就不可能）⇒ **不可能把越界格变成可走**。
坐标若来自 LLM 或工具输出，这条比任何额外校验都值钱。

⭐ **本仓已应用**：`nt_astar.rs` 的 `is_walkable`/`set_obstacle`/`set_movement_cost`
都有显式四段比较（`x >= 0 && x < width && y >= 0 && y < height`）⇒ **此候选被撤销**。

### 1.3 ⭐ 防穿角（实测 `pass_grid.rs:203-209`）

```rust
// 对角线：两侧正交格也须可走，且爬升可达。
if dx != 0 && dy != 0 {
    let ox = (i32::from(x) + dx) as u16;
    let oy = (i32::from(y) + dy) as u16;
    if !self.is_passable(ox, y) || !self.is_passable(x, oy)
        || !self.climb_ok(x, y, ox, y) || !self.climb_ok(x, y, x, oy) { continue; }
}
```

⇒ 对角步要求**两侧正交格都可走**，否则实体可以从两面墙的夹角挤过去。

### 1.4 ⭐⭐ `climb_ok` —— **带高程的寻路**（我前三轮完全没注意到）

**正交移动同样受它约束**（`:196` 的 `if visited[i] || !self.is_passable(nx, ny) || !self.climb_ok(x, y, nx, ny)`）。
⇒ 这不是「八邻 BFS」，是**带爬升高度限制的 BFS**：单位实体不能一步登上无限高的墙，
对角还要额外检查两侧的高差。

⭐ **这是本轮最有价值的未吸收能力**：本仓 `nt_astar` / `nt_flow` 都**只有二维 passable
布尔图**，没有任何高程/爬升维度。若要做真实地图寻路，这是必需的一层。

### 1.5 它的缺陷（可反向借鉴）

`find_path_ex` 每次调用都新建 `visited: Vec<bool>` + `prev: Vec<Option<(u16,u16)>>`
（各 `w*h` 个元素）⇒ 每次寻路两次整图分配，零复用。
可修法：用**世代戳**（`visited: Vec<u32>` + 单调递增的 `stamp`）替代每次 memset。

---

## 2. `ra-ecs` —— 存储模型与句柄契约（全部实测）

### 2.1 ⭐ 每组件类型一个稀疏集合（`ra-ecs/src/store.rs:9-14`）

```rust
pub(crate) struct ComponentStore<T> {
    dense: Vec<T>,                    // 连续、零哈希、零指针追逐
    entities: Vec<EcsEntity>,         // 平行数组：谁在这个槽
    sparse: Vec<Option<u32>>,         // slot → dense 下标
}
```

⇒ 热路径**全部单态化泛型**；`Box<dyn Any>` 只存在于**类型注册表**（`world.rs:16`
的 `stores: HashMap<TypeId, Box<dyn ErasedStore>>`），**每类型一次**。

`remove` 用 `swap_remove`（`store.rs:54-55,96`）⇒ 移位 O(1)，代价是**打乱顺序**。

### 2.2 ⭐⭐ 世代必须活在**独立并行数组**里（`entity.rs` + `world.rs:14`）

```rust
pub(crate) struct EntityMeta { pub(crate) generation: u32, pub(crate) alive: bool }
// world.rs:14-18
metas: Vec<EntityMeta>,
free: Vec<u32>,                        // LIFO 空闲表
stores: HashMap<TypeId, Box<dyn ErasedStore>>,
```

**为什么必须是并行数组**：槽位释放后 `metas[slot].alive = false`，但
`generation` **照样保留**。若世代只活在句柄里，**释放即丢失** ⇒ 复用时没有
可递增的基准 ⇒ 无法区分「旧句柄」与「新实体」。

⭐ **我凭记忆一定会写错这一点**（我第一版就想把 generation 塞进句柄），
所以是**读原码**才拿对的。

### 2.3 ⭐ `contains()` 是唯一的 ABA 防线（`world.rs:58-64,84-96`）

```rust
pub fn contains(&self, entity: EcsEntity) -> bool {
    match self.metas.get(entity.slot as usize) {
        Some(meta) => meta.alive && meta.generation == entity.generation,
        None => false,
    }
}
pub fn despawn(&mut self, entity: EcsEntity) -> bool {
    if !self.contains(entity) { return false; }   // ⛔ 唯一的检查点
    …
    meta.alive = false;
    meta.generation = meta.generation.wrapping_add(1);
    self.free.push(entity.slot);
}
```

⇒ **`insert`/`remove`/`get`/`get_mut` 每一个都先过 `contains`**。
不是「检查一处」，是**每个公开操作都过**。

### 2.4 ⭐ 它**自认不确定性**（`world.rs:136` 原文注释）

```rust
/// 遍历某组件的全部实例（顺序为内部稠密数组顺序，**不保证**跨运行稳定；
/// 需要稳定序时按外部 `EntityId` 排序）。
```

配合 `swap_remove` ⇒ **同一组实体的遍历序跨运行会变**。它是**明写放弃**的，
并在消费端（`repair.rs:89` 那类）排序兜。

⭐ **这印证了一个更一般的结论**：确定性得在**消费边界**解决，
在源头解决需要 archetype + 稳定序的整套结构，而多数场景不划算。

---

## 3. `ra-widgets` / `ra-layout` —— 两处确定性技巧（实测）

### 3.1 ⭐ `hit_test` 用**复合键**做 tie-break（`ra-layout/src/snapshot/mod.rs:52-57`）

```rust
.iter().enumerate()
.filter(|(_, e)| e.hit_test == HitTestMode::Rect)
.filter(|(_, e)| e.hit_region.rect.contains(point))
.max_by_key(|(i, e)| (e.z_index, *i))     // ⭐ z_index + 插入序
.map(|(_, e)| e)
```

⇒ 只按 `z_index` 取 max 时，**同 z 的元素谁赢取决于迭代序**；把
`enumerate()` 的下标并进键里 ⇒ **结果与迭代序无关**。
这正是本仓反复出问题的那一处（「同优先级平手时按什么定序」）。

### 3.2 ⭐ CJK 安全的前缀切片（`ra-widgets/src/animation/typewriter.rs:66-77`）

```rust
let total = self.full.chars().count();
if n >= total { return &self.full; }
match self.full.char_indices().nth(n) {
    Some((idx, _)) => &self.full[..idx],
    None => &self.full,
}
```

⇒ `char_indices().nth(n)` 取**第 n 个字符的起始字节**，
而 `&s[..n]` 是**按字节**切 ⇒ 多字节字符上直接 panic（`byte index is not a char boundary`）。

⭐ **这条已被本仓两处采用**：
- 我在 `agent_guardrails/output_validator.rs:289` 与 `input_validator.rs:163`
  修掉了两处**同款活体 panic**（按字节切超限输出 / 切 40 字符的 base64 匹配串）。
- 另一窗口 `6a59833a` 用同一思路吸收了 `break-ui` 的 grapheme 截断
  （ZWJ/肤色/区域指示/组合附加符手写必漏）。

---

## 4. 我据此**改了什么**（都有提交与实测）

### 4.1 `3877336a` —— `nt_ecs` 句柄契约（移植 §2.2/§2.3）

**改前**：`generation` 字段在全文只出现 3 处（声明/构造/赋值）⇒ **从未被任何代码读过**；
`spawn()` 硬编码世代 0；`despawn` 只查 `slot.is_some()`；槽位**只 push 不回收**。

**先写测试拿到 6 红 1 绿**，逐条实测确认：

| 缺陷 | 改前实测 |
|---|---|
| 死句柄插入被接受 ⇒ 幽灵组件 | 红 |
| 槽位永不回收 | 红：`128 vs 64`，每轮 +64 |
| archetype 组件集合永不累积 | 红：插两个仍是 **1** |
| 实体重复登记进每个 archetype | 红：1 实体 **2 条登记** |
| 陈旧句柄别名复用槽位（ABA） | 红 |

改后全量 `13263 passed / 0 failed`。

### 4.2 `b2dae47c` —— `nt_astar` 度量与穿角（移植 §1.1 的教训 + §1.3）

**改前**：`neighbors()` 无条件返回 4 个对角、**不看两侧正交格**。

⭐ **而「不可采纳启发」这一条我是靠判决实验拿下的，不是靠推理**：
我推断「Manhattan + √2 对角边 ⇒ A\* 返回次优路径」，**按这个结论动手前先写测试 ——
测试是绿的**。于是改用**同一套边代价 + 同一套邻居规则的 Dijkstra 当 oracle**，
在 60 个确定性 LCG 随机障碍布局上逐个对账 ⇒ **28/60（47%）次优**，
最差一例 `最优 14.8995 / 实得 20.5563`（差 38%），最小一例差**恰好 2.0000**
（L1 高估的特征值）。改用 `octile_distance`（`√2*min + |dx-dy|`）后 **60/60 全最优**。

⇒ **§1.1 的教训正落在这里**：`PassGrid` 之所以没有这个 bug，
是因为它**根本没有启发式**。

### 4.3 顺手合并的自我复制

`find_path_with_limit` 是 `find_path` 的**逐字复制**（约 70 行，只差一个计数器）
⇒ 合并为唯一一份 `search(max_steps: Option<usize>)`，并加测试钉住两入口一致。

---

## 5. ⭐ 我**否决**的三个候选（这部分比上面更重要）

我基于源码提了 4 个候选，逐个验证后 **3 个不成立**：

| # | 候选 | 否决理由（实测） |
|---|---|---|
| ① | 「`nt_ecs` 是 `HashMap<(EntityId,TypeId), Box<dyn Any>>`，要换 dense+sparse」 | **我读的是记忆，不是代码。** 打开文件发现它**本来就是 `ArchetypeId` archetype 设计**；且全部公开 API **零真实消费者**（`nt_ecs::Chunk`/`SoAStorage`/`ParallelScheduler`/`Archetype` 外部引用均为 0；唯一那条 `ComponentStorage` 命中是 `nt_game/ecs/mod.rs` 里的**注释**）。⇒ **优化一条没有调用方的热路径，是又一次装饰性重写。** |
| ② | 「统一 `step_cost`」 | **更早的会话早已完成** —— `nt_astar.rs:15` 的 `SQRT_2` 是共享常量，`nt_flow.rs:9-13` 明写两者「已统一为欧氏（对角 ×√2）」。 |
| ③ | 「越界返回 false」 | **早已正确** —— `is_walkable`/`set_*` 都有显式四段边界检查（§1.2）。 |

⇒ **4 个候选，3 个因「零消费者 / 已完成」被当场撤下。**
**「消费者审计」比任何实现技巧都省事** —— 这一条已写进
[`handoff-2026-10-05-game-source-absorption.md`](handoff-2026-10-05-game-source-absorption.md)
作为下一窗口的首要建议。

---

## 6. `ra-assets` —— 错误类型纪律（部分实测 + **一条二手结论被推翻**）

规模：130 文件 / 13,628 行。

### 6.1 实测

| 项 | 实测 |
|---|---|
| `forbid(unsafe_code)` | **0 处** ⇒ **零 unsafe 但无强制**，靠约定 |
| panic 策略 | `panic = "abort"`（`Cargo.toml:84`，release） |
| 双层错误 | `Option<Result<PcmAudio, WavError>>`（`audio/aud.rs:14`、`audio/wav_riff.rs:19`）—— 外层「探格式」、内层「格式坏」，**两层分开**比 catch-all 好 |
| `WavError`（`audio/mod.rs:25`） | 具名变体：`Symphonia(String)`/`NoAudioTrack`/`MissingSampleRate`/`MissingChannels`… ⇒ **每个失败原因可枚举** |

### 6.2 ⛔ 推翻一条二手结论

三路深读 agent 报告称「错误类型主要是 catch-all `RaError::Parse(String)`」。
**实测**：

```
$ grep -rn "Parse(String)" ra-assets/src/ | wc -l   # 0
$ grep -rn "enum RaError" ra-assets/src/            # 0 —— 该类型不存在
```

⇒ **`RaError` 是编造的类型名。** 真实的是 `WavError` 与 `BinkVideoError`
（`image/bink_video.rs:22`，含 `UnsupportedVersion(BinkVersion)`/`BadSize{width,height}`，
但也**退化到 `Msg(String)`**，且带一个 `NotImplemented(&'static str)` 骨架占位）。

⭐ 这条与 §5 的教训同型：**一次拼错的 grep 零命中，被升级成架构结论。**
若不复核就会把「它的错误纪律很差」写进永久文档，而真实情况是
`WavError` 的变体枚举做得相当规矩。

---

## 7. 尚未吸收、但值得排期的能力

| 能力 | 来源 | 本仓现状 | 成本 |
|---|---|---|---|
| ⭐ **高程/爬升约束寻路**（`climb_ok`） | §1.4 | `nt_astar`/`nt_flow` 只有二维 passable 布尔图 | 中（需加高程维度） |
| **稳定序遍历**（确定性消费边界） | §2.4 | 本仓多处按 `HashMap` 序遍历 | 中（需逐处排） |
| **`Option<Result<T,E>>` 双层错误** | §6.1 | 本仓多为单层 `Result` | 低 |
| **世代戳复用**（省每次整图分配） | §1.5 | 本仓每次寻路都分配 | 低 |

---

## 8. 三条方法论（本轮真实付出代价换来的）

1. **消费者审计优先于实现。** 4 个候选 3 个被撤（§5）。
   「这个资产有人用吗」比「怎么优化它」重要一个数量级。
2. **手推 ≠ 实证，而且会连错两次。** 一次是「不可采纳启发」——
   按推理动手前测试是绿的，靠**穷举对照实验**才拿下；
   一次是 §6.2 的编造类型名。
3. **陈旧记录比没有记录更危险。** 我自己的函数 doc 里留过一行
   「`cat data | python3 -c` 命中 ⇒ 误报」，而实测已是不命中 ——
   独立复验的 agent 直接把它标为「与实现矛盾」，并要求修正。
   同一条纪律（`R-SCAN-3`）在门、在文档、在注释上完全通用。
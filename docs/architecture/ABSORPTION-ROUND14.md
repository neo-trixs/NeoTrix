# ABSORPTION-ROUND14 — 全网技术扫荡：RLTK/bracket + refs 深挖（2026-09-23）

> 承接 GAME-SKILLS 八项。本轮：以引擎缺口（FOV/流场/快照/洗牌/击退/掉落）为靶，
> 全网定位 + 移植 + 生产接线。源：`amethyst/bracket-lib`（1.7k★ MIT；
> bracket-pathfinding/dijkstra+astar+FOV 双算法，algorithm-traits），
> `nsmryan/shadowcasting`（Albert Ford 对称算法 CC0），libtcod，
> gamedev-skills refs（generation-fov-loot / effect-resolution / feedback-recipes）。

## 一、移植（零新依赖，全部单测锁定）

| 模块 | 源与改动 | 测试 |
|---|---|---|
| `spire/nt_fov.rs`：`compute_fov` | bracket 对称阴影移植；Rational32→f64（半径≤64 精度充足）；末端越界过滤（原版有，我初版漏→已补） | 5：半径形态/墙影/对称性全对（12×12 确定性迷宫两两互见）/零半径与界外安全 |
| `spire/nt_fov.rs`：`los_clear` | 连续空间 slab 线段-AABB（中心系 Sprite 由调用方转左上系）；中点严格内判定（擦角可见，天然对称） | 3：开/挡/墙后/嵌墙判遮挡（P12→Patrol）/擦角 |
| `spire/nt_flow.rs`：`FlowMap` | bracket DijkstraMap 精神；VecDeque 松弛→二叉堆真 Dijkstra（`total_cmp`，无均匀代价限制，原版 WARNING 位直接消除）；对角禁穿角；多源/截断/上下山 | 6：切比雪夫/绕行/多源最小/截断/下山必达/上山远离/穿角禁行/单格沼泽绕行（VecDeque 版在此会错） |
| `spire/nt_net.rs`：`SeqTracker` | G2 latest-wins：首包恒收，≤last 丢弃，玩家独立 | 1 |

## 二、生产接线（game 侧行为变更，仅一处）

- `judgment.rs`：`AiObservation.visible` + 追击/攻击门（盲则归零，reason 标"看不见（墙后）"）+ 2 新单测。
- `ai.rs` Observe：每帧收一次平台矩形（`query3<Position,Sprite,PlatformMarker>`，中心系转左上系），
  逐怪 `los_clear` → `visible`。零平台时恒真（行为与旧版逐位一致）；调用点唯一（main.rs:670）免签名改。
- 效果：怪不再隔墙索敌（此前纯距离制，卡墙才靠`blocked_time`跳跃脱困）；墙后怪回 Patrol。

## 三、审计结论（valid / 不碰，附理由）

- 洗牌：已是 Fisher-Yates（cards.rs:44）✓；掉落独立 roll = loot §4 推荐模式 ✓；
  效果 data+interpreter 已是 effect-resolution 形态 ✓；连通性 pass 已有 ✓——全部 valid，无改动。
- 击退：零存量；配方要求物理侧拥有稳定性 + 我方 Execute 逐帧覆写速度（脉冲活不过 1 帧）。
  不硬塞视觉层（违 juice 不动模拟 doctrine）。下一步：Execute 内击退速度通道（blend 非覆写）。
- 定步长主循环 / 碰撞 cap / 输入缓冲：沿用 GAME-SKILLS 结论（均已记录，不碰）。

## 四、验证

spire **71/71**（57+7+6+1）· game **148/148**（146+2）· cards check 过 · 新模块零警告。

## 五、分层重申

纯算法进 spire（nt_fov/nt_flow/SeqTracker，可发布复用），行为门进 game（judgment/ai）。
bracket 依赖**不引入**（移植优于依赖：体量、f64/堆替换、MIT/CC0 合规）。

# ABSORPTION-ROUND20 — 全网扫荡熔炼：元胞洞穴＋对象池＋Celeste 下降（2026-09-24）

> 源：bobeff/open-source-games（458 行，Godot 主导；Bevy fishfolk/Jumpy+Punchy；
> Veloren Rust 体量证）、cozy 主题（状态机/可交互组件/昼夜/集中物品数据）、
> macroquad 自带 `physics-platformer/`（476 行，Celeste 模型：像素余量/单轴碰撞/
> 跳穿 descent/骑乘挤压）、rot.js cellular（born 5-8/survive 4-8）、
> GPP 对象池章（全文：固定池＋空闲栈＋满池策略＋复用初始化纪律）。

## 一、底层逆向（ substrate 共识）

- 引擎分布：Godot 压倒性；Rust 侧 Bevy（fishfolk 系动作）+ Veloren（体素 MMO
  存在性证明）+ macroquad（我方栈，自动合批/跨端/内置 UI 音频）。
- 全 genre 六件套（沿用 ROUND19）：状态机/事件/命令/集中数据/确定性/消息日志。
- 平台物理正典（Maddy/Celeste 经 macroquad 实现确认）：像素余量步进、
  单轴分离碰撞、跳穿＋descent 旗、土狼/缓冲/可变跳高（我方 nt_move 已有）。

## 二、熔炼（只收有消费者的核心）

| # | 融入 | 公理源 | 验证 |
|---|---|---|---|
| 1 | `dungeongen::generate_caves`（随机撒点→5 轮平滑→最大域保留→边界墙→最远楼梯＋全岩兜底） | rot.js cellular | 2 单测（连通/边界/占比/确定性） |
| 2 | `particles` 对象池（固定 200 槽＋空闲栈＋spawn_raw 全量初始化＋满池丢弃） | GPP 对象池 | 3 单测（含零分配：容量恒定）；附带修 `interact_hint` 无界增长旧漏 |
| 3 | `nt_platform::drop`＋`MoveState::drop/dropping`（0.25s 窗）＋ swords S+K 接线 | Celeste descent | 引擎 1＋状态 1 单测；玩法实测位（SHOT 静态，主循环已通） |
| 4 | `state_stack` 补 4 单测（生命周期/替换/命令驱动/上下文） | shelfware 除名 | — |
| 5 | `spiregen`→`towergen`（TowerNodeKind/TOWER_×63）＋删僵尸注释 | 正名收尾 | abilities 全绿 |

## 三、诚实未收（有理由）

- **角修正**：需侧向实心碰撞 substrate，我方单向顶无此结构；不建无消费者抽象。
- **像素余量步进**：float 世界 + dt 钳制 + 下落限速已覆盖穿隧类；tile 引擎才需。
- **回滚**：nt_net 现 rooms/序列/环回；真回滚需全仿真确定性（gen_range 未播种），
  史诗级，记 doctrine 不动手。
- **快照插值/空间哈希/WFC/egui**：无症状或无消费者，不碰。

## 四、验证

game **53**（40＋平台4＋状态栈4＋池3＋drop1…实计 40+13）· abilities **83** ·
swords **10**，全绿零警告；审计脚本 2/2；SHOT=play 现为故事开场（by design），
覆层渲染确认；title 字节一致。

# ABSORPTION-ROUND19 — 融合重构＋金庸叙事吸收（2026-09-24）

> 源：bobeff/open-source-games（458 行清单，Godot 主导；Bevy fishfolk/Jumpy+Punchy；
> Veloren Rust 体量证）、cozy 主题（状态机/可交互组件/昼夜/集中物品数据）、
> jynew/jynew.wiki（8.9k★ 武侠框架文档全套）、Stranne  layered 架构（正对我方三件套）。

## 一、底层逆向（共性基座）

全 genre 收敛六件套：状态机＋事件总线＋命令对象＋集中数据＋确定性＋消息日志；
cozy 另有昼夜/季节/关系（非我体裁，取模式不取系统）。

## 二、熔炼（冗余/缺陷/错位 → 执行）

| # | 靶点 | 判定 | 执行 |
|---|---|---|---|
| 1 | swords 手写物理 vs 引擎 ecs_systems | 真冗余（顶面单向只此一家） | 提炼 `nt_platform`（move_on_platforms＋overlap＋4 单测），swords 消费，本地实现删除 |
| 2 | `foes.clone()`×5 每帧分配 | 真缺陷（§6.5 零分配） | 索引直读，行为逐位一致 |
| 3 | state_stack 零测试 shelfware | 缺陷（无 single consumer 不敢删） | 补生命周期/替换/命令/上下文 4 单测，定位引擎 API |
| 4 | `spiregen` 名＋`GameWorld::spire` 僵尸注释 | 正名残留 | 改 `towergen`（TowerNodeKind/TOWER_×63，API 面已是 Tower 系）；删僵尸注释 |
| 5 | 其它候选（消息 ticker/knockback/快照插值/存量 StS 表） | 经审计：无症状或无消费者 | 不碰，记账 |

## 三、金庸吸收（jynew wiki → swords M2 叙事）

- 事件＝编号脚本＋五族命令（显示/流程/数值/场景/音乐）；触发器三槽＋ModifyEvent
  自改线；存档＝SaveablePojo 递归＋KeyValues 开关（`chest.<guid>`）；技能＝excel
  配置＋编辑器预览＋Play 实时同步。
- 落子 `story.rs`（Say/Choice/Banner→已砍/Do/End＋Effect 四种＋越界即终）：
  wave-1 开场三行＋wave-4 破庙抉择（打坐回血回氣 vs 夜练下波加精锐），
  选项改写后续波次＝ModifyEvent 精神；终幕回战斗；Esc 不打断。
- 字体 delta 6 字（嗯宿庙歇烽秉）合并；覆层 SHOT 目检通过。

## 四、验证

game **48**（40＋平台4＋状态栈4）· abilities **81** · swords **10**，
全绿零警告；swords 审计脚本 title/play 通过；SHOT 故事覆层确认。
（注：一次 Metal OOM abort，重试即过，环境抖动档已立。）

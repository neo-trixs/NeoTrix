# ABSORPTION-ROUND15 — 寻路完备 + 相机死区 + 可重绑输入（2026-09-23）

> 承接 ROUND14（FOV/流场/视野 AI）。本轮靶点：A*（单对单最优）、相机死区
> （贴脸抖动）、键位重绑表（§10.3）、对话死端校验、存档/对话审计。
> 源：Stanford Amit Patel + RedBlob（A*/流场决策规则）、Phaser/Celeste 相机
> canon（死区 25%+前视 8–15%）、input-systems skill、dialogue-systems skill。

## 一、生产接线

| # | 产物 | 测试 |
|---|---|---|
| 1 | `spire/nt_flow.rs::astar`：二叉堆 A*，8 向+禁穿角（与流场同规则），octile 启发（min 步代价 1.0 前提已文档），`max_expand` 熔断，含起终点序列 | 5：对角最优/起终同点/绕墙/围死 None/沼泽绕行+预算熔断 |
| 2 | `game/nt_camera.rs`：`Deadzone::platformer()`（半宽 140≈22%）+ `follow`（盒内静止，出盒推至盒边再 lerp，系数 5.0 与旧手感连续） | 3：盒内静止/单轴推动/收敛至盒边+dt=0 不动 |
| 3 | 主循环跟随改死区（main.rs:1162，3 行替换） | — |
| 4 | `input.rs::BindingTable`：默认表单源（与旧硬编码逐键一致）+ rebind（跨动作冲突 Err）+ export/import（排序稳定文件）+ 26 键显式名映射；`update()` 改表驱动；启动加载 `bindings.ron`（缺失/非法即默认） | 3：默认一致+自洽/重绑冲突未知空键/导出导入往返+坏输入 |
| 5 | `dialogue.rs::validate()`：悬空跳转点名 + `init_dialogue` 装配后跑一遍（有效内容零输出） | 2：悬空检出/干净通过 |

验证：spire **76/76** · game **156/156**（148+8）· cards check 过 · 新模块零 dead_code
（authoring API 按 tween.rs 惯例 allow+理由；重绑 UI 为下一步）。

## 二、寻路选型（Stanford/RedBlob 规则落地）

- 单对单 → `astar`（启发+早停）；多对一 → `build_flow` 一次算完 + `descend` 查表。
- 我方怪群：视野 AI（ROUND14 LOS 门）+ 流场下山（将来时）+ A*（单体精确，如首领走位）。
- 对角一律禁穿角（流场与 A* 同规则，无穿墙斜行）。

## 三、审计（valid / 不碰）

- 任务双执行：`NotStarted` 状态机 guard 已防 ✓；profile 元进度：无存量（roguelite meta 未来项，不动存档结构）。
- 重绑 UI（捕获流+冲突显示+落盘写回）：PauseState 无菜单渲染体，超尾预算 → API 已备，记下一步。
- 前视 lookahead：死区之后第二步，记下一步（Celeste 量级 8–15%）。
- 定步长主循环/碰撞 cap：沿用前轮结论。

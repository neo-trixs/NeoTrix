# ABSORPTION-ROUND22：引擎 Behavior 层（2026-09-24）

> Explanation 象限：外部三源 → 本轮引擎增量。来源：Dreamlab-engine（WorldQL）、
> Godot AI 插件生态（BehaviorMachine/LimboAI/BeeHave/Utility）、
> PhlloriLab JobBoard（优先级 agent 执行）＋ anysearch 趋势面（GDC 2026：36% 用 genAI；
> NVIDIA AI NPC；生成式 NPC＝开发者输入＋模型）。

## 吸收结论（只取可落地的）

| 外部做法 | 映射到 neotrix-game | 本轮落地 |
|----------|---------------------|----------|
| Dreamlab Unity 式 Behavior 脚本 | 实体＋可挂载逻辑块，终结主循环手写逻辑 | ✅ `nt_behavior.rs`（trait＋注册表＋定序 tick，6 单测） |
| Dreamlab networked-behavior（属性几行即同步） | `nt_net` 只有房间/环回，缺属性复制 | ⏳ M3：`Replicated<T>`＋脏标记同步（已记 ROADMAP 候选） |
| Godot LimboAI/BehaviorMachine（BT＋状态机＋事件驱动） | `nt_utility` 只有打分，缺树结构 | ⏳ M2-A：敌脑用 Behavior 承载，BT 按需另起（Behavior 内可嵌状态机） |
| PhlloriLab 优先级 agent（0 Critical 先取） | 进程已有 TODO 优先级；引擎无对应物 | ✅ 思想验证：不教条映射，记此一条 |
| PhlloriLab local-first（数据不出文件夹） | 与本仓"不出仓"令同构 | ✅ 验证既有规则正确 |

## 本轮交付
- `crates/neotrix-game/src/nt_behavior.rs`（~200 行＋6 测试）：attach/update/detach、
  同名替换、开关、死亡跳过、组件读写；tick 快照防借用冲突（R10）。
- `main.rs` demo：手写往返 14 行 → `PingPong` Behavior＋`tick` 1 行（R-P79 同会话接线）。
- 验证：engine **59/59**（53＋6），`--all-targets` 零警告；本地 target（新规矩）。

## 未取（有意）
- 可视化编辑器/单人导出/一键部署（Dreamlab）：M4 事，不过早铺。
- 生成式 NPC 对话： guixu 叙事走 jynew 脚本已够，模型侧不碰。

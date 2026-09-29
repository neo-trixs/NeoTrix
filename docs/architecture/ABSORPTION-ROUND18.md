# ABSORPTION-ROUND18 — 正名 abilities + 能力边界（2026-09-23）

## 一、正名：`neotrix-spire` → `neotrix-abilities`

- 理由："尖塔"名随 StS 而起，现内容早超卡牌（调参/联机/FOV/流场/决策），
  且像素游戏已死、StS 标签无产品承载。`abilities` 贴合"引擎+能力"定位。
- 执行：`mv` 目录 + Cargo 名/描述 + workspace 成员 + 依赖方 2 处 Cargo +
  代码引用 4 处（nt_juice×3、swords main×1）。历史文档保留旧名（不改写历史）。
- 验证：abilities 81 · game 40 · swords 7 全绿；swords 重构建 + SHOT 正常。

## 二、能力边界（lib.rs 头文档固化，两层）

- **数据层（StS 系）**：schema/query/spiregen/events_parse/rich/powers_math/store。
- **算法层（通用）**：dungeongen/nt_tuning/nt_net/nt_fov/nt_flow/**nt_utility**。
- 约定：纯函数优先，RNG 调用方种子传入，无 wall-clock，无渲染类型。

## 三、补齐：`nt_utility`（judgment 模式提炼，去游戏类型）

- 动因：旧 judgment（效用打分+门禁+迟滞+fail-open）随游戏代码被删，
  但模式是通用 AI 决策件；M2 敌 AI 直接可用。
- 内容：`Scored<M>`（打分/门禁）+ `hysteresis`（防抖）+ `decide`（过滤非有限/
  负分取最高，空则回退），5 单测（含 NaN/Inf/全禁回退）。
- 验证：abilities 76→81 全绿。

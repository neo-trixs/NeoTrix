/**
 * 跑轮↔任务卡归属判据 — 纯函数，零 DOM / 零 IPC（故可进 `selftest.ts` 直跑）。
 *
 * ## 为什么需要它
 *
 * 桌面聊天里发一条斜杠指令（`/help` `/stop` `/status` `/new`）**不进跑轮**：
 * Rust 侧 `neobot_run_stream` 在跑轮**之前**就把它本地回执掉了
 * （`apps/neobot-desktop/src/nt_commands/nt_cmd_run.rs` 的
 * `intercept_local_command`），只往 `Channel` 上发一条 `Delta`（整段回执）
 * 加一条 `Done`，**一个任务都不建**。
 *
 * 而流式收尾那段曾**无条件**取 `tasks[0]` —— 在队内会话里，库里那条最新
 * 任务是**上一轮真实对话**的，于是它被挂到这条本地指令气泡下面。用户看到的是
 * 「我打了个 `/help`，下面挂着一条毫不相关的任务」。
 *
 * ## 判据为什么是语义的
 *
 * 「这条消息背后有没有跑轮」只能问跑轮自己，答在 `status` 里：
 * 真实跑轮的 status 恒为 `TurnStatus` 的五个串之一（`nt_types.rs:37`），而
 * 本地指令那条路回的是**专用**常量 `LOCAL_COMMAND_STATUS = "本地指令"`
 * （`nt_cmd_run.rs` 的 `LOCAL_COMMAND_STATUS`，注释明写「不进跑轮 ⇒ 这一轮**没有** TurnStatus」）。
 * 所以判据 = `status` 是不是 `TurnStatus`，**不是**「任务列表此刻恰好空不空」——
 * 后者只在空列表时碰巧不挂，列表非空时照样挂错。
 *
 * 未知 status 一律 fail-closed（不挂）。方向是刻意选的：多挂错一条卡，
 * 比少挂一条卡更伤 —— 后者是「没显示过程件」，前者是**编造了归属**。
 * 口径与 Rust 的 `TurnStatus::parse`（「未知返回 None，调用方按 fail-closed
 * 处理」，`nt_types.rs:47`）同向。
 *
 * ## `tasks[0]` 那个假设：核实结论
 *
 * `tasks[0]` = 全库最近 50 条里**最新**的那条（`nt_cmd_tasks.rs:10` 的
 * `list_tasks(50)`，SQL 是 `ORDER BY created_at DESC` 且**不带会话过滤**，
 * `nt_store_routines.rs:264`）。它跟「本轮这条任务」没有必然关系：队内多人
 * 并行、后台例程任务、别的窗口开跑，都会让它指到别人的任务。
 *
 * ## 后端现在直接给本轮的真 id（2026-09-28 切片 C3）
 *
 * `NeobotRunResult` 多了 `task_id`，由 Rust 侧**认领本轮那一个**任务
 * （`nt_cmd_run.rs` 的 `claim_turn_task_id`：跑轮前后各读一次 id 集做集合差，
 * 再按「恰好一个 + 归属本会话」收窄，认不出就交**空串**）。
 * 于是本模块的判据是**两级**：
 *
 * 1. **优先**用后端给的真 id —— 它是本轮任务的第一手证据，比任何间接判据
 *    都强（后端就在 `run_local_turn` 那一侧，看得见自己建了谁）；
 * 2. **只在它缺失时**（`undefined` / `null` / 空串）才回落到下面那段间接判据
 *    （窗口内新建 + 归属本会话）—— 那条路**保留**是为了后端认不出时（并发的
 *    别的跑轮/例程让「恰好一个」不成立）仍能挂对，而不是一律不挂。
 *
 * 拿到真 id 时**还要核一遍归属**：`after` 读得到就核到那条任务本身
 * （核不到 / 归属对不上 ⇒ 不挂）。这不是多余的谨慎 —— 「后端说这是本轮的」
 * 正是本模块从前端猜不到的断言，万一后端口径坏了，挂一张别人的任务卡
 * 比不挂更伤（那等于**编造了归属**）。`after` 读不到（null）时以真 id 为准。 *
 * 残余风险（不隐瞒）：**间接判据那条路**（第 2 级）仍有它原先那份风险 ——
 * 同一瞬间别的跑轮在本会话也建了任务时，取的是本会话里最新那个。桌面
 * 单窗口 + `onSend` 的 `shell.running` 闸使这条路实际不可达。
 */

/**
 * 跑轮终态白名单 —— 与 Rust `TurnStatus::as_str()`（`nt_types.rs:37`）逐字对齐。
 * **闭集**：后端新增状态时这里会 fail-closed（不挂卡），这是刻意的失败方向。
 */
export const RUN_TURN_STATUSES: readonly string[] = [
  "done",
  "continue",
  "needs_clarification",
  "blocked",
  "waiting",
];

/**
 * 本地指令那一条路的专用 status，对应 Rust `LOCAL_COMMAND_STATUS`
 * （`nt_cmd_run.rs` 的 `LOCAL_COMMAND_STATUS`）。导出是为了让 `selftest.ts` 能把
 * 「它**不是**跑轮终态」这条契约钉死。
 */
export const LOCAL_COMMAND_STATUS = "本地指令";

/** 任务在判据里用到的最小结构（`TaskItem` 天然满足；本模块不 import `core.ts`，故 selftest 直跑无 DOM 依赖）。 */
export interface TurnTaskRef {
  readonly id: string;
  readonly conversation_id?: string | null;
}

/**
 * 归一后端给的 `task_id`：**空串与非串一律当缺失**（fail-closed 的方向是
 * 「没有真 id 就走间接判据」，不是「拿一个怪串去查表」）。
 */
function realTaskId(raw: unknown): string {
  return typeof raw === "string" ? raw.trim() : "";
}

/**
 * 本轮是否**真跑了轮**（= 这条消息背后有没有一个跑轮）。
 *
 * 纯语义判据：只看跑轮自报的 `status`，不碰任何列表长度。
 */
export function turnRanEngine(status: string): boolean {
  const s = typeof status === "string" ? status.trim() : "";
  if (s === LOCAL_COMMAND_STATUS) return false;
  return RUN_TURN_STATUSES.includes(s);
}

/**
 * 本轮该挂哪张任务卡 —— 判据的单一入口。
 *
 * 三道语义闸，全过才返回任务：
 * 1. `status` 是跑轮终态（本地指令 / 未知状态 → 不挂）；
 * 2. 有真 id 就用它（并核归属），没有才问「起点快照读到了没有」
 *    （`before` 为 null ⇒ 不知道本轮新建了谁 ⇒ 不猜）；
 * 3. 没有真 id 时：跑轮窗口内**新建**且 `conversation_id` 就是本会话
 *    （存量旧任务一律不算）。
 *
 * @param status 跑轮/回执自报的 status（`asRunResult` 归一后的值）。
 * @param convo  本轮所属会话 id（`runStreamTurn` 的 `convo`）。
 * @param before 跑轮**之前**的任务 id 集；读不到就传 null（fail-closed）。
 * @param after  跑轮**之后**的任务列表（`ORDER BY created_at DESC`）；没读到传 null。
 * @param backendTaskId 后端给的**本轮真 id**（`NeobotRunResult.task_id`）。
 *   **可选**是为了老调用点（`selftest.ts` 里那 10 处 4 参调用）语义不变：
 *   不传 = 走第 2 级间接判据，传了空串同「没有」。
 * @returns 该挂的任务；不该挂时 `undefined`（调用方据此不上卡）。
 */
export function turnTaskToAttach(
  status: string,
  convo: string,
  before: ReadonlySet<string> | null,
  after: readonly TurnTaskRef[] | null,
  backendTaskId?: string | null,
): TurnTaskRef | undefined {
  if (!turnRanEngine(status)) return undefined;
  // 第 1 级：后端给的真 id（优先于一切间接证据）。
  const real = realTaskId(backendTaskId);
  if (real) {
    if (after === null) return { id: real };
    const hit = after.find((t) => t.id === real);
    if (!hit) return undefined;
    if ((hit.conversation_id ?? "") !== convo) return undefined;
    return hit;
  }
  if (before === null || after === null) return undefined;
  // 第 2 级：窗口内新建的候选（每个真实跑轮恰好一个，见文件头）。
  const fresh = after.filter((t) => !before.has(t.id));
  if (fresh.length === 0) return undefined;
  // 收敛到本会话：别人会话 / 无归属的任务（后台例程、别的窗口）一律不认。
  // 认不出就不挂 —— 不退回「取最新的那条」。
  const mine = fresh.filter((t) => (t.conversation_id ?? "") === convo);
  return mine.length > 0 ? mine[0] : undefined;
}

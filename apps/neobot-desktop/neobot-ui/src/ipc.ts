/**
 * 自持 UI 的 **IPC 单一出口** + 活动记录。
 *
 * # 设计来源（吸收，非照搬）
 *
 * `bytedance/UI-TARS-desktop`（39,137★，Apache-2.0）的两条设计被采纳：
 *   · **Event Stream Viewer**（v0.3.0 特性：数据流追踪与调试）
 *   · **timing statistics for tool calls**（工具调用耗时统计）
 *
 * ⛔ **不照搬它的「Event Stream」本身**：那是协议驱动的 agent 事件流，
 *    而我们的 `neobot_send` 返回单值 `AgentRunResult` ——
 *    **后端没有可流的东西**，引入事件协议是对不存在能力的投机实现。
 *
 * ## 改为记录「我们自己发出的 IPC」
 *
 * 可行且有真实价值的等价物：自持 UI 的每一次 `invoke` 都经本模块，
 * 记录 **命令名 / 耗时 / 成功失败 / 错误摘要**，在日志面板里可查。
 *
 * ## 这补的是一个真缺陷
 *
 * 改之前：命令失败时，界面只显示一句「读取失败」，用户既不知道
 * **是哪条命令**失败，也不知道**它耗时多久**（是超时还是立刻失败）。
 * 排查只能去翻 `read_run_logs` 的文本 blob —— 而那条日志
 * **不一定包含前端 IPC 的失败**（后端没被调到就不会记）。
 *
 * ## 为什么必须走单一出口
 *
 * 各组件原本各自 `import { invoke } from '@tauri-apps/api/core'`，
 * 无处可插桩。单一出口让「记录」成为**默认行为**而非可选纪律 ——
 * 这与本仓反复吃过的教训一致：不靠「记得写」，靠**结构**。
 *
 * ## ⛔ 已知缺口（**不是**我漏了，是不能动）
 *
 * `src/pet/pet.tsx`（桌宠页）仍直连 `@tauri-apps/api/core` ——
 * 该文件属**另一窗口**的在途/已提交工作，按共享工作树纪律我**不擅自改**。
 * ⇒ 后果：桌宠的 IPC 调用**不进入本活动面板**（桌宠自己的失败仍由它自己呈现）。
 * ⇒ 收敛方式：由桌宠功能作者把该文件改为
 *   `import { invokeCmd as invoke } from '../ipc'`（一行）。
 *   `nt_neobot_ui_wiring.py` 的单一出口检查会把这条列出来。
 */
import { invoke as rawInvoke } from '@tauri-apps/api/core'

export interface ActivityEntry {
  /** 命令名，如 `neobot_convo_list` */
  cmd: string
  /** 耗时（毫秒，向上取整；<1ms 记 0） */
  ms: number
  ok: boolean
  /** 失败时的错误摘要（截断，避免把整个堆栈塞进面板） */
  error?: string
  /** 单调时钟时间戳（ms），用于排序与相对时间显示 */
  at: number
}

/** 环形缓冲上限。刻意小：这是**诊断**面板，不是日志归档。
 *  写大了会在面板里刷屏，反而看不见刚才那次失败。 */
const CAPACITY = 60

let ring: ActivityEntry[] = []
const listeners = new Set<(e: ActivityEntry) => void>()

/** 记录一条活动（内部用；导出仅为便于测试注入）。 */
export function record(e: ActivityEntry): void {
  ring.push(e)
  if (ring.length > CAPACITY) ring = ring.slice(-CAPACITY)
  for (const fn of listeners) fn(e)
}

/** 取当前活动记录（**副本**，防调用方改到内部状态）。 */
export function getActivity(): ActivityEntry[] {
  return ring.slice()
}

/** 清空（面板上的「清空」按钮用）。 */
export function clearActivity(): void {
  ring = []
}

/** 订阅单条活动（返回取消订阅函数）。 */
export function onActivity(fn: (e: ActivityEntry) => void): () => void {
  listeners.add(fn)
  return () => listeners.delete(fn)
}

/**
 * 仪表化的 `invoke`。
 *
 * ⛔ **绝不吞错误**：失败照样 reject，让调用方的 `catch` 继续按原逻辑处理
 *    （如 `setLang` 的回滚、`neobot-root` 的错误态）。仪表化是**旁路**，
 *    不能改变控制流 —— 否则「记录失败」会把「失败」变成「成功」。
 */
export async function invokeCmd<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const started = performance.now()
  try {
    const out = (await rawInvoke(cmd, args)) as T
    record({ cmd, ms: Math.round(performance.now() - started), ok: true, at: Date.now() })
    return out
  } catch (e) {
    record({
      cmd,
      ms: Math.round(performance.now() - started),
      ok: false,
      error: String(e).slice(0, 120),
      at: Date.now(),
    })
    throw e // ⛔ 旁路不改控制流
  }
}

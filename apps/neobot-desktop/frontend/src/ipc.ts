/**
 * IPC 类型表 —— 前后端契约的**唯一**前端副本。
 *
 * # 为什么不直接 `call<string>("neobot_evidence_summary")`
 *
 * ⛔ 原来的 `call<T>(cmd, args)` 用 `as T` 做无校验断言，**任何返回类型都合法**。
 *    于是把 `EvidenceReport`（对象）当 `string` 用，tsc 一声不吭，
 *    运行时炸在 `r.value.includes is not a function`。
 *    这不是「类型标注写错了」，是**断言代替了检查** ——
 *    而断言恰恰在最需要它工作的地方（跨语言边界）不工作。
 *
 * 现在返回类型由这张表**推导**：`invoke(cmd, args)` 的 `ret` 来自
 * `Commands[K]["ret"]`。后端改了形状而前端没跟上 ⇒ **编译错误**，
 * 不用等到点按钮才炸。
 *
 * # 维护约定
 *
 * 改后端命令的签名/返回结构时，**这张表必须同步**。
 * 表与 Rust 不同步的那一刻，`cargo check` 不会报、`tsc` 也不会报
 * （两边各自自洽），只有运行时才炸 —— 这类漂移只能靠人守。
 * 故在每条上标注 Rust 侧位置，见下方 `// @rust`。
 */

/** Rust: nt_core::AgentRunResult.trace 元素（TraceRow） */
export interface TraceRow {
  kind: string;
  detail: string;
  failed?: boolean;
}

/** Rust: nt_core::AgentRunResult.usage（Option<TokenUsage>） */
export interface TokenUsage {
  input_tokens: number;
  output_tokens: number;
  cost_usd: number;
  context_window?: number;
}

/** Rust: nt_core::AgentRunResult */
export interface AgentRunResult {
  status: string;
  output: string;
  trace: TraceRow[];
  model_used: string;
  /** 路由三态：direct / passthrough / fallback */
  mode: string;
  tools: string[];
  usage: TokenUsage | null;
}

/** Rust: nt_evidence::FindingKind（serde rename_all = "snake_case"） */
export type FindingKind = "unsourced" | "overclaim" | "mismatch";

/** Rust: nt_evidence::Finding */
export interface Finding {
  kind: FindingKind;
  excerpt: string;
}

/**
 * Rust: nt_evidence::EvidenceReport
 *
 * `sourced_ratio` 是 `Option<f64>` ⇒ JS 侧可能是 `number | null`。
 * ⛔ 0 断言时后端给 **null 而不是 0**：「没断言过」与「断言全无出处」
 *    是不同的两句话，前者不能显示成「0% 有出处」。
 */
export interface EvidenceReport {
  clean: boolean;
  findings: Finding[];
  summary: string;
  sourced_ratio: number | null;
}

/** Rust: main.rs::CapabilitySnapshot */
export interface CapabilitySnapshot {
  crystal_version: string;
  tool_count: number;
  model: string;
  model_source: string;
}

/** Rust: nt_panel::Answer（serde 默认字段名） */
export interface Answer {
  panel_id: string;
  option_id: string;
  /** 界面**看到**的候选集版本。骨架拿它和自己当前的比，对不上即过期。 */
  candidate_set_version: number;
}

/** Rust: nt_panel::Panel —— 跨 IPC 用 snake_case（库类型未开 camelCase）。 */
export interface DecisionPanelWire {
  id: string;
  thread_id: string;
  turn_id: string;
  candidate_set_version: number;
  kind: "clarification" | "comparison";
  title: string;
  options: { id: string; label: string; details?: string[]; sources: { title: string; url: string }[] }[];
  mode?: "sample" | "live";
}

/**
 * Rust: nt_panel::AnswerOutcome
 *
 * ⛔ `accepted` 用小写 `accepted` 而不是「两个变体」：
 *    骨架侧 `Accepted`/`Rejected(Reject)` 经 serde 变成
 *    `{ "accepted": null }` / `{ "rejected": {...} }`（externally tagged）。
 *    界面**必须**按这个形状读，不能假设 `ok: boolean`。
 */
export type AnswerOutcomeWire =
  | { accepted: null }
  | { rejected: { no_such_panel: null } | { stale: { current: number; answered: number } }
      | { no_such_option: null } | { panel_invalid: string } };

/** 命令表：键=命令名，值={args, ret}。 */
export interface Commands {
  // @rust main.rs::neobot_agent_run
  neobot_agent_run: { args: { goal: string; context?: string | null }; ret: AgentRunResult };
  // @rust main.rs::neobot_send
  neobot_send: { args: { text: string }; ret: AgentRunResult };
  // @rust main.rs::neobot_convo_group
  neobot_convo_group: { args: { title: string; members: string[] }; ret: string };
  // @rust main.rs::neobot_convo_dm
  neobot_convo_dm: { args: { me: string; peer: string }; ret: string };
  // @rust main.rs::neobot_evidence_summary
  neobot_evidence_summary: { args: { text: string }; ret: EvidenceReport };
  // @rust main.rs::neobot_core_capabilities
  neobot_core_capabilities: { args: Record<string, never>; ret: CapabilitySnapshot };
  // @rust commands.rs::neobot_panel_answer
  //
  // ⚠️ 签名里**panel 一并传入**，不是服务端按 id 查 —— 当前骨架尚未维护
  //    面板注册表（面板随事件下发）。等它有真源后改签名，此处与 Rust 同步改。
  neobot_panel_answer: {
    args: { answer: Answer; panel: DecisionPanelWire };
    ret: AnswerOutcomeWire;
  };
}

export type Result<T> = { ok: true; value: T } | { ok: false; error: string };

/**
 * 唯一 IPC 出口。
 *
 * `K` 从参数 `cmd` 推导 ⇒ `ret` 也就跟着推导出来，调用方**不能**自己声明
 * 期望的返回类型。这是本文件存在的全部理由。
 */
export async function invoke<K extends keyof Commands>(
  cmd: K,
  args: Commands[K]["args"],
  tauri: boolean,
): Promise<Result<Commands[K]["ret"]>> {
  if (!tauri) {
    return { ok: false, error: `浏览器预览下无法调用 ${cmd}（没有 Tauri 运行时）` };
  }
  try {
    const core = await import("@tauri-apps/api/core");
    // 这里仍有一个 as，但它是**收窄**而非断言：core.invoke 返回 unknown，
    // 而 ret 已由上面的表推导出来。此处不做任何运行时校验 ——
    // 那属于「后端形状与表不一致」的情况，应在类型层与测试层各自兜住，
    // 而不是在每次调用时付一次检查的钱。
    return { ok: true, value: (await core.invoke(cmd, args)) as Commands[K]["ret"] };
  } catch (e) {
    return { ok: false, error: String(e instanceof Error ? e.message : e).slice(0, 200) };
  }
}

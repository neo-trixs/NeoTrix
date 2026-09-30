/**
 * 活动岛（dynamic island）状态模型 —— 纯逻辑，零 DOM。
 *
 * # 它替掉什么
 *
 * 原来 composer 边上一个静态 status 药丸，只有「空闲/运行中」两态。
 * 问题不是简陋，是**撒谎的余地太大**：没有「骨架在等我回答」这一态，
 * 于是骨架一旦发来决策请求，界面看上去和「空闲」一模一样 ——
 * 用户以为没人在干活，而实际上球在**他**那边。
 *
 * # 为什么门控
 *
 * 活动类型与能力挂钩（跑一轮要能对话；出决策面板要能渲染）。宿主不支持时
 * 不是「藏起来」，而是显式说「这个宿主做不了」—— 理由来自
 * `CapabilityRegistry.verdict().hint`，界面不自己编。
 *
 * # 纯逻辑/渲染分离
 *
 * 与 registry、block-model 同约定：逻辑在此以便 plain node 自测，DOM 在 island.ts。
 */

import type { Block, DecisionPanel } from "./block-model.ts";
import { hasPendingPanel } from "./block-model.ts";
import type { CapabilityRegistry, HostKind } from "../plugin/contract.ts";

/** 岛上的活动类型。每个类型声明它需要哪些能力。 */
export type ActivityKind = "run" | "decide" | "observe";

/**
 * 活动类型 → 所需能力。
 *
 * 刻意**按能力声明**而不是按平台：`run` 要的是「能对话」，
 * 不是「跑在 Tauri 上」。这样浏览器预览与桌面走同一套判定。
 */
export const ACTIVITY_REQUIRES: Readonly<Record<ActivityKind, readonly string[]>> = {
  run: ["chat"],
  decide: ["chat"],
  observe: ["chat"],
};

export type IslandState =
  | { readonly kind: "idle" }
  | { readonly kind: "running"; readonly label: string; readonly seq: number }
  /** 骨架在等用户决策。这是球在用户那边，必须比「运行中」更显眼。 */
  | { readonly kind: "awaiting"; readonly panel: DecisionPanel; readonly seq: number }
  | { readonly kind: "error"; readonly label: string; readonly detail?: string }
  /** 宿主缺能力。带可读理由，不假装能做。 */
  | { readonly kind: "unavailable"; readonly label: string; readonly reason: string };

/**
 * 该活动类型在当前宿主上是否可用；不可用时给出理由。
 *
 * 理由取自注册表而不是本文件编的 —— 否则「为什么这个没出现」会有两套答案。
 */
export function activityBlocked(
  kind: ActivityKind,
  caps: CapabilityRegistry,
  host: HostKind,
): string | null {
  for (const need of ACTIVITY_REQUIRES[kind]) {
    const v = caps.verdict(need);
    if (!v.visible) return v.hint ?? `缺少能力：${need}`;
  }
  return null;
}

/**
 * 由块流与忙碌标记推出岛的状态。
 *
 * 优先级是**刻意的**：awaiting > error > running > idle。
 * 理由：球在用户那边时显示「运行中」，会让用户以为系统还在算而继续等 ——
 * 这是最坏的一种错，因为用户**等不到**任何后续。
 * 同理，出错时不该显示「运行中」。
 */
export function deriveIsland(
  blocks: readonly Block[],
  opts: { caps: CapabilityRegistry; host: HostKind; busy: boolean; seq: number },
): IslandState {
  const { caps, host, busy, seq } = opts;

  // ① 决策请求优先：它在等**用户**，不是在等机器。
  const pending = hasPendingPanel(blocks);
  if (pending) {
    const blocked = activityBlocked("decide", caps, host);
    if (blocked) {
      return { kind: "unavailable", label: "等待你的选择", reason: blocked };
    }
    return { kind: "awaiting", panel: pending, seq };
  }

  // ② 最近的系统错误
  const lastErr = [...blocks].reverse().find(
    (b): b is Extract<Block, { kind: "system" }> =>
      b.kind === "system" && b.level === "error",
  );
  if (lastErr) {
    return { kind: "error", label: lastErr.text, detail: lastErr.detail };
  }

  // ③ 运行中
  if (busy) {
    const blocked = activityBlocked("run", caps, host);
    if (blocked) return { kind: "unavailable", label: "运行一轮", reason: blocked };
    return { kind: "running", label: "运行中", seq };
  }

  return { kind: "idle" };
}

/** 岛的一行文案。纯展示，逻辑集中在上面。 */
export function islandLabel(s: IslandState): string {
  switch (s.kind) {
    case "idle": return "空闲";
    case "running": return s.label;
    case "awaiting": return "等你选择";
    case "error": return s.label;
    case "unavailable": return `${s.label} · 不可用`;
  }
}

/**
 * 岛是否「需要注意」。
 *
 * 供 a11y 用：`awaiting` 与 `error` 需要播报（role=status / alert），
 * `running` 不该反复打断读屏 —— 那会每帧念一次。
 */
export function islandIsLive(s: IslandState): "alert" | "status" | "none" {
  if (s.kind === "error") return "alert";
  if (s.kind === "awaiting" || s.kind === "unavailable") return "status";
  return "none";
}

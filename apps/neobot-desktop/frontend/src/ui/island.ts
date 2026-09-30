/**
 * 活动岛 DOM 渲染 —— 替掉 composer 边的静态 status 药丸。
 *
 * 设计取自 openbot 的 `OpenBotDynamicIsland`（PolyForm Noncommercial，
 * **只取思路不取码**）与 macOS 的动态岛：常态是一颗小药丸，
 * 需要人管时（等决策/出错）才张开并变醒目。
 *
 * ⛔ 不用 innerHTML 拼任何模型或用户产出。全走 textContent。
 */

import type { IslandState } from "./island-model.ts";
import { islandLabel, islandIsLive } from "./island-model.ts";

/** 药丸里的小圆点随状态动。running 时用 CSS 动画，不用 JS 定时器。 */
function dotClass(s: IslandState): string {
  return `is-dot is-dot--${s.kind}`;
}

export function renderIsland(host: HTMLElement, s: IslandState): void {
  const live = islandIsLive(s);
  // 先清空再填：状态种类少、结构固定，整体替换比逐字段 diff 更不容易
  // 留下上一次状态的残留类。
  host.replaceChildren();
  host.className = `island is-island--${s.kind}`;
  // a11y：等决策与出错要播报，运行中不播（否则读屏每帧念一次）。
  if (live === "alert") host.setAttribute("role", "alert");
  else if (live === "status") host.setAttribute("role", "status");
  else host.removeAttribute("role");
  host.setAttribute("aria-live", live === "none" ? "off" : "polite");

  const dot = document.createElement("span");
  dot.className = dotClass(s);
  host.appendChild(dot);

  const label = document.createElement("span");
  label.className = "is-label";
  label.textContent = islandLabel(s);
  host.appendChild(label);

  if (s.kind === "awaiting") {
    const q = document.createElement("span");
    q.className = "is-count";
    q.textContent = `${s.panel.options.length} 个选项`;
    host.appendChild(q);
  }
  if (s.kind === "unavailable") {
    // 不可用必须**看得见理由**，否则就是一个不动的药丸，用户无从判断
    // 是没在跑还是坏了。
    const r = document.createElement("span");
    r.className = "is-reason";
    r.textContent = s.reason;
    r.title = s.reason;
    host.appendChild(r);
  }
}

/**
 * 基础控件层 —— 形态与取值对齐 dsh-tauri/deepseek-harness-desktop（MIT）。
 *
 * # 复用的是什么
 *
 * 从 `src/components/primitives.ts` + `src/components/panel.tsx` 取**架构与几何**，
 * 用本仓的原生 TS 重新实现（不引 React/Tailwind/HeroUI）。取的是：
 *
 *   ① **按钮是胶囊**：`sm = h28 / 圆角 14px`、`md = h36 / 圆角 18px`。
 *      ⛔ 这一条我前面做错过：整套 UI 的按钮都是 8/10px 圆角矩形，
 *      而「胶囊」是这套设计语言最显眼的特征 —— 它同时用在按钮和消息气泡上。
 *      两者一起用才成立；只把气泡改胶囊、按钮还是方角，会像两套设计拼的。
 *
 *   ② **主按钮是「与背景反差最大的那个」**，不是品牌色。
 *      浅色下 = 近黑底白字；深色下 = 近白底深字（反相）。
 *      这是极简里最反直觉、也最有效的一条：主操作不靠颜色吸引，靠对比。
 *
 *   ③ **三档色调**（primary / ghost / danger），没有 accent/secondary。
 *      少一档就少一次「这个按钮要不要强调」的判断。
 *
 *   ④ **Panel 三态**（加载中 / 失败 / 内容）收敛到一个函数里。
 *      我之前是「有数据就渲染，没数据显示空态」，
 *      而**读失败**和**没有数据**被混成同一个分支 ——
 *      于是「store 读不了」会显示成「还没有会话」，用户以为是自己没建过。
 *      三态分开是这个模式最大的价值。
 *
 * # 不复用的是什么
 *
 * 框架（React/Tailwind/HeroUI/i18next/valtio）、Rust 侧、以及他们 iframe 上游
 * Web UI 的做法。NeoBot 没有上游 Web UI 可嵌，对话区必须自己实现。
 * 取向明确：**取设计与交互架构，不取运行时**。
 */

import { el } from "./dom.ts";

/** 按钮两档几何。数值来自 primitives.ts，逐项对应。 */
export type BtnSize = "sm" | "md";
export type BtnTone = "primary" | "ghost" | "danger";

/**
 * 造一个胶囊按钮。
 *
 * ⚠️ `label` 允许传空串 —— 那时**必须**给 `ariaLabel`，否则键盘用户
 * 聚焦到它读屏只会念「按钮」。图标按钮没有文字，这是唯一的信息来源。
 */
export function btn(
  label: string,
  opts: {
    size?: BtnSize;
    tone?: BtnTone;
    onClick?: () => void;
    icon?: Node;
    title?: string;
    ariaLabel?: string;
    type?: "button" | "submit";
  } = {},
): HTMLButtonElement {
  const size = opts.size ?? "md";
  const tone = opts.tone ?? "ghost";
  const b = el("button", `nb-btn nb-btn--${size} nb-btn--${tone}`);
  b.type = opts.type ?? "button";
  if (opts.icon) b.appendChild(opts.icon);
  if (label) b.appendChild(document.createTextNode(label));
  const tip = opts.title ?? opts.ariaLabel ?? label;
  if (tip) {
    b.title = tip;
    b.setAttribute("aria-label", tip);
  }
  if (opts.onClick) b.addEventListener("click", opts.onClick);
  return b;
}

/**
 * 列表三态。**这是本文件最有价值的一处**。
 *
 * 上一版把「读失败」和「没有数据」合并，于是：
 *   · store 读不了  → 显示「还没有会话」
 *   · 用户以为是自己没建过 → 去建会话 → 还是同样的提示 → 无从排查
 * 而 tsc 过、build 过、界面看上去正常。**状态语义被压扁了。**
 *
 * 三态分开之后，「读失败」有自己的样子（错误色 + 原因 + 重试），
 * 这与本仓记了七次的「把两种状态合并成一种」是同一类问题的不同表现。
 */
export function threeState(opts: {
  loading: boolean;
  error: string;
  isEmpty: boolean;
  onRetry?: () => void;
  render: () => Node;
  emptyText: string;
  emptyHint?: string;
}): HTMLElement {
  const box = el("div", "nb-state");

  if (opts.loading) {
    // 加载中不写「暂无」—— 那是把「还没好」说成「没有」。
    const s = el("div", "nb-state-loading");
    s.appendChild(el("span", "nb-spinner"));
    s.appendChild(el("span", "nb-state-text", "加载中"));
    box.appendChild(s);
    return box;
  }

  if (opts.error) {
    const s = el("div", "nb-state nb-state--error");
    s.appendChild(el("div", "nb-state-title", "读取失败"));
    s.appendChild(el("div", "nb-state-text", opts.error));
    if (opts.onRetry) {
      s.appendChild(btn("重试", { size: "sm", tone: "ghost", onClick: opts.onRetry }));
    }
    box.appendChild(s);
    return box;
  }

  if (opts.isEmpty) {
    const s = el("div", "nb-state");
    s.appendChild(el("div", "nb-state-text", opts.emptyText));
    if (opts.emptyHint) s.appendChild(el("div", "nb-state-hint", opts.emptyHint));
    box.appendChild(s);
    return box;
  }

  box.appendChild(opts.render());
  return box;
}

/** 面板：标题 + 说明 + 右侧动作位。结构对齐 `Panel.Header`。 */
export function panelHead(
  title: string,
  opts: { description?: string; action?: Node } = {},
): HTMLElement {
  const h = el("div", "nb-panel-head");
  const row = el("div", "nb-panel-head-row");
  row.appendChild(el("span", "nb-panel-title", title));
  if (opts.action) row.appendChild(opts.action);
  h.appendChild(row);
  if (opts.description) h.appendChild(el("div", "nb-panel-desc", opts.description));
  return h;
}

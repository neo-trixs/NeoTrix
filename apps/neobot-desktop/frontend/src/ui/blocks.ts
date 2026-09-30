/**
 * 块渲染器 —— 把 Block 变成 DOM。
 *
 * # 逐块隔离（吸收 douchat ChatErrorBoundary 的思路，AGPL 仓只取思路不取码）
 *
 * douchat 的注释原话：「Keep a broken conversation from unmounting the sidebar
 * and the whole app」，并且渲染失败时明确告诉用户「聊天记录未被删除」。
 *
 * 我这里没有 React 的 ErrorBoundary，但等价的更强一层：
 * **每个块自带 try/catch**，且失败时用**原始数据**兜底渲染。
 *
 * 为什么逐块而不是整条线程一个 try：
 * 一条会话里若有 60 块，整条包一层的话任何一块炸了，**60 块全没**。
 * 逐块隔离的代价是每块一次 try（微秒级），换来的是「一块坏、其余照常」。
 *
 * ⛔ 绝不用 innerHTML 拼**用户或模型产出**。所有文本走 textContent。
 * 唯一用 innerHTML 的是图标（来自本仓静态表，非外部输入）。
 */

import type { Block, DecisionPanel } from "./block-model.ts";
import { validatePanel, summarizeTool } from "./block-model.ts";

function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  cls?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const n = document.createElement(tag);
  if (cls) n.className = cls;
  if (text !== undefined) n.textContent = text;
  return n;
}

function renderMark(b: Extract<Block, { kind: "mark" }>): HTMLElement {
  return el("div", "session-mark", b.text);
}

function renderText(b: Extract<Block, { kind: "text" }>): HTMLElement {
  const wrap = el("div", `msg msg--${b.role === "user" ? "user" : "bot"}`);
  wrap.appendChild(el("div", "msg-bubble", b.text));
  if (b.meta) wrap.appendChild(el("div", "msg-meta", b.meta));
  return wrap;
}

function renderTool(b: Extract<Block, { kind: "tool" }>): HTMLElement {
  // details/summary：原生折叠，无 JS 依赖，键盘可达
  const box = el("details", "nb-item tool-card");
  box.appendChild(el("summary", "tool-sum", summarizeTool(b.steps)));
  for (const s of b.steps) {
    const row = el("div", "nb-item-row");
    const left = el("div", "nb-item-left");
    left.appendChild(el("span", s.failed ? "nb-item-title tool-step--bad" : "nb-item-title", s.kind));
    row.appendChild(left);
    row.appendChild(el("span", "nb-item-sub", s.detail));
    box.appendChild(row);
  }
  return box;
}

function renderArtifact(b: Extract<Block, { kind: "artifact" }>): HTMLElement {
  const box = el("details", "nb-item artifact-card");
  const sum = el("summary", "tool-sum", b.title);
  box.appendChild(sum);
  const pre = el("pre", "artifact-body");
  // 产物也是模型产出 ⇒ 同样走 textContent
  pre.textContent = b.body;
  box.appendChild(pre);
  return box;
}

function renderSystem(b: Extract<Block, { kind: "system" }>): HTMLElement {
  const box = el("div", b.level === "error" ? "nb-error" : "nb-empty");
  box.setAttribute("role", b.level === "error" ? "alert" : "status");
  box.appendChild(el("div", "nb-item-title", b.text));
  // ⛔ 「数据没丢」必须是**明确写出的一行**，不是让用户自己推断。
  //    douchat 把它写进错误文案，理由是：不给这句话，用户第一反应是
  //    「我的记录是不是没了」，然后去导出/重装，那是更贵的后果。
  if (b.dataIntact) box.appendChild(el("div", "nb-item-sub", "记录未删除，内容仍在本机。"));
  if (b.detail) {
    const d = el("details", "sys-detail");
    d.appendChild(el("summary", "sys-detail-sum", "详情"));
    const pre = el("pre", "sys-detail-body", b.detail);
    d.appendChild(pre);
    box.appendChild(d);
  }
  return box;
}

function renderPanel(b: Extract<Block, { kind: "panel" }>): HTMLElement {
  const p = b.panel;
  const bad = validatePanel(p);
  if (bad) {
    // 骨架发了非法面板：**暴露问题**，不渲染一张看着能用的坏卡。
    const box = el("div", "nb-error");
    box.setAttribute("role", "alert");
    box.appendChild(el("div", "nb-item-title", "决策面板无效，未能显示"));
    box.appendChild(el("div", "nb-item-sub", bad));
    return box;
  }
  const box = el("div", "nb-item panel-card");
  if (p.mode === "sample") {
    // sample ≠ live。混为一谈就是把「演示」说成「真实」。
    const w = el("div", "panel-warn", "示例数据（非真实候选）");
    box.appendChild(w);
  }
  const head = el("div", "nb-item-row");
  head.appendChild(el("div", "nb-item-left", ));
  head.querySelector(".nb-item-left")!.appendChild(el("span", "nb-item-title", p.title));
  head.appendChild(el("span", "nb-item-sub", `v${p.candidateSetVersion}`));
  box.appendChild(head);

  for (const o of p.options) {
    const opt = el("label", "nb-opt");
    const input = el("input");
    input.type = "radio";
    input.name = `panel-${p.id}`;
    input.value = o.id;
    input.checked = o.id === p.selectedId;
    // 过期作答：候选集版本对不上时，高亮提示「选项已更新」
    const body = el("div", "opt-body");
    body.appendChild(el("div", "nb-opt-label", o.label));
    for (const d of o.details) body.appendChild(el("div", "nb-item-sub", d));
    if (o.sources.length > 0) {
      const srcs = el("div", "opt-srcs");
      for (const s of o.sources) {
        const a = el("a", "opt-src", s.title);
        a.href = s.url;
        a.rel = "noopener noreferrer";
        a.target = "_blank";
        srcs.appendChild(a);
      }
      body.appendChild(srcs);
    }
    opt.append(input, body);
    box.appendChild(opt);
  }
  return box;
}

const RENDERERS: { [K in Block["kind"]]: (b: Extract<Block, { kind: K }>) => HTMLElement } = {
  mark: renderMark,
  text: renderText,
  tool: renderTool,
  artifact: renderArtifact,
  system: renderSystem,
  panel: renderPanel,
};

/**
 * 渲染单个块，**永不抛出**。
 *
 * 失败时返回一个用原始文本兜底的系统块 —— 这样「渲染失败」本身
 * 也是一条可读的信息，而不是线程变空白。
 */
export function renderBlock(b: Block, index: number): HTMLElement {
  try {
    const node = RENDERERS[b.kind](b as never);
    node.dataset["bk"] = `${index}:${b.kind}`;
    return node;
  } catch (e) {
    const box = el("div", "nb-error");
    box.setAttribute("role", "alert");
    const raw = "text" in b ? b.text : JSON.stringify(b).slice(0, 300);
    box.appendChild(el("div", "nb-item-title", "这一块显示失败"));
    box.appendChild(el("div", "nb-item-sub", "记录未删除，内容仍在本机。"));
    const d = el("details", "sys-detail");
    d.appendChild(el("summary", "sys-detail-sum", "详情与原文"));
    d.appendChild(el("pre", "sys-detail-body", `${String(e)}\n\n${raw}`));
    box.appendChild(d);
    return box;
  }
}

/** 挂到线程末尾（追加，不整体重绘 ⇒ 滚动位置与折叠状态都保住）。 */
export function appendBlock(thread: HTMLElement, b: Block, index: number): void {
  thread.appendChild(renderBlock(b, index));
}

export type { DecisionPanel };

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

import { el } from "./dom.ts";
import type { Block, DecisionPanel } from "./block-model.ts";
import { validatePanel, summarizeTool } from "./block-model.ts";
import { renderPanelView, type AnswerView } from "./panel-view.ts";

/** 渲染上下文：只放「需要外部参与」的东西（目前只有面板作答）。 */
export interface BlockContext {
  onAnswer?: (a: AnswerView) => void;
}

function renderMark(b: Extract<Block, { kind: "mark" }>): HTMLElement {
  return el("div", "session-mark", b.text);
}

/**
 * 对话胶囊。
 *
 * 结构（自外向内）：行 → 头像 + 列 → 发送者名 + 气泡。
 *
 * ⚠️ 三个「不得」都是踩出来的：
 *   · 头像**不得**放进气泡 —— 短消息会被头像顶成窄柱（见 capsule.css 头注）
 *   · 发送者名**不得**放进气泡 —— 会把气泡撑成两行不齐的怪东西
 *   · 气泡**不得**带尾 —— 圆角矩形在长文本下会被读成「卡片」而非「一句话」
 */
function renderText(b: Extract<Block, { kind: "text" }>): HTMLElement {
  const me = b.role === "user";
  const wrap = el("div", `msg ${me ? "msg--me" : "msg--you"}`);

  const av = el("div", "msg-avatar", me ? "我" : "N");
  av.setAttribute("aria-hidden", "true");
  wrap.appendChild(av);

  const col = el("div", "msg-col");
  // 对方气泡上方有名字，我方没有 —— 这是「谁在说」的唯一线索
  if (!me) col.appendChild(el("div", "msg-who", b.role === "bot" ? "NeoBot" : b.role));
  col.appendChild(el("div", "msg-bubble", b.text));
  // meta 走气泡**下方**小字，不进气泡
  if (b.meta) col.appendChild(el("div", "msg-meta", b.meta));
  wrap.appendChild(col);
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
    // outputs 一律按「名字 + 值」渲染，不认识具体类型 —— 这样加第 20 种工具
    // 产出时**界面不用改**（依据 cua ComputerCallOutputMessage 的「always a
    // screenshot」：输出形状统一 ⇒ 渲染器只有一套）。
    const outputs: Readonly<Record<string, string>> = s.outputs ?? {};
    for (const [name, val] of Object.entries<string>(outputs)) {
      const out = el("div", "tool-out");
      out.appendChild(el("span", "tool-out-name", name));
      out.appendChild(el("span", "tool-out-val", val));
      box.appendChild(out);
    }
  }
  return box;
}

function renderReasoning(b: Extract<Block, { kind: "reasoning" }>): HTMLElement {
  // 默认收起：思考展开会把对话变成日志。但**内容完整保留**，可随时查 ——
  // 排障时「它当时为什么这么选」看不到，等于没有推理块。
  const box = el("details", "nb-item reason-card");
  const sum = el("summary", "tool-sum");
  sum.textContent = b.tokens === undefined ? "思考过程" : `思考过程（${b.tokens} tokens）`;
  box.appendChild(sum);
  const pre = el("pre", "reason-body");
  pre.textContent = b.text;
  box.appendChild(pre);
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

/**
 * 决策面板的静态入口。
 *
 * 无回调时（纯渲染路径）画成**只读**卡片：选项可见但不可选。
 * 有回调时交给 `panel-view.ts` 的交互版本。
 * ⛔ 绝不在这里另画一张卡 —— 面板有两个版本就会有两处「何时失效」的逻辑。
 */
function renderPanel(b: Extract<Block, { kind: "panel" }>, ctx?: BlockContext): HTMLElement {
  if (!ctx?.onAnswer) {
    const ro = renderPanelView(b.panel, { onAnswer: () => {} });
    ro.querySelectorAll("input").forEach((i) => { (i as HTMLInputElement).disabled = true; });
    ro.querySelectorAll("button").forEach((x) => { (x as HTMLButtonElement).disabled = true; });
    return ro;
  }
  return renderPanelView(b.panel, { onAnswer: ctx.onAnswer });
}

type R<T extends Block["kind"]> = (b: Extract<Block, { kind: T }>, ctx?: BlockContext) => HTMLElement;
const RENDERERS: { [K in Block["kind"]]: R<K> } = {
  mark: renderMark,
  text: renderText,
  tool: renderTool,
  reasoning: renderReasoning,
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
export function renderBlock(b: Block, index: number, ctx?: BlockContext): HTMLElement {
  try {
    const node = RENDERERS[b.kind](b as never, ctx);
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
export function appendBlock(thread: HTMLElement, b: Block, index: number, ctx?: BlockContext): void {
  thread.appendChild(renderBlock(b, index, ctx));
}

export type { DecisionPanel };

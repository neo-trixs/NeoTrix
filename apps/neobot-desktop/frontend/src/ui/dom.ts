/**
 * 最小 DOM 构造器。
 *
 * 之前它住在 `ui/blocks.ts` 里且**没有导出**，所以设置面板与 rail 想用就得
 * 各写一份 `document.createElement` + `className` + `textContent` ——
 * 那正是「同一个函数抄三遍，三处写法慢慢分叉」的起点。
 * ⇒ 抽成单点真源，两边都从这里取。
 *
 * ⛔ `text` 走 `textContent`，**不是** `innerHTML`。
 * 本项目的对话内容含用户输入与模型产出，`innerHTML` 拼字符串是 XSS 直门。
 * 需要富文本的场合（链接、代码高亮）必须逐节点构造，见 blocks.ts 的来源块。
 */
export function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  cls?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const n = document.createElement(tag);
  if (cls) n.className = cls;
  if (text !== undefined) n.textContent = text;
  return n;
}

/** 带 class 的 span。列表与设置面板里用得最多，单列一个免得每次写全。 */
export function span(cls?: string, text?: string): HTMLSpanElement {
  return el("span", cls, text);
}

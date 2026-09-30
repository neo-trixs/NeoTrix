/**
 * 最小 DOM 构造器。
 *
 * ⛔ `text` 一律走 `textContent`，**不是** `innerHTML`。
 *    本项目的内容含用户输入与模型产出，`innerHTML` 拼字符串是 XSS 直门。
 *    需要富文本（链接、代码高亮）必须逐节点构造。
 */

/** 建元素。第三个参数给数字时转字符串 —— 调用点写数字比写 `String(n)` 常见。 */
export function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  cls?: string,
  text?: string | number,
): HTMLElementTagNameMap[K] {
  const n = document.createElement(tag);
  if (cls) n.className = cls;
  if (text !== undefined) n.textContent = String(text);
  return n;
}

export function span(cls?: string, text?: string | number): HTMLSpanElement {
  return el("span", cls, text);
}

/**
 * 线性图标集 —— 24 网格 / 描边 1.7px / 圆头圆角 / 不填充。
 *
 * # 规格来源与理由
 *
 * 取自 dataelement/dsh-desktop `packages` 下的 client.js 的实测值：
 * `viewBox 0 0 24 24 · fill none · stroke currentColor · stroke-width 1.7
 *  · linecap round · linejoin round`。
 *
 * **1.7 而不是 Lucide 的 2**：在 13px 字号旁边，2px 描边会显得比文字还重，
 * 而整套界面是刻意做轻的。1.7 刚好「看得见但不抢」。
 *
 * **自绘而不是引 Lucide**：对照过 Lucide 的特征路径（齿轮 `M12.22 2h-.44`、
 * 放大镜 `m21 21-4.34-4.34`、对勾 `M20 6 9 17l-5-5`）——
 * dsh-desktop 全部 0 命中，是自己写的 24 网格路径。
 * 这里同样自绘：只要 8 个图标，引一个图标库不划算，且库的默认 2px 与本套不符。
 *
 * # 「极简」不等于「少画点东西」
 *
 * 极简是**一个图标只表达一件事**。所以：
 *   · 侧栏图标只表示「去哪一类东西」，不表示「进去能干什么」
 *   · 详情/证据这类动作不进文字按钮，改成图标 + tooltip
 *   · 图标旁**不留文字标签**（唯一例外是主操作「发送」——
 *     发送是**动作**不是**目的地**，图标化后没人敢按）
 *
 * 每个图标都带 `title` 与 `aria-label`：dsh-desktop 的 workbench-standard
 * 明确写「图标只表示归属，并提供可访问名称或提示」。图标不能只有图形。
 */

/** 24 网格图标的统一外壳。尺寸由调用处给，描边跟随文字色。 */
function svgIcon(paths: string[], size = 16): SVGSVGElement {
  const NS = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(NS, "svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("width", String(size));
  svg.setAttribute("height", String(size));
  svg.setAttribute("fill", "none");
  svg.setAttribute("stroke", "currentColor");
  svg.setAttribute("stroke-width", "1.7");
  svg.setAttribute("stroke-linecap", "round");
  svg.setAttribute("stroke-linejoin", "round");
  svg.setAttribute("aria-hidden", "true");
  for (const d of paths) {
    const p = document.createElementNS(NS, "path");
    p.setAttribute("d", d);
    svg.appendChild(p);
  }
  return svg;
}

/** 路径表。key 是语义名，不是形状名 —— 换画法不该改调用处。 */
const PATHS: Record<string, string[]> = {
  // 对话：圆角方框 + 尾巴
  chat: ["M4 5.5A1.5 1.5 0 0 1 5.5 4h13A1.5 1.5 0 0 1 20 5.5v9a1.5 1.5 0 0 1-1.5 1.5H9l-4 3.5V16"],
  // 联系人：人形
  contacts: ["M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8Z", "M4.5 20a7.5 7.5 0 0 1 15 0"],
  // 群组：多一人 + 括号
  group: [
    "M10 11a3.2 3.2 0 1 0 0-6.4 3.2 3.2 0 0 0 0 6.4Z",
    "M3.5 19a6.5 6.5 0 0 1 13 0",
    "M17 7.5a2.6 2.6 0 0 1 0 5",
    "M19 15.4a5 5 0 0 1 2 3.6",
  ],
  // 设置：齿轮（比 Lucide 少两个齿，更简）
  settings: [
    "M12 15.2a3.2 3.2 0 1 0 0-6.4 3.2 3.2 0 0 0 0 6.4Z",
    "M19.2 14.4a1.5 1.5 0 0 0 .3 1.65l.06.05a1.8 1.8 0 1 1-2.55 2.55l-.05-.06a1.5 1.5 0 0 0-1.65-.3 1.5 1.5 0 0 0-.9 1.37V20a1.8 1.8 0 1 1-3.6 0v-.1a1.5 1.5 0 0 0-.98-1.37 1.5 1.5 0 0 0-1.65.3l-.05.06A1.8 1.8 0 1 1 4.5 16.2l.06-.05a1.5 1.5 0 0 0 .3-1.65 1.5 1.5 0 0 0-1.37-.9H3.4a1.8 1.8 0 1 1 0-3.6h.1a1.5 1.5 0 0 0 1.37-.98 1.5 1.5 0 0 0-.3-1.65L4.5 7.3A1.8 1.8 0 1 1 7.05 4.75l.05.06a1.5 1.5 0 0 0 1.65.3h.07a1.5 1.5 0 0 0 .9-1.37V3.4a1.8 1.8 0 1 1 3.6 0v.1a1.5 1.5 0 0 0 .9 1.37 1.5 1.5 0 0 0 1.65-.3l.05-.06A1.8 1.8 0 1 1 19.5 7.3l-.06.05a1.5 1.5 0 0 0-.3 1.65v.07a1.5 1.5 0 0 0 1.37.9h.09a1.8 1.8 0 1 1 0 3.6h-.1a1.5 1.5 0 0 0-1.37.9Z",
  ],
  // 证据：放大镜 + 勾（保留在图标层，不做文字按钮）
  evidence: ["M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14Z", "M16.2 16.2 21 21", "M8.5 11.2l1.8 1.8 3.4-3.6"],
  // 跑一轮：循环箭头
  run: ["M20 12a8 8 0 1 1-2.6-5.9", "M20 4v4h-4"],
  // 详情：右滑入的面板
  detail: ["M4 6.5A1.5 1.5 0 0 1 5.5 5h13A1.5 1.5 0 0 1 20 6.5v11a1.5 1.5 0 0 1-1.5 1.5h-13A1.5 1.5 0 0 1 4 17.5Z", "M14.5 9.5 11 12l3.5 2.5"],
  // 发送：纸飞机。⛔ 不用「↑」—— 向上箭头在输入框里会被读成「上移一行」
  send: ["M20.5 3.5 3.8 10.2a.6.6 0 0 0 .05 1.13l6.9 2.4 2.4 6.9a.6.6 0 0 0 1.13.05Z", "M20.5 3.5 10.75 13.73"],
  // 加号
  plus: ["M12 5.5v13", "M5.5 12h13"],
  // 折叠/展开侧栏
  collapse: ["M14.5 8.5 11 12l3.5 3.5"],
  // 关闭
  close: ["M6 6l12 12", "M18 6 6 18"],
  // 搜索
  search: ["M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14Z", "M16.2 16.2 21 21"],
};

/** 图标名 → 可用集合。门禁（nt_check_tokens）会核对每个用到的名字都在表里。 */
export const ICON_NAMES = Object.keys(PATHS);

/**
 * 造一个「图标按钮」。
 *
 * ⚠️ `label` 同时给 `title` 与 `aria-label`：title 给鼠标悬停的可读提示，
 *    aria-label 给读屏。**只给其中一个**是常见的漏 —— 只给 title，
 *    键盘用户聚焦时读屏不念；只给 aria-label，鼠标用户看不到字。
 */
export function iconButton(
  name: string,
  label: string,
  onClick: () => void,
  opts: { variant?: "ghost" | "solid"; size?: number } = {},
): HTMLButtonElement {
  const b = document.createElement("button");
  b.type = "button";
  b.className = opts.variant === "solid" ? "nb-icon-btn nb-icon-btn--solid" : "nb-icon-btn";
  b.title = label;
  b.setAttribute("aria-label", label);
  b.dataset["icon"] = name;
  b.appendChild(svgIcon(PATHS[name] ?? PATHS["detail"]!, opts.size ?? 16));
  b.addEventListener("click", onClick);
  return b;
}

/** 造一个纯图标（非按钮，用于装饰位）。 */
export function icon(name: string, size = 16): SVGSVGElement {
  return svgIcon(PATHS[name] ?? PATHS["detail"]!, size);
}

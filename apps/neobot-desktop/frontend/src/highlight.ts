/**
 * 零依赖语法高亮 — 手写分词器，不引任何高亮库。
 *
 * 为什么不引库：CSP 是 `script-src 'self'`（无 CDN、无 eval），任何高亮库
 * 都得整包 bundle 进 vite；而本项目只需要「关键字 / 字符串 / 注释 / 数字」
 * 四档着色，Prisma/Highlight.js 那套几百 KB 的换来的只是一份更长的正则。
 *
 * **头号不变式：先转义，再切 token。**
 * 顺序反了就是 XSS —— 模型生成的 diff、仓库里的文件都可能带 `<script>`。
 * 故 `highlight()` 的第一步永远是 `esc()`，之后的分词只在已转义文本上找
 * 关键字/字符串/注释边界（这些符号 `&lt;` 之类不会破坏边界判定）。
 *
 * 分词是**单趟扫描**（不是逐类 replace 叠加）：叠加法会让后一类的匹配
 * 吃掉前一类已插入的标签，颜色错乱且不可能修对。
 */

/** 支持的语言 id。`plain` = 不着色。 */
export type LangId =
  | "plain"
  | "rust"
  | "ts"
  | "js"
  | "python"
  | "json"
  | "markdown"
  | "shell"
  | "toml"
  | "css"
  | "html"
  | "sql";

/** HTML 转义（与 `core.ts` 的 `esc` 同口径；此处自带一份以免与壳耦合）。 */
function esc(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

/** 扩展名 → 语言。未知一律 `plain`（不瞎猜）。 */
const EXT_MAP: Record<string, LangId> = {
  rs: "rust",
  ts: "ts",
  tsx: "ts",
  mts: "ts",
  cts: "ts",
  js: "js",
  jsx: "js",
  mjs: "js",
  cjs: "js",
  py: "python",
  pyi: "python",
  json: "json",
  jsonc: "json",
  json5: "json",
  md: "markdown",
  markdown: "markdown",
  mdx: "markdown",
  sh: "shell",
  bash: "shell",
  zsh: "shell",
  toml: "toml",
  ini: "toml",
  conf: "toml",
  css: "css",
  scss: "css",
  less: "css",
  html: "html",
  htm: "html",
  vue: "html",
  svelte: "html",
  sql: "sql",
  log: "plain",
  csv: "plain",
  txt: "plain",
};

/** 无扩展名的常见文件名（按整名判）。 */
const NAME_MAP: Record<string, LangId> = {
  makefile: "shell",
  dockerfile: "shell",
  ".gitignore": "plain",
  ".env": "shell",
  "cargo.toml": "toml",
  "package.json": "json",
  "tsconfig.json": "json",
};

/** 按文件名判语言；判不出就是 `plain`。 */
export function detectLanguage(name: string): LangId {
  const base = name.slice(name.lastIndexOf("/") + 1).toLowerCase();
  const byName = NAME_MAP[base];
  if (byName) return byName;
  const dot = base.lastIndexOf(".");
  if (dot <= 0) return "plain";
  return EXT_MAP[base.slice(dot + 1)] ?? "plain";
}

/** 每种语言的规则。缺省即 C 家族那套。 */
interface Rules {
  keywords: Set<string>;
  lineComment?: string[];
  blockComment?: [string, string];
  /** 字符串定界符（JSON 只有 `"`，shell 还要 `'`）。 */
  quotes: string[];
  /** 是否数字敏感（JSON 里 `true`/`null` 归关键字）。 */
  numbers: boolean;
  /** 模板串（TS/JS 的反引号 + `${}` 插值）。 */
  template?: boolean;
}

const word = (list: string): Set<string> => new Set(list.split(/\s+/));

const RULES: Partial<Record<LangId, Rules>> = {
  rust: {
    keywords: word(
      "as async await break const continue crate dyn else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type unsafe use where while",
    ),
    lineComment: ["//"],
    blockComment: ["/*", "*/"],
    quotes: ['"'],
    numbers: true,
  },
  ts: {
    keywords: word(
      "abstract any as async await boolean break case catch class const continue debugger declare default delete do else enum export extends false finally for from function get if implements import in infer instanceof interface is keyof let namespace never new null number object of private protected public readonly return satisfies set static string super switch symbol this throw true try type typeof undefined unknown var void while with yield",
    ),
    lineComment: ["//"],
    blockComment: ["/*", "*/"],
    quotes: ['"', "'"],
    numbers: true,
    template: true,
  },
  python: {
    keywords: word(
      "and as assert async await break class continue def del elif else except False finally for from global if import in is lambda None nonlocal not or pass raise return True try while with yield match case",
    ),
    lineComment: ["#"],
    quotes: ['"', "'"],
    numbers: true,
  },
  shell: {
    keywords: word(
      "if then else elif fi for while do done case esac function in return break continue local export source echo cd exit set unset trap",
    ),
    lineComment: ["#"],
    quotes: ['"', "'"],
    numbers: true,
  },
  sql: {
    keywords: word(
      "select from where insert into values update set delete create table drop alter index view join inner left right outer on group by order having limit offset union all as distinct and or not null primary key foreign references default",
    ),
    lineComment: ["--"],
    blockComment: ["/*", "*/"],
    quotes: ["'", '"'],
    numbers: true,
  },
  toml: {
    keywords: word("true false"),
    lineComment: ["#"],
    quotes: ['"', "'"],
    numbers: true,
  },
  css: {
    keywords: word("important media import charset keyframes supports"),
    blockComment: ["/*", "*/"],
    quotes: ['"', "'"],
    numbers: true,
  },
  html: {
    keywords: word(""),
    quotes: ['"', "'"],
    numbers: false,
  },
  json: {
    keywords: word("true false null"),
    quotes: ['"'],
    numbers: true,
  },
  markdown: { keywords: word(""), quotes: ['"'], numbers: false },
  plain: { keywords: word(""), quotes: [], numbers: false },
};

/** 一段着色后的文本。 */
interface Token {
  kind: "kw" | "str" | "com" | "num" | "txt";
  text: string;
}

/**
 * 单趟分词。
 *
 * 在**已转义**的文本上扫：按优先级 注释 > 字符串 > 数字 > 关键字 > 普通文本。
 * 注释优先级最高，否则 `"// not a comment"` 里的 `//` 会被当成注释起头。
 */
function tokenize(escaped: string, rules: Rules): Token[] {
  const out: Token[] = [];
  const push = (kind: Token["kind"], text: string): void => {
    if (!text) return;
    const last = out.length > 0 ? out[out.length - 1] : null;
    // 合并相邻同类（否则输出里全是碎片 span）。
    if (last && last.kind === kind) last.text += text;
    else out.push({ kind, text });
  };

  let i = 0;
  const n = escaped.length;
  while (i < n) {
    const rest = escaped.slice(i);

    // 1) 块注释
    if (rules.blockComment) {
      const [open, close] = rules.blockComment;
      if (rest.startsWith(open)) {
        const end = escaped.indexOf(close, i + open.length);
        const stop = end === -1 ? n : end + close.length;
        push("com", escaped.slice(i, stop));
        i = stop;
        continue;
      }
    }
    // 2) 行注释（吃到行尾；转义后换行仍是换行）
    if (rules.lineComment) {
      const marker = rules.lineComment.find((m) => rest.startsWith(m));
      if (marker) {
        const nl = escaped.indexOf("\n", i);
        const stop = nl === -1 ? n : nl;
        push("com", escaped.slice(i, stop));
        i = stop;
        continue;
      }
    }
    // 3) 字符串（含 TS/JS 模板串）
    const quote = rules.quotes.find((q) => rest.startsWith(q));
    if (quote) {
      const stop = scanString(escaped, i + quote.length, quote);
      push("str", escaped.slice(i, stop));
      i = stop;
      continue;
    }
    // 4) 数字（要求前面不是标识符字符，否则 `a1` 会被切成 `a` + `1`）
    if (rules.numbers && isDigit(rest[0]) && !isWordChar(i > 0 ? escaped[i - 1] : "")) {
      const m = /^[0-9][0-9a-fA-FxXoObB._+-]*/.exec(rest);
      const text = m ? m[0] : rest[0];
      push("num", text);
      i += text.length;
      continue;
    }
    // 5) 标识符 / 关键字
    if (isWordStart(rest[0])) {
      const m = /^[A-Za-z_$][A-Za-z0-9_$]*/.exec(rest);
      const text = m ? m[0] : rest[0];
      push(rules.keywords.has(text) ? "kw" : "txt", text);
      i += text.length;
      continue;
    }
    // 6) 其余原样
    const m = /^[^\w$]+/.exec(rest);
    const text = m ? m[0] : rest[0];
    push("txt", text);
    i += text.length;
  }
  return out;
}

/** 扫到字符串结束（`stop` 是闭引号之后的位置；未闭合则到文末）。 */
function scanString(escaped: string, from: number, quote: string): number {
  const n = escaped.length;
  let i = from;
  while (i < n) {
    const ch = escaped[i];
    if (ch === "\\") {
      i += 2; // 跳过转义符与其后一字符
      continue;
    }
    if (ch === quote) return i + 1;
    if (ch === "\n") return i; // 单行串不跨行（未闭合就到此为止）
    i += 1;
  }
  return n;
}

function isDigit(ch: string | undefined): boolean {
  return ch !== undefined && ch >= "0" && ch <= "9";
}

function isWordStart(ch: string | undefined): boolean {
  if (ch === undefined) return false;
  return /[A-Za-z_$]/.test(ch);
}

function isWordChar(ch: string | undefined): boolean {
  if (ch === undefined) return false;
  return /[A-Za-z0-9_$]/.test(ch);
}

/**
 * 高亮一段文本，产出**只含我们自己的 `<span class="hl-*">`** 的 HTML。
 *
 * 不变式（`selftest.ts` 逐条盯着）：
 * 1. 输入里的 `<` `>` `&` 一律转义 —— 绝不产出用户可控标签；
 * 2. 去掉所有标签后，文本与源文**逐字符相同** —— 切错就会暴露。
 */
export function highlight(source: string, lang: LangId | string): string {
  const rules = RULES[lang as LangId] ?? RULES.plain;
  if (!rules || lang === "plain") return esc(source);
  // Markdown 单独处理（结构与代码语言不同：标题/列表/粗体/行内码）。
  if (lang === "markdown") return highlightMarkdown(source);
  const tokens = tokenize(esc(source), rules);
  let html = "";
  for (const token of tokens) {
    html += token.kind === "txt" ? token.text : `<span class="hl-${token.kind}">${token.text}</span>`;
  }
  return html;
}

/** Markdown 的四种着色：标题 / 粗斜体 / 行内码 / 其余。 */
function highlightMarkdown(source: string): string {
  const escaped = esc(source);
  const lines = escaped.split("\n");
  const out: string[] = [];
  let inFence = false;
  for (const line of lines) {
    if (/^\s*(```|~~~)/.test(line)) {
      inFence = !inFence;
      out.push(`<span class="hl-com">${line}</span>`);
      continue;
    }
    if (inFence) {
      out.push(`<span class="hl-com">${line}</span>`);
      continue;
    }
    if (/^\s{0,3}#{1,6}\s/.test(line)) {
      out.push(`<span class="hl-kw">${inlineMd(line)}</span>`);
      continue;
    }
    if (/^\s{0,3}([-*+]|\d+\.)\s/.test(line)) {
      out.push(`<span class="hl-num">${inlineMd(line)}</span>`);
      continue;
    }
    if (/^\s{0,3}>/.test(line)) {
      out.push(`<span class="hl-com">${inlineMd(line)}</span>`);
      continue;
    }
    out.push(inlineMd(line));
  }
  return out.join("\n");
}

/** 行内的行内码 / 粗体 / 斜体。 */
function inlineMd(line: string): string {
  return line
    .replace(/(`[^`]+`)/g, '<span class="hl-str">$1</span>')
    .replace(/(\*\*[^*]+\*\*)/g, '<span class="hl-kw">$1</span>')
    .replace(/(\*[^*]+\*)/g, '<span class="hl-num">$1</span>')
    .replace(/(\[[^\]]*\]\([^)]*\))/g, '<span class="hl-num">$1</span>');
}

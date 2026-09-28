/**
 * 文件树的**纯逻辑** —— 零依赖、零 IO、零 DOM。
 *
 * 为什么要单独成文件：`sidebar.ts` import 了 `@tauri-apps/api`（`invoke`），
 * 于是它**没法**在 plain node 里被单测 import。把树的形状计算摘到这里，
 * 既能单测，又正好合上本项目既有的分层约定（`board_render.ts` / `thread.ts`
 * 是「纯渲染」，`main.ts` 是「事件装配」）。
 *
 * 这段逻辑出过一次 bug：重绘时只看**一层**，于是展开目录毫无反应 ——
 * 而肉眼看「一棵树」和「一个只会显示根目录的面板」几乎分不出来。
 * 故下面的规则都有对应的自测（`selftest.ts`）。
 */

/** 目录下一项（与 Rust 侧 `nt_workspace::DirEntry` 同形，只用到树需要的字段）。 */
export interface TreeEntry {
  name: string;
  rel: string;
  is_dir: boolean;
  is_symlink?: boolean;
  broken_link?: boolean;
  size?: number;
  mtime?: number;
  /** 标记「这一项是打不开的占位」，不是真的目录内容。 */
  unreadable?: boolean;
}

/** 树里当前**可见**的一行。 */
export interface TreeRow {
  /** 相对工作区根的路径。 */
  rel: string;
  /** 缩进层级（根层 = 0）。 */
  depth: number;
  entry: TreeEntry;
}

/**
 * 按先序遍历摊平成「可见行」—— 只有 `expandedDirs` 里的目录才递归下去。
 *
 * 摊平而不是递归出嵌套 HTML：摊平后缩进只是 `depth * 14px`，
 * 而 DOM 保持一层，展开/折叠时不必重建子树。
 *
 * 三条不变量（都有自测盯着）：
 * 1. **未展开的目录不产出子项** —— 否则「懒加载」名不副实；
 * 2. **未加载的目录不产出子项** —— 刚展开、拉取失败时不能编，也不能崩；
 * 3. **文件永不可展开** —— 即使 `expandedDirs` 里混进了文件路径。
 */
export function flattenTree(
  cache: Map<string, TreeEntry[]>,
  expandedDirs: Set<string>,
  root = "",
  failed: Set<string> = new Set(),
): TreeRow[] {
  const out: TreeRow[] = [];
  // 显式栈而非递归：目录深度由用户控制，理论上可以很深，递归版会爆栈。
  const walk = (rel: string, depth: number): void => {
    const entries = cache.get(rel);
    if (!entries) return;
    for (const entry of entries) {
      // 展开过但**拉取失败**的目录：在**它自己那一行**上就地标「打不开」，
      // 不再递归进去。
      //
      // 早先的写法是在 `walk` 开头为失败目录**另加**一行占位，于是它出现两次
      // （父目录列举里一次、占位一次）—— 自测当场抓到了。
      // 就地标记顺带保证了「一行一个条目」这条不变式。
      const unreadable = entry.is_dir && failed.has(entry.rel);
      out.push({
        rel: entry.rel,
        depth,
        entry: unreadable ? { ...entry, unreadable: true } : entry,
      });
      if (entry.is_dir && expandedDirs.has(entry.rel) && !unreadable) {
        walk(entry.rel, depth + 1);
      }
    }
  };
  walk(root, 0);
  return out;
}

/**
 * 目录内容是否变了（轮询刷新用：没变就不重绘）。
 *
 * 比 `name + is_dir + size`，**不比 `mtime`** —— mtime 会因为
 * `touch`、构建产物重写之类的原因动而内容没变，每 4 秒重绘一次纯属浪费。
 * 顺序也比：Rust 侧列举已排序（目录在前 + 名序），换序即视为变了，
 * 宁可多重绘一次也不要漏掉变化。
 */
export function sameEntries(a: TreeEntry[], b: TreeEntry[]): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i += 1) {
    const x = a[i];
    const y = b[i];
    if (!x || !y) return false;
    if (x.rel !== y.rel || x.is_dir !== y.is_dir) return false;
    if ((x.size ?? 0) !== (y.size ?? 0)) return false;
  }
  return true;
}

/** 相对路径的父目录（顶层返回 `""`）。 */
export function parentOf(rel: string): string {
  const idx = rel.lastIndexOf("/");
  return idx === -1 ? "" : rel.slice(0, idx);
}

/**
 * 展开到某路径所需的祖先集合（含路径自身的目录，不含文件本身）。
 *
 * 「面包屑 = 展开到这一层」用的就是这个。
 */
export function ancestorsOf(rel: string): string[] {
  const parts = rel.split("/").filter(Boolean);
  const out: string[] = [];
  let acc = "";
  for (let i = 0; i < parts.length - 1; i += 1) {
    const part = parts[i];
    if (part === undefined) continue;
    acc = acc ? `${acc}/${part}` : part;
    out.push(acc);
  }
  return out;
}

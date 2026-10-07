/**
 * 产物新鲜度断言 —— UI 门公用件（P0-1）。
 *
 * # 为什么必须有
 *
 * 实测（2026-10-07）：**17 道**读 `neobot-ui/dist` 的门，此前**无一断言产物时效**。
 * ⓘ 逐一验证：同时含 `neobot-ui/dist` 与 `mtime/statSync` 的门 —— 零。
 * ⇒ 真实事故形态：源码改了、`dist` 没重建，门**绿着测旧界面**。
 * ⓘ 复现：只 `touch` 一个源文件（内容零改动），布局门仍 rc=0 PASS ——
 *   它无法分辨自己测的是哪一版。
 * ⇒ 这是 §4 教训 38 的形状，只是分叉在**产物时效**而非选择器：
 *   「文档说有门、实际没测到目标」。
 *
 * # 为什么做成公用件而不是各门复制一段
 *
 * ⛔ 17 道门各写一份 = 17 份待腐化的副本。本仓已经吃过「同一个东西两个数字」
 *   的亏（douchat 侧栏宽 324/322/244 三个值；`--nb-side-w` 声明了却零消费）。
 * ⇒ 单一真源，17 处引用。
 *
 * # 已知局限（必须写下来，否则会被当万能门）
 *
 * ⚠️ mtime 比对挡不住「改了又改回」：改文件又还原内容，mtime 更新但产物其实
 *   是对的 ⇒ 此时**误报**（要求重建，实际不必）。要更硬得靠构建期内容哈希
 *   （vite manifest），成本高一档，暂不做。
 * ⚠️ 只比 mtime，**不校验产物真的由当前源码生成**。手工把一个旧 dist 拷进来
 *   再 `touch` 过 index.html ⇒ 假绿。
 * ⚠️ 只覆盖 `src/`。改 `index.html` / `vite.config.*` / `package.json` 而源码
 *   未动时，本断言**不报** ⇒ 改构建配置后仍需手动重建。
 *
 * ⛔ 文案里**不得出现反引号包命令示例**（§4 教训 R36：`check-disk.sh` 那次
 *   反引号被 bash 当命令替换，真删了 115.2 GiB）。
 */
import { existsSync, readdirSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';

/** statSync 的安全版：共享工作树里文件可能在遍历途中消失。 */
function safeStat(p) {
  try { return statSync(p); } catch { return null; }
}

/** 递归找 dir 下最新的文件 mtime（毫秒）。返回 { newest, newestRel }。 */
export function newestMtime(dir) {
  let newest = 0, newestRel = '';
  if (!existsSync(dir)) return { newest, newestRel };
  for (const e of readdirSync(dir)) {
    if (e === 'node_modules' || e === '.git' || e === 'target' || e === 'dist') continue;
    const p = join(dir, e);
    const st = safeStat(p);
    if (!st) continue;
    if (st.isDirectory()) {
      const sub = newestMtime(p);
      if (sub.newest > newest) { newest = sub.newest; newestRel = sub.newestRel; }
    } else if (st.isFile() && st.mtimeMs > newest) {
      newest = st.mtimeMs; newestRel = p;
    }
  }
  return { newest, newestRel };
}

/**
 * 比对 dist 与 src 的新鲜度，**不退出**，由调用方决定怎么处理。
 * @returns {{ ok: boolean, reason?: string, newestSrc?: string, srcMtime?: number, distMtime?: number }}
 */
export function checkDistFresh({ dist, src, label = '门' }) {
  const index = join(dist, 'index.html');
  if (!existsSync(index)) {
    return { ok: false, reason: `dist 不存在：${index}` };
  }
  const distMtime = statSync(index).mtimeMs;
  const { newest, newestRel } = newestMtime(src);
  if (newest > distMtime) {
    return {
      ok: false,
      reason: `产物比源码旧 —— ${label}会测到旧界面`,
      newestSrc: newestRel ? relative(process.cwd(), newestRel) : '(未知)',
      srcMtime: newest,
      distMtime,
    };
  }
  return { ok: true, srcMtime: newest, distMtime };
}

/**
 * 断言产物新鲜；不新鲜则打印**可执行**的修法并 `process.exit(1)`。
 * 每道门在开浏览器**之前**调用 —— 免得白等几十秒再失败。
 */
export function requireFreshDist({ dist, src, label = '本门' }) {
  const r = checkDistFresh({ dist, src, label });
  if (r.ok) return true;
  console.error(`${label} FAIL: ${r.reason}`);
  if (r.newestSrc) {
    console.error(`  最新源码: ${r.newestSrc}`);
  }
  console.error(`  修法: cd apps/neobot-desktop/neobot-ui && pnpm run build`);
  process.exit(1);
}

/** 默认坐标（自持交付树），供绝大多数门直接用。 */
export function neobotDistFresh(label) {
  const ui = new URL('../../apps/neobot-desktop/neobot-ui/', import.meta.url).pathname;
  return requireFreshDist({ dist: join(ui, 'dist'), src: join(ui, 'src'), label });
}
#!/usr/bin/env node
// 能力涌现门 —— 断言「能力树在运行期真的会被写入，且判定区分『没长』与『长停了』」。
//
// ## 判据来源
// `docs/architecture/EMERGENCE-ROADMAP-2026-10-03.md` §1：
//   > 涌现 = 能力树在无人工干预下新增了节点，且该节点通过 `audit-maturity --strict`。
//
// ## ⭐⭐ 为什么本门是**三态**而不是红/绿二值（这是本门唯一真正的设计点）
//
// 我第一版想把「节点数没增长」直接判红，写的时候立刻卡住：**冷启动**。
// 首批节点由**第一次低质 critique** 产生 ⇒ 进程刚起来、还没积累到任何一次
// 低质输出时，节点数必然是 0 ⇒ 二值门会**天天红**，然后被加白名单，然后门就死了。
//
// ⭐ 解法来自外部吸收（`docs/architecture/ABSORPTION-ZERON-2026-10-03.md` §2）：
// zeron 的 `crates/harness/src/code_signature.rs` 有
// `pub const SUPPORTED: bool = cfg!(any(target_os="macos", windows));`
// 并明写「linux 构建无签名 ⇒ pinned digest 是**唯一**可接受的证明」。
// ⇒ **证明能力不可用时，必须显式降级并说明替代证明，绝不静默 pass。**
//
// 本门同构地分三态：
//   PASS      —— 生产侧有写入点（接线在），成熟度审计绿
//   COLD_START—— 接线在，但**尚无生产写入发生过**（不可证伪，不是缺陷）
//   FAIL      —— 接线**不在**，或成熟度审计红（这才是缺陷）
//
// ⛔ COLD_START **不是**绿灯豁免：它是「本轮证据不足」，必须每次复跑都重新判，
// 一旦接线被删、审计转红，立即翻成 FAIL。

import { readFileSync, existsSync, mkdtempSync, cpSync, writeFileSync, rmSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const REPO = join(import.meta.dirname, '..', '..');
const RUNTIME = 'crates/neotrix-neobot/src/nt_capability_registry.rs';
const PRODUCER = 'neotrix-core/src/l5_cognition/nt_core_consciousness/consciousness_runtime.rs';

const results = [];
const ok = (m) => results.push(['PASS', m]);
const bad = (m) => results.push(['FAIL', m]);

if (!existsSync(join(REPO, RUNTIME))) {
  console.error(`FAIL: 运行期注册表缺失: ${RUNTIME}`);
  process.exit(1);
}
if (!existsSync(join(REPO, PRODUCER))) {
  console.error(`FAIL: 生产者文件缺失: ${PRODUCER}`);
  process.exit(1);
}

// ⭐ 静态判定抽成**纯函数**（吃源码文本）—— 正向与负向自测**共用同一份逻辑**。
// ⛔ 旧版自测自己重写了一遍正则，那只能证明「我的第二个正则也能摘掉」，
//    **证明不了门本身会红**。负向测试的价值全在「跑的是同一段代码」。
function staticVerdict(runtime, producer) {
  const out = [];
  if (/^pub fn register_node/m.test(runtime)) out.push(['PASS', '导出 register_node']);
  else out.push(['FAIL', '未导出 register_node（模块是死的）']);
  if (/^pub fn node_count/m.test(runtime)) out.push(['PASS', '导出 node_count（成长可观测）']);
  else out.push(['FAIL', '未导出 node_count ⇒ 无法盯成长']);
  const testStart = producer.indexOf('#[cfg(test)]');
  const head = testStart === -1 ? producer : producer.slice(0, testStart);
  const prodCalls = [...head.matchAll(/\bregister_node\s*\(/g)].length;
  if (prodCalls > 0) out.push(['PASS', `生产侧 register_node 调用点 x${prodCalls}`]);
  else out.push(['FAIL', '生产侧无 register_node 调用 ⇒ 接线已丢失']);
  if (/quality\s*<\s*0\.3/.test(head) && /quality\s*<\s*0\.3[\s\S]{0,2000}\bregister_node\s*\(/.test(head))
    out.push(['PASS', '成长判据锚在 `quality < 0.3`']);
  else out.push(['FAIL', 'register_node 未锚低质判据 ⇒ 涌现退化为刷节点']);
  return out;
}

const runtime = readFileSync(join(REPO, RUNTIME), 'utf8');
const producer = readFileSync(join(REPO, PRODUCER), 'utf8');
for (const [st, m] of staticVerdict(runtime, producer)) (st === 'PASS' ? ok : bad)(m);
const prodCalls = [...producer.slice(0, producer.indexOf('#[cfg(test)]') === -1
  ? undefined : producer.indexOf('#[cfg(test)]')).matchAll(/\bregister_node\s*\(/g)].length;

// ── ⑤ 反虚标：成熟度审计必须绿 ────────────────────────────────────────────────
let auditGreen = false;
try {
  // ⭐ 命令形态**照抄 CI**（`.github/workflows/ci.yml:375-377`）：
  //   cargo build -p nt_core_capability_tree --bin neotrix-capability
  //   ./target/debug/neotrix-capability audit-maturity --strict
  // ⛔ 我第一版写成 `cargo run -p nt-core-capability-tree` ⇒ **包名分隔符又错了**
  //   （真名是下划线 `nt_core_capability_tree`）⇒ 门当场红。
  //   ⭐ 这是我在 `1b37c1ec` 刚记录过的同一个坑，**一天内踩了两次**
  //   ⇒ 该命名陷阱必须当作常驻风险看待。
  execFileSync('cargo', ['build', '-q', '-p', 'nt_core_capability_tree',
    '--bin', 'neotrix-capability'], { cwd: REPO, stdio: ['ignore', 'pipe', 'pipe'] });
  execFileSync(join(REPO, 'target/debug/neotrix-capability'),
    ['audit-maturity', '--strict'], { cwd: REPO, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
  auditGreen = true;
  ok('成熟度审计（反虚标）通过');
} catch (e) {
  const detail = String((e.stdout || '') + (e.stderr || '')).slice(0, 400);
  bad(`成熟度审计红 —— 有节点声称的成熟度超过证据支撑\n${detail}`);
}

// ── ⑥ ⭐ 三态判定 ────────────────────────────────────────────────────────────
const wired = prodCalls > 0 && auditGreen;
// ⭐⭐⭐ 涌现状态**提为模块级变量**（⭐⭐ 单一真源）：⭐⭐ 上面的分支要写它，
// ⭐⭐ 文件末尾的**汇总行**也要读它 ⇒ ⭐⭐ 放函数体内会 ReferenceError。
// ⭐⭐⭐ 而**不是**让汇总行硬编码字符串 —— ⭐⭐ 那正是我刚修掉的「摘要撒谎」。
let emergentState = 'UNKNOWN';
if (!wired) {
  console.error('FAIL: 涌现接线不在（见上方逐项）');
} else {
  // ⭐⭐⭐⭐ **运行期探针**（⭐⭐ 路线 §5 第 2 步，⭐⭐ 本轮实现）。
  // ⭐⭐⭐ 静态门拿不到「节点数是否增长」⇒ ⭐⭐ 于是只能报 COLD_START。
  // ⭐⭐⭐ 探针走**真实构造路径**（`ConsciousnessRuntime::new()`），
  // ⭐⭐⭐ ⛔ 不许自证（不直接调 bootstrap）。
  const probe = (() => {
    try {
      const out = execFileSync(
        'cargo',
        ['test', '-p', 'neotrix', '--lib', '运行期探针', '--', '--nocapture'],
        { cwd: REPO, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], timeout: 900_000 },
      );
      const m = out.match(/^EMERGENCE_PROBE (\{.*\})$/m);
      return m ? JSON.parse(m[1]) : null;
    } catch (e) {
      const so = `${e.stdout || ''}`;
      const m = so.match(/^EMERGENCE_PROBE (\{.*\})$/m);
      return m ? JSON.parse(m[1]) : null;
    }
  })();

  if (!probe) {
    // ⭐⭐⭐ ⛔ **探针跑不出来 ⇒ FAIL，不许静默回落成 COLD_START**。
    // ⭐⭐ 理由：⭐⭐ 回落 = ⭐⭐「拿不到证据」被当成「没有缺陷」⇒
    // ⭐⭐⭐ **这正是本门自己文档里警告的「COLD_START 不是绿灯豁免」**。
    console.error('FAIL: 运行期探针跑不出结果（无法判定是否已成长）');
    console.error('  ⛔ ⭐⭐ 不回落成 COLD_START：⭐⭐ 拿不到证据 ≠ 没有缺陷');
    process.exit(1);
  }

  const findings = probe.maturity_findings || [];
  if (findings.length) {
    console.error(`FAIL: 成熟度审计（反虚标）红：${findings.join('; ')}`);
    process.exit(1);
  }
  if (!(probe.abilities >= 5)) {
    console.error(`FAIL: 运行期真实能力节点 ${probe.abilities} 个（应 ≥5）`
      + ' ⇒ ⭐⭐ 播种没发生 or 被回退 ⇒ ⭐⭐ 涌现链断在这里');
    process.exit(1);
  }

  // ⭐⭐⭐ 第四态：**已成长**（⭐⭐ 静态门 + 运行期证据都成立）
  emergentState = 'WARM';   // ⭐⭐ 供末尾汇总行读（⭐⭐ 单一真源）
  console.log(`STATE: WARM ✅（运行期实测：能力节点 ${probe.abilities} / 总节点 ${probe.total}`
    + ` · 缺口节点 ${probe.gaps} · 成熟度审计 0 条）`);
  console.log(`  能力节点清单：${probe.ids.join(', ')}`);
  console.log('  ⭐⭐ 本状态由**运行期探针**判定，⭐⭐ 不是静态推断。');
  // ⭐⭐⭐ 三段式能力播报（backburner 范式）：⭐⭐ **第 3 段 = 「被调用了吗」**。
  // ⭐⭐⭐ 前面的「STATE: WARM」证明「已注册」，⭐⭐ 这一段证明「是否真被用过」。
  // ⭐⭐⭐ 「注册了」与「被用了」是**两个不同的事实** —— ⭐⭐ 而三家对标仓库
  // ⭐⭐⭐ （os-taxonomy / lcu / backburner）⭐⭐ **全部只能证明前者**。
  const neverInvoked = probe.never_invoked || [];
  const invokedCounts = probe.invoked_counts || [];
  console.log('  ── 调用面（三段式的第 3 段）──');
  for (const v of invokedCounts) {
    console.log(`    ${v.invoked > 0 ? '✅' : 'ℹ️ '} ${v.id} 被调用 ${v.invoked} 次`);
  }
  if (neverInvoked.length) {
    // ⭐⭐ ⛔ 只报告 ⛔ **不阻断**：⭐⭐「本次进程该调用几次」无可证伪定义，
    // ⭐⭐ 阻断它等于造假（照 lcu `tested.py` 的「Informational, never refuses」）。
    console.log(`    ℹ️ 已注册但**本次进程零调用**：${neverInvoked.join(', ')}`);
    console.log('    ⭐⭐ ⛔ **刻意不阻断**（理由同上）');
  }
}

for (const [s, m] of results) console.log(`${s}: ${m}`);
const failed = results.filter(([s]) => s === 'FAIL').length;
if (failed) { console.error(`\nFAIL: ${failed} 项红`); process.exit(1); }
// ⭐⭐⭐ 汇总行**必须**与真实状态一致（⭐⭐ 修一处「摘要撒谎」）。
// ⛔ 改前这行**硬编码** `(STATE: COLD_START)`，⭐⭐⭐ 而上面已打印
// ⭐⭐ `STATE: WARM` ⇒ ⭐⭐⭐ **汇总与真实状态矛盾** ⇒ ⭐⭐ 读者只看
// ⭐⭐ 最后一行就会得到**相反结论**。⇒ 改为引用同一个变量。
console.log(`\nOK: ${results.length} 项全绿（STATE: ${emergentState}）`);

// ── ⭐ 负向自测：摘掉接线，**同一份** staticVerdict 必须翻红 ──────────────────
// ⛔ 三次修正才让它有意义：
//   ① 最初只重写正则 ⇒ 证明不了门会红（假自测）
//   ② 变异标记 `unwired_register_node(` **含** `register_node` 子串 ⇒ 变异无效
//   ③ 现改为共用 staticVerdict + 不含子串的标记 `NOT_WIRED(`
if (process.argv.includes('--self-test')) {
  for (const [label, mutate] of [
    ['摘除生产侧接线', t => t.replace(/\bregister_node\s*\(/g, 'NOT_WIRED(')],
    ['摘除 node_count 导出', t => t.replace(/^pub fn node_count/m, 'fn node_count')],
    ['摘除低质判据锚', t => t.replace(/quality\s*<\s*0\.3/g, 'quality < 0.9')],
  ]) {
    const v = staticVerdict(mutate(runtime), mutate(producer));
    const red = v.filter(([st]) => st === 'FAIL').length;
    if (red === 0) { console.error(`SELFTEST FAIL: ${label} 后仍全绿（变异无效）`); process.exit(1); }
    console.log(`SELFTEST OK: ${label} -> ${red} 项红`);
  }
  console.log('\nPASS: 产出侧仍全绿 + 三个变异均被同一份判定翻红（负向测试有效）');
}

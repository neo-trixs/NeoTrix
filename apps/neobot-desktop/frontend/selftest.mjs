// 自测入口。node ≥ 22 直接跑 TS 源（类型擦除），零构建步骤。
// 不在 .mjs 里写 TS 注解 —— node 不会对 .mjs 擦除类型。
import { runSelfTest } from "./src/selftest.ts";
const failures = await runSelfTest();
if (failures.length === 0) console.log("  neobot 架构自测: 全部通过");
else {
  console.error(`  neobot 架构自测: ${failures.length} 项失败`);
  for (const f of failures) {
    console.error(`  ✗ ${f.what}`);
    if ("got" in f) console.error(`      实际 ${JSON.stringify(f.got)}\n      期望 ${JSON.stringify(f.want)}`);
  }
  process.exitCode = 1;
}

#!/usr/bin/env python3
"""扩展点接线门 —— 断言「注册式扩展 API 的零生产调用方数量不增长」。

## ⭐ 本门度量的是**接线**，不是「有没有建」

本日（三次实测）沉淀的核心判据，**接线必须分三级看**：

| 级 | 问什么 | 可机器判定 |
|---|---|---|
| L1 声明 | 有 `pub fn` 吗？ | ✅ 文本可数 |
| L2 分发 | 生产路径会**跑**它吗？ | ⚠️ 需数据流 |
| ⭐ **L3 使用** | ⭐ **有生产代码真的注册它吗？** | ✅ **本门度量这个** |

⭐ 为什么只看 L3：本日实测两例「声称已接线实则未用」——
`nt_core_dispatch`（354 行 waterfall，真实消费者只有 L0 一处）与
`McpBridge::on_tool_call`（接上后 `new()` 注册了默认守卫才达 L3）。
⇒ **L1/L2 绿 ≠ 能力被使用。**

## ⭐ 为什么是**棘轮基线**而不是「必须为 0」
实测：209 个注册式扩展 API 中 **50 个零生产调用方**。
⇒ 「全部接线」不是本门能推动的目标（那是产品排期，不是 CI 判据）。
⇒ 本门只做一件**诚实**的事：**这个数不许增长**。
⛔ 本门**不声称**基线内那 50 个是死代码，也**不声称**它们该被接线 ——
其中可能有正当理由（如仅供外部 crate 使用的公开 API）。
新增即红 ⇒ 须当场定性并写进基线。

## ⭐ 口径（错一次就得出假结论，务必看清）
· 扫描根 = **全工作区**（`neotrix-core/src` + `crates` + `apps` + `neobot`），
  ⛔ **不能只扫 `neotrix-core/src`**：
  实测若只扫它，`register_stdio_global` 等会被误判为死 API，
  而它们在 `crates/`/`apps/` 有调用方（本日已踩过一次「搜索范围窄 ⇒ 假阴性」）。
· ⛔ **排除 `#[cfg(test)]` 之后的内容** —— 测试里的调用**不算**生产调用方。
  ⭐ 实测踩过反向坑：用未排除测试的 `rg` 抽查，得出「有 2 个调用方」的**错**结论，
  差点否掉一个**正确**的测量。⇒ 两边都要查，别只信一边。
· 判「调用」用**裸标识符 + `(`**，⛔ 不用路径限定（`mod::name(`），
  否则跨模块重导出后漏计。

## 用法
    python3 scripts/check-ext-wiring.py            # 棘轮检查
    python3 scripts/check-ext-wiring.py --audit    # 只打印
    python3 scripts/check-ext-wiring.py --rebaseline
    python3 scripts/check-ext-wiring.py --self-test  # ⭐ 负向测试
"""
import os
import re
import sys

ROOTS = ["neotrix-core/src", "crates", "apps", "neobot"]
BASELINE = "scripts/ext-wiring-baseline.txt"
SKIP = ("/target", "/node_modules", "/.git")
# 注册式扩展 API 的命名形态
API = re.compile(r"pub fn ((?:on_|register_|add_hook|subscribe)\w*)\s*\(")


def rust_files():
    for r in ROOTS:
        if not os.path.isdir(r):
            continue
        for dp, _dn, fn in os.walk(r):
            if any(s in dp for s in SKIP):
                continue
            for f in fn:
                if f.endswith(".rs"):
                    yield os.path.join(dp, f)


def load():
    src = {}
    for p in rust_files():
        try:
            with open(p, encoding="utf-8", errors="ignore") as fh:
                src[p] = fh.read()
        except OSError:
            pass
    return src


# ⭐⭐ SIM-27 同款处理：剥掉**整行注释**再扫。
# ⭐ 起因（2026-10-03 实测，**本门自己的假阳性**）：
#   `nt_capability_registry.rs` 的 `with_registry` 文档里有 ```` ```ignore ```` 代码块
#   写着示例签名 `pub fn register_xxx_capability(registry: &mut …)`，
#   ⭐⭐ 那是**文档示例**，不是真 API ⇒ 被本门当成「零生产调用的扩展点」而误报。
#   ⭐ 与 `nt_layer_deps.sh:66`（SIM-27：drop full-line comments）
#   以及 `nt_sampler.rs:17` 的注释误报**同型** —— ⭐ 而这次是我自己引入的。
# ⛔ **只剥整行注释**（`//`、`///`、`//!`）；行尾注释与字符串字面量**保留**
#   （保守：可能过度报告，绝不漏报）—— 与 layer-deps 的口径一致。
_LINE_COMMENT = re.compile(r"^\s*(//+[!/]?).*$", re.M)


def prod_body(text):
    """剥掉整行注释 + `#[cfg(test)]` 之后的内容。

    ⭐ 两处都要：测试里的调用不算生产调用方；文档里的示例签名不算真 API。
    """
    return _LINE_COMMENT.sub("", text).split("#[cfg(test)]")[0]


def measure():
    src = load()
    unwired = []
    total = 0
    for p, text in src.items():
        # ⭐ 枚举候选时同样要剥注释（否则注释里的示例签名会被当成真 API）
        for m in API.finditer(prod_body(text)):
            name = m.group(1)
            total += 1
            pat = re.compile(r"\b" + re.escape(name) + r"\s*\(")
            prod = 0
            for q, t2 in src.items():
                body = prod_body(t2)
                if q == p:
                    # 定义处不算；本文件内 `#[cfg(test)]` 之前还有其它调用才算
                    if len(pat.findall(body)) > 1:
                        prod += 1
                    continue
                if pat.search(body):
                    prod += 1
            if prod == 0:
                unwired.append(f"{p}::{name}")
    return total, sorted(unwired)


def main():
    argv = sys.argv[1:]
    total, unwired = measure()

    if "--rebaseline" in argv:
        with open(BASELINE, "w", encoding="utf-8") as fh:
            # ⭐⭐ 保留既有 `#` 理由注释：⭐ 重算基线时**丢掉理由**比多一条更糟
            # （下一个读到「某个 API 零调用」却不知道它是否已定性）。
            keep = []
            if os.path.exists(BASELINE):
                with open(BASELINE, encoding="utf-8") as old:
                    for ln in old:
                        if ln.lstrip().startswith("#"):
                            keep.append(ln.rstrip())
            if keep:
                fh.write("\n".join(keep) + "\n")
            fh.write("\n".join(unwired) + "\n")
        print(f"基线已重建: {total} 个注册 API / {len(unwired)} 个零生产调用方")
        return 0

    known = set()
    if os.path.exists(BASELINE):
        with open(BASELINE, encoding="utf-8") as fh:
            # ⭐ 支持 `#` 理由注释行（写入时保留、读取时忽略）
            known = {
                ln.strip() for ln in fh
                if ln.strip() and not ln.lstrip().startswith("#")
            }

    fresh = [u for u in unwired if u not in known]

    if "--audit" in argv:
        print(f"注册式扩展 API: {total} · 零生产调用方: {len(unwired)} · 基线: {len(known)} · 新增: {len(fresh)}")
        for f in fresh:
            print("  NEW ", f)
        return 0

    print(f"注册式扩展 API: {total} · 零生产调用方: {len(unwired)} · 基线: {len(known)}")

    if fresh:
        print(f"\nFAIL: 新增 {len(fresh)} 个零生产调用的扩展 API：", file=sys.stderr)
        for f in fresh:
            print("  " + f, file=sys.stderr)
        print(
            "\n⛔ 这**不等于**死代码（可能是仅供外部 crate 使用的公开 API）。\n"
            "   但新增即须当场定性：\n"
            "  · 有正当理由 ⇒ 加进 scripts/ext-wiring-baseline.txt 并注明理由\n"
            "  · 是漏接线 ⇒ 先接线（本门度量的是 L3「有生产注册」）\n"
            "  · 定性不了 ⇒ 留红，别加白名单",
            file=sys.stderr,
        )
        return 1

    stale = known - set(unwired)
    if stale:
        print(f"ℹ️ 基线中 {len(stale)} 条已消失（可清理，不判红）")
    print("PASS: 无新增零生产调用的扩展 API")
    print("⛔ 本门不判定基线内的 API 是否死代码 —— 静态匹配无法判定「该不该接线」。")
    return 0


if __name__ == "__main__":
    if "--self-test" in sys.argv:
        # ⭐ 负向自测：注入一个**新增**的零调用 `pub fn on_*`，同一份判定必须翻红。
        # ⭐ 教训（本日 3 次）：① 必须跑**同一份**逻辑；② 变异标记**不能含被测子串**；
        #   ③ 变异方向必须与被测性质相关（这里是「新增未接线 API」）。
        import shutil
        import tempfile

        victim = "crates/neotrix-neobot/src/nt_provider.rs"
        if not os.path.exists(victim):
            print("SELFTEST FAIL: 注入目标不存在: " + victim)
            sys.exit(2)
        before_total, before_unwired = measure()
        with open(victim, encoding="utf-8") as fh:
            original = fh.read()
        tmpdir = tempfile.mkdtemp()
        try:
            with open(victim, "w", encoding="utf-8") as fh:
                fh.write("pub fn on_selftest_probe_zzz() {}\n" + original)
            after_total, after_unwired = measure()
            if after_total != before_total + 1:
                print(f"SELFTEST FAIL: 注入后总数未 +1（{before_total} → {after_total}）")
                sys.exit(1)
            new_unwired = [u for u in after_unwired if u not in before_unwired]
            if not new_unwired:
                print("SELFTEST FAIL: 注入后未被判为零生产调用方")
                sys.exit(1)
            print(f"\nSELFTEST OK: 注入后 {before_total} → {after_total}，"
                  f"新增零调用 {len(new_unwired)} 个 ⇒ 判定会红")
        finally:
            with open(victim, "w", encoding="utf-8") as fh:
                fh.write(original)
            shutil.rmtree(tmpdir, ignore_errors=True)
        # 还原必须彻底（否则自测污染工作树 —— 本日实测教训）
        _, restored = measure()
        if restored != before_unwired:
            print("SELFTEST FAIL: 还原后不一致（污染工作树）")
            sys.exit(1)
        print("SELFTEST OK: 还原后与注入前一致")
        sys.exit(0)
    sys.exit(main())
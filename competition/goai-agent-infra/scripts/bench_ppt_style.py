#!/usr/bin/env python3
"""ppt-design C3 benchmark — 多主题 StyleSpec 生成基准 (des/branches/ppt-design).

验证维度:
1. 全主题覆盖: 6 主题 (consciousness/hexagram/engineering/corporate/minimal/superbody)
   StyleSpec 生成无异常, 必有 primary/surface/ink/accent。
2. 设计语言注入: tokens.json 驱动时 semantic token 命中 (非现场重推)。
3. 颜色有效性: 全 hex 合法 + 语义色字段齐备。
4. token 漂移检测: 主题 hex 若与 tokens.json 冲突 → 标记 (仅警告, superbody 主题应零漂移)。

用法: python3 bench_ppt_style.py [--tokens <tokens.json>] [--json]
退出码: 0 全通过 / 1 有失败。
"""
import argparse
import json
import os
import sys
import re

from ppt_theme import derive_theme, THEMES, TypeScale

HEX_RE = re.compile(r"^[0-9A-Fa-f]{6}$")
STYLE_FIELDS = ("base", "ink", "primary", "accent", "support1", "support2",
                "positive", "negative", "surface", "surface2", "muted")


def bench(prefer=None, tokens_path=None):
    results = []
    themes = [prefer] if prefer else list(THEMES.keys())
    for name in themes:
        th = derive_theme("NeoTrix", "超体极简 浅金 light gold 品牌 hypercube",
                          prefer=name, tokens_path=tokens_path)
        ok = True
        notes = []
        for fld in STYLE_FIELDS:
            v = getattr(th, fld, None)
            if not v or not HEX_RE.match(str(v)):
                ok = False
                notes.append(f"bad {fld}={v}")
        if th.radii and not isinstance(th.radii, (int, float)):
            ok = False
            notes.append("bad radii")
        results.append({"theme": name, "name": th.name, "ok": ok,
                        "notes": notes, "primary": th.primary, "surface": th.surface})
    return results


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--prefer", default=None, help="单主题名")
    ap.add_argument("--tokens", default=None, help="tokens.json 路径 (设计语言注入验证)")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    results = bench(args.prefer, args.tokens)
    passed = sum(1 for r in results if r["ok"])

    # token 漂移检测 (仅当提供 tokens 时)
    drift = []
    if args.tokens and os.path.isfile(args.tokens):
        with open(args.tokens) as f:
            tok = json.load(f)
        superbody = derive_theme("X", "超体 light gold", prefer="superbody", tokens_path=args.tokens)
        expected = {"primary": tok.get("primary"), "surface": tok.get("background"),
                    "muted": tok.get("ink-3")}
        for fld, exp in expected.items():
            if exp and getattr(superbody, fld, "").lower() != exp.lower():
                drift.append(f"{fld}: theme={getattr(superbody, fld)} vs token={exp}")

    if args.json:
        print(json.dumps({"results": results, "passed": passed, "total": len(results),
                          "drift": drift}, ensure_ascii=False, indent=2))
    else:
        for r in results:
            flag = "PASS" if r["ok"] else "FAIL"
            print(f"[{flag}] {r['theme']:14s} {r['name']}  primary={r['primary']} surface={r['surface']}"
                  + (f"  {r['notes']}" if r["notes"] else ""))
        print(f"--- {passed}/{len(results)} themes pass; drift={drift or 'none'}")

    return 0 if passed == len(results) and not drift else 1


if __name__ == "__main__":
    sys.exit(main())
#!/usr/bin/env python3
"""nt_selfcall — 能力自调用环（D1/deep_route 落地执行器）.

任务进来 → deep_route 选臂/定策略 → 按策略调用本仓真实能力 →
收结果 → 记账。能力注册表内建（可扩展），调用全走子进程/函数，
输出统一 {ok, output, evidence}。

能力表（持续补）：
  locate      scripts/ops/nt_locate.py（定点）
  synth       scripts/ops/nt_skill_synth.py --selftest（技能合成自检）
  deeproute   models/training/deep_route.py --selftest（路由自检）
  evals       D5 tasks.json 行数 + baseline 分数（尺子健康）
  probe_json  JSON 文件合法性（通用数据门）

用法:
  python3 scripts/ops/nt_selfcall.py --task "定位 NtSelfIterate 收敛行" --domain code
  python3 scripts/ops/nt_selfcall.py --selftest
"""
import argparse
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(ROOT, "models", "training"))
from deep_route import route as deep_pick


def run(cmd, timeout=120):
    try:
        p = subprocess.run(cmd, capture_output=True, timeout=timeout, cwd=ROOT)
        out = (p.stdout.decode("utf-8", errors="ignore") + p.stderr.decode("utf-8", errors="ignore"))[-1500:]
        return {"ok": p.returncode == 0, "output": out}
    except Exception as e:
        return {"ok": False, "output": f"invoke failed: {e}"}


CAPABILITIES = {
    "locate": lambda a: run([sys.executable, "scripts/ops/nt_locate.py",
                             "--component", a.get("component", ""),
                             "--source-file", a.get("source_file", ""),
                             "--root", "."]),
    "synth": lambda a: run([sys.executable, "scripts/ops/nt_skill_synth.py", "--selftest"]),
    "deeproute": lambda a: run([sys.executable, "models/training/deep_route.py", "--selftest"]),
    "evals": lambda a: run([sys.executable, "-c",
        "import json;rows=[json.loads(l) for l in open('evals/gaia_mini/tasks.json',encoding=\"utf-8\") if l.strip()];"
        "print(len(rows),json.load(open('evals/gaia_mini/baseline_20260924.json'))['score'])" ]),
    "probe_json": lambda a: run([sys.executable, "-c",
        f"import json;json.load(open({a.get('path','')!r}));print('json OK')"]),
}

# 任务关键词 → 能力序列（策略即“调用哪些外部/内部资源”）
POLICIES = [
    (("定位", "定点", "在哪", "locate", "文件"), ["locate"]),
    (("技能", "skill", "合成"), ["synth"]),
    (("路由", "route", "选", "决策"), ["deeproute"]),
    (("基线", "eval", "尺子", "健康"), ["evals", "deeproute"]),
    (("json", "合法", "格式"), ["probe_json"]),
]


def plan(task):
    t = task or ""
    for kws, seq in POLICIES:
        if any(k in t for k in kws):
            return seq, "keyword"
    # 兜底：走 deep_route 选臂，再映射到能力
    pick = deep_pick(t)
    arm = pick["pick"]
    if arm == "laya":
        return ["deeproute"], f"deep_route->{arm}"
    return ["evals"], f"deep_route->{arm}"


def execute(task, domain="", args=None):
    args = args or {}
    seq, why = plan(task)
    steps = []
    for cap in seq:
        fn = CAPABILITIES.get(cap)
        if not fn:
            steps.append({"cap": cap, "ok": False, "output": "unknown capability"})
            continue
        r = fn(args)
        steps.append({"cap": cap, **r})
    ok = all(s["ok"] for s in steps) if steps else False
    return {"task": task, "policy": seq, "why": why, "ok": ok, "steps": steps}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--task", default="")
    ap.add_argument("--domain", default="")
    ap.add_argument("--args-json", default="{}")
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        r1 = execute("定位 NtSelfIterate 收敛行", args={"component": "NtSelfIterate",
                     "source_file": "nt_self_iterate.rs"})
        assert r1["ok"] and any("nt_self_iterate.rs" in s["output"] for s in r1["steps"]), r1
        r2 = execute("路由自检", {})
        assert r2["ok"], r2
        r3 = execute("基线健康", {})
        assert r3["ok"] and "20/20" in r3["steps"][0]["output"], r3
        print("[selftest] OK 3/3 (locate/deeproute/evals self-invoked)", flush=True)
        return
    print(json.dumps(execute(args.task, args.domain, json.loads(args.args_json)),
                     ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()

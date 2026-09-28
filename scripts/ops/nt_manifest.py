#!/usr/bin/env python3
"""nt_manifest.py — Context Manifest：把「你是在哪个环境里验证的」变成记录。

## 为什么存在
2026-09-28 一个会话里出现 4 次「断言与实测矛盾」：
  ① 误删 ratchet worktree —— 同一命令块内两个判据互相矛盾仍下结论
  ② pre-push 收工门是死代码 —— 测逻辑（mock 三档全对）≠ 测可达性
  ③ layer-map 数字过期 —— 治理数据不随代码刷新
  ④ 采信了外部 agent 的结论，未实测就写进报告

**共同结构都是「断言与证据脱钩」**。`docs/architecture/LESSONS-20260928-fresh-checkout.md`
已写元教训：「任何『X 是好的/坏的』断言都要问『我是在哪个环境里验证的』；答『我的
工作树』就等于还没有证据」。但那是散文要求，**没有任何机制能检查它**。

本命令把该要求变成可机读记录（吸收 aliyun/ai-agent-handbook §15.9.5 的
Context Manifest 概念）：一条结论 + 它所依赖的环境指纹 + 证据来源。

## 用法
    # 记录一条结论（会连同环境指纹一起存）
    python3 scripts/ops/nt_manifest.py add "13/58 → 58/58 路径解析已修" \
        --claim skill_index_path_resolution \
        --evidence "neotrix-core/src/skill_loader.rs:267" \
        --evidence "python3 复测 index.json 58/58 命中" \
        --env clean-checkout          # 可选：声明测量台

    # 查
    python3 scripts/ops/nt_manifest.py list
    python3 scripts/ops/nt_manifest.py show skill_index_path_resolution
    python3 scripts/ops/nt_manifest.py stale            # 环境已变但结论没重验
    python3 scripts/ops/nt_manifest.py audit            # 门：file:line 是否还指向实文件
"""
import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
MANIFEST = os.path.join(REPO, ".neotrix", "context-manifest.json")


def sh(*args):
    try:
        return subprocess.run(
            args, cwd=REPO, capture_output=True, text=True, timeout=30
        ).stdout.strip()
    except Exception:
        return ""


def git(*args):
    return sh("git", *args)


def env_fingerprint():
    """当前环境的事实指纹。任何一项变化都会让旧结论进入 stale。"""
    head = git("rev-parse", "HEAD")
    dirty = [
        ln.split(None, 1)[1]
        for ln in git("status", "--porcelain").split("\n")
        if ln.strip() and not ln.startswith("?? models/training/")
    ]
    cargo_lock = os.path.isfile(os.path.join(REPO, "Cargo.lock"))
    # 只用「事实」参与指纹，不用时间戳 —— 时间戳进指纹会导致每次跑都变
    payload = json.dumps(
        {"head": head, "dirty": sorted(dirty), "cargo_lock": cargo_lock},
        sort_keys=True,
        ensure_ascii=False,
    )
    return {
        "head": head[:12],
        "dirty_count": len(dirty),
        "dirty": sorted(dirty)[:20],
        "cargo_lock": cargo_lock,
        "fingerprint": hashlib.sha256(payload.encode()).hexdigest()[:16],
    }


def load():
    if not os.path.isfile(MANIFEST):
        return {"_about": "Context Manifest（2026-09-28 立）", "claims": {}}
    with open(MANIFEST, encoding="utf-8") as f:
        return json.load(f)


def save(d):
    os.makedirs(os.path.dirname(MANIFEST), exist_ok=True)
    with open(MANIFEST, "w", encoding="utf-8") as f:
        json.dump(d, f, ensure_ascii=False, indent=2)
        f.write("\n")


def cmd_add(args):
    d = load()
    env = env_fingerprint()
    d.setdefault("claims", {})[args.claim] = {
        "statement": args.statement,
        "evidence": args.evidence,
        "env": env,
        "declared_env": args.env or "working-tree",
        "recorded_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%MZ"),
    }
    save(d)
    print(f"manifest: recorded '{args.claim}' @ {env['fingerprint']} "
          f"(head={env['head']}, dirty={env['dirty_count']})")
    for e in args.evidence:
        print(f"  evidence: {e}")
    return 0


def cmd_list(args):
    d = load()
    claims = d.get("claims", {})
    if not claims:
        print("manifest: 空（用 `nt_manifest.py add` 记录第一条）")
        return 0
    cur = env_fingerprint()["fingerprint"]
    print(f"manifest: {len(claims)} claims   当前环境 {cur}\n")
    for k, v in claims.items():
        same = v["env"]["fingerprint"] == cur
        mark = "✅" if same else "⚠️ 环境已变"
        print(f"  [{mark}] {k}")
        print(f"      {v['statement']}")
        print(f"      @{v['recorded_at']}  declared={v['declared_env']}  fp={v['env']['fingerprint']}")
    return 0


def cmd_show(args):
    d = load()
    v = d.get("claims", {}).get(args.claim)
    if not v:
        print(f"manifest: 无此 claim: {args.claim}")
        return 1
    print(json.dumps(v, ensure_ascii=False, indent=2))
    return 0


def cmd_stale(args):
    d = load()
    cur = env_fingerprint()
    stale = [
        k for k, v in d.get("claims", {}).items()
        if v["env"]["fingerprint"] != cur["fingerprint"]
    ]
    if not stale:
        print(f"manifest: 0 stale（{len(d.get('claims', {}))} claims，环境指纹 {cur['fingerprint']} 全对）")
        return 0
    print(f"manifest: {len(stale)} stale claim(s) —— 环境已变但结论未重验：")
    for k in stale:
        v = d["claims"][k]
        print(f"  - {k}  @fp {v['env']['fingerprint']} → 现 {cur['fingerprint']}")
        print(f"      旧环境: head={v['env']['head']} dirty={v['env']['dirty_count']}")
    print(f"  现环境: head={cur['head']} dirty={cur['dirty_count']}")
    print("  ⇒ 逐条重验后 `nt_manifest.py add` 覆盖，或确认失效后从 manifest 删除")
    return 1


FILELINE = re.compile(r"([\w./-]+\.(?:rs|py|sh|json|md)):(\d+)")


def cmd_audit(args):
    """门：file:line 证据是否还指向真实文件/行。

    这条比 env 漂移更重要 —— R-DISK-8 的教训是「测逻辑 ≠ 测可达性」，
    同类地「记了 file:line ≠ 它还指在那里」。行号漂了就是假证据。
    """
    d = load()
    problems = []
    checked = 0
    for k, v in d.get("claims", {}).items():
        for e in v.get("evidence", []):
            for path, line in FILELINE.findall(e):
                checked += 1
                full = os.path.join(REPO, path)
                if not os.path.isfile(full):
                    problems.append(f"{k}: 证据文件不存在 {path}")
                    continue
                n = int(line)
                with open(full, encoding="utf-8", errors="replace") as fh:
                    total = sum(1 for _ in fh)
                if n > total:
                    problems.append(
                        f"{k}: {path}:{n} 超出文件行数 {total}（行号漂移 ⇒ 假证据）"
                    )
    if problems:
        print(f"manifest --audit: {len(problems)} problem(s) / 检查 {checked} 个 file:line")
        for p in problems:
            print("  " + p)
        return 1
    print(f"manifest --audit: {len(d.get('claims', {}))} claims / {checked} 个 file:line 全部有效 ✅")
    return 0


def main():
    ap = argparse.ArgumentParser(description="Context Manifest：记录结论 + 其环境指纹 + 证据")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("add", help="记录一条结论")
    p.add_argument("statement", help="结论本身")
    p.add_argument("--claim", required=True, help="稳定 slug")
    p.add_argument("--evidence", action="append", default=[], help="证据（可多次；file:line 会被 audit 校验）")
    p.add_argument("--env", help="声明测量台，如 clean-checkout / worktree:<name>")

    p = sub.add_parser("list", help="列出全部 claim")
    p = sub.add_parser("stale", help="环境已变但结论未重验的 claim")
    p = sub.add_parser("audit", help="门：file:line 证据是否仍有效")
    p = sub.add_parser("show")
    p.add_argument("claim")

    args = ap.parse_args()
    return {
        "add": cmd_add, "list": cmd_list, "show": cmd_show,
        "stale": cmd_stale, "audit": cmd_audit,
    }[args.cmd](args)


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""nt_retry — 失败签名 → 修复指引（OmO retry-patterns 思想吸收）.

门脚本/长任务只记 EXIT 码是不够的：把已知的失败签名变成
(errorType, fixHint, autoAction)，下次命中直接给药方，严重的自动执行。

用法:
  python3 scripts/ops/nt_retry.py --match "text of failure"   # 查药方
  python3 scripts/ops/nt_retry.py --selftest
  from nt_retry import advise  # autoAction: none|retry|rebuild-gate|notify
"""
import argparse
import json
import re

PATTERNS = [
    {"pattern": r"cargo.*lock|Blocking waiting for file lock|lock.*held",
     "errorType": "cargo_lock_held",
     "fixHint": "锁被邻窗占用：退避等待，勿抢跑；门控脚本自动等。",
     "autoAction": "none"},
    {"pattern": r"EXIT:137|SIGKILL|signal 9|Out of memory|OOM",
     "errorType": "oom_killed",
     "fixHint": "被 OOM-killer 处决：查 ckpt 是否完整，用 --resume 续跑；以后 heavy 任务串行 + 内存门。",
     "autoAction": "none"},
    {"pattern": r"cdp.*timeout|Request timed out|websocket",
     "errorType": "cdp_timeout",
     "fixHint": "CDP 超时：先查 tab 是否堆积、浏览器是否还活着、页面是否半加载；重启浏览器+单 tab 重试。",
     "autoAction": "none"},
    {"pattern": r"429|rate limit|Too Many Requests",
     "errorType": "rate_limited",
     "fixHint": "被限流：退避 + 换源/降频；搜索类任务切备用 query。",
     "autoAction": "retry"},
    {"pattern": r"Singleton|profile.*lock|user-data-dir.*in use",
     "errorType": "profile_lock",
     "fixHint": "profile 文件锁：先全退 Chrome 再起自动化；或换副本目录。",
     "autoAction": "none"},
    {"pattern": r"flock|lock_exclusive.*hang|kb.*hang|hang.*kb",
     "errorType": "kb_flock_hang",
     "fixHint": "KB flock 挂起：改 try_lock 非阻塞 + 降级（见 kb_core.rs fix）；探针 kb_probe 复现。",
     "autoAction": "none"},
    {"pattern": r"non-default data directory|remote-debugging.*refus",
     "errorType": "chrome_debug_refused",
     "fixHint": "Chrome 拒默认目录调试：改副本目录 + --remote-debugging-port，或走 connect 附着活体。",
     "autoAction": "none"},
    {"pattern": r"ModuleNotFound|No module named|ImportError",
     "errorType": "missing_dep",
     "fixHint": "缺依赖：pip install --break-system-packages <pkg>（homebrew python），或查 venv。",
     "autoAction": "none"},
    {"pattern": r"E0432|E0433|E0425|cannot find|unresolved import",
     "errorType": "rust_unresolved",
     "fixHint": "Rust 未决符号：先定责（报错文件是否本窗）；邻窗热区不代修，等合上重跑 verify 门。",
     "autoAction": "none"},
    {"pattern": r"panic|assertion.*failed|FAILED",
     "errorType": "test_failure",
     "fixHint": "单测挂：读完整输出定位首个失败；本窗的修，邻窗的记 handoff 不碰。",
     "autoAction": "none"},
]


def advise(text):
    hits = []
    for p in PATTERNS:
        if re.search(p["pattern"], text or "", re.IGNORECASE):
            hits.append({"errorType": p["errorType"], "fixHint": p["fixHint"],
                         "autoAction": p["autoAction"]})
    return hits


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--match", default="")
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        assert advise("cargo test EXIT:137 killed")[0]["errorType"] == "oom_killed"
        assert advise("error[E0433]: unresolved import")[0]["errorType"] == "rust_unresolved"
        assert advise("flock hang in kb open")[0]["errorType"] == "kb_flock_hang"
        assert advise("429 POST failed")[0]["errorType"] == "rate_limited"
        assert advise("all good")[0:] == []
        print("[selftest] OK 5/5 (10 patterns loaded)", flush=True)
        return
    hits = advise(args.match)
    print(json.dumps(hits, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()

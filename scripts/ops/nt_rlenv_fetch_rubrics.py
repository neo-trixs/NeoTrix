#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""nt_rlenv_fetch_rubrics — MiMo RL 环境的 925 份 rubric + 共享判分骨架落本地.

背景（2026-09-28 实测决定本脚本只拉这两样）：
  `general/envs/` 下 925 个环境目录，每目录 ~11 个文件（verify.py / verify~
  .py / verifier_meta.json / instruction.md / manifest.json / workspace/ …），
  全量约 10000 文件 ~83MB。但抽样 3 个环境比对 sha256 发现：
    - `verify.py`   **925 份逐字节相同**（唯一哈希 1 个）→ 只需拉 1 份
    - `verifier_meta.json` 每环境唯一        → 这才是独有知识，必须全量
  故本脚本只拉 925 份 rubric（~6.8MB）+ 1 份判分骨架，省掉 ~9000 次请求。

产出（项目内，datasets/ 已 gitignore）：
  datasets/hf_raw/mimo/general/envs/{env_id}.rubric.json   925 份
  datasets/hf_raw/mimo/general/verify_shared.py            1 份（925 份的共同哈希）
  datasets/hf_raw/mimo/general/envs_index.json            目录清单 + 拉取状态

用法:
  python3 scripts/ops/nt_rlenv_fetch_rubrics.py --dry-run
  python3 scripts/ops/nt_rlenv_fetch_rubrics.py
  python3 scripts/ops/nt_rlenv_fetch_rubrics.py --selftest
"""
import argparse
import concurrent.futures as cf
import hashlib
import json
import os
import sys
import time
import urllib.error
import urllib.request

DS = "XiaomiMiMo/MiMo-V2.6-RL-oss"
RESOLVE = "https://huggingface.co/datasets/%s/resolve/main" % DS
TREE = "https://huggingface.co/api/datasets/%s/tree/main" % DS
ROOT = os.path.join("datasets", "hf_raw", "mimo", "general")
ENV_DIR = os.path.join(ROOT, "envs")
SHARED = os.path.join(ROOT, "verify_shared.py")
INDEX = os.path.join(ROOT, "envs_index.json")
UA = "NeoTrix/0.19 (nt_rlenv_fetch_rubrics.py)"
TIMEOUT = 60
WORKERS = 8          # 对 HF 保持温和并发，避免 429
RETRIES = 3


def api_get(url, timeout=TIMEOUT):
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return r.read()


def list_envs():
    """general/envs 下的一级目录名（925 个；单页 <1000 一次取全）。"""
    d = json.loads(api_get(TREE + "/general/envs"))
    return sorted(e["path"].split("/")[-1] for e in d if e.get("type") == "directory")


def valid_rubric(path):
    try:
        with open(path, encoding="utf-8") as fh:
            d = json.load(fh)
        return isinstance(d, dict) and isinstance(d.get("items"), list)
    except (OSError, ValueError):
        return False


def fetch_one(env_id, dest, force=False):
    if not force and valid_rubric(dest):
        return ("skip", env_id, "")
    url = "%s/general/envs/%s/verifier_meta.json" % (RESOLVE, env_id)
    last = ""
    for k in range(RETRIES):
        try:
            body = api_get(url)
            d = json.loads(body)
            if not isinstance(d.get("items"), list):
                last = "no items list"
                continue
            tmp = dest + ".tmp"
            with open(tmp, "wb") as fh:
                fh.write(body)
            os.replace(tmp, dest)
            return ("ok", env_id, hashlib.sha256(body).hexdigest()[:12])
        except (urllib.error.URLError, OSError, ValueError) as e:
            last = str(e)[:80]
            time.sleep(0.6 * (k + 1))
    return ("fail", env_id, last)


def selftest():
    assert WORKERS <= 16, "别把 HF 打挂"
    assert RESOLVE.endswith("resolve/main")
    assert ENV_DIR.endswith(os.path.join("general", "envs"))
    # 合法性判据：必须是 dict 且 items 是 list（拒绝 HTML 错误页/空 body）
    assert not valid_rubric("/nonexistent/x.json")
    import tempfile
    with tempfile.TemporaryDirectory() as t:
        p = os.path.join(t, "a.json")
        with open(p, "w", encoding="utf-8") as fh:
            fh.write('{"items": []}')
        assert valid_rubric(p)
        with open(p, "w", encoding="utf-8") as fh:
            fh.write("<html>429</html>")
        assert not valid_rubric(p), "错误页必须判为非法"
        with open(p, "w", encoding="utf-8") as fh:
            fh.write('{"items": {}}')
        assert not valid_rubric(p), "items 非 list 必须判为非法"
    print("selftest ok: url-shape/validity-gate(拒错误页与错类型)")
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--force", action="store_true")
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        return selftest()

    envs = list_envs()
    if args.limit:
        envs = envs[:args.limit]
    print("envs discovered: %d  (general/envs)" % len(envs))
    if args.dry_run:
        print("dry-run: 将拉 %d 份 verifier_meta.json + 1 份 verify.py" % len(envs))
        return 0

    os.makedirs(ENV_DIR, exist_ok=True)
    t0 = time.time()
    results = []
    with cf.ThreadPoolExecutor(max_workers=WORKERS) as ex:
        futs = {ex.submit(fetch_one, e, os.path.join(ENV_DIR, e + ".rubric.json"),
                          args.force): e for e in envs}
        for i, f in enumerate(cf.as_completed(futs), 1):
            results.append(f.result())
            if i % 100 == 0:
                print("  %d/%d  (%.0fs)" % (i, len(envs), time.time() - t0))
    ok = [r for r in results if r[0] == "ok"]
    sk = [r for r in results if r[0] == "skip"]
    fl = [r for r in results if r[0] == "fail"]

    # 共享判分骨架（925 份 verify.py 相同，抽首份即可）
    shared_sha = ""
    if not os.path.exists(SHARED):
        body = api_get("%s/general/envs/%s/verify.py" % (RESOLVE, envs[0]))
        tmp = SHARED + ".tmp"
        with open(tmp, "wb") as fh:
            fh.write(body)
        os.replace(tmp, SHARED)
        shared_sha = hashlib.sha256(body).hexdigest()
        print("verify_shared.py: %d bytes  sha=%s" % (len(body), shared_sha[:12]))
    else:
        shared_sha = hashlib.sha256(open(SHARED, "rb").read()).hexdigest()

    index = {
        "dataset": DS, "envs": len(envs),
        "rubrics_ok": len(ok), "rubrics_skipped": len(sk), "rubrics_failed": len(fl),
        "verify_shared_sha256": shared_sha,
        "note": "verify.py 925 份逐字节相同（抽样 3 环境 sha256 唯一值=1），故只存 1 份",
        "failed": [{"env": e, "err": m} for _s, e, m in fl],
    }
    tmp = INDEX + ".tmp"
    with open(tmp, "w", encoding="utf-8") as fh:
        json.dump(index, fh, ensure_ascii=False, indent=1)
    os.replace(tmp, INDEX)
    tot = sum(os.path.getsize(os.path.join(ENV_DIR, f)) for f in os.listdir(ENV_DIR))
    print("done: ok=%d skip=%d fail=%d  rubrics=%.1fMB  %.0fs"
          % (len(ok), len(sk), len(fl), tot / 1e6, time.time() - t0))
    print("index: %s" % INDEX)
    if fl:
        for _s, e, m in fl[:10]:
            print("  FAIL %s: %s" % (e, m))
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main() or 0)

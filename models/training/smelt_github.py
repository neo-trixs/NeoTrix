#!/usr/bin/env python3
"""Trendshift+用户名单 GitHub 元数据熔炼（GitHub API 未认证 60/hr，65s 起步）.

输入: models/training/smelt/repo_index.json {repo: {boards,...}}
输出:
- models/training/repo_cards.jsonl   crawl_queue 消费格式 {url,title,content,domain}
- models/training/repo_meta.jsonl    原始元数据 sidecar（stars/lang/topics/license/updated）
- models/training/smelt/smelt_progress.json     断点续跑（已抓集合）
用法: python3 models/training/smelt_github.py [--limit N]
"""
import argparse
import json
import os
import subprocess
import sys
import time

BASE = "https://api.github.com/repos/"
PACE = 65.0


def fetch(repo):
    # urllib 直连被防火墙杀，用 curl 走系统代理（已验证通）
    try:
        p = subprocess.run(
            ["curl", "-s", "-D", "-", "--max-time", "30",
             "-H", "User-Agent: NeoTrix-smelt/1.0",
             "-H", "Accept: application/vnd.github+json",
             BASE + repo],
            capture_output=True,
            timeout=45,
        )
        out = p.stdout.decode("utf-8", errors="ignore")
        remaining, reset, body = 60, 0, "{}"
        if "\r\n\r\n" in out:
            head, body = out.split("\r\n\r\n", 1)
        elif "\n\n" in out:
            head, body = out.split("\n\n", 1)
        else:
            head, body = out, "{}"
        for line in head.splitlines():
            ll = line.lower()
            if ll.startswith("x-ratelimit-remaining:"):
                remaining = int(ll.split(":", 1)[1].strip() or 60)
            elif ll.startswith("x-ratelimit-reset:"):
                reset = int(ll.split(":", 1)[1].strip() or 0)
        meta = json.loads(body) if body.strip().startswith("{") else {"_error": body[:120]}
        if isinstance(meta, dict) and meta.get("message", "").startswith("API rate limit"):
            return {"_error": "rate-limited"}, 0, reset
        return meta, remaining, reset
    except Exception as e:
        return {"_error": str(e)[:120]}, 999, 0


def card(repo, meta, boards):
    desc = (meta.get("description") or "").strip()
    topics = ", ".join(meta.get("topics") or [])
    content = (
        f"[{repo}] {desc}\n"
        f"stars={meta.get('stargazers_count', '?')} lang={meta.get('language') or '?'} "
        f"license={(meta.get('license') or {}).get('spdx_id') or '?'} "
        f"updated={meta.get('updated_at', '?')[:10]} topics={topics}"
    )
    return {
        "url": f"github://{repo}",
        "title": repo,
        "content": content,
        "domain": "github-smelt",
        "boards": boards,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--limit", type=int, default=0)
    args = ap.parse_args()
    root = os.getcwd()
    idx = json.load(open(os.path.join(os.path.dirname(__file__), "smelt/repo_index.json")))
    repos = [r for r in idx if not r.startswith("#")]
    if args.limit:
        repos = repos[: args.limit]
    prog_path = os.path.join(os.path.dirname(__file__), "smelt/smelt_progress.json")
    done = set(json.load(open(prog_path)) if os.path.exists(prog_path) else [])
    cards_f = open(os.path.join(root, "models/training/repo_cards.jsonl"), "a", encoding="utf-8")
    meta_f = open(os.path.join(root, "models/training/repo_meta.jsonl"), "a", encoding="utf-8")
    n_new = n_err = 0
    for i, repo in enumerate(repos):
        if repo in done:
            continue
        meta, remaining, reset = fetch(repo)
        if "_error" in meta:
            n_err += 1
            print(f"[{i}/{len(repos)}] {repo} ERR {meta['_error']}", flush=True)
        else:
            boards = idx[repo].get("boards", [])
            cards_f.write(json.dumps(card(repo, meta, boards), ensure_ascii=False) + "\n")
            meta_f.write(json.dumps({"repo": repo, "boards": boards, "meta": meta}) + "\n")
            n_new += 1
            print(
                f"[{i}/{len(repos)}] {repo} ★{meta.get('stargazers_count')} "
                f"{meta.get('language')} ({remaining} left)",
                flush=True,
            )
        done.add(repo)
        if (n_new + n_err) % 10 == 0:
            json.dump(sorted(done), open(prog_path, "w"))
        if remaining is not None and remaining < 5:
            wait = max(reset - int(time.time()) + 5, 60)
            print(f"[smelt] rate low, sleep {wait}s", flush=True)
            time.sleep(wait)
        else:
            time.sleep(PACE)
    json.dump(sorted(done), open(prog_path, "w"))
    cards_f.close()
    meta_f.close()
    print(f"[smelt] done: new={n_new} err={n_err}", flush=True)


if __name__ == "__main__":
    main()

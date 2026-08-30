#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
NeoTrix 批吸收器 (Batch Absorber) — 批量 URL → 知识库
============================================================
从 URL 列表文件抓取信息并入库, 复用阵法引擎的代理/网络能力。
节点格式与已有库一致:
  - github.com  → node_type='repository', 走 GitHub API 拿 stars/language/topics
  - arxiv.org   → node_type='paper', 走 export.arxiv.org API 拿 Abstract
  - 其他站点    → node_type='article', 抓 HTML 提取正文

架构 (R-P97, Cycle 232): 本脚本仅做【数据 prep】— 并发抓取/解析/组装 node 字段;
知识写入 (nodes/nodes_fts 双写 + URL 去重) 全部委托 Rust CLI
`neotrix-experience absorb-node` (单一事实源, 防 PA011 FTS desync)。
本地 insert_node 已退役删除。

用法:
  python3 scripts/kb_batch_absorb.py --urls /tmp/missing_urls.txt        # 入库
  python3 scripts/kb_batch_absorb.py --urls /tmp/missing_urls.txt --dry-run
  python3 scripts/kb_batch_absorb.py --urls X --limit 10                 # 只跑前 10
"""

import argparse
import concurrent.futures
import json
import os
import re
import subprocess
import sys
import tempfile
import threading
import time
import urllib.parse

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

UA = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36"
BLOCKED_DOMAINS = {
    'books.google.co.jp', 'books.google.com',
    'web.archive.org', 'web.archive.org',
}

# ── R-P97: 知识写入单一事实源 — Rust CLI (neotrix-experience absorb-node) ──
# Python 仅做数据 prep (抓取/解析/组装 node 字段), 不再直接写 nodes/nodes_fts。
# 语义对齐 kb_batch_absorb.py 原 insert_node: URL 去重 + FTS 显式双写 (防 PA011 desync)。
RUST_BIN = os.environ.get("NEOTRIX_EXPERIENCE_BIN", "neotrix-experience")


def rust_absorb_nodes(nodes, dry_run=False):
    """把 node 数组交给 Rust CLI 批量写入。返回 (inserted, duplicated) 统计。"""
    if not nodes:
        return (0, 0)
    tmp = tempfile.NamedTemporaryFile(
        mode='w', suffix='.json', prefix='nt_rust_', delete=False, encoding='utf-8')
    with tmp:
        json.dump(nodes, tmp, ensure_ascii=False)
    cmd = [RUST_BIN, 'absorb-node', tmp.name]
    if dry_run:
        cmd.append('--dry-run')
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    except FileNotFoundError:
        print(f"  ✗ {RUST_BIN} 未找到 — 请先 cargo build --release --bin neotrix-experience 并安装到 PATH",
              flush=True)
        return (0, 0)
    out = (r.stdout or '') + (r.stderr or '')
    for line in out.splitlines():
        if 'absorb-node' in line:
            print(f"  {line}", flush=True)
    inserted = duplicated = 0
    m = re.search(r"(\d+) inserted, (\d+) duplicated", out)
    if m:
        inserted, duplicated = int(m.group(1)), int(m.group(2))
    return (inserted, duplicated)


def url_valid(url):
    """HEAD request with redirect follow; discard 404/410/403 immediately."""
    domain = urllib.parse.urlparse(url).netloc
    if domain in BLOCKED_DOMAINS:
        return False
    try:
        r = subprocess.run(
            ['curl', '-s', '-o', '/dev/null', '-w', '%{http_code}',
             '-L', '--connect-timeout', '5', '--max-time', '10',
             '-A', UA, '-I', url],
            capture_output=True, text=True, timeout=15)
        code = r.stdout.strip()
        return code in ('200', '301', '302', '307', '308')
    except Exception:
        return False


_curl_counter = 0


def curl(url, timeout=15, headers=None):
    """Fetch a URL body. Each call uses a unique temp file to avoid
    concurrent-writer race (shared /tmp/_batch_out corrupted node data
    under ThreadPoolExecutor workers)."""
    global _curl_counter
    _curl_counter += 1
    out = f'/tmp/_batch_out_{os.getpid()}_{_curl_counter}_{threading.get_ident()}'
    cmd = ['curl', '-s', '-L', '-m', str(timeout), '--connect-timeout', '10',
           '-A', UA, '-o', out, '-w', '%{http_code}', url]
    if headers:
        for k, v in headers.items():
            cmd += ['-H', f'{k}: {v}']
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout + 15)
        if r.stdout.strip() == '200':
            try:
                with open(out, encoding='utf-8', errors='ignore') as f:
                    body = f.read()
                if body.strip():
                    return body
            except OSError:
                pass
    except (subprocess.TimeoutExpired, OSError):
        pass
    finally:
        try:
            os.remove(out)
        except OSError:
            pass
    return None


def norm_github(url):
    m = re.match(r'https?://github\.com/([^/]+)/([^/]+)', url)
    if m:
        return m.group(1), m.group(2)
    return None, None


def fetch_github_repo(url):
    owner, repo = norm_github(url)
    if not owner:
        return None
    api = f'https://api.github.com/repos/{owner}/{repo}'
    body = curl(api, headers={'Accept': 'application/vnd.github+json'})
    if not body:
        return fetch_github_html(url, owner, repo)
    try:
        d = json.loads(body)
    except json.JSONDecodeError:
        return fetch_github_html(url, owner, repo)
    if 'full_name' not in d:
        return None
    title = d.get('name', repo)
    desc = d.get('description') or ''
    lang = d.get('language') or 'en'
    stars = d.get('stargazers_count') or 0
    topics = d.get('topics') or []
    license_name = (d.get('license') or {}).get('spdx_id', '')
    default_branch = d.get('default_branch', 'main')
    # README
    readme = ''
    for branch in (default_branch, 'main', 'master'):
        raw = curl(f'https://raw.githubusercontent.com/{owner}/{repo}/{branch}/README.md', timeout=12)
        if raw:
            readme = raw
            break
    if not readme:
        rm = curl(f'https://raw.githubusercontent.com/{owner}/{repo}/{default_branch}/readme.md', timeout=12)
        if rm:
            readme = rm
    if len(readme) > 4000:
        readme = readme[:4000]
    meta = {
        'stars': stars, 'language': lang, 'topics': topics, 'owner': owner,
        'forks': d.get('forks_count') or 0, 'license': license_name,
        'default_branch': default_branch,
        'updated_at': d.get('updated_at', ''),
        'enriched_at': int(time.time()),
    }
    return {
        'node_type': 'repository', 'title': title, 'language': lang,
        'summary': f'Software repository: {title}. {desc}'.strip(),
        'content': readme or f'{title} is a software repository from github.com. Contains source code, documentation, and related project resources.',
        'url': f'https://github.com/{owner}/{repo}', 'domain': 'github.com',
        'importance': min(1.0, max(0.5, stars / 200000.0)),
        'meta': json.dumps(meta, ensure_ascii=False),
    }


def fetch_github_html(url, owner, repo):
    """HTML fallback when GitHub API is rate-limited. Uses OG meta + README raw."""
    page = curl(f'https://github.com/{owner}/{repo}', timeout=15)
    if not page:
        return None
    def og(name):
        m = re.search(rf'property=["\']og:{name}["\'][^>]+content=["\']([^"\']+)', page)
        if not m:
            m = re.search(rf'content=["\']([^"\']+)["\'][^>]+property=["\']og:{name}["\']', page)
        return m.group(1).strip() if m else ''
    desc = og('description')
    title = og('title') or repo
    readme = ''
    for branch in ('main', 'master'):
        raw = curl(f'https://raw.githubusercontent.com/{owner}/{repo}/{branch}/README.md', timeout=12)
        if raw:
            readme = raw
            break
    meta = {
        'owner': owner, 'language': og('language') or 'en',
        'enriched_at': int(time.time()), 'html_fallback': True,
    }
    if len(readme) > 4000:
        readme = readme[:4000]
    return {
        'node_type': 'repository', 'title': title, 'language': meta['language'],
        'summary': f'Software repository: {title}. {desc}'.strip(),
        'content': readme or f'{title} is a software repository from github.com. Contains source code, documentation, and related project resources.',
        'url': f'https://github.com/{owner}/{repo}', 'domain': 'github.com',
        'importance': 0.5,
        'meta': json.dumps(meta, ensure_ascii=False),
    }


def fetch_arxiv(url, mirror_source=None):
    m = re.search(r'arxiv\.org/(?:abs|pdf)/([0-9]+(?:\.[0-9]+)?)', url)
    if not m:
        return None
    aid = m.group(1)
    api = f'https://export.arxiv.org/api/query?id_list={aid}'
    body = curl(api, timeout=20)
    if not body:
        return None
    def tag(name):
        mm = re.search(rf'<{name}[^>]*>(.*?)</{name}>', body, re.S)
        return re.sub(r'\s+', ' ', mm.group(1)).strip() if mm else ''
    title = tag('title')
    if title.startswith('arXiv Query'):
        mm = re.findall(r'<title[^>]*>(.*?)</title>', body, re.S)
        title = re.sub(r'\s+', ' ', mm[1]).strip() if len(mm) > 1 else title
    abstract = tag('summary')
    if not title or not abstract:
        return None
    meta = {'arxiv_id': aid, 'enriched_at': int(time.time())}
    if mirror_source:
        # W0.2: 镜像源 (paperswithcode.co / alphaxiv.org) 归并到 canonical arxiv URL,
        # 记录来源防信息丢失 (Cycle 232 redirected_from 先例)
        meta['mirror_source'] = mirror_source
        meta['redirected_from'] = url
    return {
        'node_type': 'paper', 'title': title, 'language': 'en',
        'summary': f'arXiv paper {aid}: {title}',
        'content': f'Abstract: {abstract}',
        'url': f'https://arxiv.org/abs/{aid}', 'domain': 'arxiv.org',
        'importance': 0.8,
        'meta': json.dumps(meta, ensure_ascii=False),
    }


# W0.2: arxiv 论文镜像域 — 同 ID 论文经这些域名发布, 必须归并到 canonical
# arxiv.org/abs/<id> 否则产生平行节点 (batch3 2026-08-26: 2608.23552 双节点教训)
ARXIV_MIRROR_DOMAINS = ('paperswithcode.co', 'alphaxiv.org', 'paperswithcode.com',
                        'huggingface.co', 'openreview.net')


def extract_arxiv_id(url):
    """从 arxiv.org 或其镜像 URL 提取 arxiv ID; 非论文 URL 返回 None。"""
    m = re.search(r'arxiv\.org/(?:abs|pdf)/([0-9]+(?:\.[0-9]+)?)', url)
    if m:
        return m.group(1)
    host = urllib.parse.urlparse(url).netloc.replace('www.', '').lower()
    if any(host == d or host.endswith('.' + d) for d in ARXIV_MIRROR_DOMAINS):
        # 镜像路径形态: /paper/<id> /abs/<id> /papers/<id>
        m = re.search(r'(?:/paper/|/abs/|/papers/)([0-9]{4}\.[0-9]{4,5})(?:v[0-9]+)?', url)
        if m:
            return m.group(1)
    return None


def fetch_article(url):
    body = curl(url, timeout=15)
    if not body:
        return None
    t = re.search(r'<title[^>]*>(.*?)</title>', body, re.S)
    title = re.sub(r'\s+', ' ', t.group(1)).strip() if t else url
    desc = re.search(r'<meta[^>]+name=["\']description["\'][^>]+content=["\']([^"\']+)', body)
    if not desc:
        desc = re.search(r'<meta[^>]+content=["\']([^"\']+)["\'][^>]+name=["\']description["\']', body)
    dtext = re.sub(r'\s+', ' ', desc.group(1)).strip() if desc else ''
    # strip HTML tags from title
    title = re.sub(r'<[^>]+>', '', title)
    text = re.sub(r'<script[\s\S]*?</script>|<style[\s\S]*?</style>', ' ', body)
    text = re.sub(r'<[^>]+>', ' ', text)
    text = re.sub(r'[ \t]+', ' ', text)
    lines = [l.strip() for l in text.split('\n') if len(l.strip()) > 30]
    content = '\n'.join(lines[:25])[:4000]
    if not content and dtext:
        content = dtext[:2000]
    if len(content) < 60:
        return None
    return {
        'node_type': 'article', 'title': title, 'language': 'en',
        'summary': dtext or f'Article: {title}',
        'content': content, 'url': url,
        'domain': urllib.parse.urlparse(url).netloc.replace('www.', ''),
        'importance': 0.5,
        'meta': json.dumps({'enriched_at': int(time.time())}, ensure_ascii=False),
    }


def fetch_one(url, skip_prefilter=False):
    if not skip_prefilter and not url_valid(url):
        return None
    if 'github.com' in url:
        return fetch_github_repo(url)
    if 'arxiv.org' in url:
        return fetch_arxiv(url)
    # W0.2: 镜像域论文 → canonical arxiv 节点 (Rust CLI URL 去重据此判重)
    aid = extract_arxiv_id(url)
    if aid:
        host = urllib.parse.urlparse(url).netloc.replace('www.', '')
        return fetch_arxiv(f'https://arxiv.org/abs/{aid}', mirror_source=host)
    return fetch_article(url)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--urls', default='/tmp/missing_urls.txt')
    ap.add_argument('--limit', type=int, default=0)
    ap.add_argument('--dry-run', action='store_true')
    ap.add_argument('--workers', type=int, default=8)
    ap.add_argument('--skip-prefilter', action='store_true',
                    help='input already HEAD-prefiltered externally; skip sequential prefilter & per-URL HEAD check')
    args = ap.parse_args()

    urls = [l.strip() for l in open(args.urls) if l.strip()]
    if args.limit:
        urls = urls[:args.limit]
    if args.skip_prefilter:
        print(f'[batch] {len(urls)} URLs (prefilter skipped)', flush=True)
    else:
        # Pre-filter: discard dead URLs via HEAD request before entering fetch pipeline
        pre_filtered = []
        pre_failed = 0
        for u in urls:
            if url_valid(u):
                pre_filtered.append(u)
            else:
                pre_failed += 1
        urls = pre_filtered
        print(f'[batch] {len(urls)} URLs after pre-filter (-{pre_failed} dead/404)', flush=True)

    # W0.1 (batch3-2026-08-26): inserted/duplicate 唯一来源 = Rust CLI 真实写回计数。
    # 旧 bug: fetch 成功时即计 'inserted', flush 时又累加 CLI 计数 → 双计 (47 源报 94)。
    stats = {'url_total': len(urls), 'fetched': 0, 'inserted': 0,
             'duplicate': 0, 'in_batch_dup': 0, 'failed': 0}
    pending: list = []
    seen_urls: set = set()
    done = 0
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.workers) as ex:
        futs = {ex.submit(fetch_one, u, args.skip_prefilter): u for u in urls}
        for fut in concurrent.futures.as_completed(futs):
            u = futs[fut]
            done += 1
            try:
                node = fut.result()
            except Exception as e:
                node = None
                print(f'  ✗ {u[:70]}  (exception {e})', flush=True)
            if node:
                # W0.2: 批内 canonical URL 去重 — 多镜像归并后同批只保留首个
                if node['url'] in seen_urls:
                    stats['in_batch_dup'] += 1
                    print(f'  = [in-batch dup] {node["title"][:60]} → {node["url"][:60]}', flush=True)
                    continue
                seen_urls.add(node['url'])
                pending.append(node)
                tag = '◇' if args.dry_run else '≈'
                print(f'  {tag} [{node["domain"]}] {node["title"][:60]}', flush=True)
                stats['fetched'] += 1
            else:
                stats['failed'] += 1
                print(f'  ✗ {u[:70]}', flush=True)
            # 每批 50 节点交给 Rust CLI 写入 (R-P97 单一事实源), 控内存与进度
            if len(pending) >= 50:
                ins, dup = rust_absorb_nodes(pending, args.dry_run)
                stats['inserted'] += ins
                stats['duplicate'] += dup
                pending = []

    # 尾批
    if pending:
        ins, dup = rust_absorb_nodes(pending, args.dry_run)
        stats['inserted'] += ins
        stats['duplicate'] += dup

    mode = ' (dry-run)' if args.dry_run else ''
    print(f'\n[batch] done{mode}: urls={stats["url_total"]} fetched={stats["fetched"]} '
          f'inserted={stats["inserted"]} duplicate={stats["duplicate"]} '
          f'in_batch_dup={stats["in_batch_dup"]} failed={stats["failed"]}', flush=True)
    # 对账不变量: fetched == inserted + duplicate + (CLI 未返回部分应为 0)
    mismatch = stats['fetched'] - stats['inserted'] - stats['duplicate']
    if mismatch != 0 and not args.dry_run:
        print(f'[batch] ⚠ 对账差额 {mismatch} — 检查 Rust CLI 输出解析 (R-P16)', flush=True)


if __name__ == '__main__':
    main()

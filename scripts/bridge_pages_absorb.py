#!/usr/bin/env python3
"""
Bridge absorber: absorb *.github.io URLs via GitHub API channel (bypass GitHub Pages CDN).

当 GitHub Pages CDN (185.199.x) 被网络出口白名单阻断时，站点内容 = 对应 repo 源码
(has_pages 站点的部署源)。经 api.github.com contents API 拉取源文件，组装 node 字段。

架构 (R-P97, Cycle 232): 本脚本仅做【数据 prep】— 经 GitHub API 拉源 + 组装 node 字段;
知识写入 (nodes/nodes_fts 双写 + URL 去重 + capability 四元组) 委托 Rust CLI
`neotrix-experience absorb-node` (单一事实源)。本地 insert_node/apply_capability 已退役删除。

用法:
  python3 scripts/bridge_pages_absorb.py --dry-run                    # 预览
  python3 scripts/bridge_pages_absorb.py                              # 入库
  python3 scripts/bridge_pages_absorb.py --config pages.json --apply  # 写 capability

配置 (JSON):
[
  {
    "url": "https://example.github.io/project/",
    "owner": "org", "repo": "project",
    "mode": "static",              # static: 从 markdown 源取正文
    "content_paths": ["article.md", "README.md"],
    "index_path": "index.html",    # title 提取 (可省略 → 用 url)
    "language": "en",
    "summary": "...",
    "importance": 0.7,
    "meta_extra": {"author": "..."},
    "capability": {"branch": "NT-MIND", "capability": "generate", "evidence": "..."}
  },
  {
    "url": "https://...github.io/spa/",
    "mode": "spa",                 # spa: 从 TS/TSX 字符串字面量取正文
    "content_paths": ["src/content/chapter1.ts"],
    ...
  }
]
"""
import argparse, base64, json, os, re, subprocess, tempfile, time

# R-P97: 知识写入单一事实源 — Rust CLI (neotrix-experience absorb-node)
RUST_BIN = os.environ.get('NEOTRIX_EXPERIENCE_BIN', 'neotrix-experience')
PROXY = os.environ.get('NEOTRIX_PROXY', 'http://127.0.0.1:1082')
API = 'https://api.github.com'


def get_token():
    try:
        return subprocess.check_output(
            ['security', 'find-internet-password', '-s', 'github.com', '-w']
        ).decode().strip()
    except Exception:
        return os.environ.get('GITHUB_TOKEN', '')


TOKEN = get_token()


def api(path):
    r = subprocess.run(
        ['curl', '-s', '-m', '20', '-x', PROXY,
         '-H', f'Authorization: Bearer {TOKEN}',
         '-H', 'Accept: application/vnd.github+json',
         API + path],
        capture_output=True, text=True)
    return json.loads(r.stdout)


def api_text(owner, repo, path):
    d = api(f'/repos/{owner}/{repo}/contents/{path}')
    if isinstance(d, dict) and 'content' in d:
        return base64.b64decode(d['content']).decode('utf-8', errors='replace')
    return ''


def md_to_content(md):
    """markdown → 可读文本 (strip 格式符/图片/链接语法)."""
    content = re.sub(r'!\[[^\]]*\]\([^)]*\)', '', md)
    content = re.sub(r'\[([^\]]+)\]\([^)]*\)', r'\1', content)
    content = re.sub(r'[#>*_`~|:]{1,}', ' ', content)
    content = re.sub(r'[ \t]+', ' ', content)
    lines = [l.strip() for l in content.split('\n') if len(l.strip()) > 30]
    return '\n'.join(lines[:25])[:4000]


def ts_to_content(ts):
    """TS/TSX 字符串字面量 → 正文 (SPA 站点正文嵌在代码里)."""
    strings = re.findall(r'"([^"\\]*(?:\\.[^"\\]*)*)"', ts)
    strings = [s for s in strings if len(s) > 30]
    return '\n'.join(strings[:25])[:4000]


def build_node(cfg):
    url = cfg['url']
    owner, repo = cfg['owner'], cfg['repo']
    mode = cfg.get('mode', 'static')

    # title: 从 index.html 提取, 否则用 url
    title = url
    index_path = cfg.get('index_path')
    if index_path:
        index = api_text(owner, repo, index_path)
        t = re.search(r'<title[^>]*>(.*?)</title>', index, re.S)
        if t:
            title = re.sub(r'\s+', ' ', t.group(1)).strip()
            title = re.sub(r'<[^>]+>', '', title)
    if not title or title == url:
        title = cfg.get('title', url)

    # content: 按 mode 提取, 多个 path 顺序取第一个非空
    content = ''
    for p in cfg.get('content_paths', []):
        raw = api_text(owner, repo, p)
        if not raw:
            continue
        if mode == 'spa':
            content = ts_to_content(raw)
        else:
            content = md_to_content(raw)
        if len(content) >= 60:
            break
    if len(content) < 60 and index_path:
        content = strip_tags(api_text(owner, repo, index_path))

    if len(content) < 60:
        content = cfg.get('summary', '')[:2000]

    meta = {'enriched_at': int(time.time()), 'bridge': 'github-api'}
    meta.update(cfg.get('meta_extra', {}))
    return {
        'node_type': cfg.get('node_type', 'article'),
        'title': title, 'language': cfg.get('language', 'en'),
        'summary': cfg.get('summary', f'Article: {title}'),
        'content': content, 'url': url,
        'domain': cfg.get('domain', urllib_parse_netloc(url)),
        'importance': cfg.get('importance', 0.5),
        'meta': json.dumps(meta, ensure_ascii=False),
    }


def strip_tags(html):
    text = re.sub(r'<script[\s\S]*?</script>|<style[\s\S]*?</style>', ' ', html)
    text = re.sub(r'<[^>]+>', ' ', text)
    text = re.sub(r'[ \t]+', ' ', text)
    lines = [l.strip() for l in text.split('\n') if len(l.strip()) > 30]
    return '\n'.join(lines[:25])[:4000]


def urllib_parse_netloc(url):
    from urllib.parse import urlparse
    return urlparse(url).netloc.replace('www.', '')


def rust_absorb_nodes(nodes, dry_run=False, apply_capability=False):
    """把 node 数组交给 Rust CLI 批量写入 (R-P97 单一事实源)。

    Rust CLI 原生支持: URL 去重 + nodes/nodes_fts 双写 + capability 四元组写入。
    返回 (inserted, duplicated)。
    """
    if not nodes:
        return (0, 0)
    tmp = tempfile.NamedTemporaryFile(
        mode='w', suffix='.json', prefix='nt_rust_', delete=False, encoding='utf-8')
    with tmp:
        json.dump(nodes, tmp, ensure_ascii=False)
    cmd = [RUST_BIN, 'absorb-node', tmp.name]
    if dry_run:
        cmd.append('--dry-run')
    if apply_capability:
        cmd.append('--apply-capability')
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    except FileNotFoundError:
        print(f"  ✗ {RUST_BIN} 未找到 — 请先构建并安装 neotrix-experience",
              flush=True)
        return (0, 0)
    out = (r.stdout or '') + (r.stderr or '')
    for line in out.splitlines():
        if 'absorb-node' in line:
            print(f"  {line}", flush=True)
    m = re.search(r"(\d+) inserted, (\d+) duplicated", out)
    if m:
        return (int(m.group(1)), int(m.group(2)))
    return (0, 0)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--config', default='scripts/pages-bridge-config.json',
                    help='JSON 配置文件 (站点清单)')
    ap.add_argument('--dry-run', action='store_true')
    ap.add_argument('--apply', action='store_true',
                    help='同时写 capability 映射')
    args = ap.parse_args()

    if not os.path.exists(args.config):
        print(f"config not found: {args.config}")
        sys_exit(1)

    configs = json.load(open(args.config))
    nodes = []
    for cfg in configs:
        node = build_node(cfg)
        print(f"\n=== {node['url']}")
        print(f"  title: {node['title'][:70]}")
        print(f"  content_len: {len(node['content'])}B")
        if cfg.get('capability'):
            node['capability'] = cfg['capability']
        nodes.append(node)
    ins, dup = rust_absorb_nodes(nodes, dry_run=args.dry_run,
                                 apply_capability=args.apply)
    print(f"\nDone. inserted={ins}, duplicated={dup}")


def sys_exit(code):
    import sys
    sys.exit(code)


if __name__ == '__main__':
    main()

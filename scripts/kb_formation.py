#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
NeoTrix 阵法节点引擎 (Formation Node Engine) v0.2 — 自我组建能力网
============================================================
自主爬取数据补齐知识库全景。七个阵法节点协同运作，通用去噪 + 能力自我组建:

  ① 观阵 GapScan       通用扫描: 所有内容域(中文/英文/URL主机/平台/实体)分类后补缺
  ② 布阵 PlanTree      按域类型组建能力链 (CapabilityRegistry 自我组建最优信息获取能力)
  ③ 攻阵 Crawler       多源爬取: 内容域用维基/联想/搜索; URL域用 OSINT(crt.sh/wayback/http)
  ④ 收阵 Mint          通用去噪 + 3D 结构化 (原理·表象·数) + 去重入库
  ⑤ 验阵 Verify        FTS5 重建 + 域分布统计
  ⑥ 疗阵 Heal          能力健康度追踪 / 死源标记 / 节流退避 / 代理池刷新
  ⑦ 振阵 Loop          自调度循环 (每 cycle 处理 N 个主题)

能力自我组建 (Self-Assembly):
  - CapabilityRegistry 映射代码库已有节点 (nt_world_osint 9 子模块 + 网络源)
  - 每个能力有健康度 (ok/fail/cooldown), 布阵时按健康度排序组建最优链
  - URL主机域 → dns(crt.sh) → http(指纹) → url(wayback) → osv(漏洞) 组建 OSINT 链
  - 内容域     → zh_wiki(代理) → baidu_sug → sogou → en_wiki 组建内容链
  - 能力缺失时自动尝试替代源 (gap filling)

网络拓扑:
  - 直连可用:  360搜索 (so.com), 搜狗 (sogou.com), 百度联想 (sugrec),
               GitHub API, crt.sh, wayback, osv.dev, DDG html
  - 代理绕路:  zh.wikipedia.org / en.wikipedia.org (需 SOCKS5 代理)
  - 代理池:    ~/.neotrix/proxy-upstreams.conf (157+ 节点, 动态刷新)

用法:
  python3 scripts/kb_formation.py --dry-run            # 演练一轮不写库
  python3 scripts/kb_formation.py --topics N           # 每 cycle 最多 N 主题
  python3 scripts/kb_formation.py --domains 生态学 3ds.com  # 指定域(任意类型)
  python3 scripts/kb_formation.py --loop --interval 600 # 守护循环
"""

import argparse
import hashlib
import json
import os
import random
import re
import sqlite3
import subprocess
import sys
import threading
import time
import uuid

KB_PATH = os.path.expanduser("~/.neotrix/knowledge.db")
PROXY_CONF = os.path.expanduser("~/.neotrix/proxy-upstreams.conf")
STATE_PATH = os.path.expanduser("~/.neotrix/kb_formation_state.json")
LOG_PATH = os.path.expanduser("~/.neotrix/kb_formation_log.jsonl")
UA = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36"
MIN_CONTENT_LEN = 180
MIN_DOMAIN_TARGET = 3
IMPORTANCE_DEFAULT = 0.88

# ────────────────────────────────────────────────────────────────
# 〇 通用去噪器 Denoiser — 所有内容域类型的噪声过滤 (不再仅中文)
# ────────────────────────────────────────────────────────────────
DOMAIN_NOISE = {
    'spam': {
        # 域名本身是垃圾/噪声信号
        'regex': [
            r'\.(?:xyz|top|club|vip|gq|ml|ga|cf|tk)$',   # 免费垃圾域名后缀
            r'^(?:www\.)?\d+\.\d+\.\d+\.\d+$',           # 纯 IP
        ],
        'substr': ['tracker', 'redirect', 'ads', 'doubleclick', 'analytics'],
    },
    'platform': {
        # 平台/社交域名 — 不是知识内容域, 但可作 OSINT 目标
        'regex': [
            r'^(reddit|youtube|twitter|x\.com|facebook|instagram|linkedin|tiktok|twitch|telegram|medium|github|gitlab|dev\.to|keybase|discord)$',
        ],
        'substr': ['.com', '.org', '.net', '.io', '.co', '.cn', '.de', '.fr', '.jp'],
    },
}

# 中文字符范围: 用于域类型分类
CJK_RE = re.compile(r'[\u4e00-\u9fff]')


class Denoiser:
    """通用去噪器: 对任何域/标题/内容做分类与噪声过滤。"""

    @staticmethod
    def domain_type(domain):
        """分类域类型: chinese | url | platform | internal | code | entity
        chinese  = 中文内容域 (含 CJK)
        url      = 域名/IP (含 TLD, 数字)
        platform = 已知平台名 (reddit/youtube/...)
        internal = 内部知识链标签 (含 →·_ 等拓扑符号)
        code     = 程序/技术实体 (可能含字母但非域名)
        entity   = 实体名 (人名/组织/混合)
        """
        if not domain or not isinstance(domain, str):
            return 'unknown'
        # 内部知识链标签 (NeoTrix taxonomy): → · _ 分隔
        if re.search(r'[→·_／/]', domain):
            return 'internal'
        if CJK_RE.search(domain):
            return 'chinese'
        low = domain.lower().strip()
        # platform 优先
        for p in DOMAIN_NOISE['platform']['regex']:
            if re.search(p, low):
                return 'platform'
        # 含 TLD 或数字或点/短横 → url
        if re.search(r'(\.(com|org|net|io|co|cn|de|fr|jp|uk|ru|info|xyz|top|club|vip|gq|ml|ga|cf|tk)$|\.\d+|^[\w-]+\.\w+$)', low):
            return 'url'
        if re.search(r'\d', low):
            return 'url'
        # 纯英文字母且像缩写 → code/entity
        if re.match(r'^[a-z]{2,20}$', low):
            return 'entity'
        return 'entity'

    @staticmethod
    def is_noise_domain(domain):
        """URL/平台/内部标签域是噪声, 不作为内容补缺目标。但 URL 仍是 OSINT 目标。"""
        t = Denoiser.domain_type(domain)
        return t in ('url', 'platform', 'unknown', 'internal')

    @staticmethod
    def clean_title(title):
        """去除标题噪声: 问句尾巴 / 广告词 / 观看类 / 考试就业。"""
        if not title:
            return ''
        t = title.strip()
        # 问句尾
        t = re.sub(r'[？?].*$', '', t)
        # 广告/娱乐词
        for bad in ['韩剧', '电影', '电视剧', '在线观看', '高清', '免费观看',
                    '全集', '国语版', '破解版', '下载', 'app下载', '安装']:
            if bad in t:
                return ''
        # 考试/就业/招聘/排名 这些是营销噪声
        if re.search(r'(考试|就业|招聘|面试|大学排名|专业代码)', t):
            return ''
        t = t.strip(' .·:：-_')
        if 2 <= len(t) <= 24:
            return t
        return ''

    @staticmethod
    def clean_extract(text):
        """通用内容去噪: 去 HTML/脚本/重复行/营销话术。适用于任何语言。"""
        if not text:
            return ''
        t = text
        t = re.sub(r'<[^>]+>', '', t)
        t = re.sub(r'\b(function|var|window\.|document\.|script)\b.*', '', t, flags=re.I)
        lines = [l.strip() for l in t.splitlines() if l.strip()]
        seen = set()
        out = []
        for l in lines:
            if l in seen:
                continue
            seen.add(l)
            if len(l) < 12:
                continue
            out.append(l)
        return '\n'.join(out)

    @staticmethod
    def is_spam_topic(topic):
        """主题是否为营销/娱乐噪声。"""
        if not topic:
            return True
        for bad in ['韩剧', '电影', '电视剧', '在线观看', '高清', '免费', '全集',
                    '破解', '下载', '安装', 'app', '软件', '版本', '考试', '就业',
                    '招聘', '面试', '专业代码', '大学排名', 'HD', '国语']:
            if bad in topic:
                return True
        if re.search(r'[怎如吗那哪]$', topic.strip()):
            return True
        return False


# ────────────────────────────────────────────────────────────────
# 〇 能力注册表 CapabilityRegistry — 自我组建信息获取能力
# ────────────────────────────────────────────────────────────────
# 每个能力映射代码库已有节点 (nt_world_osint 子模块 + 网络源)。
# health = ok/(ok+fail), cooldown 用于退避。布阵时按健康度组建链。
class CapabilityRegistry:
    def __init__(self, state, net):
        self.state = state  # 持久状态: {cap_name: {ok, fail, last, cooldown}}
        self.net = net
        self.caps = self._build()

    def _build(self):
        caps = {
            # ── 内容获取 (content) ──
            'zh_wiki':     {'node': 'nt_world_crawl::wikipedia',   'type': 'content', 'proxy': True,  'cost': 3},
            'en_wiki':     {'node': 'nt_world_crawl::wikipedia',   'type': 'content', 'proxy': True,  'cost': 3},
            'baidu_sug':   {'node': 'nt_world_osint::social',      'type': 'content', 'proxy': False, 'cost': 1},
            'sogou':       {'node': 'nt_world_crawl::search',      'type': 'content', 'proxy': False, 'cost': 2},
            'arxiv':       {'node': 'nt_world_crawl::academic',    'type': 'content', 'proxy': False, 'cost': 2},
            # ── OSINT (url/entity/platform 域) ──
            'dns_crt':     {'node': 'nt_world_osint::dns',         'type': 'osint',   'proxy': False, 'cost': 2},
            'http_probe':  {'node': 'nt_world_osint::http',        'type': 'osint',   'proxy': False, 'cost': 2},
            'url_wayback': {'node': 'nt_world_osint::url',         'type': 'osint',   'proxy': False, 'cost': 2},
            'osv_vuln':    {'node': 'nt_world_osint::vuln',        'type': 'osint',   'proxy': False, 'cost': 1},
            'ddg_search':  {'node': 'nt_world_osint::social',      'type': 'osint',   'proxy': False, 'cost': 2},
            'github_api':  {'node': 'nt_world_osint::http',        'type': 'code',    'proxy': False, 'cost': 2},
        }
        return caps

    def _meta(self, name):
        return self.state.setdefault('capabilities', {}).setdefault(
            name, {'ok': 0, 'fail': 0, 'last': 0, 'cooldown': 0})

    def health(self, name):
        m = self._meta(name)
        total = m['ok'] + m['fail']
        if total == 0:
            return 0.5  # unknown
        return m['ok'] / total

    def mark(self, name, ok):
        m = self._meta(name)
        m['last'] = time.time()
        if ok:
            m['ok'] += 1
            m['cooldown'] = 0
        else:
            m['fail'] += 1
            # 失败2次 → cooldown 60s, 4次 → 300s, 8次 → 600s
            m['cooldown'] = min(600, 60 * (2 ** (m['fail'] // 2)))

    def in_cooldown(self, name):
        m = self._meta(name)
        if m['cooldown'] <= 0:
            return False
        return (time.time() - m['last']) < m['cooldown']

    def assemble(self, domain_type):
        """按域类型组建最优能力链 (按健康度 + cost 排序, 排除冷却/失败>3)。"""
        chain = []
        if domain_type == 'chinese':
            order = ['zh_wiki', 'baidu_sug', 'sogou']
        elif domain_type in ('url', 'platform'):
            order = ['dns_crt', 'url_wayback', 'http_probe', 'osv_vuln', 'ddg_search']
        elif domain_type == 'code':
            order = ['github_api', 'osv_vuln']
        else:  # entity
            order = ['en_wiki', 'ddg_search', 'baidu_sug']
        for name in order:
            if self.in_cooldown(name):
                continue
            m = self._meta(name)
            if m['fail'] >= 5 and (time.time() - m['last']) < 1800:
                continue  # 软性禁用
            chain.append(name)
        # 健康度降序稳定 (同域类型内仍按 cost 加权)
        chain.sort(key=lambda n: (-self.health(n), self.caps[n]['cost']))
        return chain

    def report(self):
        lines = []
        for name, cap in self.caps.items():
            m = self._meta(name)
            total = m['ok'] + m['fail']
            h = m['ok'] / total if total else 0.5
            lines.append(f"{name}({cap['type']}, h={h:.2f}, ok={m['ok']}, fail={m['fail']})")
        return ' '.join(lines)


# ────────────────────────────────────────────────────────────────
# ① 观阵 GapScan — 通用扫描薄弱域 (含 URL/平台/实体域)
# ────────────────────────────────────────────────────────────────
class GapScan:
    def __init__(self, conn):
        self.conn = conn

    def weak_domains(self, max_entries=4, exclude=None):
        exclude = exclude or set()
        rows = self.conn.execute(
            """SELECT domain, COUNT(*) FROM nodes
               WHERE domain IS NOT NULL GROUP BY domain HAVING COUNT(*) <= ?
            """, (max_entries,)).fetchall()
        weak = []
        for d, cnt in rows:
            if not isinstance(d, str):
                continue
            if '/' in d or '@' in d or '→' in d or len(d) < 2 or len(d) > 30:
                continue
            if d in exclude:
                continue
            dtype = Denoiser.domain_type(d)
            # 噪声域(纯 URL/平台)不补内容, 但保留供 OSINT 或跳过
            if Denoiser.is_noise_domain(d):
                continue
            weak.append((cnt, d, dtype))
        weak.sort(key=lambda x: (x[0], x[1]))
        return weak

    def total_entries(self, domain):
        r = self.conn.execute("SELECT COUNT(*) FROM nodes WHERE domain=?", (domain,)).fetchone()
        return r[0] if r else 0

    def existing_titles(self, domain):
        return set(r[0] for r in self.conn.execute(
            "SELECT title FROM nodes WHERE domain=?", (domain,)))

    def domain_list(self, exclude=None, limit=60):
        """返回所有域(含 URL/平台), 供 OSINT 批量处理。"""
        exclude = exclude or set()
        rows = self.conn.execute(
            """SELECT DISTINCT domain FROM nodes WHERE domain IS NOT NULL
               ORDER BY domain""").fetchall()
        out = []
        for (d,) in rows:
            if not isinstance(d, str) or d in exclude:
                continue
            if '/' in d or '@' in d or len(d) < 2 or len(d) > 30:
                continue
            out.append(d)
            if len(out) >= limit:
                break
        return out


# ────────────────────────────────────────────────────────────────
# 网络层 — 代理绕路 + 直连 + 重试
# ────────────────────────────────────────────────────────────────
class NetLayer:
    def __init__(self, state):
        self.state = state  # dict {proxy: {fail, ok, last}}
        self.proxy_pool = self._load_proxies()
        self._last_refresh = 0

    def _load_proxies(self):
        try:
            lines = [l.strip() for l in open(PROXY_CONF, encoding='utf-8')
                     if l.strip() and not l.startswith('#') and '://' in l]
            socks = [l for l in lines if 'socks5://' in l]
            return socks or lines
        except FileNotFoundError:
            return []

    def _healthy_proxies(self):
        now = time.time()
        if now - self._last_refresh > 600:
            self.proxy_pool = self._load_proxies()
            self._last_refresh = now
        cands = []
        for p in self.proxy_pool:
            s = self.state.get(p, {})
            if s.get('fail', 0) >= 3 and now - s.get('last', 0) < 600:
                continue
            cands.append(p)
        random.shuffle(cands)
        return cands

    def _curl(self, url, proxy=None, timeout=12, headers=None, retries=1):
        cmd = ['curl', '-s', '-L', '-m', str(timeout), '--connect-timeout', '10',
               '-A', UA, '-o', '/tmp/_fmt.json', '-w', '%{http_code}', url]
        if proxy:
            cmd += ['-x', proxy]
        for i in range(retries + 1):
            try:
                r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout + 15)
                code = r.stdout.strip()
                if code == '200':
                    body = open('/tmp/_fmt.json', encoding='utf-8', errors='ignore').read()
                    if body.strip():
                        self._mark(proxy, True)
                        return body
                elif proxy:
                    self._mark(proxy, False)
            except (subprocess.TimeoutExpired, OSError):
                if proxy:
                    self._mark(proxy, False)
            time.sleep(1.0 * (i + 1))
        return None

    def _mark(self, proxy, ok):
        if not proxy:
            return
        s = self.state.setdefault(proxy, {'fail': 0, 'ok': 0, 'last': 0})
        s['last'] = time.time()
        if ok:
            s['ok'] = s.get('ok', 0) + 1
        else:
            s['fail'] = s.get('fail', 0) + 1

    # ── wiki extract (zh/en, 代理绕路) ──
    def wiki_extract(self, lang, title, retries=1):
        """Fetch {lang}.wikipedia.org intro via proxy."""
        url = (f"https://{lang}.wikipedia.org/w/api.php?action=query&format=json"
               f"&prop=extracts&explaintext=1&exintro=1&redirects=1&titles={title}")
        proxies = self._healthy_proxies()
        for proxy in proxies[:retries]:
            body = self._curl(url, proxy=proxy, timeout=10)
            if body:
                try:
                    data = json.loads(body)
                    pages = data.get('query', {}).get('pages', {})
                    for pid, p in pages.items():
                        ext = p.get('extract')
                        if ext:
                            return {'title': p.get('title', title),
                                    'extract': ext,
                                    'source': f'{lang}_wikipedia',
                                    'url': f'https://{lang}.wikipedia.org/wiki/{p.get("title", title)}'}
                except (json.JSONDecodeError, AttributeError):
                    pass
        return None

    def zh_wiki_extract(self, title, retries=1):
        return self.wiki_extract('zh', title, retries)

    def en_wiki_extract(self, title, retries=1):
        return self.wiki_extract('en', title, retries)

    # ── 搜索联想 (直连) ──
    def baidu_sug(self, word):
        url = f"https://www.baidu.com/sugrec?prod=pc&wd={word}"
        body = self._curl(url, proxy=None, timeout=10)
        if body:
            try:
                d = json.loads(body)
                return [g['q'] for g in d.get('g', [])]
            except (json.JSONDecodeError, KeyError):
                pass
        return []

    def sogou_search(self, query, limit=5):
        url = f"https://www.sogou.com/web?query={query}"
        body = self._curl(url, proxy=None, timeout=12)
        if not body:
            return []
        results = []
        for m in re.findall(r'<h3[^>]*>\s*<a[^>]+href="([^"]+)"[^>]*>(.*?)</a>', body)[:limit]:
            u, t = m
            t = re.sub(r'<[^>]+>', '', t).strip()
            if t:
                results.append({'url': u.replace('&amp;', '&'), 'title': t})
        return results

    def ddg_html(self, query, limit=5):
        url = f"https://html.duckduckgo.com/html/?q={query}"
        body = self._curl(url, proxy=None, timeout=12)
        if not body:
            return []
        results = []
        for m in re.findall(r'class="result__a"[^>]*href="([^"]+)"[^>]*>(.*?)</a>', body)[:limit]:
            u, t = m
            t = re.sub(r'<[^>]+>', '', t).strip()
            if t:
                results.append({'url': u.replace('&amp;', '&'), 'title': t})
        return results

    # ── OSINT 直连 ──
    def crt_sh(self, domain, limit=8):
        """crt.sh certificate transparency — subdomain/domain info."""
        url = f"https://crt.sh/?q={domain}&output=json"
        body = self._curl(url, proxy=None, timeout=12)
        if not body:
            return []
        try:
            d = json.loads(body)
            out = {}
            for e in d:
                name = e.get('name_value', '')
                for n in name.split('\n'):
                    n = n.strip().lstrip('*.').lower()
                    if n and n not in out:
                        out[n] = e.get('common_name', '')
                if len(out) >= limit:
                    break
            return list(out.items())
        except (json.JSONDecodeError, TypeError):
            return []

    def wayback(self, url, limit=5):
        """Wayback CDX — URL 历史快照。"""
        u = f"http://web.archive.org/cdx/search/cdx?url={url}&output=json&fl=timestamp,statuscode,original&limit={limit}"
        body = self._curl(u, proxy=None, timeout=10)
        if not body:
            return []
        try:
            d = json.loads(body)
            if len(d) < 2:
                return []
            return d[1:limit + 1]
        except (json.JSONDecodeError, IndexError):
            return []

    def osv_vuln(self, name, ecosystem='PyPI', limit=3):
        """OSV.dev — 漏洞查询 (按包名)。"""
        import urllib.request
        payload = json.dumps({"package": {"name": name, "ecosystem": ecosystem}}).encode()
        req = urllib.request.Request("https://api.osv.dev/v1/query", data=payload,
                                     headers={'Content-Type': 'application/json'})
        try:
            with urllib.request.urlopen(req, timeout=10) as r:
                d = json.loads(r.read().decode())
            return [v.get('id') for v in d.get('vulns', [])][:limit]
        except Exception:
            return []

    def github_search(self, query, limit=3):
        url = ("https://api.github.com/search/repositories"
               f"?q={query}&sort=stars&per_page={limit}")
        body = self._curl(url, proxy=None, timeout=15)
        if not body:
            return []
        try:
            d = json.loads(body)
            return [{'name': r['full_name'], 'desc': r.get('description') or '',
                     'stars': r.get('stargazers_count', 0)}
                    for r in d.get('items', [])[:limit]]
        except (json.JSONDecodeError, KeyError):
            return []

    def arxiv_search(self, query, limit=3):
        url = (f"http://export.arxiv.org/api/query?search_query=all:{query}"
               f"&max_results={limit}")
        body = self._curl(url, proxy=None, timeout=10)
        if not body:
            return []
        entries = re.findall(r'<entry>.*?</entry>', body, re.S)[:limit]
        out = []
        for e in entries:
            t = re.search(r'<title>(.*?)</title>', e, re.S)
            s = re.search(r'<summary>(.*?)</summary>', e, re.S)
            if t:
                out.append({'title': re.sub(r'\s+', ' ', t.group(1)).strip(),
                            'summary': re.sub(r'\s+', ' ', s.group(1)).strip()[:200] if s else ''})
        return out


# ────────────────────────────────────────────────────────────────
# ② 布阵 PlanTree — 按域类型组建能力链 + 主题树
# ────────────────────────────────────────────────────────────────
class PlanTree:
    SEEDS = {
        '生态学': ['生态系统', '食物链', '生物多样性', '生态系统服务', '生态位', '群落演替', '生态金字塔'],
        '量子力学': ['量子纠缠', '波函数坍缩', '不确定性原理', '量子隧道效应', '叠加态', '薛定谔方程'],
        '相对论': ['狭义相对论', '广义相对论', '时空弯曲', '引力波', '质能等价', '光速不变原理'],
        '地质学': ['板块构造论', '岩石循环', '地质年代', '断层与褶皱', '火山学', '矿物学'],
        '植物学': ['光合作用', '植物组织', '根茎叶结构', '传粉机制', '种子传播', '植物激素'],
        '动物学': ['动物分类学', '动物行为学', '迁徙现象', '冬眠机制', '拟态', '捕食关系'],
        '气候学': ['大气环流', '季风系统', '厄尔尼诺', '温室效应', '古气候', '气候带分布'],
        '语言学': ['音位学', '句法学', '语义学', '语用学', '语言演化', '方言学'],
        '神话学': ['创世神话', '英雄神话', '洪水神话', '神话母题', '原始宗教', '太阳崇拜'],
        '符号学': ['能指与所指', '图像符号', '指示符号', '象征符号', '索绪尔语言学', '皮尔斯三分法'],
        '密码学': ['对称加密', '非对称加密', '哈希函数', '数字签名', 'RSA算法', '量子密码'],
        '控制论': ['反馈控制', '伺服系统', 'PID控制', '自动驾驶', '稳定性理论', '控制系统'],
        '统计学': ['概率分布', '假设检验', '回归分析', '置信区间', '贝叶斯统计', '方差分析'],
        '运筹学': ['线性规划', '排队论', '博弈论', '动态规划', '最优化方法', '决策论'],
        '教育学': ['认知发展理论', '建构主义', '学习理论', '教育评估', '课程设计', '元认知'],
        '人类学': ['文化相对主义', '民族志方法', '亲属制度', '仪式与象征', '进化人类学', '田野调查'],
        '认知科学': ['工作记忆', '模式识别', '心智模型', '意识研究', '神经可塑性', '认知偏差'],
        '地缘政治': ['地缘战略', '海权论', '陆权论', '边境地理', '资源地缘政治', '能源走廊'],
        '建筑学': ['建筑类型学', '空间组织', '结构体系', '可持续建筑', '地域主义', '建筑符号学'],
        '机械工程': ['机构运动学', '齿轮传动', '内燃机', '材料力学', '公差配合', '机械振动'],
        '电磁学': ['静电场', '电磁感应', '麦克斯韦方程组', '交流电', '电磁波', '洛伦兹力'],
        '农业': ['作物育种', '土壤肥力', '灌溉技术', '农业生态', '病虫害防治', '精准农业'],
        '农业育种': ['杂交育种', '基因编辑育种', '分子标记', '品种选育', '种质资源', '倍性育种'],
        '民俗学': ['民间故事', '节庆仪式', '口头传统', '民间信仰', '民俗分类', '地方知识'],
        '文学理论': ['叙事学', '读者反应理论', '结构主义文论', '后殖民批评', '女性主义批评', '形式主义'],
        '医学基础': ['细胞生物学', '人体解剖', '病理学', '药理学', '免疫学', '病原微生物'],
        '博弈论': ['纳什均衡', '囚徒困境', '帕累托最优', '重复博弈', '零和博弈', '机制设计'],
        '系统科学': ['系统论', '反馈系统', '自组织', '复杂性', '熵与信息', '涌现现象'],
        '计算机科学': ['算法复杂度', '数据结构', '编译原理', '操作系统', '分布式系统', '数据库原理'],
        '金融学': ['风险管理', '资产定价', '投资组合', '货币政策', '金融衍生品', '行为金融'],
        '天文学': ['恒星演化', '宇宙学原理', '黑洞物理', '行星系统', '星系分类', '暗物质'],
        '艺术史': ['文艺复兴艺术', '巴洛克艺术', '印象派', '现代主义', '后现代艺术', '中国绘画史'],
        '翻译学': ['翻译策略', '功能对等', '直译意译', '语域转换', '翻译伦理', '机器翻译评估'],
    }

    def __init__(self, net, registry):
        self.net = net
        self.registry = registry

    def topics_for_domain(self, domain, existing_titles, limit=8):
        """按域类型生成主题: 中文域用 seed + baidu_sug, 英文/实体域用 wiki 直查。"""
        dtype = Denoiser.domain_type(domain)
        topics = []
        seen = set()
        if dtype == 'chinese':
            for t in self.SEEDS.get(domain, []):
                if t not in seen:
                    seen.add(t)
                    topics.append(t)
            for t in self.net.baidu_sug(domain):
                clean = Denoiser.clean_title(t)
                if clean and clean not in seen:
                    seen.add(clean)
                    topics.append(clean)
        else:
            # 英文/实体: 直接当作主题本身, 由 OSINT 链补内容
            topics.append(domain)
        fresh = [t for t in topics if t not in existing_titles and not Denoiser.is_spam_topic(t)]
        return fresh[:limit]


# ────────────────────────────────────────────────────────────────
# ③ 攻阵 Crawler — 能力链组装爬取
# ────────────────────────────────────────────────────────────────
class Crawler:
    def __init__(self, net, registry):
        self.net = net
        self.registry = registry

    def _fetch_content(self, topic, domain, lang='zh'):
        """内容域: wiki(代理) 优先, baidu_sug/sogou 兜底。"""
        result = {}
        done = threading.Event()

        def _wiki():
            fn = self.net.en_wiki_extract if lang == 'en' else self.net.zh_wiki_extract
            r = fn(topic)
            if r and not done.is_set():
                result.update(r)
                done.set()

        def _sug():
            sugs = self.net.baidu_sug(topic)
            for s in sugs[:4]:
                clean = Denoiser.clean_title(s)
                if clean:
                    if not done.is_set():
                        result.update({'title': topic, 'extract': '',
                                       'source': 'baidu', 'url': None,
                                       'domain': domain, 'topic': topic})
                        done.set()
                    return
            if not done.is_set():
                result.update({'title': topic, 'extract': '',
                               'source': 'baidu', 'url': None,
                               'domain': domain, 'topic': topic})
                done.set()

        threads = [threading.Thread(target=_wiki, daemon=True),
                   threading.Thread(target=_sug, daemon=True)]
        for t in threads:
            t.start()
        done.wait(timeout=15)
        if result:
            result['domain'] = domain
            result['topic'] = topic
            return result
        return None

    def _fetch_osint(self, topic, domain):
        """URL/平台域: OSINT 链 (crt.sh → wayback → http → osv → ddg)。"""
        findings = {}
        caps = self.registry.assemble(Denoiser.domain_type(domain))
        for cap in caps:
            try:
                if cap == 'dns_crt':
                    r = self.net.crt_sh(topic, limit=6)
                    if r:
                        findings['subdomains'] = [n for n, _ in r[:6]]
                        self.registry.mark(cap, True)
                        break
                    self.registry.mark(cap, False)
                elif cap == 'url_wayback':
                    r = self.net.wayback(topic, limit=4)
                    if r:
                        findings['snapshots'] = r[:4]
                        self.registry.mark(cap, True)
                        break
                    self.registry.mark(cap, False)
                elif cap == 'http_probe':
                    r = self.net._curl(f"https://{topic}", proxy=None, timeout=8)
                    if r:
                        title = re.search(r'<title[^>]*>(.*?)</title>', r, re.S | re.I)
                        findings['http'] = {'reachable': True,
                                            'title': title.group(1).strip()[:80] if title else ''}
                        self.registry.mark(cap, True)
                    else:
                        self.registry.mark(cap, False)
                elif cap == 'osv_vuln':
                    r = self.net.osv_vuln(topic)
                    if r:
                        findings['vulns'] = r[:3]
                        self.registry.mark(cap, True)
                    else:
                        self.registry.mark(cap, False)
                elif cap == 'ddg_search':
                    r = self.net.ddg_html(topic, limit=3)
                    if r:
                        findings['references'] = r[:3]
                        self.registry.mark(cap, True)
                    else:
                        self.registry.mark(cap, False)
            except Exception:
                self.registry.mark(cap, False)
        if findings:
            return {'title': topic, 'extract': json.dumps(findings, ensure_ascii=False),
                    'source': 'osint', 'url': f"https://{topic}",
                    'domain': domain, 'topic': topic}
        return None

    def fetch(self, topic, domain):
        dtype = Denoiser.domain_type(domain)
        if dtype == 'chinese':
            return self._fetch_content(topic, domain, lang='zh')
        if dtype in ('url', 'platform'):
            return self._fetch_osint(topic, domain)
        if dtype == 'code':
            r = self.net.github_search(topic, limit=3)
            if r:
                return {'title': topic, 'extract': json.dumps(r, ensure_ascii=False),
                        'source': 'github', 'url': f"https://github.com/{topic}",
                        'domain': domain, 'topic': topic}
            return None
        # entity / english → en_wiki + ddg
        result = {}
        done = threading.Event()

        def _wiki():
            r = self.net.en_wiki_extract(topic)
            if r and not done.is_set():
                result.update(r)
                done.set()

        def _ddg():
            r = self.net.ddg_html(topic, limit=3)
            if r and not done.is_set():
                result.update({'title': topic, 'extract': json.dumps(r, ensure_ascii=False),
                               'source': 'ddg', 'url': None, 'domain': domain, 'topic': topic})
                done.set()

        threads = [threading.Thread(target=_wiki, daemon=True),
                   threading.Thread(target=_ddg, daemon=True)]
        for t in threads:
            t.start()
        done.wait(timeout=15)
        if result:
            result['domain'] = domain
            result['topic'] = topic
            return result
        return None


# ────────────────────────────────────────────────────────────────
# ④ 收阵 Mint — 通用去噪 + 3D 结构化 + 入库
# ────────────────────────────────────────────────────────────────
class Mint:
    def to_3d(self, raw):
        """OSINT 域: 直接结构化 findings; 内容域: 3D 拆分/合成。"""
        text = Denoiser.clean_extract(raw.get('extract', ''))
        topic = raw.get('topic', '')
        domain = raw.get('domain', '')
        source = raw.get('source', '')

        if source == 'osint':
            try:
                d = json.loads(text)
                parts = []
                for k, v in d.items():
                    if isinstance(v, list) and v:
                        parts.append(f"{k}: {'; '.join(str(x) for x in v[:4])}")
                    elif v:
                        parts.append(f"{k}: {v}")
                return {'principle': f"{domain} OSINT 画像 —— {topic}",
                        'surface': '\n'.join(parts),
                        'number': f"发现 {sum(len(v) if isinstance(v, list) else 1 for v in d.values())} 类情报"}
            except json.JSONDecodeError:
                pass

        n = len(text)
        if n < 40 or text.endswith('?') or text.endswith('怎'):
            return self._synthesize(topic, domain, source)
        if n <= MIN_CONTENT_LEN:
            return {'principle': text[:max(60, n // 2)], 'surface': '', 'number': ''}
        third = max(MIN_CONTENT_LEN, n // 3)

        def split_near(pos):
            for step in range(0, 40):
                for i in (pos + step, pos - step):
                    if i < n and i > third // 2 and text[i] in '。！？.!?':
                        return i + 1
            return min(n, pos + 40)

        p_end = split_near(third)
        s_end = split_near(third * 2)
        return {'principle': text[:p_end].strip(),
                'surface': text[p_end:s_end].strip(),
                'number': text[s_end:].strip()}

    def _synthesize(self, topic, domain, source=''):
        src_tag = f" (source={source})" if source else ''
        principle = f"{domain}中的'{topic}'——研究{topic}的核心概念、定义、基本原理与内在机制。{src_tag}"
        surface = f"{topic}的具体表现：相关的现象、实例、应用场景与自然/社会表现。"
        number = f"与{topic}相关的关键数据、规模、比例及可量化指标。"
        return {'principle': principle, 'surface': surface, 'number': number}

    def _content_from_3d(self, d3):
        c = []
        if d3['principle']:
            c.append(f"原理：{d3['principle']}")
        if d3['surface']:
            c.append(f"表象：{d3['surface']}")
        if d3['number']:
            c.append(f"数：{d3['number']}")
        return '\n'.join(c)

    def insert(self, conn, raw, d3, domain, dry_run=False):
        title = raw.get('title', raw.get('topic', ''))
        if not title or title in [r[0] for r in conn.execute(
                "SELECT title FROM nodes WHERE domain=? AND title=?", (domain, title))]:
            return None, 'duplicate'
        if dry_run:
            return 'dry-run', 'would_insert'
        now = int(time.time())
        eid = f"canon_{now}_{uuid.uuid4().hex[:8]}"
        content = self._content_from_3d(d3)
        summary = f"{domain}——{title}：{raw.get('extract', '')[:80]}"
        meta = json.dumps({'domain': domain, 'source': raw.get('source', 'formation'),
                           'layer': 'theory', 'url': raw.get('url'),
                           'formation_cycle': True}, ensure_ascii=False)
        conn.execute(
            """INSERT INTO nodes(id,node_type,title,summary,content,url,domain,language,
               confidence,importance,created_at,updated_at,access_count,metadata,
               data_tier,temporal,supersedes,source_episode,tier)
               VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)""",
            (eid, 'concept', title, summary, content, raw.get('url'),
             domain, 'zh', 1.0, IMPORTANCE_DEFAULT, now, now, 0, meta,
             'core', None, None, None, 'warm'))
        # 同步写 FTS5 索引 — 普通 fts5 表 (非 external content) 的 'rebuild'
        # 只重建 shadow 表已有行, 不会拉取 nodes 新数据; 必须显式插入.
        conn.execute(
            "INSERT INTO nodes_fts(rowid, title, summary, content, domain) "
            "VALUES(last_insert_rowid(), ?, ?, ?, ?)",
            (title, summary or '', content or '', domain or ''))
        return eid, 'inserted'


# ────────────────────────────────────────────────────────────────
# ⑤ 验阵 Verify — FTS5 重建 + 统计
# ────────────────────────────────────────────────────────────────
class Verify:
    @staticmethod
    def rebuild_fts(conn):
        try:
            conn.execute("INSERT INTO nodes_fts(nodes_fts) VALUES('rebuild')")
            return True
        except sqlite3.Error as e:
            return f"fts rebuild failed: {e}"

    @staticmethod
    def distribution(conn, limit=12):
        rows = conn.execute(
            """SELECT domain, COUNT(*) FROM nodes WHERE domain IS NOT NULL
               GROUP BY domain ORDER BY COUNT(*) DESC LIMIT ?""", (limit,)).fetchall()
        return rows

    @staticmethod
    def weak_remaining(conn, max_entries=4):
        rows = conn.execute(
            """SELECT domain, COUNT(*) FROM nodes WHERE domain IS NOT NULL
               GROUP BY domain HAVING COUNT(*) <= ?""", (max_entries,)).fetchall()
        weak = []
        for d, cnt in rows:
            if not isinstance(d, str):
                continue
            if '/' in d or '@' in d or '→' in d or len(d) < 2 or len(d) > 30:
                continue
            if Denoiser.is_noise_domain(d):
                continue
            weak.append((d, cnt))
        return sorted(weak, key=lambda x: (x[1], x[0]))


# ────────────────────────────────────────────────────────────────
# ⑥ 疗阵 Heal — 代理池 / 死源 / 节流 / 能力健康
# ────────────────────────────────────────────────────────────────
class Heal:
    @staticmethod
    def next_backoff(state_key, attempts):
        return min(60, 5 * (2 ** attempts))


# ────────────────────────────────────────────────────────────────
# ⑦ 振阵 Loop — 主循环
# ────────────────────────────────────────────────────────────────
def log(level, phase, msg, extra=None):
    entry = {'ts': int(time.time()), 'level': level, 'phase': phase, 'msg': msg}
    if extra:
        entry['data'] = extra
    try:
        with open(LOG_PATH, 'a') as f:
            f.write(json.dumps(entry, ensure_ascii=False) + '\n')
            f.flush()
    except OSError:
        pass
    tag = {'INFO': '', 'OK': '✅', 'WARN': '⚠️', 'ERROR': '❌'}.get(level, '')
    print(f"[{time.strftime('%H:%M:%S')}][{phase}] {tag} {msg}", flush=True)


def run_cycle(conn, net, state, args):
    phase_stats = {'scanned': 0, 'planned': 0, 'fetched': 0, 'inserted': 0,
                   'duplicates': 0, 'failed': 0}
    registry = CapabilityRegistry(state, net)

    # ① 观阵
    gap = GapScan(conn)
    if args.domains:
        weak = [(gap.total_entries(d), d) for d in args.domains]
    else:
        weak = gap.weak_domains(max_entries=args.max_entries)
    phase_stats['scanned'] = len(weak)
    log('INFO', '观阵', f'扫描到 {len(weak)} 个薄弱域 (含 URL/平台/实体域由 OSINT 链处理)')

    total_topics = 0
    per_domain = max(1, args.topics // max(1, len(weak))) if args.domains else None
    # 真实循环
    for item in weak:
        if total_topics >= args.topics:
            break
        if len(item) == 3:
            cnt, domain, dtype = item
        else:
            cnt, domain = item
            dtype = Denoiser.domain_type(domain)
        existing = gap.existing_titles(domain)
        plan = PlanTree(net, registry)
        budget = per_domain if per_domain else max(1, args.topics - total_topics)
        topics = plan.topics_for_domain(domain, existing, limit=budget)
        if not topics:
            log('INFO', '布阵', f'{domain}: 无新主题 (已有 {len(existing)})')
            continue
        phase_stats['planned'] += len(topics)
        log('INFO', '布阵', f'{domain} [{dtype}]: 规划 {len(topics)} 主题 -> {topics[:5]}')

        for topic in topics:
            if total_topics >= args.topics:
                break
            total_topics += 1
            crawler = Crawler(net, registry)
            raw = crawler.fetch(topic, domain)
            if not raw:
                phase_stats['failed'] += 1
                log('WARN', '攻阵', f'{topic} 抓取失败 (跳过)')
                continue
            phase_stats['fetched'] += 1
            mint = Mint()
            d3 = mint.to_3d(raw)
            eid, status = mint.insert(conn, raw, d3, domain, dry_run=args.dry_run)
            if status == 'inserted':
                phase_stats['inserted'] += 1
                conn.commit()
                log('OK', '收阵', f'{domain}/{topic} 入库 [{raw.get("source")}]')
            elif status == 'would_insert':
                phase_stats['inserted'] += 1
                log('INFO', '收阵', f'[dry-run] {domain}/{topic} 将入库 [{raw.get("source")}]')
            else:
                phase_stats['duplicates'] += 1
                log('INFO', '收阵', f'{topic} 重复 (跳过)')
            time.sleep(args.sleep)

    # ⑤ 验阵
    if phase_stats['inserted'] > 0:
        Verify.rebuild_fts(conn)
        conn.commit()
    dist = Verify.distribution(conn)
    remaining = Verify.weak_remaining(conn, args.max_entries)
    log('INFO', '验阵', f'本轮: 扫描{phase_stats["scanned"]} 规划{phase_stats["planned"]} '
                       f'抓取{phase_stats["fetched"]} 入库{phase_stats["inserted"]} '
                       f'重复{phase_stats["duplicates"]} 失败{phase_stats["failed"]}')
    log('INFO', '验阵', f'剩余薄弱域: {len(remaining)}')
    log('INFO', '验阵', 'Top 域分布: ' + ', '.join(f'{d}({c})' for d, c in dist[:6]))
    log('INFO', '疗阵', '能力健康: ' + registry.report())
    return phase_stats


def main():
    ap = argparse.ArgumentParser(description='NeoTrix 阵法节点引擎 v0.2 (自我组建能力网)')
    ap.add_argument('--dry-run', action='store_true', help='演练一轮不写库')
    ap.add_argument('--topics', type=int, default=30, help='每 cycle 最多主题数')
    ap.add_argument('--sleep', type=float, default=2.0, help='主题间间隔秒')
    ap.add_argument('--domains', nargs='*', help='只处理指定域 (任意类型)')
    ap.add_argument('--max-entries', type=int, default=4, help='薄弱域阈值')
    ap.add_argument('--loop', action='store_true', help='守护循环')
    ap.add_argument('--interval', type=int, default=600, help='循环间隔秒')
    args = ap.parse_args()

    state = {}
    if os.path.exists(STATE_PATH):
        try:
            state = json.load(open(STATE_PATH))
        except (json.JSONDecodeError, OSError):
            state = {}

    conn = sqlite3.connect(KB_PATH, timeout=30)
    conn.execute("PRAGMA busy_timeout=30000")
    net = NetLayer(state)

    log('INFO', '启动', '阵法节点引擎 v0.2 启动 (dry_run=%s)' % args.dry_run)

    while True:
        try:
            run_cycle(conn, net, state, args)
        except KeyboardInterrupt:
            break
        except Exception as e:
            log('ERROR', '振阵', f'cycle 异常: {e}')
            import traceback
            traceback.print_exc()
        if not args.loop:
            break
        log('INFO', '振阵', f'sleep {args.interval}s...')
        time.sleep(args.interval)

    if state:
        with open(STATE_PATH, 'w') as f:
            json.dump(state, f, ensure_ascii=False)
    conn.close()
    log('INFO', '退出', '阵法节点引擎结束')


if __name__ == '__main__':
    main()

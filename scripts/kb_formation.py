#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
NeoTrix 阵法节点引擎 (Formation Node Engine) v0.1 — Python 原型
============================================================
自主爬取数据补齐知识库全景。七个阵法节点协同运作:

  ① 观阵 GapScan    扫描 KB,找出薄弱域 (≤4 entries)
  ② 布阵 PlanTree   为薄弱域生成主题树 (维基分类 + 搜索联想 + 主题库)
  ③ 攻阵 Crawler    多源爬取 (中文维基 API + 360/搜狗 + GitHub API + 代理绕路)
  ④ 收阵 Mint       3D 结构化 (原理·表象·数) + 去重入库
  ⑤ 验阵 Verify     FTS5 重建 + 域分布统计
  ⑥ 疗阵 Heal       代理池刷新 / 死源标记 / 节流退避
  ⑦ 振阵 Loop       自调度循环 (每 cycle 处理 N 个主题)

网络拓扑:
  - 直连可用:  360搜索 (so.com), 搜狗 (sogou.com), 百度联想 (sugrec),
               GitHub API, 中科院/教育网
  - 代理绕路:  zh.wikipedia.org API (需 SOCKS5 代理)
  - 代理池:    ~/.neotrix/proxy-upstreams.conf (157+ 节点, 动态刷新)

用法:
  python3 scripts/kb_formation.py --dry-run            # 演练一轮不写库
  python3 scripts/kb_formation.py --topics N           # 每 cycle 最多 N 主题
  python3 scripts/kb_formation.py --domains 生态学 量子力学  # 指定域
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
import time
import uuid
from concurrent.futures import ThreadPoolExecutor, as_completed

KB_PATH = os.path.expanduser("~/.neotrix/knowledge_base.db")
PROXY_CONF = os.path.expanduser("~/.neotrix/proxy-upstreams.conf")
STATE_PATH = os.path.expanduser("~/.neotrix/kb_formation_state.json")
LOG_PATH = os.path.expanduser("~/.neotrix/kb_formation_log.jsonl")
UA = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36"
MIN_CONTENT_LEN = 180
MIN_DOMAIN_TARGET = 3
IMPORTANCE_DEFAULT = 0.88

# ────────────────────────────────────────────────────────────────
# ① 观阵 GapScan — 扫描 KB 薄弱域
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
            if re.search(r'[a-zA-Z0-9]', d):
                continue
            if '/' in d or '@' in d or '→' in d or len(d) < 2 or len(d) > 30:
                continue
            if d in exclude:
                continue
            weak.append((cnt, d))
        weak.sort(key=lambda x: (x[0], x[1]))
        return weak

    def total_entries(self, domain):
        r = self.conn.execute("SELECT COUNT(*) FROM nodes WHERE domain=?", (domain,)).fetchone()
        return r[0] if r else 0

    def existing_titles(self, domain):
        return set(r[0] for r in self.conn.execute(
            "SELECT title FROM nodes WHERE domain=?", (domain,)))

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
            # prefer socks5, skip duplicates
            socks = [l for l in lines if 'socks5://' in l]
            return socks or lines
        except FileNotFoundError:
            return []

    def _healthy_proxies(self):
        now = time.time()
        if now - self._last_refresh > 600:  # refresh pool every 10min
            self.proxy_pool = self._load_proxies()
            self._last_refresh = now
        cands = []
        for p in self.proxy_pool:
            s = self.state.get(p, {})
            if s.get('fail', 0) >= 3 and now - s.get('last', 0) < 600:
                continue  # temporarily banned
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

    def zh_wiki_extract(self, title, retries=1):
        """Fetch zh.wikipedia.org article intro via proxy (fast, 10s timeout)."""
        url = ("https://zh.wikipedia.org/w/api.php?action=query&format=json"
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
                                    'extract': ext, 'source': 'zh_wikipedia',
                                    'url': f'https://zh.wikipedia.org/wiki/{p.get("title",title)}'}
                except (json.JSONDecodeError, AttributeError):
                    pass
        return None

    def zh_wiki_search(self, term, limit=8):
        url = ("https://zh.wikipedia.org/w/api.php?action=query&format=json"
               f"&list=search&srsearch={term}&srlimit={limit}")
        proxies = self._healthy_proxies()
        for proxy in proxies[:1]:
            body = self._curl(url, proxy=proxy, timeout=12)
            if body:
                try:
                    d = json.loads(body)
                    return [s['title'] for s in d.get('query', {}).get('search', [])]
                except json.JSONDecodeError:
                    pass
        return []

    def zh_wiki_category_members(self, category, limit=8):
        url = ("https://zh.wikipedia.org/w/api.php?action=query&format=json"
               f"&list=categorymembers&cmtitle=Category:{category}&cmlimit={limit}&cmtype=page")
        proxies = self._healthy_proxies()
        for proxy in proxies[:1]:
            body = self._curl(url, proxy=proxy, timeout=12)
            if body:
                try:
                    d = json.loads(body)
                    return [m['title'] for m in d.get('query', {}).get('categorymembers', [])]
                except json.JSONDecodeError:
                    pass
        return []

    def baidu_sug(self, word):
        """Baidu suggestion API (direct, no proxy needed)."""
        url = f"https://www.baidu.com/sugrec?prod=pc&wd={word}"
        body = self._curl(url, proxy=None, timeout=10)
        if body:
            try:
                d = json.loads(body)
                return [g['q'] for g in d.get('g', [])]
            except (json.JSONDecodeError, KeyError):
                pass
        return []

    def so360_search(self, query, limit=5):
        """360 search results (direct)."""
        url = f"https://www.so.com/s?q={query}"
        body = self._curl(url, proxy=None, timeout=15)
        if not body:
            return []
        links = re.findall(r'<a[^>]+data-mdurl="(http[^"]+)"[^>]*>(.*?)</a>', body)
        results = []
        for u, t in links[:limit]:
            t = re.sub(r'<[^>]+>', '', t).strip()
            if t and '360.com' not in u and 'bing.com' not in u:
                results.append({'url': u.replace('&amp;', '&'), 'title': t})
        return results

    def github_search(self, query, limit=3):
        url = ("https://api.github.com/search/repositories"
               f"?q={query}&sort=stars&per_page={limit}")
        body = self._curl(url, proxy=None, timeout=15,
                          headers={'Accept': 'application/vnd.github+json'})
        if not body:
            return []
        try:
            d = json.loads(body)
            return [{'name': r['full_name'], 'desc': r.get('description') or '',
                     'stars': r.get('stargazers_count', 0)}
                    for r in d.get('items', [])[:limit]]
        except (json.JSONDecodeError, KeyError):
            return []

# ────────────────────────────────────────────────────────────────
# ② 布阵 PlanTree — 生成主题树
# ────────────────────────────────────────────────────────────────
class PlanTree:
    def __init__(self, net):
        self.net = net
        self.seed_topics = {
            '生态学': ['生态系统', '食物链', '生物多样性', '生态系统服务', '生态位',
                       '群落演替', '生态金字塔', '种群生态学'],
            '量子力学': ['量子纠缠', '波函数坍缩', '不确定性原理', '量子隧道效应',
                        '叠加态', '量子退相干', '薛定谔方程'],
            '相对论': ['狭义相对论', '广义相对论', '时空弯曲', '引力波', '质能等价',
                      '光速不变原理', '双生子佯谬'],
            '地质学': ['板块构造论', '岩石循环', '地质年代', '断层与褶皱', '火山学',
                      '沉积作用', '矿物学'],
            '植物学': ['光合作用', '植物组织', '根茎叶结构', '传粉机制', '种子传播',
                      '植物激素', '苔藓植物'],
            '动物学': ['动物分类学', '动物行为学', '迁徙现象', '冬眠机制', '拟态',
                      '捕食关系', '节肢动物'],
            '气候学': ['大气环流', '季风系统', '厄尔尼诺', '温室效应', '古气候',
                      '气候带分布', '气候模型'],
            '语言学': ['音位学', '句法学', '语义学', '语用学', '语言演化', '方言学',
                      '文字起源'],
            '神话学': ['创世神话', '英雄神话', '洪水神话', '神话母题', '原始宗教',
                      '太阳崇拜', '中国神话'],
            '符号学': ['能指与所指', '图像符号', '指示符号', '象征符号', '索绪尔语言学',
                      '皮尔斯三分法', '文化符号'],
            '密码学': ['对称加密', '非对称加密', '哈希函数', '数字签名', 'RSA算法',
                      '量子密码', '密码分析'],
            '控制论': ['反馈控制', '伺服系统', 'PID控制', '自动驾驶', '稳定性理论',
                      '控制系统', '信息论与控制论'],
            '统计学': ['概率分布', '假设检验', '回归分析', '置信区间', '贝叶斯统计',
                      '方差分析', '抽样调查'],
            '运筹学': ['线性规划', '排队论', '博弈论', '动态规划', '最优化方法',
                      '决策论', '库存控制'],
            '教育学': ['认知发展理论', '建构主义', '学习理论', '教育评估', '课程设计',
                      '元认知', '终身学习'],
            '人类学': ['文化相对主义', '民族志方法', '亲属制度', '仪式与象征', '进化人类学',
                      '结构主义人类学', '田野调查'],
            '认知科学': ['工作记忆', '模式识别', '心智模型', '意识研究', '神经可塑性',
                        '语言与认知', '认知偏差'],
            '地缘政治': ['地缘战略', '海权论', '陆权论', '边境地理', '资源地缘政治',
                        '能源走廊', '一带一路地理'],
            '建筑学': ['建筑类型学', '空间组织', '结构体系', '可持续建筑', '地域主义',
                      '建筑符号学', '城市空间'],
            '机械工程': ['机构运动学', '齿轮传动', '内燃机', '材料力学', '公差配合',
                        '液压传动', '机械振动'],
            '电磁学': ['静电场', '电磁感应', '麦克斯韦方程组', '交流电', '电磁波',
                      '洛伦兹力', '电路分析'],
            '密码学与安全': ['零知识证明', '身份认证', '访问控制', '入侵检测', '安全协议',
                           '恶意软件', '网络隔离'],
            '农业': ['作物育种', '土壤肥力', '灌溉技术', '农业生态', '病虫害防治',
                    '精准农业', '粮食安全'],
            '农业育种': ['杂交育种', '基因编辑育种', '分子标记', '品种选育', '种质资源',
                        '倍性育种', '诱变育种'],
            '民俗学': ['民间故事', '节庆仪式', '口头传统', '民间信仰', '民俗分类',
                      '地方知识', '民俗学方法'],
            '文学理论': ['叙事学', '读者反应理论', '结构主义文论', '后殖民批评', '女性主义批评',
                        '形式主义', '解释学'],
            '医学基础': ['细胞生物学', '人体解剖', '病理学', '药理学', '免疫学',
                        '病原微生物', '生理学'],
            '博弈论': ['纳什均衡', '囚徒困境', '帕累托最优', '重复博弈', '零和博弈',
                      '机制设计', '演化博弈'],
            '系统科学': ['系统论', '反馈系统', '自组织', '复杂性', '熵与信息',
                        '涌现现象', '系统动力学'],
            '计算机科学': ['算法复杂度', '数据结构', '编译原理', '操作系统', '分布式系统',
                          '数据库原理', '计算机体系结构'],
            '金融学': ['风险管理', '资产定价', '投资组合', '货币政策', '金融衍生品',
                      '行为金融', '市场微观结构'],
            '天文学': ['恒星演化', '宇宙学原理', '黑洞物理', '行星系统', '星系分类',
                      '暗物质', '宇宙微波背景'],
            '地质学与地理': ['地貌学', '水文循环', '土壤地理', '区域地理', '地图学',
                           '遥感技术', '地理信息系统'],
            '艺术史': ['文艺复兴艺术', '巴洛克艺术', '印象派', '现代主义', '后现代艺术',
                      '中国绘画史', '艺术风格演变'],
            '翻译学': ['翻译策略', '功能对等', '直译意译', '语域转换', '翻译伦理',
                      '机器翻译评估', '文化翻译'],
        }

    def topics_for_domain(self, domain, net_layer, existing_titles, limit=8):
        """Combine: seed topics + wiki search + wiki category + baidu sug, skip existing titles."""
        topics = []
        seen = set()
        for t in self.seed_topics.get(domain, []):
            if t not in seen:
                seen.add(t)
                topics.append(t)
        # expand via wiki search on the domain term
        for t in self.net.zh_wiki_search(domain, limit=10):
            clean = t.replace('（', ' ').split(' ')[0].strip()
            if clean and clean not in seen and len(clean) <= 20:
                seen.add(clean)
                topics.append(clean)
        # expand via wiki category members (domain as category name)
        for t in self.net.zh_wiki_category_members(domain, limit=10):
            clean = t.replace('（', ' ').split(' ')[0].strip()
            if clean and clean not in seen and len(clean) <= 20:
                seen.add(clean)
                topics.append(clean)
        # filter to topics not already in KB for this domain
        fresh = [t for t in topics if t not in existing_titles]
        return fresh[:limit]

# ────────────────────────────────────────────────────────────────
# ③ 攻阵 Crawler — 多源爬取
# ────────────────────────────────────────────────────────────────
class Crawler:
    def __init__(self, net):
        self.net = net

    def _fetch_wiki(self, topic):
        """Try zh wikipedia via proxy (fastest available)."""
        body = self.net.zh_wiki_extract(topic)
        if body and len(body.get('extract') or '') >= MIN_CONTENT_LEN:
            return body
        return None

    def _fetch_search(self, topic):
        """Fallback: 360 search snippet."""
        hits = self.net.so360_search(topic, limit=3)
        if hits:
            h = hits[0]
            return {'title': topic, 'extract': h.get('title', ''),
                    'source': 'search_snippet', 'url': h.get('url'),
                    'domain': None, 'topic': topic}
        return None

    def fetch(self, topic, domain):
        """Race: try wiki + search in parallel, use first successful result."""
        import threading
        result = {}
        event = threading.Event()

        def _wiki():
            r = self._fetch_wiki(topic)
            if r and not event.is_set():
                result.update(r)
                event.set()

        def _search():
            r = self._fetch_search(topic)
            if r and not event.is_set():
                result.update(r)
                event.set()

        t1 = threading.Thread(target=_wiki, daemon=True)
        t2 = threading.Thread(target=_search, daemon=True)
        t1.start(); t2.start()
        event.wait(timeout=12)
        if result:
            result['domain'] = domain
            result['topic'] = topic
            return result
        return None

# ────────────────────────────────────────────────────────────────
# ④ 收阵 Mint — 3D 结构化 + 入库
# ────────────────────────────────────────────────────────────────
class Mint:
    STRUCTURE_MARKERS = {
        'principle': ['原理', '本质', '核心', '基本', '机制', '规律', '定义', '由'],
        'surface': ['表现', '现象', '特征', '外观', '形式', '例如', '如'],
        'number': ['%', '亿', '万', '千', '百', '公里', '米', '年', '个数', '约', '平均'],
    }

    def to_3d(self, raw):
        """Split extract into 原理·表象·数 three sections."""
        text = raw.get('extract', '')
        n = len(text)
        if n <= MIN_CONTENT_LEN:
            return {'principle': text[:max(60, n // 2)],
                    'surface': '', 'number': ''}
        # heuristic split into ~3 balanced-ish chunks
        third = max(MIN_CONTENT_LEN, n // 3)
        # find sentence boundaries near each split point
        def split_near(pos):
            for step in range(0, 40):
                for i in (pos + step, pos - step):
                    if i < n and i > third // 2 and text[i] in '。！？.!?':
                        return i + 1
            return min(n, pos + 40)
        p_end = split_near(third)
        s_end = split_near(third * 2)
        principle = text[:p_end].strip()
        surface = text[p_end:s_end].strip()
        number = text[s_end:].strip()
        return {'principle': principle, 'surface': surface, 'number': number}

    def _content_from_3d(self, d3, raw):
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
        content = self._content_from_3d(d3, raw)
        summary = f"{domain}——{title}：{raw.get('extract','')[:80]}"
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
               AND domain NOT GLOB '*[a-zA-Z0-9]*' AND domain NOT GLOB '*/ *'
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
            if re.search(r'[a-zA-Z0-9]', d) or '/' in d or '@' in d or '→' in d:
                continue
            if len(d) < 2 or len(d) > 30:
                continue
            weak.append((d, cnt))
        return sorted(weak, key=lambda x: (x[1], x[0]))

# ────────────────────────────────────────────────────────────────
# ⑥ 疗阵 Heal — 代理池 / 死源 / 节流
# ────────────────────────────────────────────────────────────────
class Heal:
    @staticmethod
    def refresh_proxy_pool(state):
        """Re-read upstreams conf; if stale (>1h), try to refresh via network."""
        try:
            mtime = os.path.getmtime(PROXY_CONF)
            if time.time() - mtime > 3600:
                # best-effort: reload from pool state json (in-memory healthy nodes)
                pass
        except OSError:
            pass
        return state

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
    with open(LOG_PATH, 'a') as f:
        f.write(json.dumps(entry, ensure_ascii=False) + '\n')
        f.flush()
    tag = {'INFO': '', 'OK': '✅', 'WARN': '⚠️', 'ERROR': '❌'}.get(level, '')
    print(f"[{time.strftime('%H:%M:%S')}][{phase}] {tag} {msg}", flush=True)

def run_cycle(conn, net, state, args):
    phase_stats = {'scanned': 0, 'planned': 0, 'fetched': 0, 'inserted': 0,
                   'duplicates': 0, 'failed': 0}

    # ① 观阵
    gap = GapScan(conn)
    weak = gap.weak_domains(max_entries=args.max_entries)
    if args.domains:
        weak = [(gap.total_entries(d), d) for d in args.domains]
    phase_stats['scanned'] = len(weak)
    log('INFO', '观阵', f'扫描到 {len(weak)} 个薄弱域')

    targets = weak
    if args.topics:
        # prioritize domains closest to target, cap total topics
        pass

    total_topics = 0
    for cnt, domain in targets:
        if total_topics >= args.topics:
            break
        existing = gap.existing_titles(domain)
        plan = PlanTree(net)
        topics = plan.topics_for_domain(domain, net, existing, limit=args.topics - total_topics)
        if not topics:
            log('INFO', '布阵', f'{domain}: 无新主题 (已有 {len(existing)})')
            continue
        phase_stats['planned'] += len(topics)
        log('INFO', '布阵', f'{domain}: 规划 {len(topics)} 主题 -> {topics[:5]}')

        for topic in topics:
            if total_topics >= args.topics:
                break
            total_topics += 1
            crawler = Crawler(net)
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
    return phase_stats

def main():
    ap = argparse.ArgumentParser(description='NeoTrix 阵法节点引擎')
    ap.add_argument('--dry-run', action='store_true', help='演练一轮不写库')
    ap.add_argument('--topics', type=int, default=30, help='每 cycle 最多主题数')
    ap.add_argument('--sleep', type=float, default=2.0, help='主题间间隔秒')
    ap.add_argument('--domains', nargs='*', help='只处理指定域')
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

    log('INFO', '启动', '阵法节点引擎 v0.1 启动 (dry_run=%s)' % args.dry_run)

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

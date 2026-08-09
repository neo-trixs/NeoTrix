#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
kb_book_fetch.py — 书籍元数据获取节点（数据摄取能力）

让 KB 的 book 节点持续从外部书目源补全元数据（标题/作者/年份/出版社/ISBN/主题/简介）。
对标网文"设定库持续考据"：知识库不是一次性灌入，而是可重复运行的摄取管线。

合规边界（HARD）:
  - 只获取【书目元数据】（bibliographic metadata），绝不下载/存储书籍全文
  - 数据源: OpenLibrary(免费无key) / Google Books(免费key) / Anna's Archive 元数据 API(第三方,需key)
  - 写入 KB 的 url 是书目页链接（来源引用），不是文件下载链接

用法:
  python3 scripts/kb_book_fetch.py --mode search --query "philosophy of mind" --limit 20
  python3 scripts/kb_book_fetch.py --mode fill            # 补全 KB 中缺 summary 的 book 节点
  python3 scripts/kb_book_fetch.py --mode report          # KB 书籍元数据健康报告
  python3 scripts/kb_book_fetch.py --mode search --query "量子力学" --source openlibrary

环境变量:
  GOOGLE_BOOKS_API_KEY   # Google Books API key (可选)
  ANNA_ARCHIVE_API_KEY   # Anna's Archive 第三方元数据 API key (可选)
"""

import argparse
import hashlib
import json
import os
import re
import sqlite3
import sys
import time
import urllib.parse
import urllib.request
from pathlib import Path

KB = os.environ.get("NEOTRIX_KB", str(Path.home() / ".neotrix" / "knowledge.db"))

# ---------------------------------------------------------------- 规范化

_REMOVE = re.compile(r"[\s\-_·•:：,，.。;；!！?？()（）\[\]【】\"'“”‘’/\\|]+")


def to_simplified(s: str) -> str:
    """繁→简（无 opencc 时降级为原样）"""
    try:
        from opencc import OpenCC
        return OpenCC("t2s").convert(s)
    except Exception:
        return s


def norm_title(t: str) -> str:
    """规范化书名 → 去重键: 繁→简 + 去标点空格"""
    return _REMOVE.sub("", to_simplified(t or "").strip())


def node_id(prefix: str, key: str) -> str:
    return f"{prefix}-{hashlib.md5(key.encode()).hexdigest()[:16]}"


def book_id(title: str) -> str:
    return node_id("bk", norm_title(title))


def topic_id(topic: str) -> str:
    return node_id("topic", norm_title(topic))


# ---------------------------------------------------------------- DB 写入层
# 复用 absorb_guji.py 的 KBWriter 模式（镜像 nt_memory_store::insert_node 事务双写）

def _now_ts() -> int:
    return int(time.time())


class KBWriter:
    def __init__(self, path=os.environ.get("NEOTRIX_KB", str(Path.home() / ".neotrix" / "knowledge.db"))):
        self.conn = sqlite3.connect(path, timeout=60)
        self.conn.execute("PRAGMA journal_mode=WAL")
        self.conn.execute("PRAGMA synchronous=NORMAL")
        self.conn.execute("PRAGMA cache_size=-200000")
        self.cur = self.conn.cursor()
        self.stats = {"insert_book": 0, "merge_book": 0, "insert_topic": 0, "insert_edge": 0,
                      "insert_concept": 0, "insert_source": 0}

    def node_exists(self, nid: str) -> bool:
        self.cur.execute("SELECT 1 FROM nodes WHERE id=?", (nid,))
        return self.cur.fetchone() is not None

    def get_node(self, nid: str):
        self.cur.execute(
            "SELECT node_type,title,summary,content,url,domain,language,metadata,source_episode "
            "FROM nodes WHERE id=?", (nid,))
        return self.cur.fetchone()

    def insert_node(self, nid, node_type, title, summary, content, url, domain, language,
                    importance=0.5, metadata=None, source_episode=None):
        if metadata is None:
            metadata = {}
        mjson = json.dumps(metadata, ensure_ascii=False)
        now = _now_ts()
        self.cur.execute(
            "INSERT INTO nodes (id,node_type,title,summary,content,url,domain,language,"
            "confidence,importance,created_at,updated_at,access_count,metadata,"
            "data_tier,temporal,supersedes,source_episode,tier) "
            "VALUES (?,?,?,?,?,?,?,?,?,?,?,?,0,?,'core',NULL,NULL,?,'warm')",
            (nid, node_type, title, summary, content, url, domain, language,
             1.0, importance, now, now, mjson, source_episode))
        self.cur.execute(
            "INSERT INTO nodes_fts (rowid,title,summary,content,domain) "
            "VALUES (last_insert_rowid(),?,?,?,?)",
            (title, summary or "", content or "", domain))
        self.stats["insert_" + ("book" if node_type == "book" else node_type)] += 1

    def merge_node(self, nid, node_type, title, summary, content, url, domain, language,
                   importance=0.5, metadata=None, source_episode=None):
        """已存在 → 合并: 保留已有 content, 补充 url/metadata (已有键优先)"""
        row = self.get_node(nid)
        if row is None:
            self.insert_node(nid, node_type, title, summary, content, url, domain, language,
                             importance, metadata, source_episode)
            return
        _, old_title, old_summary, old_content, old_url, old_domain, old_lang, old_meta, old_ep = row
        merged = {}
        try:
            merged = json.loads(old_meta) if old_meta else {}
        except Exception:
            pass
        if metadata:
            for k, v in metadata.items():
                if k == "sources":
                    srcs = merged.get("sources", [])
                    for s in v:
                        if s not in srcs:
                            srcs.append(s)
                    merged["sources"] = srcs
                elif k not in merged:
                    merged[k] = v
        final_content = old_content or content or ""
        final_url = old_url or url or ""
        now = _now_ts()
        self.cur.execute(
            "UPDATE nodes SET summary=?, content=?, url=?, updated_at=?, metadata=? WHERE id=?",
            (old_summary or summary, final_content, final_url, now,
             json.dumps(merged, ensure_ascii=False), nid))
        self.cur.execute(
            "UPDATE nodes_fts SET title=?, summary=?, content=?, domain=? "
            "WHERE rowid=(SELECT rowid FROM nodes WHERE id=?)",
            (old_title or title, old_summary or summary, final_content, domain, nid))
        self.stats["merge_book"] += 1

    def upsert_node(self, nid, node_type, title, summary, content, url, domain, language,
                    importance=0.5, metadata=None, source_episode=None):
        if self.node_exists(nid):
            self.merge_node(nid, node_type, title, summary, content, url, domain, language,
                            importance, metadata, source_episode)
        else:
            self.insert_node(nid, node_type, title, summary, content, url, domain, language,
                             importance, metadata, source_episode)

    def add_edge(self, src, tgt, rel, weight=1.0, desc=""):
        eid = hashlib.md5(f"{src}|{tgt}|{rel}".encode()).hexdigest()
        try:
            self.cur.execute(
                "INSERT OR IGNORE INTO edges (id,source_id,target_id,relation_type,weight,description,created_at) "
                "VALUES (?,?,?,?,?,?,?)",
                (eid, src, tgt, rel, weight, desc, _now_ts()))
            self.stats["insert_edge"] += 1
        except sqlite3.IntegrityError:
            pass

    def commit(self):
        self.conn.commit()

    def report(self):
        return dict(self.stats)


# ---------------------------------------------------------------- 数据源适配器

class BookRecord:
    """规范化书目元数据"""
    def __init__(self, title, author="", year="", publisher="", isbn="",
                 topic="", summary="", url="", source="", extra=None):
        self.title = title
        self.author = author
        self.year = year
        self.publisher = publisher
        self.isbn = isbn
        self.topic = topic
        self.summary = summary
        self.url = url
        self.source = source
        self.extra = extra or {}

    def to_metadata(self):
        m = {
            "author": self.author, "year": self.year, "publisher": self.publisher,
            "isbn": self.isbn, "topic": self.topic, "source": self.source,
        }
        m.update(self.extra)
        return m

    def to_summary(self):
        parts = []
        if self.author:
            parts.append(f"By {self.author}")
        if self.year:
            parts.append(self.year)
        if self.publisher:
            parts.append(f"Publisher: {self.publisher}")
        if self.isbn:
            parts.append(f"ISBN: {self.isbn}")
        return " · ".join(parts)


def _http_get_json(url, timeout=20):
    req = urllib.request.Request(url, headers={"User-Agent": "NeoTrix-KB-Fetch/1.0"})
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read().decode("utf-8"))


class OpenLibraryAdapter:
    """OpenLibrary Search API — 免费无 key"""
    name = "openlibrary"
    base = "https://openlibrary.org/search.json"

    def search(self, query: str, limit: int = 20):
        params = urllib.parse.urlencode({"q": query, "limit": min(limit, 100), "fields": "title,author_name,first_publish_year,publisher,isbn,subject,key"})
        data = _http_get_json(f"{self.base}?{params}")
        records = []
        for d in data.get("docs", [])[:limit]:
            title = d.get("title") or ""
            if not title:
                continue
            subjects = d.get("subject") or []
            topic = subjects[0] if subjects else ""
            records.append(BookRecord(
                title=title,
                author=", ".join(d.get("author_name") or [])[:200],
                year=str(d.get("first_publish_year") or ""),
                publisher=", ".join(d.get("publisher") or [])[:200],
                isbn=", ".join(d.get("isbn") or [])[:200],
                topic=topic,
                url=f"https://openlibrary.org{d.get('key','')}" if d.get("key") else "",
                source="openlibrary",
            ))
        return records


class GoogleBooksAdapter:
    """Google Books API — 免费 key (可选)"""
    name = "googlebooks"
    base = "https://www.googleapis.com/books/v1/volumes"

    def search(self, query: str, limit: int = 20):
        key = os.environ.get("GOOGLE_BOOKS_API_KEY", "")
        params = {"q": query, "maxResults": min(limit, 40)}
        if key:
            params["key"] = key
        data = _http_get_json(f"{self.base}?{urllib.parse.urlencode(params)}")
        records = []
        for item in data.get("items", [])[:limit]:
            vi = item.get("volumeInfo", {})
            title = vi.get("title") or ""
            if not title:
                continue
            authors = vi.get("authors") or []
            industry = vi.get("industryIdentifiers") or []
            isbn = next((i.get("identifier", "") for i in industry if i.get("type") in ("ISBN_13", "ISBN_10")), "")
            records.append(BookRecord(
                title=title,
                author=", ".join(authors)[:200],
                year=str(vi.get("publishedDate") or "")[:4],
                publisher=(vi.get("publisher") or "")[:200],
                isbn=isbn,
                topic=", ".join(vi.get("categories") or [])[:200],
                summary=(vi.get("description") or "")[:2000],
                url=vi.get("infoLink", ""),
                source="googlebooks",
            ))
        return records


class AnnaArchiveAdapter:
    """Anna's Archive 元数据源 — 第三方元数据 API (需 key, 只返回书目元数据)

    说明: Anna's Archive 无官方公开 API; 本适配器对接第三方元数据 API
    (如 parse.bot / tribestick 等, 见 websearch 调研)。只取 title/author/year/md5 等
    书目元数据, 不请求任何下载链接。未配置 key 时自动跳过。
    """
    name = "anna_archive"

    def __init__(self):
        self.key = os.environ.get("ANNA_ARCHIVE_API_KEY", "")
        self.endpoint = os.environ.get(
            "ANNA_ARCHIVE_API_ENDPOINT",
            "https://api.parse.bot/scraper/5bcf2b80-0b98-4ac4-925d-60ac03365462/search",
        )

    def available(self) -> bool:
        return bool(self.key)

    def search(self, query: str, limit: int = 20):
        if not self.available():
            return []
        params = urllib.parse.urlencode({"query": query, "limit": min(limit, 50)})
        req = urllib.request.Request(
            f"{self.endpoint}?{params}",
            headers={"X-API-Key": self.key, "User-Agent": "NeoTrix-Kot-Fetch/1.0"},
        )
        with urllib.request.urlopen(req, timeout=25) as resp:
            data = json.loads(resp.read().decode("utf-8"))
        records = []
        for r in data.get("results", [])[:limit]:
            title = r.get("title") or ""
            if not title:
                continue
            records.append(BookRecord(
                title=title,
                author=(r.get("author") or "")[:200],
                year=str(r.get("year") or ""),
                url=r.get("url") or "",
                topic=r.get("content_type") or "",
                source="anna_archive",
                extra={"md5": r.get("md5", ""), "extension": r.get("extension", "")},
            ))
        return records


ADAPTERS = {
    "openlibrary": OpenLibraryAdapter,
    "googlebooks": GoogleBooksAdapter,
    "anna_archive": AnnaArchiveAdapter,
}


# ---------------------------------------------------------------- 吸收

def absorb_records(w: KBWriter, records, domain="book"):
    """把书目元数据写入 KB: book 节点 + topic 概念节点 + book→topic 边"""
    for rec in records:
        if not rec.title:
            continue
        nid = book_id(rec.title)
        w.upsert_node(
            nid, "book", rec.title, rec.to_summary(), rec.summary,
            rec.url, domain, "en", importance=0.6,
            metadata=rec.to_metadata(), source_episode=f"fetch:{rec.source}",
        )
        # 主题概念节点 + 边
        if rec.topic:
            tid = topic_id(rec.topic)
            w.upsert_node(tid, "concept", rec.topic, "", "", "", domain, "en",
                          importance=0.4, metadata={"source": rec.source})
            w.add_edge(nid, tid, "about_topic", weight=0.8, desc=f"topic from {rec.source}")


def mode_search(w: KBWriter, query: str, source: str, limit: int):
    """搜索模式: 从指定源搜索书目元数据并吸收"""
    adapter_cls = ADAPTERS.get(source)
    if adapter_cls is None:
        print(f"[error] 未知数据源: {source} (可选: {', '.join(ADAPTERS)})")
        return
    adapter = adapter_cls()
    if source == "anna_archive" and not adapter.available():
        print("[warn] ANNA_ARCHIVE_API_KEY 未配置, 跳过 anna_archive 源 (仅元数据, 无全文)")
        return
    print(f"[fetch] {source}: query='{query}' limit={limit}")
    records = adapter.search(query, limit)
    print(f"[fetch] 获取 {len(records)} 条书目元数据")
    absorb_records(w, records)
    w.commit()
    print(f"[absorb] {w.report()}")


def mode_fill(w: KBWriter, limit: int):
    """补全模式: 从 KB 读取缺 summary 的 book 节点, 用 OpenLibrary 补全元数据"""
    w.cur.execute(
        "SELECT id, title FROM nodes "
        "WHERE node_type='book' AND (summary IS NULL OR summary='') "
        "AND domain NOT LIKE '%.%' AND domain NOT IN ('guji','scripta','ancient') "
        "LIMIT ?", (limit,))
    rows = w.cur.fetchall()
    print(f"[fill] 待补全 book 节点: {len(rows)}")
    adapter = OpenLibraryAdapter()
    filled = 0
    for nid, title in rows:
        try:
            records = adapter.search(title, limit=1)
            if records:
                rec = records[0]
                w.merge_node(nid, "book", rec.title, rec.to_summary(), rec.summary,
                             rec.url, "book", "en", importance=0.6,
                             metadata=rec.to_metadata(), source_episode="fetch:openlibrary")
                filled += 1
        except Exception as e:
            print(f"  [warn] {title[:40]}: {e}")
        time.sleep(0.3)  # 礼貌限速
    w.commit()
    print(f"[fill] 补全 {filled}/{len(rows)} 节点")


def mode_report(w: KBWriter):
    """健康报告: KB 书籍元数据现状"""
    w.cur.execute("SELECT COUNT(*) FROM nodes WHERE node_type='book'")
    total = w.cur.fetchone()[0]
    w.cur.execute("SELECT COUNT(*) FROM nodes WHERE node_type='book' AND (summary IS NULL OR summary='')")
    no_summary = w.cur.fetchone()[0]
    w.cur.execute("SELECT COUNT(*) FROM nodes WHERE node_type='book' AND metadata LIKE '%\"isbn\"%'")
    has_isbn = w.cur.fetchone()[0]
    w.cur.execute("SELECT COUNT(*) FROM nodes WHERE node_type='book' AND metadata LIKE '%\"topic\"%'")
    has_topic = w.cur.fetchone()[0]
    print("=== KB 书籍元数据健康报告 ===")
    print(f"  book 节点总数: {total}")
    print(f"  缺 summary: {no_summary} ({100*no_summary/max(total,1):.0f}%)")
    print(f"  有 isbn: {has_isbn} ({100*has_isbn/max(total,1):.0f}%)")
    print(f"  有 topic: {has_topic} ({100*has_topic/max(total,1):.0f}%)")


# ---------------------------------------------------------------- main

def main():
    import argparse
    ap = argparse.ArgumentParser(description="书籍元数据获取节点 (KB 摄取管线)")
    ap.add_argument("--mode", choices=["search", "fill", "report"], default="report")
    ap.add_argument("--query", default="")
    ap.add_argument("--source", choices=list(ADAPTERS), default="openlibrary")
    ap.add_argument("--limit", type=int, default=20)
    args = ap.parse_args()

    w = KBWriter()
    if args.mode == "search":
        if not args.query:
            print("[error] --mode search 需要 --query")
            return
        mode_search(w, args.query, args.source, args.limit)
    elif args.mode == "fill":
        mode_fill(w, args.limit)
    else:
        mode_report(w)


if __name__ == "__main__":
    main()
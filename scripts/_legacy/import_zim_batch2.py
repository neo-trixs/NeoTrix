#!/usr/bin/env python3
"""
ZIM 批次导入脚本 - 第二批次：高价值 Comprehensive 文件
目标: Electronics Q&A (1.2M) + Wikipedia Popular (927K) + iFixit (894K) + 其他
使用: python3 scripts/import_zim_batch2.py
"""
import sqlite3, json, uuid, time, os, re, hashlib, struct, glob, sys
import numpy as np

# 配置
DB = os.path.expanduser("~/.neotrix/knowledge.db")
ZIM_DIR = "/Volumes/NeoTrixBrain/cortex-archive/zim"
DIM = 384
BATCH_SIZE = 5000
MAX_PER_FILE = 100000  # 每文件上限

# 第二批次优先级文件
PRIORITY_FILES = [
    "Electronics_Q&A_(Comprehensive)_-_Electronics_Stack_Exchange.zim",
    "Wikipedia__Popular_Articles.zim",
    "DIY___Repair_(Comprehensive)_-_iFixit_Repair_Guides.zim",
    "Wikipedia__Quick_Reference.zim",
    "DIY___Repair_(Standard)_-_DIY___Home_Improvement.zim",
    "Medicine_(Comprehensive)_-_Wikipedia_Medicine.zim",
    "Education___Reference_(Comprehensive)_-_Wikibooks.zim",
    "Education___Reference_(Standard)_-_LibreTexts_Chemistry.zim",
    "Education___Reference_(Essential)_-_Wikibooks.zim",
    "Education___Reference_(Standard)_-_Wikiversity.zim",
]

# 加载 MiniLM 模型 (如果可用)
def load_embedder():
    try:
        from sentence_transformers import SentenceTransformer
        model = SentenceTransformer('all-MiniLM-L6-v2')
        print("✅ 已加载 MiniLM (all-MiniLM-L6-v2)")
        return model
    except ImportError:
        print("⚠️ 未安装 sentence_transformers，使用 hash-kernel")
        return None

MODEL = load_embedder()

def mini_embed(text, dim=384):
    if MODEL:
        vec = MODEL.encode(text[:500], normalize_embeddings=True)
        return vec.astype(np.float32)
    # fallback: hash-kernel
    h = hashlib.sha256(text.encode()).digest()
    seed = struct.unpack('<Q', h[:8])[0]
    rng = np.random.default_rng(seed)
    v = rng.standard_normal(dim).astype(np.float32)
    n = np.linalg.norm(v)
    return v/n if n > 0 else v

TAG_RE = re.compile(r'<(script|style)[^>]*>.*?</\1>', re.I|re.S)
HTML_RE = re.compile(r'<[^>]+>')

def html_to_text(raw):
    try: s = raw.decode('utf-8', errors='replace')
    except: return ""
    s = TAG_RE.sub(' ', s)
    s = re.sub(r'<br\s*/?>', '\n', s, flags=re.I)
    s = HTML_RE.sub('', s).strip()[:8000]
    from html import unescape
    return unescape(s)

def import_file(zp_path, conn, cur, file_stats):
    """导入单个 ZIM 文件"""
    base = os.path.basename(zp_path).replace(".zim", "")[:45]
    
    try:
        sys.path.insert(0, os.path.expanduser("~/.neotrix/venv/lib/python3.14/site-packages"))
        from libzim.reader import Archive
        arc = Archive(zp_path)
        zuuid = str(arc.uuid)
    except Exception as e:
        print(f"  ✗ {base}: 打开失败 - {e}")
        return 0
    
    # 检查已导入数量
    cur.execute("SELECT COUNT(*) FROM nodes WHERE url LIKE ?", (f"zimid://{zuuid}/%",))
    existing = cur.fetchone()[0]
    total = arc.entry_count
    pct = int(100 * existing / max(total, 1))
    
    if pct >= 80:
        print(f"  ⏭ {base}: 已覆盖 {pct}% ({existing}/{total})")
        return 0
    
    print(f"  📥 {base}: 总计 {total:,}, 已有 {existing:,} ({pct}%)")
    
    # 创建源节点
    source_id = cur.execute("SELECT id FROM nodes WHERE url=?", (f"zim_source://{zuuid}",)).fetchone()
    if not source_id:
        source_id = str(uuid.uuid4())
        now = int(time.time())
        cur.execute("""INSERT OR IGNORE INTO nodes
            (id,node_type,title,summary,content,url,domain,language,confidence,importance,created_at,updated_at,metadata,tier)
            VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)""",
            (source_id, "source", f"ZIM: {base}", f"ZIM archive with {total} entries",
             f"ZIM archive: {base} ({total} entries)", f"zim_source://{zuuid}", "zim", "en",
             1.0, 0.9, now, now, json.dumps({"zim_uuid": zuuid, "entry_count": total}), "core"))
        # embedding
        h = hashlib.sha256(base.encode()).digest()
        seed = struct.unpack('<Q', h[:8])[0]
        rng = np.random.default_rng(seed)
        v = rng.standard_normal(DIM).astype(np.float32)
        n = np.linalg.norm(v)
        if n > 0: v = v/n
        blob = struct.pack('384f', *v)
        cur.execute("INSERT OR REPLACE INTO embeddings VALUES (?,?,?,?)", (source_id, blob, DIM, "hash-kernel-v1"))
    
    imported = 0
    batch_nodes = []
    batch_embs = []
    
    start_idx = min(existing, total - 1)
    
    for idx in range(max(0, start_idx), total):
        if imported >= MAX_PER_FILE:
            break
            
        try:
            entry = arc._get_entry_by_id(idx)
            p = entry.path.lstrip('./')
            url_key = f"zimid://{zuuid}/{p}"
            
            # 去重检查
            if cur.execute("SELECT 1 FROM nodes WHERE url=?", (url_key,)).fetchone():
                continue
            
            item = entry.get_item()
            if not item.mimetype or 'html' not in item.mimetype: continue
            if p.startswith('-/'): continue
            if any(p.lower().endswith(e) for e in ['.png','.jpg','.css','.js','.svg','.ico','.woff','.ttf']): continue
            
            raw = bytes(item.content)
            if len(raw) < 500: continue  # 太小跳过
            
            clean = html_to_text(raw)
            if len(clean) < 200: continue
            
            title = (entry.title or p.split('/')[-1])[:180]
            nid = str(uuid.uuid4())
            
            # 生成 embedding
            vec = mini_embed(clean[:500])
            blob = vec.astype(np.float32).tobytes()
            
            batch_nodes.append((nid, "article", title, clean[:300], clean[:5000], url_key,
                               base[:30], "en", 1.0, 0.5, int(time.time()), int(time.time()),
                               json.dumps({"zim_uuid": zuuid, "zim_path": p}), "warm"))
            batch_embs.append((nid, blob, DIM, "all-MiniLM-L6-v2" if MODEL else "hash-kernel-v1"))
            
            imported += 1
            
            if len(batch_nodes) >= BATCH_SIZE:
                cur.executemany("""INSERT OR IGNORE INTO nodes
                    (id,node_type,title,summary,content,url,domain,language,
                     confidence,importance,created_at,updated_at,metadata,tier)
                    VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)""", batch_nodes)
                cur.executemany("INSERT OR REPLACE INTO embeddings VALUES (?,?,?,?)", batch_embs)
                # part_of 边
                cur.executemany("""INSERT OR IGNORE INTO edges
                    (id, source_id, target_id, relation_type, weight, description, created_at, metadata, valid_from, valid_until, superseded_by)
                    VALUES (?, ?, ?, 'part_of', 0.9, ?, ?, '{}', 0, 0, NULL)""",
                    [(str(uuid.uuid4()), nid, source_id, f"entry of {base}", int(time.time())) 
                     for nid, *_ in batch_nodes])
                conn.commit()
                batch_nodes = []
                batch_embs = []
                print(f"    进度: +{imported}/{total} ({100*imported/total:.1f}%)")
                
        except Exception as e:
            continue
    
    # 写入剩余
    if batch_nodes:
        cur.executemany("""INSERT OR IGNORE INTO nodes
            (id,node_type,title,summary,content,url,domain,language,
             confidence,importance,created_at,updated_at,metadata,tier)
            VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)""", batch_nodes)
        cur.executemany("INSERT OR REPLACE INTO embeddings VALUES (?,?,?,?)", batch_embs)
        cur.executemany("""INSERT OR IGNORE INTO edges
            (id, source_id, target_id, relation_type, weight, description, created_at, metadata, valid_from, valid_until, superseded_by)
            VALUES (?, ?, ?, 'part_of', 0.9, ?, ?, '{}', 0, 0, NULL)""",
            [(str(uuid.uuid4()), nid, source_id, f"entry of {base}", int(time.time())) 
             for nid, *_ in batch_nodes])
        conn.commit()
    
    # FTS 更新
    cur.execute("""
        INSERT OR IGNORE INTO nodes_fts(rowid, title, summary, content)
        SELECT rowid, title, summary, content FROM nodes
        WHERE rowid NOT IN (SELECT rowid FROM nodes_fts) AND content IS NOT NULL AND length(content)>50
    """)
    conn.commit()
    
    print(f"  ✅ {base}: 完成 +{imported:,} 条目")
    file_stats[base] = {"imported": imported, "total": total, "existing": existing}
    return imported

def main():
    print("=" * 60)
    print("ZIM 批次导入 - 第二批次 (高价值 Comprehensive 文件)")
    print("=" * 60)
    
    conn = sqlite3.connect(DB, timeout=120)
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA busy_timeout=60000")
    cur = conn.cursor()
    
    file_stats = {}
    total_imported = 0
    
    for fname in PRIORITY_FILES:
        zp = os.path.join(ZIM_DIR, fname)
        if not os.path.exists(zp):
            print(f"  ⚠️ 文件不存在: {fname}")
            continue
        
        imported = import_file(zp, conn, cur, file_stats)
        total_imported += imported
        
        # 每文件后 checkpoint
        cur.execute("PRAGMA wal_checkpoint(TRUNCATE)")
    
    print("\n" + "=" * 60)
    print("导入完成汇总")
    print("=" * 60)
    for base, stats in file_stats.items():
        print(f"  {base}: +{stats['imported']:,} / {stats['total']:,} ({100*stats['imported']/max(stats['total'],1):.1f}%)")
    print(f"总计导入: {total_imported:,} 条目")
    
    # 最终统计
    cur.execute("SELECT COUNT(*) FROM nodes WHERE url LIKE 'zimid://%'")
    print(f"ZIM 总节点: {cur.fetchone()[0]:,}")
    cur.execute("SELECT COUNT(*) FROM nodes")
    print(f"KB 总节点: {cur.fetchone()[0]:,}")
    
    conn.close()

if __name__ == "__main__":
    main()

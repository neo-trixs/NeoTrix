#!/usr/bin/env python3
"""Clean up crawl queue by marking completed arxiv.org entries already in KB."""
import sqlite3, time, os

def main():
    db = os.environ.get("KB_PATH", os.path.expanduser("~/.neotrix/knowledge.db"))
    conn = sqlite3.connect(db)
    c = conn.cursor()

    c.execute("SELECT id, url FROM crawl_queue WHERE domain='arxiv.org' AND status='pending'")
    pending = c.fetchall()
    print(f"Pending: {len(pending)}")

    now = int(time.time())
    cnt = 0
    for cq_id, url in pending:
        pid = url.split('/')[-1].strip()
        like_pattern = f'%arxiv.org/abs/{pid}%'
        c.execute("SELECT count(*) FROM knowledge_nodes WHERE url LIKE ?", (like_pattern,))
        if c.fetchone()[0] > 0:
            c.execute("UPDATE crawl_queue SET status='completed', last_attempt=? WHERE id=?", (now, cq_id))
            cnt += 1

    conn.commit()
    print(f"Marked completed (already in KB): {cnt}")

    c.execute("SELECT status, count(*) FROM crawl_queue WHERE domain='arxiv.org' GROUP BY status")
    for r in c.fetchall():
        print(f"  {r[0]}: {r[1]}")
    conn.close()

if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""
外贸知识库快速查询工具

提供对导入的富通天下 CRM 数据的命令行查询接口。

使用方式:
    python3 trade_query.py stats
    python3 trade_query.py search <query>
    python3 trade_query.py country <country>
    python3 trade_query.py grade <grade>
    python3 trade_query.py followup [days]
    python3 trade_query.py operator <name>
    python3 trade_query.py product <query>
"""

import sqlite3
import sys
from pathlib import Path
from datetime import datetime

# ── 配置 ──────────────────────────────────────────────────────

NEOTRIX_DB_PATH = Path.home() / ".neotrix" / "trade_knowledge.db"


def open_db():
    if not NEOTRIX_DB_PATH.exists():
        print(f"❌ 数据库不存在: {NEOTRIX_DB_PATH}")
        sys.exit(1)
    return sqlite3.connect(str(NEOTRIX_DB_PATH))


def cmd_stats(conn):
    """显示统计信息"""
    total = conn.execute("SELECT COUNT(*) FROM customers").fetchone()[0]
    
    grade_dist = conn.execute(
        "SELECT grade, COUNT(*) FROM customers GROUP BY grade ORDER BY COUNT(*) DESC"
    ).fetchall()
    
    country_dist = conn.execute(
        "SELECT country, COUNT(*) FROM customers WHERE country != '' GROUP BY country ORDER BY COUNT(*) DESC LIMIT 10"
    ).fetchall()
    
    operator_count = conn.execute("SELECT COUNT(*) FROM operators").fetchone()[0]
    product_count = conn.execute("SELECT COUNT(*) FROM product_categories").fetchone()[0]
    country_count = conn.execute("SELECT COUNT(*) FROM countries").fetchone()[0]
    
    print("📊 Trade Knowledge Base Stats")
    print("═" * 40)
    print(f"Customers:      {total}")
    print(f"Operators:      {operator_count}")
    print(f"Products:       {product_count}")
    print(f"Countries:      {country_count}")
    print(f"\n📈 Grade Distribution:")
    for grade, count in grade_dist:
        pct = int(count / total * 100) if total > 0 else 0
        print(f"  {grade:<4} {count:<6} ({pct}%)")
    print(f"\n🌍 Top Countries:")
    for country, count in country_dist:
        print(f"  {country:<20} {count}")


def cmd_search(conn, query):
    """搜索客户"""
    pattern = f"%{query}%"
    rows = conn.execute(
        """SELECT customer_id, name, grade, country, owner, contact_name, channel
           FROM customers
           WHERE name LIKE ? OR contact_name LIKE ? OR description LIKE ?
           ORDER BY grade DESC, last_follow_at DESC
           LIMIT 20""",
        (pattern, pattern, pattern)
    ).fetchall()
    
    if not rows:
        print(f"🔍 未找到匹配 '{query}' 的客户")
        return
    
    print(f"🔍 Search Results for '{query}' ({len(rows)} found):")
    print()
    for row in rows:
        print(f"[{row[0]}] {row[1]} | Grade:{row[2]} | {row[3]} | Contact:{row[5]} | Owner:{row[4]} | {row[6]}")


def cmd_country(conn, country):
    """按国家搜索"""
    pattern = f"%{country.upper()}%"
    rows = conn.execute(
        """SELECT customer_id, name, grade, contact_name, owner, channel
           FROM customers
           WHERE country LIKE ? OR region LIKE ?
           ORDER BY grade DESC
           LIMIT 20""",
        (pattern, pattern)
    ).fetchall()
    
    if not rows:
        print(f"🌍 未找到 '{country}' 的客户")
        return
    
    print(f"🌍 Customers in '{country}' ({len(rows)} found):")
    print()
    for row in rows:
        print(f"[{row[0]}] {row[1]} | Grade:{row[2]} | Contact:{row[3]} | Owner:{row[4]} | {row[5]}")


def cmd_grade(conn, grade):
    """按等级搜索"""
    grade = grade.upper()
    rows = conn.execute(
        """SELECT customer_id, name, country, contact_name, owner, channel
           FROM customers
           WHERE grade = ?
           ORDER BY last_follow_at DESC
           LIMIT 20""",
        (grade,)
    ).fetchall()
    
    total = conn.execute(
        "SELECT COUNT(*) FROM customers WHERE grade = ?", (grade,)
    ).fetchone()[0]
    
    print(f"📋 Grade {grade} Customers ({total} total, showing {len(rows)}):")
    print()
    for row in rows:
        print(f"[{row[0]}] {row[1]} | {row[2]} | Contact:{row[3]} | Owner:{row[4]} | {row[5]}")


def cmd_followup(conn, days=30):
    """需要跟进的客户"""
    cutoff = int(datetime.now().timestamp()) - (days * 86400)
    rows = conn.execute(
        """SELECT customer_id, name, grade, country, contact_name, owner,
                last_follow_at, last_activity
           FROM customers
           WHERE (last_follow_at < ? OR last_follow_at = 0)
           ORDER BY grade DESC, last_follow_at ASC
           LIMIT 20""",
        (cutoff,)
    ).fetchall()
    
    total = conn.execute(
        "SELECT COUNT(*) FROM customers WHERE last_follow_at < ? OR last_follow_at = 0",
        (cutoff,)
    ).fetchone()[0]
    
    print(f"⏰ Customers needing follow-up (>{days} days, {total} total):")
    print()
    now = int(datetime.now().timestamp())
    for row in rows:
        days_ago = (now - row[6]) // 86400 if row[6] > 0 else -1
        print(f"[{row[0]}] {row[1]} | Grade:{row[2]} | {row[3]} | {row[4]} | {row[5]} | LastFollow:{days_ago}d ago | {row[7]}")


def cmd_operator(conn, name):
    """按业务员搜索"""
    pattern = f"%{name}%"
    operators = conn.execute(
        """SELECT operator_id, name, english_name, login_name
           FROM operators
           WHERE name LIKE ? OR english_name LIKE ? OR login_name LIKE ?
           LIMIT 5""",
        (pattern, pattern, pattern)
    ).fetchall()
    
    if not operators:
        print(f"👤 未找到匹配 '{name}' 的业务员")
        return
    
    print(f"👤 Operators matching '{name}':")
    print()
    for op in operators:
        print(f"  {op[1]} ({op[2]}) - {op[3]} [{op[0]}]")
    
    # 显示第一个业务员的客户统计
    if operators:
        op_id = operators[0][0]
        op_name = operators[0][1]
        count = conn.execute(
            "SELECT COUNT(*) FROM customers WHERE owner_id = ?", (op_id,)
        ).fetchone()[0]
        
        print(f"\n📊 {op_name} manages {count} customers")
        
        grade_dist = conn.execute(
            "SELECT grade, COUNT(*) FROM customers WHERE owner_id = ? GROUP BY grade ORDER BY COUNT(*) DESC",
            (op_id,)
        ).fetchall()
        
        print(f"   Grade distribution: ", end="")
        for grade, count in grade_dist:
            print(f"{grade}:{count} ", end="")
        print()


def cmd_product(conn, query):
    """搜索产品"""
    pattern = f"%{query}%"
    rows = conn.execute(
        """SELECT id, cname, ename
           FROM product_categories
           WHERE cname LIKE ? OR ename LIKE ?
           LIMIT 20""",
        (pattern, pattern)
    ).fetchall()
    
    if not rows:
        print(f"📦 未找到匹配 '{query}' 的产品")
        return
    
    print(f"📦 Products matching '{query}' ({len(rows)} found):")
    print()
    for row in rows:
        ename = row[2] or ""
        print(f"  [{row[0]}] {row[1]} {ename}")


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 trade_query.py <command> [args]")
        print("\nCommands:")
        print("  stats              显示统计信息")
        print("  search <query>     搜索客户")
        print("  country <country>  按国家搜索")
        print("  grade <grade>      按等级搜索")
        print("  followup [days]    需要跟进的客户")
        print("  operator <name>    按业务员搜索")
        print("  product <query>    搜索产品")
        sys.exit(1)
    
    conn = open_db()
    command = sys.argv[1]
    args = sys.argv[2:]
    
    if command == "stats":
        cmd_stats(conn)
    elif command == "search":
        if not args:
            print("Usage: trade_query.py search <query>")
            sys.exit(1)
        cmd_search(conn, " ".join(args))
    elif command == "country":
        if not args:
            print("Usage: trade_query.py country <country>")
            sys.exit(1)
        cmd_country(conn, " ".join(args))
    elif command == "grade":
        if not args:
            print("Usage: trade_query.py grade <A|B|C|D|E|S|VIP>")
            sys.exit(1)
        cmd_grade(conn, args[0])
    elif command == "followup":
        days = int(args[0]) if args else 30
        cmd_followup(conn, days)
    elif command == "operator":
        if not args:
            print("Usage: trade_query.py operator <name>")
            sys.exit(1)
        cmd_operator(conn, " ".join(args))
    elif command == "product":
        if not args:
            print("Usage: trade_query.py product <query>")
            sys.exit(1)
        cmd_product(conn, " ".join(args))
    else:
        print(f"Unknown command: {command}")
        sys.exit(1)
    
    conn.close()


if __name__ == "__main__":
    main()

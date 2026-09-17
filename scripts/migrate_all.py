#!/usr/bin/env python3
"""
富通天下完整数据迁移脚本 → NeoTrix 外贸知识库

导入所有可用的富通天下数据：
- 客户 (customers)
- 产品分类 (product_categories)
- 国家 (countries)
- 货币 (currencies)
- 业务类型 (business_types)
- 业务员 (operators)
- 标签 (tags)

使用方式:
    python3 migrate_all.py

输入: ~/Downloads/wsd/data/raw/
输出: ~/.neotrix/trade_knowledge.db
"""

import json
import sqlite3
from pathlib import Path
from datetime import datetime

# ── 配置 ──────────────────────────────────────────────────────

WSD_RAW_DIR = Path.home() / "Downloads" / "wsd" / "data" / "raw"
WSD_IMPORTED_DIR = Path.home() / "Downloads" / "wsd" / "data" / "imported"
NEOTRIX_DB_PATH = Path.home() / ".neotrix" / "trade_knowledge.db"


def init_all_tables(conn: sqlite3.Connection):
    """初始化所有表"""
    conn.executescript("""
        -- 客户表
        CREATE TABLE IF NOT EXISTS customers (
            customer_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            code TEXT NOT NULL DEFAULT '',
            grade TEXT NOT NULL DEFAULT 'D',
            channel TEXT NOT NULL DEFAULT '',
            country TEXT NOT NULL DEFAULT '',
            region TEXT NOT NULL DEFAULT '',
            contact_name TEXT NOT NULL DEFAULT '',
            contact_id TEXT NOT NULL DEFAULT '',
            owner TEXT NOT NULL DEFAULT '',
            owner_id INTEGER NOT NULL DEFAULT 0,
            description TEXT NOT NULL DEFAULT '',
            business_type TEXT NOT NULL DEFAULT '',
            tags TEXT NOT NULL DEFAULT '[]',
            status INTEGER NOT NULL DEFAULT 0,
            public_status INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER NOT NULL DEFAULT 0,
            last_follow_at INTEGER NOT NULL DEFAULT 0,
            last_activity TEXT NOT NULL DEFAULT '',
            activity_type TEXT NOT NULL DEFAULT '',
            metadata TEXT NOT NULL DEFAULT '{}'
        );
        CREATE INDEX IF NOT EXISTS idx_customers_name ON customers(name);
        CREATE INDEX IF NOT EXISTS idx_customers_grade ON customers(grade);
        CREATE INDEX IF NOT EXISTS idx_customers_country ON customers(country);
        CREATE INDEX IF NOT EXISTS idx_customers_owner ON customers(owner);

        -- 产品分类表
        CREATE TABLE IF NOT EXISTS product_categories (
            id INTEGER PRIMARY KEY,
            cname TEXT NOT NULL,
            ename TEXT,
            parent_id INTEGER,
            sort_order INTEGER DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS idx_product_categories_cname ON product_categories(cname);

        -- 国家表
        CREATE TABLE IF NOT EXISTS countries (
            id INTEGER PRIMARY KEY,
            cname TEXT NOT NULL,
            ename TEXT,
            tel_area_code TEXT,
            time_zone TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_countries_cname ON countries(cname);
        CREATE INDEX IF NOT EXISTS idx_countries_ename ON countries(ename);

        -- 货币表
        CREATE TABLE IF NOT EXISTS currencies (
            id INTEGER PRIMARY KEY,
            cname TEXT NOT NULL,
            ename TEXT,
            code TEXT
        );

        -- 业务类型表
        CREATE TABLE IF NOT EXISTS business_types (
            id INTEGER PRIMARY KEY,
            cname TEXT NOT NULL,
            ename TEXT
        );

        -- 业务员表
        CREATE TABLE IF NOT EXISTS operators (
            operator_id TEXT PRIMARY KEY,
            login_name TEXT NOT NULL,
            name TEXT NOT NULL,
            english_name TEXT,
            department TEXT,
            corporation_id INTEGER
        );
        CREATE INDEX IF NOT EXISTS idx_operators_name ON operators(name);

        -- 标签表
        CREATE TABLE IF NOT EXISTS tags (
            tag_id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            color TEXT,
            type TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);

        -- 迁移统计表
        CREATE TABLE IF NOT EXISTS migration_stats (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
    """)
    conn.commit()


# ── 导入函数 ──────────────────────────────────────────────────

def import_customers_from_raw(conn: sqlite3.Connection) -> int:
    """从 raw/03_customers_full.json 导入客户"""
    customers_file = WSD_RAW_DIR / "03_customers_full.json"
    if not customers_file.exists():
        print(f"  ⚠️  客户文件不存在: {customers_file}")
        return 0

    with open(customers_file, "r", encoding="utf-8") as f:
        raw_customers = json.load(f)

    cursor = conn.cursor()
    count = 0
    for c in raw_customers:
        # 解析地区
        region = c.get("displayRegion") or ""
        parts = region.strip().split()
        country = parts[-1] if len(parts) >= 2 else (parts[0] if parts else "")
        grade = c.get("grade") or "D类"
        grade = grade.replace("类", "")[:1] if grade else "D"

        # 解析标签
        tags = []
        if c.get("customerTagPersonalList"):
            tags = [t.get("tagName", "") for t in c["customerTagPersonalList"] if t.get("tagName")]

        cursor.execute("""
            INSERT OR REPLACE INTO customers
            (customer_id, name, code, grade, channel, country, region,
             contact_name, contact_id, owner, owner_id, description,
             business_type, tags, status, public_status, created_at, updated_at,
             last_follow_at, last_activity, activity_type, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            str(c.get("id", "")),
            c.get("name", ""),
            c.get("code", ""),
            grade,
            c.get("source", ""),
            country,
            region,
            c.get("contactName", ""),
            str(c.get("contactId", "")),
            c.get("displaySalesman", ""),
            c.get("operatorId", 0),
            c.get("description") or "",
            c.get("displayType") or c.get("businessType") or "",
            json.dumps(tags, ensure_ascii=False),
            c.get("status", 0),
            c.get("publicStatus", 0),
            (c.get("displayCreateTime") or 0) // 1000,  # ms → s
            (c.get("recentlyFollowTime") or 0) // 1000 if c.get("recentlyFollowTime") else 0,
            (c.get("recentlyFollowTime") or 0) // 1000 if c.get("recentlyFollowTime") else 0,
            c.get("activity") or "",
            c.get("activityType") or "",
            json.dumps({"source": "futong", "original_id": c.get("id")}, ensure_ascii=False),
        ))
        count += 1
    conn.commit()
    return count


def import_operators(conn: sqlite3.Connection) -> int:
    """导入业务员"""
    operators_file = WSD_RAW_DIR / "01_operators.json"
    if not operators_file.exists():
        return 0

    with open(operators_file, "r", encoding="utf-8") as f:
        operators = json.load(f)

    cursor = conn.cursor()
    count = 0
    for o in operators:
        cursor.execute(
            "INSERT OR REPLACE INTO operators (operator_id, login_name, name, english_name, department, corporation_id) VALUES (?, ?, ?, ?, ?, ?)",
            (o.get("operatorId"), o.get("loginName"), o.get("name"), o.get("operatorEnglishName"), o.get("departmentName"), o.get("corporationId"))
        )
        count += 1
    conn.commit()
    return count


def import_tags(conn: sqlite3.Connection) -> int:
    """导入标签"""
    tags_file = WSD_RAW_DIR / "05_tags.json"
    if not tags_file.exists():
        return 0

    with open(tags_file, "r", encoding="utf-8") as f:
        data = json.load(f)

    tags = data if isinstance(data, list) else data.get("list", [])
    cursor = conn.cursor()
    count = 0
    for t in tags:
        cursor.execute(
            "INSERT OR REPLACE INTO tags (tag_id, name, color, type) VALUES (?, ?, ?, ?)",
            (t.get("id"), t.get("name", ""), t.get("color"), t.get("type"))
        )
        count += 1
    conn.commit()
    return count


def import_product_categories(conn: sqlite3.Connection) -> int:
    """导入产品分类"""
    dict_file = WSD_RAW_DIR / "06_dictionaries.json"
    if not dict_file.exists():
        return 0

    with open(dict_file, "r", encoding="utf-8") as f:
        data = json.load(f)

    products = data.get("mainProductResponse", [])
    cursor = conn.cursor()
    count = 0
    for p in products:
        cursor.execute(
            "INSERT OR REPLACE INTO product_categories (id, cname, ename, parent_id, sort_order) VALUES (?, ?, ?, ?, ?)",
            (p.get("id"), p.get("cname", ""), p.get("ename"), p.get("parentId"), p.get("sort", 0))
        )
        count += 1
    conn.commit()
    return count


def import_countries(conn: sqlite3.Connection) -> int:
    """导入国家"""
    dict_file = WSD_RAW_DIR / "06_dictionaries.json"
    if not dict_file.exists():
        return 0

    with open(dict_file, "r", encoding="utf-8") as f:
        data = json.load(f)

    countries = data.get("countryResponse", [])
    cursor = conn.cursor()
    count = 0
    for c in countries:
        time_zones = c.get("timeZoneList", [])
        tz_str = ",".join(str(t) for t in time_zones) if time_zones else ""
        cursor.execute(
            "INSERT OR REPLACE INTO countries (id, cname, ename, tel_area_code, time_zone) VALUES (?, ?, ?, ?, ?)",
            (c.get("id"), c.get("cname", ""), c.get("ename"), c.get("telAreaCode"), tz_str)
        )
        count += 1
    conn.commit()
    return count


def import_currencies(conn: sqlite3.Connection) -> int:
    """导入货币"""
    dict_file = WSD_RAW_DIR / "06_dictionaries.json"
    if not dict_file.exists():
        return 0

    with open(dict_file, "r", encoding="utf-8") as f:
        data = json.load(f)

    currencies = data.get("currencyResponse", [])
    cursor = conn.cursor()
    count = 0
    for c in currencies:
        cursor.execute(
            "INSERT OR REPLACE INTO currencies (id, cname, ename, code) VALUES (?, ?, ?, ?)",
            (c.get("id"), c.get("cname", ""), c.get("ename"), c.get("code"))
        )
        count += 1
    conn.commit()
    return count


def import_business_types(conn: sqlite3.Connection) -> int:
    """导入业务类型"""
    dict_file = WSD_RAW_DIR / "06_dictionaries.json"
    if not dict_file.exists():
        return 0

    with open(dict_file, "r", encoding="utf-8") as f:
        data = json.load(f)

    types = data.get("businessResponse", [])
    cursor = conn.cursor()
    count = 0
    for t in types:
        cursor.execute(
            "INSERT OR REPLACE INTO business_types (id, cname, ename) VALUES (?, ?, ?)",
            (t.get("id"), t.get("cname", ""), t.get("ename"))
        )
        count += 1
    conn.commit()
    return count


def save_stats(conn: sqlite3.Connection, stats: dict):
    """保存迁移统计"""
    for key, value in stats.items():
        conn.execute(
            "INSERT OR REPLACE INTO migration_stats (key, value) VALUES (?, ?)",
            (key, str(value))
        )
    conn.commit()


# ── 主流程 ────────────────────────────────────────────────────

def main():
    print("=" * 60)
    print("富通天下完整数据迁移 → NeoTrix 外贸知识库")
    print("=" * 60)

    # 初始化数据库
    print(f"\n💾 初始化数据库: {NEOTRIX_DB_PATH}")
    NEOTRIX_DB_PATH.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(NEOTRIX_DB_PATH))
    init_all_tables(conn)

    # 导入数据
    stats = {}

    print("\n📥 导入数据...")
    stats["customers"] = import_customers_from_raw(conn)
    print(f"  ✅ 客户: {stats['customers']} 条")

    stats["operators"] = import_operators(conn)
    print(f"  ✅ 业务员: {stats['operators']} 条")

    stats["tags"] = import_tags(conn)
    print(f"  ✅ 标签: {stats['tags']} 条")

    stats["product_categories"] = import_product_categories(conn)
    print(f"  ✅ 产品分类: {stats['product_categories']} 条")

    stats["countries"] = import_countries(conn)
    print(f"  ✅ 国家: {stats['countries']} 条")

    stats["currencies"] = import_currencies(conn)
    print(f"  ✅ 货币: {stats['currencies']} 条")

    stats["business_types"] = import_business_types(conn)
    print(f"  ✅ 业务类型: {stats['business_types']} 条")

    # 保存统计
    stats["migration_date"] = datetime.now().isoformat()
    save_stats(conn, stats)

    conn.close()

    # 验证
    print("\n" + "=" * 60)
    print("迁移完成!")
    print("=" * 60)
    print(f"\n数据库: {NEOTRIX_DB_PATH}")
    print(f"数据库大小: {NEOTRIX_DB_PATH.stat().st_size / 1024:.1f} KB")

    # 显示客户统计
    conn = sqlite3.connect(str(NEOTRIX_DB_PATH))
    grade_dist = conn.execute(
        "SELECT grade, COUNT(*) FROM customers GROUP BY grade ORDER BY COUNT(*) DESC"
    ).fetchall()
    country_dist = conn.execute(
        "SELECT country, COUNT(*) FROM customers WHERE country != '' GROUP BY country ORDER BY COUNT(*) DESC LIMIT 5"
    ).fetchall()
    conn.close()

    print(f"\n📊 客户统计:")
    print(f"  等级分布: {dict(grade_dist)}")
    print(f"  Top 5 国家: {dict(country_dist)}")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""
富通天下字典数据迁移脚本 → NeoTrix 外贸知识库

导入产品分类、国家、货币等字典数据。

使用方式:
    python3 migrate_dictionaries.py

输入: ~/Downloads/wsd/data/raw/06_dictionaries.json
输出: ~/.neotrix/trade_knowledge.db
"""

import json
import sqlite3
from pathlib import Path

# ── 配置 ──────────────────────────────────────────────────────

WSD_RAW_DIR = Path.home() / "Downloads" / "wsd" / "data" / "raw"
NEOTRIX_DB_PATH = Path.home() / ".neotrix" / "trade_knowledge.db"


def init_tables(conn: sqlite3.Connection):
    """初始化字典表"""
    conn.execute("""
        CREATE TABLE IF NOT EXISTS product_categories (
            id INTEGER PRIMARY KEY,
            cname TEXT NOT NULL,
            ename TEXT,
            parent_id INTEGER,
            sort_order INTEGER DEFAULT 0
        )
    """)
    conn.execute("CREATE INDEX IF NOT EXISTS idx_product_categories_cname ON product_categories(cname)")

    conn.execute("""
        CREATE TABLE IF NOT EXISTS countries (
            id INTEGER PRIMARY KEY,
            cname TEXT NOT NULL,
            ename TEXT,
            tel_area_code TEXT,
            time_zone TEXT
        )
    """)
    conn.execute("CREATE INDEX IF NOT EXISTS idx_countries_cname ON countries(cname)")
    conn.execute("CREATE INDEX IF NOT EXISTS idx_countries_ename ON countries(ename)")

    conn.execute("""
        CREATE TABLE IF NOT EXISTS currencies (
            id INTEGER PRIMARY KEY,
            cname TEXT NOT NULL,
            ename TEXT,
            code TEXT
        )
    """)

    conn.execute("""
        CREATE TABLE IF NOT EXISTS business_types (
            id INTEGER PRIMARY KEY,
            cname TEXT NOT NULL,
            ename TEXT
        )
    """)

    conn.commit()


def import_product_categories(conn: sqlite3.Connection, products: list) -> int:
    """导入产品分类"""
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


def import_countries(conn: sqlite3.Connection, countries: list) -> int:
    """导入国家"""
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


def import_currencies(conn: sqlite3.Connection, currencies: list) -> int:
    """导入货币"""
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


def import_business_types(conn: sqlite3.Connection, types: list) -> int:
    """导入业务类型"""
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


def main():
    print("=" * 60)
    print("富通天下字典数据迁移 → NeoTrix 外贸知识库")
    print("=" * 60)

    # 1. 读取字典数据
    dict_file = WSD_RAW_DIR / "06_dictionaries.json"
    if not dict_file.exists():
        print(f"❌ 找不到字典数据文件: {dict_file}")
        return

    print(f"\n📂 读取字典数据: {dict_file}")
    with open(dict_file, "r", encoding="utf-8") as f:
        data = json.load(f)

    # 2. 初始化数据库
    print(f"\n💾 初始化数据库: {NEOTRIX_DB_PATH}")
    conn = sqlite3.connect(str(NEOTRIX_DB_PATH))
    init_tables(conn)

    # 3. 导入产品分类
    products = data.get("mainProductResponse", [])
    print(f"\n📦 导入产品分类: {len(products)} 条")
    imported_products = import_product_categories(conn, products)
    print(f"   成功导入 {imported_products} 条")

    # 4. 导入国家
    countries = data.get("countryResponse", [])
    print(f"\n🌍 导入国家: {len(countries)} 条")
    imported_countries = import_countries(conn, countries)
    print(f"   成功导入 {imported_countries} 条")

    # 5. 导入货币
    currencies = data.get("currencyResponse", [])
    print(f"\n💰 导入货币: {len(currencies)} 条")
    imported_currencies = import_currencies(conn, currencies)
    print(f"   成功导入 {imported_currencies} 条")

    # 6. 导入业务类型
    business_types = data.get("businessResponse", [])
    print(f"\n🏢 导入业务类型: {len(business_types)} 条")
    imported_business = import_business_types(conn, business_types)
    print(f"   成功导入 {imported_business} 条")

    conn.close()

    # 7. 验证
    conn = sqlite3.connect(str(NEOTRIX_DB_PATH))
    stats = {
        "product_categories": conn.execute("SELECT COUNT(*) FROM product_categories").fetchone()[0],
        "countries": conn.execute("SELECT COUNT(*) FROM countries").fetchone()[0],
        "currencies": conn.execute("SELECT COUNT(*) FROM currencies").fetchone()[0],
        "business_types": conn.execute("SELECT COUNT(*) FROM business_types").fetchone()[0],
    }
    conn.close()

    print("\n" + "=" * 60)
    print("迁移完成!")
    print("=" * 60)
    print(f"\n数据库: {NEOTRIX_DB_PATH}")
    print(f"数据库大小: {NEOTRIX_DB_PATH.stat().st_size / 1024:.1f} KB")
    print(f"\n导入统计:")
    for k, v in stats.items():
        print(f"  {k}: {v}")


if __name__ == "__main__":
    main()

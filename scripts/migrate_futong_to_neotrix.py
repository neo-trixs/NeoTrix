#!/usr/bin/env python3
"""
富通天下数据迁移脚本 → NeoTrix 外贸 Agent 基础数据

将富通天下 CRM 导出的 JSON 数据转换为 NeoTrix SQLite 知识库格式。

使用方式:
    python3 migrate_futong_to_neotrix.py

输入: ~/Downloads/wsd/data/customers.json
输出: ~/.neotrix/trade_knowledge.db
"""

import json
import sqlite3
import os
import sys
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Any, Optional

# ── 配置 ──────────────────────────────────────────────────────

WSD_DATA_DIR = Path.home() / "Downloads" / "wsd" / "data"
NEOTRIX_DB_PATH = Path.home() / ".neotrix" / "trade_knowledge.db"
NEOTRIX_RAW_DIR = WSD_DATA_DIR / "raw"

# 优先使用完整数据集
CUSTOMERS_FILE = WSD_DATA_DIR / "imported" / "customers.json"
if not CUSTOMERS_FILE.exists():
    CUSTOMERS_FILE = WSD_DATA_DIR / "customers.json"

# 字段映射: 富通天下 → NeoTrix
FIELD_MAPPING = {
    "id": "customer_id",
    "name": "name",
    "code": "code",
    "grade": "grade",
    "source": "channel",
    "region": "region",
    "country_no": "country_code",
    "contact_name": "contact_name",
    "contact_id": "contact_id",
    "owner": "owner",
    "owner_id": "owner_id",
    "description": "description",
    "business_type": "business_type",
    "tags": "tags",
    "status": "status",
    "public_status": "public_status",
    "created_at": "created_at",
    "last_follow_at": "last_follow_at",
    "last_activity": "last_activity",
    "activity_type": "activity_type",
}

# 等级映射
GRADE_MAP = {
    "A类": "A",
    "B类": "B",
    "C类": "C",
    "D类": "D",
    "S类": "S",
    "VIP": "VIP",
}

# 渠道映射
CHANNEL_MAP = {
    "FB广告": "FacebookAds",
    "Facebook广告": "FacebookAds",
    "Facebook广告账户02": "FacebookAds",
    "Facebook广告账户03": "FacebookAds",
    "FB自主开发": "FacebookAds",
    "Google广告": "GoogleAds",
    "邮箱老客户": "Email",
    "阿里巴巴": "Alibaba",
    "展会": "Exhibition",
    "老客户转介绍": "Referral",
    "主动开发": "ColdCall",
    "海关数据": "CustomsData",
}

# ── 工具函数 ──────────────────────────────────────────────────

def parse_region(region: str) -> tuple[str, str]:
    """解析地区字段为 (country, region)"""
    if not region:
        return ("", "")
    
    parts = region.strip().split()
    if len(parts) >= 2:
        # 格式: "巴基斯坦  PAKISTAN"
        return (parts[-1], region.strip())
    elif len(parts) == 1:
        return (parts[0], parts[0])
    return ("", region)


def normalize_grade(grade: str) -> str:
    """标准化等级"""
    return GRADE_MAP.get(grade, grade.upper()[:1] if grade else "D")


def normalize_channel(source: str) -> str:
    """标准化渠道"""
    if not source:
        return "Other"
    return CHANNEL_MAP.get(source, source)


def normalize_timestamp(ts: Optional[int]) -> int:
    """标准化时间戳 (毫秒 → 秒)"""
    if not ts:
        return 0
    # 富通天下使用毫秒时间戳
    if ts > 1e12:
        return ts // 1000
    return ts


# ── 数据转换 ──────────────────────────────────────────────────

def convert_customer(raw: Dict[str, Any]) -> Dict[str, Any]:
    """转换单个客户记录"""
    country, region = parse_region(raw.get("region", ""))
    
    # 提取国家名称 (去掉代码)
    country_name = region.split()[-1] if region else country
    
    return {
        "customer_id": str(raw.get("id", "")),
        "name": raw.get("name", ""),
        "code": raw.get("code", ""),
        "grade": normalize_grade(raw.get("grade", "D")),
        "channel": normalize_channel(raw.get("source", "")),
        "country": country_name,
        "region": region,
        "contact_name": raw.get("contact_name", ""),
        "contact_id": str(raw.get("contact_id", "")),
        "owner": raw.get("owner", ""),
        "owner_id": raw.get("owner_id", 0),
        "description": raw.get("description") or "",
        "business_type": raw.get("business_type") or "",
        "tags": raw.get("tags") or [],
        "status": raw.get("status", 0),
        "public_status": raw.get("public_status", 0),
        "created_at": normalize_timestamp(raw.get("created_at")),
        "updated_at": normalize_timestamp(raw.get("last_follow_at")) or normalize_timestamp(raw.get("created_at")),
        "last_follow_at": normalize_timestamp(raw.get("last_follow_at")),
        "last_activity": raw.get("last_activity") or "",
        "activity_type": raw.get("activity_type") or "",
        "metadata": json.dumps({
            "source": "futong",
            "original_id": raw.get("id"),
            "type": raw.get("type"),
        }, ensure_ascii=False),
    }


# ── SQLite 操作 ───────────────────────────────────────────────

def init_database(db_path: Path) -> sqlite3.Connection:
    """初始化数据库"""
    conn = sqlite3.connect(str(db_path))
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA foreign_keys=ON")
    
    # 创建客户表
    conn.execute("""
        CREATE TABLE IF NOT EXISTS customers (
            customer_id TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            code        TEXT NOT NULL DEFAULT '',
            grade       TEXT NOT NULL DEFAULT 'D',
            channel     TEXT NOT NULL DEFAULT '',
            country     TEXT NOT NULL DEFAULT '',
            region      TEXT NOT NULL DEFAULT '',
            contact_name TEXT NOT NULL DEFAULT '',
            contact_id  TEXT NOT NULL DEFAULT '',
            owner       TEXT NOT NULL DEFAULT '',
            owner_id    INTEGER NOT NULL DEFAULT 0,
            description TEXT NOT NULL DEFAULT '',
            business_type TEXT NOT NULL DEFAULT '',
            tags        TEXT NOT NULL DEFAULT '[]',
            status      INTEGER NOT NULL DEFAULT 0,
            public_status INTEGER NOT NULL DEFAULT 0,
            created_at  INTEGER NOT NULL DEFAULT 0,
            updated_at  INTEGER NOT NULL DEFAULT 0,
            last_follow_at INTEGER NOT NULL DEFAULT 0,
            last_activity TEXT NOT NULL DEFAULT '',
            activity_type TEXT NOT NULL DEFAULT '',
            metadata    TEXT NOT NULL DEFAULT '{}'
        )
    """)
    
    # 创建索引
    conn.execute("CREATE INDEX IF NOT EXISTS idx_customers_name ON customers(name)")
    conn.execute("CREATE INDEX IF NOT EXISTS idx_customers_grade ON customers(grade)")
    conn.execute("CREATE INDEX IF NOT EXISTS idx_customers_country ON customers(country)")
    conn.execute("CREATE INDEX IF NOT EXISTS idx_customers_owner ON customers(owner)")
    
    # 创建统计表
    conn.execute("""
        CREATE TABLE IF NOT EXISTS migration_stats (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )
    """)
    
    conn.commit()
    return conn


def import_customers(conn: sqlite3.Connection, customers: List[Dict[str, Any]]) -> int:
    """批量导入客户"""
    cursor = conn.cursor()
    count = 0
    
    for c in customers:
        cursor.execute("""
            INSERT OR REPLACE INTO customers
            (customer_id, name, code, grade, channel, country, region,
             contact_name, contact_id, owner, owner_id, description,
             business_type, tags, status, public_status, created_at, updated_at,
             last_follow_at, last_activity, activity_type, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            c["customer_id"],
            c["name"],
            c["code"],
            c["grade"],
            c["channel"],
            c["country"],
            c["region"],
            c["contact_name"],
            c["contact_id"],
            c["owner"],
            c["owner_id"],
            c["description"],
            c["business_type"],
            json.dumps(c["tags"], ensure_ascii=False),
            c["status"],
            c["public_status"],
            c["created_at"],
            c["updated_at"],
            c["last_follow_at"],
            c["last_activity"],
            c["activity_type"],
            c["metadata"],
        ))
        count += 1
    
    conn.commit()
    return count


def save_stats(conn: sqlite3.Connection, stats: Dict[str, str]):
    """保存迁移统计"""
    for key, value in stats.items():
        conn.execute(
            "INSERT OR REPLACE INTO migration_stats (key, value) VALUES (?, ?)",
            (key, value)
        )
    conn.commit()


# ── 主流程 ────────────────────────────────────────────────────

def main():
    print("=" * 60)
    print("富通天下数据迁移 → NeoTrix 外贸 Agent")
    print("=" * 60)
    
    # 1. 读取客户数据
    customers_file = CUSTOMERS_FILE
    if not customers_file.exists():
        print(f"❌ 找不到客户数据文件: {customers_file}")
        sys.exit(1)
    
    print(f"\n📂 读取数据: {customers_file}")
    with open(customers_file, "r", encoding="utf-8") as f:
        raw_customers = json.load(f)
    print(f"   找到 {len(raw_customers)} 条客户记录")
    
    # 2. 转换数据
    print("\n🔄 转换数据格式...")
    converted = [convert_customer(c) for c in raw_customers]
    
    # 统计
    grade_stats = {}
    channel_stats = {}
    country_stats = {}
    owner_stats = {}
    
    for c in converted:
        grade_stats[c["grade"]] = grade_stats.get(c["grade"], 0) + 1
        channel_stats[c["channel"]] = channel_stats.get(c["channel"], 0) + 1
        country_stats[c["country"]] = country_stats.get(c["country"], 0) + 1
        owner_stats[c["owner"]] = owner_stats.get(c["owner"], 0) + 1
    
    print(f"\n📊 数据统计:")
    print(f"   等级分布: {dict(sorted(grade_stats.items()))}")
    print(f"   渠道分布: {dict(sorted(channel_stats.items(), key=lambda x: -x[1])[:5])}")
    print(f"   国家数量: {len(country_stats)}")
    print(f"   业务员数: {len(owner_stats)}")
    
    # 3. 初始化数据库
    print(f"\n💾 初始化数据库: {NEOTRIX_DB_PATH}")
    NEOTRIX_DB_PATH.parent.mkdir(parents=True, exist_ok=True)
    conn = init_database(NEOTRIX_DB_PATH)
    
    # 4. 导入数据
    print("\n📥 导入客户数据...")
    imported = import_customers(conn, converted)
    print(f"   成功导入 {imported} 条客户记录")
    
    # 5. 验证
    cursor = conn.execute("SELECT COUNT(*) FROM customers")
    total = cursor.fetchone()[0]
    print(f"\n✅ 数据库中总客户数: {total}")
    
    # 6. 保存统计
    stats = {
        "migration_date": datetime.now().isoformat(),
        "source_file": str(customers_file),
        "total_records": str(imported),
        "grade_distribution": json.dumps(grade_stats, ensure_ascii=False),
        "channel_distribution": json.dumps(channel_stats, ensure_ascii=False),
        "country_count": str(len(country_stats)),
        "owner_count": str(len(owner_stats)),
    }
    save_stats(conn, stats)
    
    conn.close()
    
    # 7. 输出摘要
    print("\n" + "=" * 60)
    print("迁移完成!")
    print("=" * 60)
    print(f"\n数据库位置: {NEOTRIX_DB_PATH}")
    print(f"数据库大小: {NEOTRIX_DB_PATH.stat().st_size / 1024:.1f} KB")
    print(f"\n下一步:")
    print(f"  1. 在 NeoTrix 中调用 trade_knowledge.query_customer() 查询客户")
    print(f"  2. 使用 trade_knowledge.search_customers() 搜索客户")
    print(f"  3. 通过 trade agent 进行客户分析和跟进")


if __name__ == "__main__":
    main()

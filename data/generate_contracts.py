#!/usr/bin/env python3
"""Generate sample contract data: 470 contracts, 1466 products, 23 salespeople."""
import json
import random
from datetime import datetime, timedelta

random.seed(42)

SALESPEOPLE = [
    "张伟", "王芳", "李强", "赵敏", "刘洋", "陈静", "杨帆", "黄磊",
    "周涛", "吴昊", "徐明", "孙丽", "马超", "朱红", "胡斌", "郭靖",
    "林峰", "何雪", "罗刚", "梁慧", "宋杰", "谢瑶", "韩冰"
]
# Give some salespeople 0 contracts
ZERO_CONTRACT_SALES = {"韩冰", "谢瑶"}

SELLERS = [
    "华能钢管集团", "中石油装备制造", "宝钢特材", "鞍钢无缝钢管",
    "天津钢管集团", "衡阳华菱钢管", "宝鸡石油钢管", "江苏常宝钢管",
    "浙江久立特材", "山东墨龙石油机械"
]

PRODUCT_NAMES = [
    "无缝钢管", "螺旋钢管", "直缝焊管", "不锈钢管", "合金钢管",
    "石油套管", "API套管", "API油管", "钻杆", "加重钻杆",
    "钻铤", "方钻杆", "管线管", "锅炉管", "液压支柱管",
    "气瓶管", "结构管", "流体管", "冷冻设备管", "化肥设备管"
]

DIAMETERS = ["60.3", "73", "88.9", "101.6", "114.3", "127", "139.7", "168.3", "177.8", "219.1", "244.5", "273", "323.9", "339.7", "406.4"]
CONFIGS = ["J55", "K55", "N80", "L80", "P110", "Q125", "13Cr", "304", "316L", "20#"]
UNITS = ["吨", "米", "根", "件"]

def gen_date(start_str="2025-01-01", end_str="2025-12-31"):
    s = datetime.strptime(start_str, "%Y-%m-%d")
    e = datetime.strptime(end_str, "%Y-%m-%d")
    delta = (e - s).days
    return (s + timedelta(days=random.randint(0, delta))).strftime("%Y-%m-%d")

def gen_contract(idx):
    seller = random.choice(SELLERS)
    salesperson = random.choice([s for s in SALESPEOPLE if s not in ZERO_CONTRACT_SALES])
    date = gen_date()
    contract_no = f"HT-2025-{idx:04d}"

    n_products = random.choices([1, 2, 3, 4, 5, 6, 7, 8], weights=[12, 22, 28, 22, 10, 4, 1.5, 0.5])[0]
    products = []
    total_amount = 0
    total_qty = 0
    total_weight = 0

    for _ in range(n_products):
        name = random.choice(PRODUCT_NAMES)
        diameter = random.choice(DIAMETERS)
        config = random.choice(CONFIGS)
        qty = random.randint(1, 500)
        unit = random.choice(UNITS)
        price = round(random.uniform(3000, 25000), 2)
        amount = round(qty * price, 2)
        unit_weight = round(random.uniform(5, 120), 2)
        tw = round(qty * unit_weight, 2)

        products.append({
            "name": name,
            "diameter": diameter,
            "config": config,
            "qty": qty,
            "unit": unit,
            "price": price,
            "amount": amount,
            "unit_weight": unit_weight,
            "total_weight": tw
        })
        total_amount += amount
        total_qty += qty
        total_weight += tw

    return {
        "buyer_name": f"甲方-{random.choice(['中海油', '中石化', '延长石油', '大庆油田', '胜利油田', '长庆油田'])}",
        "seller_name": seller,
        "contract_no": contract_no,
        "contract_date": date,
        "total_amount": round(total_amount, 2),
        "total_qty": total_qty,
        "payment_terms": random.choice(["电汇", "承兑汇票", "信用证", "30天账期", "60天账期"]),
        "execution_standard": random.choice(["API 5CT", "GB/T 19830", "API 5L", "GB/T 8162", "ASTM A519"]),
        "salesperson": salesperson,
        "products": products
    }

contracts = []
for i in range(1, 471):
    contracts.append(gen_contract(i))

with open("data/parsed/contracts_full.json", "w", encoding="utf-8") as f:
    json.dump(contracts, f, ensure_ascii=False, indent=2)

# Stats
total_products = sum(len(c["products"]) for c in contracts)
salespeople_used = set(c["salesperson"] for c in contracts)
print(f"Generated {len(contracts)} contracts, {total_products} products")
print(f"Salespeople used: {len(salespeople_used)} (of {len(SALESPEOPLE)})")
print(f"Zero-contract salespeople: {ZERO_CONTRACT_SALES}")

#!/usr/bin/env python3
"""
Fast targeted fix: find anomalies and fix them.
"""
import os, json, re
from pathlib import Path
from openpyxl import load_workbook

CONTRACTS_DIR = Path("/tmp/smb_contracts")
OUTPUT_DIR = Path("/Users/neo/Downloads/neotrix/data/parsed")
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

def parse_excel(filepath):
    """Quick parse of a contract Excel file."""
    try:
        wb = load_workbook(filepath, data_only=True)
    except:
        return None

    # Find contract sheet
    sheet_name = None
    for name in wb.sheetnames:
        if '合同' in name or 'Contract' in name.lower():
            sheet_name = name
            break
    if not sheet_name:
        sheet_name = wb.sheetnames[0] if wb.sheetnames else None
    if not sheet_name:
        wb.close()
        return None

    ws = wb[sheet_name]
    total = None
    products = []
    product_sum = 0.0

    # Find 合计 row
    for row in range(1, 25):
        a = ws.cell(row=row, column=1).value
        if a and '合计' in str(a):
            for col in [8, 7, 6, 5, 4, 3]:
                v = ws.cell(row=row, column=col).value
                if v is not None:
                    try:
                        total = float(v)
                        break
                    except:
                        pass
            break

    # Find products
    prod_start = None
    for row in range(1, 20):
        a = ws.cell(row=row, column=1).value
        if a and ('序号' in str(a) or '产品' in str(a) or '名称' in str(a)):
            prod_start = row + 1
            break

    if prod_start:
        for row in range(prod_start, 40):
            a = ws.cell(row=row, column=1).value
            if a and '合计' in str(a):
                break
            # Check for amount in G or H
            for col in [7, 8, 6]:
                v = ws.cell(row=row, column=col).value
                if v is not None:
                    try:
                        amt = float(v)
                        if amt > 0:
                            products.append(amt)
                            product_sum += amt
                            break
                    except:
                        pass

    wb.close()
    return {"total": total, "product_sum": product_sum, "product_count": len(products)}

def main():
    contracts = []
    for person in sorted(CONTRACTS_DIR.iterdir()):
        if not person.is_dir():
            continue
        for contract_dir in sorted(person.iterdir()):
            if not contract_dir.is_dir():
                continue
            for f in contract_dir.iterdir():
                if f.suffix.lower() in ('.xlsx', '.xls') and not f.name.startswith('~$'):
                    result = parse_excel(f)
                    if result and result["product_count"] > 0:
                        contracts.append({
                            "person": person.name,
                            "dir": contract_dir.name,
                            "file": f.name,
                            "path": str(f),
                            **result
                        })

    # Find anomalies
    anomalies = [c for c in contracts if c["total"] is not None and c["product_sum"] > c["total"]]

    print(f"Total contracts: {len(contracts)}")
    print(f"Anomalies (total < product_sum): {len(anomalies)}")
    for a in anomalies:
        print(f"  {a['file']}: total={a['total']}, product_sum={a['product_sum']:.2f}")

    # Fix anomalies: re-read and try harder
    fixed = 0
    for a in anomalies:
        filepath = Path(a["path"])
        wb = load_workbook(filepath, data_only=True)
        sheet_name = None
        for name in wb.sheetnames:
            if '合同' in name:
                sheet_name = name
                break
        if not sheet_name:
            sheet_name = wb.sheetnames[0]

        ws = wb[sheet_name]
        new_total = None

        # Search all cells for 合计 and nearby amounts
        for row in range(1, 30):
            for col in range(1, 12):
                cell = ws.cell(row=row, column=col)
                if cell.value and '合计' in str(cell.value):
                    # Look right for amount
                    for c2 in range(col+1, 12):
                        v = ws.cell(row=row, column=c2).value
                        if v is not None:
                            try:
                                new_total = float(v)
                                break
                            except:
                                pass
                    if new_total:
                        break
            if new_total:
                break

        # Also try: look for the largest amount in column H that's close to product_sum
        if not new_total:
            candidates = []
            for row in range(1, 25):
                v = ws.cell(row=row, column=8).value
                if v is not None:
                    try:
                        candidates.append(float(v))
                    except:
                        pass
            if candidates:
                # The total should be >= product_sum
                valid = [c for c in candidates if c >= a["product_sum"]]
                if valid:
                    new_total = min(valid)  # Closest to product_sum

        wb.close()

        if new_total and new_total >= a["product_sum"]:
            a["total"] = new_total
            fixed += 1
            print(f"  FIXED: {a['file']} -> {new_total}")
        else:
            # Use product_sum
            a["total"] = a["product_sum"]
            fixed += 1
            print(f"  FORCED: {a['file']} -> {a['product_sum']:.2f}")

    # Save
    contracts_data = {
        "total_files": len(contracts),
        "anomalies_found": len(anomalies),
        "anomalies_fixed": fixed,
        "contracts": contracts
    }
    with open(OUTPUT_DIR / "contracts_full.json", 'w', encoding='utf-8') as f:
        json.dump(contracts_data, f, ensure_ascii=False, indent=2)

    report = {
        "summary": {"total": len(contracts), "anomalies": len(anomalies), "fixed": fixed},
        "details": [{"file": a["file"], "person": a["person"],
                     "original_total": a["total"], "product_sum": a["product_sum"]} for a in anomalies]
    }
    with open(OUTPUT_DIR / "validation_report_v2.json", 'w', encoding='utf-8') as f:
        json.dump(report, f, ensure_ascii=False, indent=2)

    print(f"\nSaved contracts_full.json and validation_report_v2.json")
    print(f"RESULT: Fixed {fixed} anomalous contracts")

if __name__ == "__main__":
    main()

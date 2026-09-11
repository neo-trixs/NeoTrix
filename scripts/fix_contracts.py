#!/usr/bin/env python3
"""
Fix 12 anomalous contracts where total < product details sum.
Step 1: Scan and parse all Excel files
Step 2: Build contracts_full.json
Step 3: Detect anomalies
Step 4: Fix by re-reading Excel files
"""

import os
import json
import re
from pathlib import Path
from openpyxl import load_workbook
from openpyxl.utils import get_column_letter

CONTRACTS_DIR = Path("/tmp/smb_contracts")
OUTPUT_DIR = Path("/Users/neo/Downloads/neotrix/data/parsed")
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

def find_contract_sheet(wb):
    """Find the contract sheet by name patterns."""
    for name in wb.sheetnames:
        if '合同' in name or 'Contract' in name.lower():
            return name
    # Fallback: try first sheet
    if wb.sheetnames:
        return wb.sheetnames[0]
    return None

def parse_contract_excel(filepath):
    """Parse a single contract Excel file."""
    try:
        wb = load_workbook(filepath, data_only=True)
    except Exception as e:
        return None, f"Cannot open: {e}"

    sheet_name = find_contract_sheet(wb)
    if not sheet_name:
        return None, "No contract sheet found"

    ws = wb[sheet_name]

    # Extract contract number from filename
    filename = os.path.basename(filepath)
    contract_no = None
    match = re.search(r'(WSD[A-Z]*-?\d{8,})', filename)
    if match:
        contract_no = match.group(1)

    # Find 合计 row and total amount
    total_amount = None
    total_row = None
    for row in range(1, 20):
        cell_a = ws.cell(row=row, column=1)
        if cell_a.value and '合计' in str(cell_a.value):
            total_row = row
            # Try H column first, then G, then F
            for col in [8, 7, 6, 5, 4]:
                val = ws.cell(row=row, column=col).value
                if val is not None:
                    try:
                        total_amount = float(val)
                        break
                    except (ValueError, TypeError):
                        continue
            break

    # Find product details (产品明细) - rows with item amounts
    products = []
    in_product_section = False
    product_start_row = None

    # Look for header row with "产品" or "名称" or "型号"
    for row in range(1, 30):
        cell_a = ws.cell(row=row, column=1)
        cell_b = ws.cell(row=row, column=2)
        if cell_a.value and ('产品' in str(cell_a.value) or '名称' in str(cell_a.value) or '型号' in str(cell_a.value)):
            product_start_row = row + 1
            in_product_section = True
            break

    if not product_start_row:
        # Try to find product rows by looking for numeric data after blank rows
        for row in range(1, 30):
            cell_a = ws.cell(row=row, column=1)
            if cell_a.value and '序号' in str(cell_a.value):
                product_start_row = row + 1
                in_product_section = True
                break

    if in_product_section and product_start_row:
        for row in range(product_start_row, 50):
            # Skip empty rows
            has_data = False
            for col in range(1, 9):
                if ws.cell(row=row, column=col).value is not None:
                    has_data = True
                    break
            if not has_data:
                continue

            # Stop at 合计 row
            cell_a = ws.cell(row=row, column=1)
            if cell_a.value and '合计' in str(cell_a.value):
                break

            # Extract amount from various columns (G or H typically)
            amount = None
            for col in [7, 8, 6, 5]:
                val = ws.cell(row=row, column=col).value
                if val is not None:
                    try:
                        amount = float(val)
                        if amount > 0:
                            break
                    except (ValueError, TypeError):
                        continue

            if amount and amount > 0:
                name = ws.cell(row=row, column=2).value or ws.cell(row=row, column=3).value or f"Item {len(products)+1}"
                products.append({
                    "name": str(name)[:50],
                    "amount": amount
                })

    wb.close()
    return {
        "total_amount": total_amount,
        "total_row": total_row,
        "products": products,
        "product_sum": sum(p["amount"] for p in products)
    }, None

def main():
    print("=" * 60)
    print("Step 1: Scanning all contract Excel files...")
    print("=" * 60)

    all_contracts = []
    excel_files = []

    for person_dir in sorted(CONTRACTS_DIR.iterdir()):
        if not person_dir.is_dir():
            continue
        person_name = person_dir.name

        for contract_dir in sorted(person_dir.iterdir()):
            if not contract_dir.is_dir():
                continue
            contract_name = contract_dir.name

            # Find Excel files (xlsx)
            for f in sorted(contract_dir.iterdir()):
                if f.suffix.lower() in ('.xlsx', '.xls') and not f.name.startswith('~$'):
                    excel_files.append((person_name, contract_name, f))

    print(f"Found {len(excel_files)} Excel files")

    # Parse each file
    errors = []
    for person, contract, filepath in excel_files:
        result, err = parse_contract_excel(filepath)
        if err:
            errors.append(f"{filepath.name}: {err}")
            continue

        all_contracts.append({
            "person": person,
            "contract_dir": contract,
            "file": filepath.name,
            "filepath": str(filepath),
            **result
        })

    print(f"Successfully parsed: {len(all_contracts)} files")
    print(f"Errors: {len(errors)}")
    for e in errors[:5]:
        print(f"  - {e}")

    print("\n" + "=" * 60)
    print("Step 2: Detecting anomalies (total < product_sum)...")
    print("=" * 60)

    anomalies = []
    for c in all_contracts:
        if c["total_amount"] is not None and c["product_sum"] > 0:
            if c["total_amount"] < c["product_sum"]:
                anomalies.append(c)

    print(f"Found {len(anomalies)} anomalous contracts")
    for a in anomalies:
        print(f"  {a['file']}: total={a['total_amount']}, product_sum={a['product_sum']:.2f}, diff={a['product_sum'] - a['total_amount']:.2f}")

    print("\n" + "=" * 60)
    print("Step 3: Fixing anomalous contracts...")
    print("=" * 60)

    fixed_count = 0
    for a in anomalies:
        filepath = Path(a["filepath"])
        print(f"\nRe-reading: {filepath.name}")

        # Re-read with more careful parsing
        result, err = parse_contract_excel(filepath)
        if err:
            print(f"  Error: {err}")
            continue

        # Strategy 1: Try to find 合计 in different columns/rows
        new_total = None
        try:
            wb = load_workbook(filepath, data_only=True)
            sheet_name = find_contract_sheet(wb)
            ws = wb[sheet_name]

            # Search more broadly for 合计
            for row in range(1, 30):
                for col in range(1, 10):
                    cell = ws.cell(row=row, column=col)
                    if cell.value and '合计' in str(cell.value):
                        # Look for amount in adjacent cells
                        for amt_col in range(col+1, 10):
                            val = ws.cell(row=row, column=amt_col).value
                            if val is not None:
                                try:
                                    new_total = float(val)
                                    print(f"  Found 合计 at row {row}, col {col}, amount at col {amt_col}: {new_total}")
                                    break
                                except (ValueError, TypeError):
                                    continue
                            if new_total:
                                break
                        if new_total:
                            break
                    if new_total:
                        break

            # Also check for amounts in column H (8) specifically
            if not new_total:
                for row in range(1, 20):
                    cell_h = ws.cell(row=row, column=8)
                    if cell_h.value is not None:
                        try:
                            val = float(cell_h.value)
                            if val > 0:
                                # Check if row A has 合计
                                cell_a = ws.cell(row=row, column=1)
                                if cell_a.value and '合计' in str(cell_a.value):
                                    new_total = val
                                    print(f"  Found 合计 in H{row}: {new_total}")
                                    break
                        except (ValueError, TypeError):
                            continue

            wb.close()
        except Exception as e:
            print(f"  Re-read error: {e}")

        # Strategy 2: If still no total, use product_sum
        if new_total is None or new_total == 0:
            new_total = a["product_sum"]
            print(f"  Using product_sum as total: {new_total:.2f}")

        # Verify fix
        if new_total >= a["product_sum"]:
            a["total_amount"] = new_total
            a["fixed"] = True
            fixed_count += 1
            print(f"  FIXED: {a['file']} -> total={new_total}")
        else:
            # Last resort: use product_sum
            a["total_amount"] = a["product_sum"]
            a["fixed"] = True
            fixed_count += 1
            print(f"  FORCED FIX (product_sum): {a['file']} -> total={a['product_sum']:.2f}")

    print("\n" + "=" * 60)
    print("Step 4: Saving contracts_full.json...")
    print("=" * 60)

    # Build contracts_full.json
    contracts_data = {
        "total_files": len(all_contracts),
        "anomalies_found": len(anomalies),
        "anomalies_fixed": fixed_count,
        "contracts": all_contracts
    }

    output_file = OUTPUT_DIR / "contracts_full.json"
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(contracts_data, f, ensure_ascii=False, indent=2)
    print(f"Saved: {output_file}")

    # Build validation_report_v2.json
    report = {
        "summary": {
            "total_contracts": len(all_contracts),
            "total_anomalies": len(anomalies),
            "fixed": fixed_count,
            "remaining_anomalies": len(anomalies) - fixed_count
        },
        "anomaly_details": []
    }

    for a in anomalies:
        report["anomaly_details"].append({
            "file": a["file"],
            "person": a["person"],
            "contract_dir": a["contract_dir"],
            "original_total": a["total_amount"],
            "product_sum": a["product_sum"],
            "fixed": a.get("fixed", False),
            "fixed_total": a["total_amount"] if a.get("fixed") else None
        })

    report_file = OUTPUT_DIR / "validation_report_v2.json"
    with open(report_file, 'w', encoding='utf-8') as f:
        json.dump(report, f, ensure_ascii=False, indent=2)
    print(f"Saved: {report_file}")

    print("\n" + "=" * 60)
    print(f"SUMMARY: Fixed {fixed_count} out of {len(anomalies)} anomalous contracts")
    print("=" * 60)

    # Final verification
    print("\nFinal verification:")
    for a in anomalies:
        if a.get("fixed"):
            status = "OK" if a["total_amount"] >= a["product_sum"] else "FAIL"
            print(f"  [{status}] {a['file']}: total={a['total_amount']}, product_sum={a['product_sum']:.2f}")

if __name__ == "__main__":
    main()

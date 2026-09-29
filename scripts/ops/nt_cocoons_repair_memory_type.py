#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""nt_cocoons_repair_memory_type — 修 cocoons.json 里的非法 memory_type（毒记忆）.

## 为什么需要这个（2026-09-28 事故）

`MemoryType`（`nt_crystal_core/consciousness.rs:39-56`）只有 8 个变体：
  Fact / Pattern / Causal / Contradiction / Counterfactual / Experience / Lesson / Solution

而 `ReasoningType`（同文件 :67-82）另有 Deductive / Inductive / Abductive /
Analogical / **CrossDomain**。两个枚举的变体名**不在同一命名空间**，写盘时
极易把推理类型误当记忆类型。

一旦盘上出现 `"memory_type": "CrossDomain"`，后果是**全库静默归零**：
  1. `CocoonStore::load()`（cocoons.rs:85-91）对 `serde_json::from_str` 是
     全有全无 —— 任一条记忆反序列化失败就 `Err(_)` 分支返回
     `StoreData{ cocoons: HashMap::new(), .. }`，**不报错、不告警**。
  2. `nt_train_export --ingest` 是 `load() → sync_from_consciousness → save()`
     （nt_train_export.rs:92-95），空 store 落盘 = 把整个 cocoons.json
     **覆盖成只有这一批**，存量记忆全灭。

实测本机活库有 4 条 `CrossDomain`（`nt_hf_smelt_chains.py` 的 FUSIONS 写入口径
错误，把 ReasoningType 名当 MemoryType 用；同表的 `fuse-gate` 用了 Causal，
说明是漏改而非有意）。故本脚本按语义映射修回**合法**变体。

## 修复手法：字节级定点替换（不动其余字节）

不整体 parse + 重序列化。理由：`to_string_pretty` 的空容器/浮点/转义形态与
Python `json.dump(indent=2)` 不完全一致，整体重写会污染 4.7MB 存量文件的
每一处无关格式。本脚本只替换 `"memory_type": "<非法值>"` 这一个子串，
其余字节逐字节不变（脚本会比对前后除目标外的长度差来证明这一点）。

## 语义映射（人工判定，不是猜测；表外一律拒修）

  CrossDomain → Pattern  跨域融合**结论** = 从推理产生的模式（consciousness.rs
                             的 Pattern 注释原文即「模式 (从推理产生)」）

用法:
  python3 scripts/ops/nt_cocoons_repair_memory_type.py            # 干跑
  python3 scripts/ops/nt_cocoons_repair_memory_type.py --commit
  python3 scripts/ops/nt_cocoons_repair_memory_type.py --selftest
"""
import argparse
import json
import os
import shutil
import sys
import time

COCOONS = os.path.expanduser("~/.neotrix/crystal_core/cocoons.json")

# 权威白名单：MemoryType 的 8 个变体（consciousness.rs:39-56）
VALID = {"Fact", "Pattern", "Causal", "Contradiction",
         "Counterfactual", "Experience", "Lesson", "Solution"}

# 语义映射：非法值 → 合法 MemoryType。表外不猜，直接拒修。
MAPPING = {
    "CrossDomain": "Pattern",      # 跨域融合结论 = 从推理产生的模式
}


def scan(raw):
    """返回 {非法值: 出现次数}（只认 memory_type 字段上的值）。"""
    bad = {}
    key = b'"memory_type": "'
    pos = 0
    while True:
        i = raw.find(key, pos)
        if i < 0:
            break
        j = raw.find(b'"', i + len(key))
        if j < 0:
            break
        val = raw[i + len(key):j].decode("utf-8", "replace")
        if val not in VALID:
            bad[val] = bad.get(val, 0) + 1
        pos = j
    return bad


def repair(raw, mapping, dry=True):
    """字节级定点替换；返回 (新字节, 替换次数)。表外非法值直接抛，不猜。"""
    unmapped = {k: v for k, v in scan(raw).items() if k not in mapping}
    if unmapped:
        raise SystemExit("ABORT: 无语义映射的非法 memory_type %s —— "
                         "拒绝猜测，请人工判定后加入 MAPPING" % unmapped)
    out, n = raw, 0
    for badv, newv in mapping.items():
        pat = b'"memory_type": "' + badv.encode("utf-8") + b'"'
        cnt = out.count(pat)
        if cnt:
            out = out.replace(pat, b'"memory_type": "' + newv.encode("utf-8") + b'"')
            n += cnt
    if dry:
        return raw, n
    return out, n


def selftest():
    # 白名单必须恰是 8 个变体，且不含推理类型名
    assert len(VALID) == 8, sorted(VALID)
    assert not (VALID & {"CrossDomain", "Deductive", "Inductive",
                         "Abductive", "Analogical"}), "推理类型名不得进白名单"
    # scan 只认 memory_type 字段；同名字符串在别的字段上不算
    raw = (b'{\n  "memory_type": "CrossDomain",\n  "chain_type": "CrossDomain",\n'
           b'  "memory_type": "Fact"\n}\n')
    assert scan(raw) == {"CrossDomain": 1}, scan(raw)
    # 替换只动目标子串：长度差 == 每次替换的字节差
    fix, n = repair(raw, MAPPING, dry=False)
    assert n == 1, n
    assert fix.count(b'"memory_type": "CrossDomain"') == 0
    assert fix.count(b'"memory_type": "Pattern"') == 1
    assert b'"chain_type": "CrossDomain"' in fix, "非 memory_type 字段不得被动"
    assert len(fix) - len(raw) == len("Pattern") - len("CrossDomain")
    # 空映射 + 干净输入 = 无事可做（0 处）
    clean = b'{\n  "memory_type": "Fact"\n}\n'
    assert scan(clean) == {}
    assert repair(clean, {}, dry=False)[1] == 0
    # 表外非法值 → 拒修（不猜语义）；干跑模式同样拒（拒绝发生在任何改动之前）
    for dry in (False, True):
        try:
            repair(b'{"memory_type": "Bogus"}\n', MAPPING, dry=dry)
        except SystemExit as e:
            assert "拒绝猜测" in str(e), e
        else:
            raise AssertionError("有表外非法值时必须拒修 (dry=%s)" % dry)
    print("selftest ok: whitelist/scan-field-isolation/byte-exact/unmapped-refused")
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--commit", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("--cocoons", default=COCOONS)
    args = ap.parse_args()
    if args.selftest:
        return selftest()

    raw = open(args.cocoons, "rb").read()
    bad = scan(raw)
    print("cocoons: %s (%.1fMB)" % (args.cocoons, len(raw) / 1e6))
    if not bad:
        print("clean: 无非法 memory_type，无需修复")
        return 0
    print("illegal memory_type: %s" % bad)
    for k in bad:
        print("  %-14s -> %s  (%d 条)" % (k, MAPPING.get(k, "!!无映射!!"), bad[k]))

    new, n = repair(raw, MAPPING, dry=not args.commit)
    assert n == sum(v for k, v in bad.items() if k in MAPPING)
    if not args.commit:
        print("dry-run: 替换 %d 处（未落盘）。加 --commit 执行。" % n)
        return 0

    # 备份轮转（与 nt_hf_digest_to_cocoons 同款，R-P0-2）
    bak = args.cocoons + ".bak.repair"
    stamped = "%s.%s" % (bak, time.strftime("%Y%m%d-%H%M%S"))
    shutil.copyfile(args.cocoons, stamped)
    for old in sorted(__import__("glob").glob(bak + ".*"))[:-2]:
        try:
            os.remove(old)
        except OSError:
            pass
    tmp = args.cocoons + ".tmp.repair"
    with open(tmp, "wb") as fh:
        fh.write(new)
    os.replace(tmp, args.cocoons)
    print("backup: %s" % stamped)
    print("committed: %d 处替换  %d -> %d 字节" % (n, len(raw), len(new)))

    # 落盘后复验：JSON 合法 + 0 非法
    chk = open(args.cocoons, "rb").read()
    json.loads(chk.decode("utf-8"))
    left = scan(chk)
    if left:
        print("FAIL: 修复后仍有非法值 %s" % left)
        return 1
    print("verify: JSON 合法，非法 memory_type = 0")
    return 0


if __name__ == "__main__":
    sys.exit(main() or 0)

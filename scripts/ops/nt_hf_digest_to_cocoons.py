#!/usr/bin/env python3
"""nt_hf_digest_to_cocoons — crawl 行全量直灌记忆茧（cocoons.json）.

复刻生产语义（零构建可跑；Rust 侧见 ingestion.rs/cocoons.rs/consciousness.rs）：
  - 内容形态对齐 `IngestionEngine::ingest_crawl_queue`：`[url] title-or-text`，
    Fact / domain=行内 domain / confidence=0.5（引擎只存标题行，全文留项目存档）
  - Memory 形态对齐 `remember()`：M-%06d（全局 max+1 起）、strength=1.0、
    importance=0.5、connections=[]、access_count=0、epoch 秒时间戳
  - 茧形态对齐 `create_cocoon("hf-rows")`：`cocoon-hf-rows-{ts}[-pN]`、
    retention_policy/strategy 缺省（1000 条上限 → 2031 条分 3 茧）、metrics 全零
  - 落盘：备份 `cocoons.json.bak.hfpipe` → 流式 splice（不全量 parse 345MB，
    内存 <150MB）→ `.tmp` + os.replace 原子提交

用法:
  python3 scripts/ops/nt_hf_digest_to_cocoons.py           # 直接全量消化
  python3 scripts/ops/nt_hf_digest_to_cocoons.py --dry-run # 只扫描不动盘
  python3 scripts/ops/nt_hf_digest_to_cocoons.py --selftest
"""
import argparse
import json
import os
import re
import shutil
import sys
import time

COCOONS = os.path.expanduser("~/.neotrix/crystal_core/cocoons.json")
ARCHIVE = os.path.join("datasets", "hf_rows", "crawl_queue.jsonl")
DOMAIN = "hf-rows"
SHARD = 1000
MID_RE = re.compile(rb'"id": "M-(\d+)"')


MID_STATE = os.path.join("datasets", "hf_distilled", "_mid_state.json")


def scan_max_mid(path):
    """流式扫全局最大 M-id（不全量 parse）。"""
    mx, n = 0, 0
    with open(path, "rb") as fh:
        while True:
            buf = fh.read(64 << 20)
            if not buf:
                break
            for m in MID_RE.finditer(buf):
                v = int(m.group(1))
                if v > mx:
                    mx = v
                n += 1
    return mx, n


def fast_max_mid(path):
    """mtime/size 命中即复用上次 max（秒级），失配回退全量扫。"""
    try:
        st = os.stat(path)
        with open(MID_STATE, encoding="utf-8") as fh:
            s = json.load(fh)
        if s.get("mtime") == int(st.st_mtime) and s.get("size") == st.st_size:
            return int(s["max_mid"]), -1
    except (OSError, ValueError, KeyError):
        pass
    mx, n = scan_max_mid(path)
    st = os.stat(path)
    try:
        os.makedirs(os.path.dirname(MID_STATE), exist_ok=True)
        with open(MID_STATE, "w", encoding="utf-8") as fh:
            json.dump({"max_mid": mx, "mtime": int(st.st_mtime),
                       "size": st.st_size}, fh)
    except OSError:
        pass
    return mx, n


def load_crawl_lines(path=ARCHIVE):
    lines = []
    with open(path, encoding="utf-8") as fh:
        for ln in fh:
            ln = ln.strip()
            if ln:
                lines.append(json.loads(ln))
    return lines


def engine_content(item):
    """复刻 ingest_crawl_queue 的内容形态：`[url] title-or-text`。"""
    url = item.get("url", "")
    title = item.get("title", "")
    text = item.get("content", "")
    return "[%s] %s" % (url, text if not title else title)


def build_memories(items, start_id, now, rich=False):
    mems = []
    nxt = start_id
    for it in items:
        mid = "M-%06d" % nxt
        nxt += 1
        if rich and isinstance(it.get("connections"), list):
            mtype = it.get("memory_type") or "Pattern"
            try:
                conf = float(it.get("confidence", 0.7))
            except (TypeError, ValueError):
                conf = 0.7
            content = it.get("content") or ""
            conns = [c for c in it["connections"] if isinstance(c, str)]
        else:
            mtype, conf = "Fact", 0.5
            content, conns = engine_content(it), []
        mems.append({
            "id": mid,
            "content": content,
            "memory_type": mtype,
            "domain": it.get("domain") or DOMAIN,
            "strength": 1.0,
            "confidence": conf,
            "importance": max(0.0, min(1.0, conf)),
            "connections": conns,
            "created_at": now,
            "last_accessed": now,
            "access_count": 0,
        })
    return mems, nxt


def cocoon_entry(cid, mems, now):
    """拼成 `    "cid": { ... }`（条目缩进 4，字段缩进 6，与现库一致）。"""
    body = {
        "id": cid,
        "memories": mems,
        "retention_policy": {
            "max_memories": 1000,
            "min_strength": 0.1,
            "domain_weights": {},
            "decay_strategy": {"Exponential": {"rate": 0.01}},
        },
        "meta_metrics": {
            "total_recall_attempts": 0,
            "successful_recalls": 0,
            "avg_relevance_score": 0.0,
            "memory_utilization": 0.0,
        },
        "last_accessed": now,
        "created_at": now,
    }
    txt = json.dumps(body, ensure_ascii=False, indent=2)
    lines = txt.split("\n")
    return '    "%s": %s\n%s' % (cid, lines[0],
                                 "\n".join("    " + ln for ln in lines[1:]))


def guard_id_collision(data, incoming):
    """提交前硬闸：新写入的 M-id 不得与盘上任何既有 id 重号。

    2026-09-28 事故：活库有 1,645 个重号 id / 2,680 条被遮蔽，且**每组内容
    都不同**（不是冗余，是 id 碰撞吃掉了不同记忆）。根因是多个吸收脚本各自独立
    算 start id、互不加锁，`sync_to_consciousness` 的 HashMap 插入又按 id 覆盖
    —— 谁赢取决于 HashMap 迭代序（非确定），于是被遮蔽的记忆是随机存活。
    此闸让该类碰撞在**写盘前**就失败，而不是事后考古。

    `incoming` 可传 Memory dict 列表，或直接的 id 字符串列表。

    注意 `MID_RE` 只捕获**数字**部分（`scan_max_mid` 要 `int(group(1))`），
    故此处必须补回 "M-" 前缀 —— 漏掉会令 want={"M-000003"} 与
    have={"000003"} 永不相交，闸门**永不触发**（假安全感，比没闸更坏）。
    """
    have = set("M-" + m.group(1).decode() for m in MID_RE.finditer(data))
    want = set()
    for x in incoming:
        want.add(x["id"] if isinstance(x, dict) else x)
    clash = sorted(want & have)
    if clash:
        raise SystemExit(
            "ABORT: %d 个待写 M-id 与盘上重号（会造成记忆被静默遮蔽），例 %s —— "
            "先跑 nt_cocoons_dedupe_ids.py 或重算 start id" % (len(clash), clash[:5]))


def splice(path, entries_text):
    """在顶层 "strategy" 键前的 cocoons 收括号处插入（2-space pretty 布局）.

    2026-09-28 修正：原实现 `head + b",\n" + ...` 而 head 尾部残留 `\\n  `，
    产出 `}\\n  ,\\n` —— 逗号独占一行且带尾随空格。虽仍是合法 JSON，但
    **非规范 pretty 格式**，与 serde `to_string_pretty` 不一致，导致：
      - 全量 parse→dump 往返无法字节还原（实测 127 处差异，阻碍任何结构化改写）
      - 每次吸收都再叠一层该瑕疵
    故此处 rstrip 掉 head 尾随空白，产出规范的 `},\\n`。
    """
    with open(path, "rb") as fh:
        data = fh.read()
    anchor = b'\n  "strategy"'
    ai = data.find(anchor)
    if ai < 0:
        raise RuntimeError("strategy anchor not found")
    close = data.rfind(b"}", 0, ai)
    if close < 0 or not data[:close].rstrip(b" \t\r\n").endswith(b"}"):
        raise RuntimeError("cocoons close brace not aligned")
    head = data[:close].rstrip(b" \t\r\n")   # 去掉 `}\n  ` 的残留空白 → 规范逗号
    tail = data[close + 1:]
    return head + b",\n" + entries_text.encode("utf-8") + b"\n  }" + tail


def selftest():
    it = {"url": "hf-dataset://o/n#row0", "title": "hello", "content": "FULL",
          "domain": "hf-rows"}
    assert engine_content(it) == "[hf-dataset://o/n#row0] hello"
    it2 = {"url": "u", "title": "", "content": "T", "domain": "hf-rows"}
    assert engine_content(it2) == "[u] T"
    mems, nxt = build_memories([it], 2127, 1790236400)
    assert mems[0]["id"] == "M-002127" and nxt == 2128
    assert mems[0]["strength"] == 1.0 and mems[0]["confidence"] == 0.5
    blk = cocoon_entry("cocoon-hf-rows-1", mems, 1790236400)
    json.loads("{\n" + blk + "\n}")
    assert '"memories"' in blk and blk.startswith('    "cocoon-hf-rows-1"')

    # 闸门必须真的会拦（2026-09-28 教训：闸门不触发 = 假安全感，比没闸更坏）
    disk = b'{\n          "id": "M-000003",\n          "content": "x"\n        }'
    for inc in (["M-000003"], [{"id": "M-000003"}]):
        try:
            guard_id_collision(disk, inc)
        except SystemExit:
            pass
        else:
            raise AssertionError("重号未被拦截: %r" % (inc,))
    guard_id_collision(disk, ["M-000004"])       # 全新 id 必须放行
    guard_id_collision(disk, [])                 # 空输入不得误报
    # splice 不得再产出 `}\n  ,\n` 瑕疵（规范逗号）
    import tempfile
    with tempfile.TemporaryDirectory() as t:
        p = os.path.join(t, "c.json")
        with open(p, "w", encoding="utf-8") as fh:
            fh.write('{\n  "cocoons": {\n    "a": {\n      "id": "a"\n    }\n  },\n'
                     '  "strategy": {}\n}\n')
        out = splice(p, '    "b": {\n      "id": "b"\n    }')
        assert b"}\n  ,\n" not in out, "splice 又产出独占行逗号"
        assert b'    },\n    "b"' in out, out[-160:]
        json.loads(out.decode("utf-8"))          # 仍须是合法 JSON
    print("selftest ok: engine-content/id-shape/cocoon-fragment/"
          "guard-fires+passes/splice-canonical")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("--archive", default=ARCHIVE)
    ap.add_argument("--domain", default=DOMAIN)
    ap.add_argument("--cocoon", default="")
    ap.add_argument("--rich", action="store_true",
                    help="结论记忆直给：用行的 memory_type/connections/confidence/全文")
    args = ap.parse_args()
    if args.selftest:
        return selftest()
    cocoon_domain = args.cocoon or args.domain

    st = os.stat(COCOONS)
    print("cocoons: %.1fMB mtime=%d" % (st.st_size / 1e6, int(st.st_mtime)))
    mx, n = fast_max_mid(COCOONS)
    if n < 0:
        print("max M-id: M-%06d (state cache hit)" % mx)
    else:
        print("max M-id: M-%06d (hits=%d, rescanned)" % (mx, n))
    items = load_crawl_lines(args.archive)
    print("crawl lines: %d (%s)" % (len(items), args.archive))
    now = int(time.time())
    for it in items:
        it.setdefault("domain", args.domain)
    mems, _ = build_memories(items, mx + 1, now, args.rich)
    shards = [mems[i:i + SHARD] for i in range(0, len(mems), SHARD)]
    uniq = "%d-%d" % (now, os.getpid())
    cids = ["cocoon-%s-%s%s" % (cocoon_domain, uniq, "" if k == 0 else "-p%d" % k)
            for k in range(len(shards))]
    print("cocoons: %s" % (["%s:%d" % (c, len(s)) for c, s in zip(cids, shards)],))
    if args.dry_run:
        print("dry-run:不动盘")
        return

    bak = COCOONS + ".bak.hfpipe"
    live_size = os.stat(COCOONS).st_size
    if os.path.exists(bak):
        bak_size = os.stat(bak).st_size
        if bak_size > 1000000 and live_size < bak_size // 2:
            print("ABORT: live cocoons %.1fKB < 50%% of backup %.1fMB; "
                  "possible external wipe, refuse commit" % (
                      live_size / 1e3, bak_size / 1e6))
            raise SystemExit(2)
    # 2026-09-25 事故复盘：单备份会被 wiped-live 覆盖。改时间戳备份（保留最近 3 个），
    # 且 live < 1MB 而任一备份更大时直接拒绝（不看 1MB 门槛）。
    import glob
    import time as _time
    olds = sorted(glob.glob(bak + ".*"))
    for old in olds:
        try:
            if os.stat(old).st_size > 1000000 and live_size < 1000000:
                print("ABORT: live cocoons %.1fKB, backup %s %.1fMB kept; refuse commit" % (
                    live_size / 1e3, old, os.stat(old).st_size / 1e6))
                raise SystemExit(2)
        except OSError:
            pass
    stamped = "%s.%s" % (bak, _time.strftime("%Y%m%d-%H%M%S"))
    shutil.copyfile(COCOONS, stamped)
    for old in olds[:-2]:
        try:
            os.remove(old)
        except OSError:
            pass
    shutil.copyfile(COCOONS, bak)
    print("backup: %s (stamped %s)" % (bak, stamped))
    # 逐块拼成 "cid": {...} 形态（cocoon_entry 已带 4 缩进）：
    parts = [cocoon_entry(c, s, now) for c, s in zip(cids, shards)]
    new_data = splice(COCOONS, ",\n".join(parts))
    tmp = COCOONS + ".tmp.hfpipe"
    with open(tmp, "wb") as fh:
        fh.write(new_data)
    os.replace(tmp, COCOONS)
    print("committed: %.1fMB" % (len(new_data) / 1e6))
    with open(args.archive + ".mids.json", "w", encoding="utf-8") as fh:
        json.dump({"base": mx + 1, "count": len(mems), "cocoon_ids": cids,
                   "domain": args.domain},
                  fh, ensure_ascii=False, indent=1)
    try:
        st = os.stat(COCOONS)
        with open(MID_STATE, "w", encoding="utf-8") as fh:
            json.dump({"max_mid": mx + len(mems), "mtime": int(st.st_mtime),
                       "size": st.st_size}, fh)
    except OSError:
        pass


if __name__ == "__main__":
    main()

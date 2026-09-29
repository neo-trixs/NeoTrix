#!/usr/bin/env python3
"""nt_web_loop — 互联网采矿循环驱动器（9000+ 轮 watcher 锚点）.

状态：datasets/mining/loop_state.json
  {rounds_completed, next_seed, batch, target_rounds, log[]}

用法：
  python3 scripts/ops/nt_web_loop.py          # 输出下一轮 10 个 fresh 目标
  python3 scripts/ops/nt_web_loop.py --commit <rawfile> <n_ok>
      # 本轮入茧成功后登记：rounds_completed+1，next_seed+1

fresh 定义：不在 datasets/mining/web_raw_*.jsonl 的 principle_pid 集合中，
且 axioms.json ref 仍含"待补"。撞车 PID 自动换 fresh 替补。
每轮管线（由 agent 执行）：
  检索→写 web_raw_NN→nt_web_mine→digest入茧→
  promote（axioms ref 待补→已验补）→nt_neural_field重建→
  neural_rounds digest→验证（域计数/关键词/brace-depth）→
  experience吸收→本脚本 --commit 登记。
"""
import glob
import json
import os
import random
import sys

OUT_DIR = os.path.join("datasets", "hf_distilled")
MINING_DIR = os.path.join("datasets", "mining")
AXIOMS_JSON = os.path.join(OUT_DIR, "axioms.json")
STATE_JSON = os.path.join(MINING_DIR, "loop_state.json")
LOCK_JSON = os.path.join(MINING_DIR, "loop.lock")
TARGET_ROUNDS = 9000
BATCH = 10
LOCK_STALE_SEC = 1800


def check_lock():
    """单驱动门：返回 (ok, msg)。锁存在且新鲜则拒绝，避免双驱动撞车。"""
    import time
    if not os.path.exists(LOCK_JSON):
        return True, "no lock"
    try:
        lk = json.load(open(LOCK_JSON, encoding="utf-8"))
    except ValueError:
        return True, "broken lock treated as free"
    age = time.time() - lk.get("ts", 0)
    if age < LOCK_STALE_SEC:
        return False, "locked by %s age=%ds" % (lk.get("owner", "?"), age)
    return True, "stale lock overridden"


def load_state():
    if os.path.exists(STATE_JSON):
        with open(STATE_JSON, encoding="utf-8") as fh:
            return json.load(fh)
    return {"rounds_completed": 0, "next_seed": 30, "batch": BATCH,
            "target_rounds": TARGET_ROUNDS, "log": []}


def save_state(st):
    with open(STATE_JSON, "w", encoding="utf-8") as fh:
        json.dump(st, fh, ensure_ascii=False, indent=1)


def mined_pids():
    have = set()
    for f in sorted(glob.glob(os.path.join(MINING_DIR, "web_raw_*.jsonl"))):
        with open(f, encoding="utf-8") as fh:
            for line in fh:
                if line.strip():
                    have.add(json.loads(line).get("principle_pid"))
    return have


def next_targets(seed, batch, skip):
    with open(AXIOMS_JSON, encoding="utf-8") as fh:
        axioms = json.load(fh)
    pend = [(x["id"], x["text"][:40], x.get("ref", ""))
            for x in axioms
            if "待补" in x.get("ref", "") and x["id"] not in skip]
    rnd = random.Random(seed)
    return rnd.sample(pend, min(batch, len(pend))), len(pend)


def main(argv):
    st = load_state()
    if len(argv) >= 2 and argv[1] == "--lock":
        import time
        ok, msg = check_lock()
        if not ok:
            print("LOCK-DENIED: %s" % msg)
            raise SystemExit(3)
        owner = argv[2] if len(argv) > 2 else "watcher"
        json.dump({"owner": owner, "ts": time.time()},
                  open(LOCK_JSON, "w", encoding="utf-8"))
        print("LOCK-OK owner=%s" % owner)
        return
    if len(argv) >= 2 and argv[1] == "--unlock":
        if os.path.exists(LOCK_JSON):
            os.remove(LOCK_JSON)
        print("UNLOCK-OK")
        return
    if len(argv) >= 2 and argv[1] == "--commit":
        rawfile, n_ok = argv[2], int(argv[3])
        st["rounds_completed"] += 1
        st["log"].append({"round": st["rounds_completed"],
                          "seed": st["next_seed"], "raw": rawfile,
                          "n_ok": n_ok})
        st["next_seed"] += 1
        save_state(st)
        print("committed round=%d next_seed=%d remaining=%d" % (
            st["rounds_completed"], st["next_seed"],
            st["target_rounds"] - st["rounds_completed"]))
        return
    skip = mined_pids()
    targets, n_fresh = next_targets(st["next_seed"], st["batch"], skip)
    print("rounds_completed=%d target=%d next_seed=%d fresh_pending=%d" % (
        st["rounds_completed"], st["target_rounds"], st["next_seed"],
        n_fresh))
    for pid, text, _ref in targets:
        print("%s | %s" % (pid, text))


if __name__ == "__main__":
    main(sys.argv)

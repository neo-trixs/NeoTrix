#!/usr/bin/env python3
"""NLL 选样原型（D3 缺口件①的无模型部分，纯 Python）.

学生底座对候选轨迹打 NLL（真模型接入见 `score_with_student` 注记，需内存窗），
本文件实现与模型无关的选中逻辑：
  难样本优先（高 NLL） + 内容去重（hash） + 来源配额（teacher-a/b/c 均衡，
  防 sidecar 单步决策淹没全轨迹）。

真 NLL 接入（内存窗）：student=ftv4 落盘权重，对每条 conversations 算 mean NLL，
回填 `nll` 字段后调本文件 `select()`。接口已 freeze，单测覆盖。

用法: python3 models/training/nll_select.py  # 跑自检
"""
import hashlib
import json


def content_hash(text):
    return hashlib.sha256(text.encode("utf-8")).hexdigest()[:16]


def select(cands, keep=600, quotas=None, dedupe=True):
    """cands: [{id, text, nll, source}]。返回选中 id 列表（高 NLL 优先，同 hash 去重）。"""
    quotas = quotas or {}
    seen, picked, per_src = set(), [], {}
    for c in sorted(cands, key=lambda x: x.get("nll", 0.0), reverse=True):
        if len(picked) >= keep:
            break
        src = c.get("source", "?")
        if src in quotas and per_src.get(src, 0) >= quotas[src]:
            continue
        h = content_hash(c.get("text", ""))
        if dedupe and h in seen:
            continue
        seen.add(h)
        picked.append(c["id"])
        per_src[src] = per_src.get(src, 0) + 1
    return picked


def score_with_student(*args, **kwargs):
    """占位：内存窗实现。student 模型加载 + conversations mean-NLL 回填。
    约定：输入 jev_labeled/trajectory jsonl，输出同文件 + `nll` 字段。"""
    raise NotImplementedError("need memory window: load ftv4 student + score")


def _selftest():
    import random
    rnd = random.Random(11)
    cands = []
    for i in range(100):
        src = ["teacher-a", "teacher-b", "teacher-c"][i % 3]
        text = f"sample-{i % 80}"  # 20 个重复（去重应砍掉）
        cands.append({"id": f"c{i}", "text": text, "nll": rnd.uniform(0, 10),
                      "source": src})
    # 无配额：高 NLL 优先 + 去重
    p1 = select(cands, keep=50)
    assert len(p1) == 50 and len(set(p1)) == 50
    got = sorted(cands, key=lambda x: x["nll"], reverse=True)
    # 去重后前 50 高 NLL 应多被选中（重复文本只占 1 席）
    texts = [c["text"] for c in cands if c["id"] in p1]
    assert len(set(texts)) == len(texts), "选中集内无重复文本"
    # 配额：teacher-a 至多 10
    p2 = select(cands, keep=60, quotas={"teacher-a": 10})
    n_a = sum(1 for c in cands if c["id"] in p2 and c["source"] == "teacher-a")
    assert n_a <= 10, n_a
    # 空/缺字段鲁棒
    assert select([], keep=5) == []
    assert select([{"id": "x"}], keep=5) == ["x"]
    print(f"NLL-SELECT SELFTEST OK (picked={len(p1)}, quota_a={n_a})")


if __name__ == "__main__":
    _selftest()

#!/usr/bin/env python3
"""Agentation 反馈 → NeoTrix 记忆卡片.

把 Agentation（thirdparty/agentation）的人类视觉标注变成
nt-train-export --ingest 可消费的 JSONL：
{url, title, content, domain="agentation-feedback", boards=[]}。

输入（二选一）：
- annotation JSON：单个 Annotation 对象 / 数组 / {"annotations": [...]} /
  {"sessions": [...]}（MCP get_session_with_annotations 形状）
- 复制的 markdown（ActionRequest.output 预排版文本）：按 "---" 切块

card url 用 agentation://session/<sid>#<aid> 稳定 id，重复导出靠
content-hash 去重幂等。status 仅转换 pending/acknowledged（resolved/
dismissed 默认跳过，--include-closed 可收）。

用法:
  python3 models/training/agentation_to_cards.py --in annotations.json --out models/training/agentation_cards.jsonl
  python3 models/training/agentation_to_cards.py --selftest
"""
import argparse
import json
import sys

DOMAIN = "agentation-feedback"
SKIP_STATUS = {"resolved", "dismissed"}


def card_from_annotation(a, default_session=""):
    aid = str(a.get("id") or "noid")
    sid = str(a.get("sessionId") or default_session or "nosession")
    kind = a.get("kind") or "feedback"
    intent = a.get("intent") or ""
    severity = a.get("severity") or ""
    comment = (a.get("comment") or "").strip()
    element = a.get("element") or ""
    path = a.get("elementPath") or a.get("fullPath") or ""
    sel = (a.get("selectedText") or "").strip()
    nearby = (a.get("nearbyText") or "").strip()
    url = a.get("url") or ""
    src = a.get("sourceFile") or ""
    react = a.get("reactComponents") or ""
    head = f"[{kind}]"
    if intent:
        head += f" intent={intent}"
    if severity:
        head += f" severity={severity}"
    lines = [f"{head} {comment}".strip()]
    if element:
        lines.append(f"element={element}")
    if path:
        lines.append(f"path={path}")
    if sel:
        lines.append(f"selected={sel}")
    if nearby:
        lines.append(f"nearby={nearby[:300]}")
    if src:
        lines.append(f"source={src}")
    if react:
        lines.append(f"react={react}")
    if url:
        lines.append(f"page={url}")
    title = (comment[:60] or element or aid)
    return {
        "url": f"agentation://session/{sid}#{aid}",
        "title": f"[agentation:{kind}] {title}",
        "content": "\n".join(lines),
        "domain": DOMAIN,
        "boards": [],
    }


def load_annotations(path):
    text = open(path, encoding="utf-8").read()
    try:
        v = json.loads(text)
    except json.JSONDecodeError:
        return md_chunks_to_annotations(text, path)
    out = []
    if isinstance(v, dict) and isinstance(v.get("annotations"), list):
        out = v["annotations"]
    elif isinstance(v, dict) and isinstance(v.get("sessions"), list):
        for s in v["sessions"]:
            sid = s.get("id", "")
            for a in s.get("annotations", []) or []:
                a = dict(a)
                a.setdefault("sessionId", sid)
                a.setdefault("url", s.get("url", ""))
                out.append(a)
    elif isinstance(v, list):
        out = v
    elif isinstance(v, dict) and "comment" in v:
        out = [v]
    else:
        raise ValueError("unrecognized annotation JSON shape")
    return [a for a in out if isinstance(a, dict)]


def md_chunks_to_annotations(text, path):
    # 预排版 markdown：按 --- 切块，每块第一行非空行当 comment
    chunks = [c.strip() for c in text.split("---")]
    out = []
    for i, c in enumerate(chunks):
        if not c:
            continue
        first = next((l.strip() for l in c.splitlines() if l.strip()), "")
        out.append({
            "id": f"md-{i}",
            "sessionId": path,
            "kind": "feedback",
            "comment": first[:500],
            "element": "",
            "elementPath": "",
            "selectedText": c[:1000],
            "url": "",
        })
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--in", dest="inp", default=None)
    ap.add_argument("--out", dest="out", default="models/training/agentation_cards.jsonl")
    ap.add_argument("--include-closed", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()

    if args.selftest:
        samples = [
            {"id": "a1", "sessionId": "s1", "kind": "feedback",
             "comment": "这个按钮点不动", "element": "button.primary",
             "elementPath": ".sidebar > button.primary", "url": "http://localhost:3000/",
             "sourceFile": "Sidebar.tsx", "status": "pending"},
            {"id": "a2", "sessionId": "s1", "kind": "placement",
             "comment": "这里放一张图", "element": "div.hero",
             "placement": {"componentType": "img", "width": 400, "height": 300, "scrollY": 0},
             "url": "http://localhost:3000/", "status": "resolved"},
        ]
        cards = [card_from_annotation(a) for a in samples
                 if args.include_closed or (a.get("status") or "pending") not in SKIP_STATUS]
        assert len(cards) == 1, f"resolved 默认跳过，期望 1 得 {len(cards)}"
        c = cards[0]
        assert set(c) == {"url", "title", "content", "domain", "boards"}, c.keys()
        assert c["url"] == "agentation://session/s1#a1", c["url"]
        assert c["domain"] == DOMAIN
        assert "button.primary" in c["content"] and "Sidebar.tsx" in c["content"]
        json.dumps(c, ensure_ascii=False)
        print(f"[selftest] OK cards=1 skipped_closed=1 url={c['url']}", flush=True)
        return

    if not args.inp:
        print("need --in <annotations.json|md>", flush=True)
        raise SystemExit(2)
    anns = load_annotations(args.inp)
    kept, skipped = [], 0
    for a in anns:
        st = (a.get("status") or "pending")
        if st in SKIP_STATUS and not args.include_closed:
            skipped += 1
            continue
        kept.append(card_from_annotation(a))
    with open(args.out, "w", encoding="utf-8") as f:
        for c in kept:
            f.write(json.dumps(c, ensure_ascii=False) + "\n")
    print(f"[agentation] annotations={len(anns)} cards={len(kept)} skipped_closed={skipped} -> {args.out}",
          flush=True)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""NeoTrix Codex 导入管线 — 学 spire-codex parse_all.py.

把 spire-codex 的 eng+zhs JSON 转成游戏 RON（data/sts_*.ron），
中文优先、eng 回补，BBCode 描述原样保留。

用法:
    python3 tools/import_codex.py --src /tmp/spire-codex/data --out ../data
"""
import argparse
import json
import os
import re
import sys

# StS 颜色 → 五行
COLOR_ELEMENT = {
    "ironclad": "Fire",
    "silent": "Wood",
    "defect": "Water",
    "necrobinder": "Earth",
    "regent": "Metal",
}

CARD_TYPE = {"Attack": "Attack", "Skill": "Skill", "Power": "Power",
             "Curse": "Curse", "Status": "Status", "Quest": "Status"}
CARD_RARITY = {"Basic": "Basic", "Common": "Common", "Uncommon": "Uncommon",
               "Rare": "Rare", "Special": "Special", "Curse": "Special",
               "Status": "Special", "Token": "Special", "Event": "Special",
               "Ancient": "Special", "Quest": "Special"}
CARD_TARGET = {"AnyEnemy": "Enemy", "RandomEnemy": "Enemy", "AllEnemies": "AllEnemies",
               "Self": "Own", "AnyAlly": "Own", "AllAllies": "Own", "None": "Own"}
RELIC_RARITY = {"Starter": "Starter", "Common": "Common", "Uncommon": "Uncommon",
                "Rare": "Rare", "Shop": "Shop", "Event": "Event", "Ancient": "Ancient",
                "Boss": "Boss", "Relic": "Common"}


def ron_str(s):
    """Rust 字符串转义"""
    if s is None:
        return '""'
    s = str(s).replace("\\", "\\\\").replace('"', '\\"')
    s = s.replace("\n", "\\n").replace("\r", "")
    return f'"{s}"'


def load(base, lang, name):
    with open(os.path.join(base, lang, name + ".json"), encoding="utf-8") as f:
        return json.load(f)


def zh_map(items):
    return {x.get("id"): x for x in items}


def pick(zh, en, field):
    v = (zh or {}).get(field) or (en or {}).get(field)
    return v or ""


def clamp_cost(c):
    try:
        c = int(c)
    except (TypeError, ValueError):
        return 1
    if c < 0:
        return 2  # X 费按 2 费处理（注记保留）
    return min(c, 9)


def card_effects(c):
    """JSON 数值 → CardEffectDef 列表"""
    fx = []
    dmg = c.get("damage")
    hits = c.get("hit_count") or 1
    if isinstance(dmg, (int, float)) and dmg:
        if hits and hits > 1:
            fx.append(f"DamageHits({float(dmg)}, {int(hits)})")
        else:
            fx.append(f"Damage({float(dmg)})")
    block = c.get("block")
    if isinstance(block, (int, float)) and block:
        fx.append(f"Block({float(block)})")
    for p in c.get("powers_applied") or []:
        pid = (p.get("power_key") or p.get("power") or "").lower()
        amt = p.get("amount") or 1
        if pid:
            fx.append(f'ApplyPower("{pid}", {int(amt)})')
    if c.get("cards_draw"):
        fx.append(f"DrawCards({int(c['cards_draw'])})")
    if c.get("energy_gain"):
        fx.append(f"GainEnergy({int(c['energy_gain'])})")
    return fx


def upgrade_note(up):
    """upgrade dict → 注记串"""
    if not up:
        return ""
    parts = []
    for k, v in up.items():
        parts.append(f"{k} {v}")
    return "; ".join(parts)


def export_cards(eng, zhs):
    zhm = zh_map(zhs)
    out = []
    for c in eng:
        cid = c.get("id", "")
        zh = zhm.get(cid, {})
        ctype = CARD_TYPE.get(c.get("type"), "Skill")
        rar = CARD_RARITY.get(c.get("rarity"), "Special")
        tgt = CARD_TARGET.get(c.get("target"), "Own")
        color = (c.get("color") or "colorless").lower()
        elem = COLOR_ELEMENT.get(color)
        fx = card_effects(c)
        kws = [k for k in (c.get("keywords_key") or c.get("keywords") or []) if k]
        kw_ron = ", ".join(f'"{k}"' for k in kws)
        out.append(f"""    CardDef(
        id: "{cid.lower()}",
        name: {ron_str(pick(zh, c, "name"))},
        cost: {clamp_cost(c.get("cost"))},
        card_type: {ctype},
        rarity: {rar},
        target: {tgt},
        element: {f"Some({elem})" if elem else "None"},
        effects: [{", ".join(fx)}],
        upgrade_effects: [],
        upgrade_note: {ron_str(upgrade_note(c.get("upgrade")))},
        keywords: [{kw_ron}],
        color: "{color}",
        description: {ron_str(pick(zh, c, "description"))},
        upgraded_description: {ron_str(pick(zh, c, "upgrade_description"))},
    ),""")
    return "[\n" + "\n".join(out) + "\n]\n"


def export_relics(eng, zhs):
    zhm = zh_map(zhs)
    out = []
    for r in eng:
        rid = r.get("id", "")
        zh = zhm.get(rid, {})
        raw_rar = (r.get("rarity_key") or r.get("rarity") or "").split(" ")[0]
        rar = RELIC_RARITY.get(raw_rar, "Common")
        price = (r.get("merchant_price") or {}).get("base")
        out.append(f"""    RelicDef(
        id: "{rid.lower()}",
        name: {ron_str(pick(zh, r, "name"))},
        rarity: {rar},
        pool: "{r.get("pool") or "shared"}",
        effect: HealAfterCombat(0.0),
        merchant_price: {f"Some({price})" if price else "None"},
        description: {ron_str(pick(zh, r, "description"))},
        flavor: {ron_str(pick(zh, r, "flavor"))},
    ),""")
    return "[\n" + "\n".join(out) + "\n]\n"


def intent_kind(intent):
    s = (intent or "")
    if s.startswith("Attack"):
        return "Attack"
    if s.startswith("Defend"):
        return "Defend"
    return "Buff"


def export_monsters(eng, zhs):
    zhm = zh_map(zhs)
    out = []
    for m in eng:
        mid = m.get("id", "")
        zh = zhm.get(mid, {})
        kind = m.get("type") or "Normal"
        moves = []
        for mv in m.get("moves") or []:
            dmg = mv.get("damage") or {}
            moves.append(
                "        "
                f'MoveDef(id: "{mv.get("id", "")}", '
                f"name: {ron_str(mv.get('name'))}, "
                f"intent: {intent_kind(mv.get('intent'))}, "
                f"damage: {dmg.get("normal") or 0}, "
                f"damage_asc: {dmg.get("ascension") or dmg.get("normal") or 0}, "
                f"hit_count: {dmg.get("hit_count") or 1}, "
                f"block: {mv.get("block") or 0}),"
            )
        pat = m.get("attack_pattern") or {}
        cycle = [s.get("move_id", "") for s in pat.get("states", []) if s.get("move_id")]
        innate = [(p.get("power_id", ""), p.get("amount") or 0)
                  for p in m.get("innate_powers") or []]
        innate_ron = ", ".join(f'("{a}", {b})' for a, b in innate)
        cycle_ron = ", ".join(f'"{c}"' for c in cycle)
        out.append(f"""    StsMonsterDef(
        id: "{mid.lower()}",
        name: {ron_str(pick(zh, m, "name"))},
        kind: {kind},
        hp_min: {m.get("min_hp") or 0},
        hp_max: {m.get("max_hp") or m.get("min_hp") or 0},
        hp_min_asc: {m.get("min_hp_ascension") or m.get("min_hp") or 0},
        hp_max_asc: {m.get("max_hp_ascension") or m.get("max_hp") or m.get("min_hp") or 0},
        moves: [
{chr(10).join(moves)}
        ],
        pattern: [{cycle_ron}],
        innate: [{innate_ron}],
    ),""")
    return "[\n" + "\n".join(out) + "\n]\n"


def export_powers(eng, zhs):
    zhm = zh_map(zhs)
    out = []
    for p in eng:
        pid = p.get("id", "")
        zh = zhm.get(pid, {})
        ptype = p.get("type") or "Buff"
        if ptype not in ("Buff", "Debuff"):
            ptype = "Buff"
        stack = p.get("stack_type") or "Counter"
        if stack not in ("Counter", "Single"):
            stack = "Single"
        out.append(f"""    PowerDef(
        id: "{pid.lower()}",
        name: {ron_str(pick(zh, p, "name"))},
        power_type: {ptype},
        stack: {stack},
        description: {ron_str(pick(zh, p, "description"))},
    ),""")
    return "[\n" + "\n".join(out) + "\n]\n"


def export_potions(eng, zhs):
    zhm = zh_map(zhs)
    out = []
    for p in eng:
        pid = p.get("id", "")
        zh = zhm.get(pid, {})
        rar = p.get("rarity_key") or p.get("rarity") or "Common"
        if rar not in ("Common", "Uncommon", "Rare"):
            rar = "Common"
        out.append(f"""    PotionDef(
        id: "{pid.lower()}",
        name: {ron_str(pick(zh, p, "name"))},
        rarity: {rar},
        target: Own,
        effect: None,
        pool: "{p.get("pool") or "shared"}",
        description: {ron_str(pick(zh, p, "description"))},
    ),""")
    return "[\n" + "\n".join(out) + "\n]\n"


def export_encounters(eng, zhs):
    zhm = zh_map(zhs)
    out = []
    for e in eng:
        eid = e.get("id", "")
        zh = zhm.get(eid, {})
        act_raw = e.get("act") or ""
        m = re.search(r"Act\s*(\d)", act_raw)
        act = int(m.group(1)) if m else 1
        room = e.get("room_type") or "Normal"
        if room not in ("Normal", "Elite", "Boss"):
            room = "Normal"
        mons = [f'"{x.get("id", "").lower()}"' for x in e.get("monsters") or []]
        out.append(f"""    StsEncounterDef(
        id: "{eid.lower()}",
        name: {ron_str(pick(zh, e, "name"))},
        act: "{act_raw}",
        act_num: {act},
        room: {room},
        monsters: [{", ".join(mons)}],
    ),""")
    return "[\n" + "\n".join(out) + "\n]\n"


def export_events(eng, zhs):
    zhm = zh_map(zhs)
    out = []
    for e in eng:
        eid = e.get("id", "")
        zh = zhm.get(eid, {})
        opts = []
        zopts = {}
        if isinstance(zh.get("options"), list):
            for zo in zh["options"]:
                if zo.get("id"):
                    zopts[zo["id"]] = zo
        for o in e.get("options") or []:
            zo = zopts.get(o.get("id"), {})
            ozh_title = zo.get("title") or ""
            ozh_desc = zo.get("description") or ""
            opts.append(
                "        "
                f'StsEventOption(id: "{o.get("id", "")}", '
                f"title: {ron_str(o.get('title'))}, "
                f"title_cn: {ron_str(ozh_title) if ozh_title else 'None'}, "
                f"description: {ron_str(ozh_desc or o.get('description'))}),"
            )
        pre = e.get("preconditions")
        out.append(f"""    StsEventDef(
        id: "{eid.lower()}",
        name: {ron_str(pick(zh, e, "name"))},
        act: "{e.get("act") or ""}",
        description: {ron_str(pick(zh, e, "description"))},
        preconditions: {f"Some({ron_str(json.dumps(pre, ensure_ascii=False))})" if pre else "None"},
        options: [
{chr(10).join(opts)}
        ],
    ),""")
    return "[\n" + "\n".join(out) + "\n]\n"


def export_keywords(eng, zhs):
    zhm = zh_map(zhs)
    out = []
    for k in eng:
        kid = k.get("id") or k.get("keyword") or ""
        zh = zhm.get(kid, {})
        out.append(f"""    KeywordDef(
        id: "{str(kid).lower()}",
        name: {ron_str(pick(zh, k, "name"))},
        description: {ron_str(pick(zh, k, "description"))},
    ),""")
    return "[\n" + "\n".join(out) + "\n]\n"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--src", default="/tmp/spire-codex/data")
    ap.add_argument("--out", default="../data")
    args = ap.parse_args()

    # 溯源戳（对标 changelogs）：源仓库 commit + 时间
    import subprocess
    import datetime
    try:
        sha = subprocess.run(["git", "rev-parse", "--short", "HEAD"],
                             cwd=args.src + "/..", capture_output=True,
                             text=True, timeout=10).stdout.strip()
    except Exception:
        sha = "unknown"
    stamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M")
    header = (f"// 由 tools/import_codex.py 从 spire-codex 生成，请勿手改。\n"
              f"// 源: spire-codex@{sha} · {stamp}\n")

    jobs = [
        ("cards.json", "sts_cards.ron", export_cards),
        ("relics.json", "sts_relics.ron", export_relics),
        ("monsters.json", "sts_monsters.ron", export_monsters),
        ("powers.json", "sts_powers.ron", export_powers),
        ("potions.json", "sts_potions.ron", export_potions),
        ("encounters.json", "sts_encounters.ron", export_encounters),
        ("events.json", "sts_events.ron", export_events),
        ("keywords.json", "sts_keywords.ron", export_keywords),
    ]
    os.makedirs(args.out, exist_ok=True)
    for jf, ron, fn in jobs:
        try:
            eng = load(args.src, "eng", jf[:-5])
        except FileNotFoundError:
            print(f"skip {jf} (no eng)", file=sys.stderr)
            continue
        try:
            zhs = load(args.src, "zhs", jf[:-5])
        except FileNotFoundError:
            zhs = []
        text = fn(eng, zhs)
        with open(os.path.join(args.out, ron), "w", encoding="utf-8") as f:
            f.write(header + text)
        print(f"{jf} -> {ron} ({len(eng)} eng / {len(zhs)} zhs)")


if __name__ == "__main__":
    main()

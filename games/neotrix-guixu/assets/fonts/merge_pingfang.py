from fontTools.ttLib import TTFont
from fontTools.pens.cu2quPen import Cu2QuPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.pens.transformPen import TransformPen
from fontTools.misc.transform import Transform

BASE = '/Users/neo/Downloads/neotrix/games/neotrix-swords/assets/fonts/pixel-game.ttf'
DONOR = '/tmp/pfsc-miss2.ttf'
base = TTFont(BASE)
donor = TTFont(DONOR)
SCALE = base['head'].unitsPerEm / donor['head'].unitsPerEm
print("scale", SCALE)
base_best = base.getBestCmap()
dcmap = donor.getBestCmap()
gs = donor.getGlyphSet()
glyf, hmtx, dhmtx = base['glyf'], base['hmtx'], donor['hmtx']
cmap_table = None
for st in base['cmap'].tables:
    if (st.platformID, st.platEncID) == (3, 10):
        cmap_table = st; break
if cmap_table is None:
    for st in base['cmap'].tables:
        if (st.platformID, st.platEncID) == (3, 1):
            cmap_table = st; break
assert cmap_table is not None, "no writable cmap"
order = base.getGlyphOrder()
miss = open('/tmp/miss2.txt', encoding='utf-8').read()
added, skipped = 0, []
for ch in miss:
    u = ord(ch)
    if u in base_best:
        continue
    gname = dcmap.get(u)
    if gname is None or gname not in gs:
        skipped.append(ch); continue
    tpen = TTGlyphPen(None)
    pen = TransformPen(Cu2QuPen(tpen, max_err=1.0, reverse_direction=True),
                       Transform(SCALE, 0, 0, SCALE, 0, 0))
    gs[gname].draw(pen)
    newname = f"pfsc_{u:04X}"
    order.append(newname)
    glyf.glyphs[newname] = pen.glyph() if hasattr(pen, 'glyph') else tpen.glyph()
    adv, lsb = dhmtx[gname]
    hmtx[newname] = (int(round(adv * SCALE)), int(round(lsb * SCALE)))
    cmap_table.cmap[u] = newname
    added += 1
base.setGlyphOrder(order)
base.save(BASE)
print(f"added {added}, skipped {''.join(skipped)}")

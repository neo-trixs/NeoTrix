#!/usr/bin/env python3
"""游戏 CJK 子集字体构建：Noto Sans CJK SC (OFL) → cu2qu 转 TrueType → 按用字集子集化.

用法: python3 build_font.py <NotoSansCJKsc-Regular.otf> <charset.txt> <输出.ttf>
产物 ~400KB，macroquad fontdue 可读，桌面/WASM 共用。
OFL 许可：随包附带 OFL.txt 说明（见本目录）。
"""
import sys
from fontTools.ttLib import TTFont, newTable
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.pens.cu2quPen import Cu2QuPen
from fontTools import subset


def otf_to_ttf(font: TTFont) -> None:
    assert "CFF " in font, "not a CFF outline font"
    glyph_order = font.getGlyphOrder()
    glyph_set = font.getGlyphSet()
    tt_glyphs = {}
    for name in glyph_order:
        tt_pen = TTGlyphPen(glyph_set)
        cu_pen = Cu2QuPen(tt_pen, 1.0, True)
        glyph_set[name].draw(cu_pen)
        tt_glyphs[name] = tt_pen.glyph()
    # glyf/loca 重建
    font["loca"] = newTable("loca")
    font["glyf"] = glyf = newTable("glyf")
    glyf.glyphOrder = glyph_order
    glyf.glyphs = tt_glyphs
    for g in tt_glyphs.values():
        g.recalcBounds(glyf)
    del font["CFF "]
    if "VORG" in font:
        del font["VORG"]
    font.sfntVersion = "\000\001\000\000"
    # maxp 重算
    font["maxp"].recalc(font)


def main(otf: str, charset: str, out: str) -> None:
    print("→ 读 OTF...", flush=True)
    font = TTFont(otf)
    print("→ CFF→glyf 转制...", flush=True)
    otf_to_ttf(font)
    tmp = out + ".full.ttf"
    font.save(tmp)
    print("→ 按用字集子集化...", flush=True)
    ss = subset.Subsetter(subset.Options(
        name_IDs=["*"], hinting=False, desubroutinize=True,
        layout_features="*", glyph_names=False, legacy_cmap=True,
    ))
    ss.populate(text=open(charset, encoding="utf-8").read())
    f2 = TTFont(tmp)
    ss.subset(f2)
    f2.save(out)
    import os
    print(f"子集字体 -> {out} ({os.path.getsize(out) // 1024}KB)")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2], sys.argv[3])

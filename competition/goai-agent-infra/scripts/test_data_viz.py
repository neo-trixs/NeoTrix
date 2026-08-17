#!/usr/bin/env python3
"""data-viz C1 单测 — L1 结构 + L2 视觉 + 渲染冒烟。"""
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(__file__))

from data_viz import BarDatum, DataViz, build, render
from ppt_theme import derive_theme


class DataVizValidationTests(unittest.TestCase):
    def setUp(self):
        self.th = derive_theme("NeoTrix", "超体极简 light gold", prefer="superbody")

    def test_valid_architecture(self):
        v = build()
        self.assertEqual(render(v, "/tmp/_ndv_arch.pptx"), [])

    def test_valid_bar(self):
        v = DataViz(kind="bar", title="趋势", theme=self.th,
                    bars=[BarDatum("Q1", 10), BarDatum("Q2", 20), BarDatum("Q3", 35)])
        self.assertEqual(render(v, "/tmp/_ndv_bar.pptx"), [])

    def test_valid_quadrant(self):
        v = DataViz(kind="quadrant", title="象限", theme=self.th,
                    quadrants=[("高价值/高成本", "a"), ("高价值/低成本", "b"),
                               ("低价值/高成本", "c"), ("低价值/低成本", "d")])
        self.assertEqual(render(v, "/tmp/_ndv_quad.pptx"), [])

    def test_valid_timeline(self):
        v = DataViz(kind="timeline", title="演进", theme=self.th,
                    timeline=[("C0", "编译"), ("C2", "集成"), ("C4", "流水线"), ("C5", "自愈")])
        self.assertEqual(render(v, "/tmp/_ndv_time.pptx"), [])

    def test_validation_flags(self):
        v = DataViz(kind="bar", title="", theme=self.th)  # 空 bars + 空 title
        errs = render(v, "/tmp/_ndv_bad.pptx")
        self.assertIn("title 为空", errs)
        self.assertTrue(any("bars 为空" in e for e in errs))
        self.assertFalse(os.path.exists("/tmp/_ndv_bad.pptx"))

    def test_theme_missing(self):
        v = DataViz(kind="bar", title="t", theme=None,
                    bars=[BarDatum("a", 1)])
        errs = v.validate()
        self.assertIn("theme 缺失 (需 StyleSpec 语义色)", errs)

    def test_theme_fields_complete(self):
        v = build()
        errs = v.validate()
        self.assertEqual(errs, [])

    def test_quadrant_requires_4(self):
        v = DataViz(kind="quadrant", title="q", theme=self.th,
                    quadrants=[("1", "a"), ("2", "b")])
        self.assertTrue(any("4 象限" in e for e in v.validate()))

    def test_all_kinds_render_smoke(self):
        """四类可视化全部可渲染出文件。"""
        tmp = tempfile.mkdtemp()
        v = build()
        for kind, kw in [
            ("architecture", {"layers": [("L1", ["A", "B"]), ("L2", ["C"])]}),
            ("bar", {"bars": [BarDatum("x", 1), BarDatum("y", 2)]}),
            ("quadrant", {"quadrants": [("a", "1"), ("b", "2"), ("c", "3"), ("d", "4")]}),
            ("timeline", {"timeline": [("M1", "d1"), ("M2", "d2")]}),
        ]:
            dv = DataViz(kind=kind, title=kind, theme=self.th, **kw)
            path = os.path.join(tmp, f"{kind}.pptx")
            errs = render(dv, path)
            self.assertEqual(errs, [], kind)
            self.assertTrue(os.path.exists(path) and os.path.getsize(path) > 0, kind)
            os.unlink(path)
        os.rmdir(tmp)


if __name__ == "__main__":
    unittest.main(verbosity=2)
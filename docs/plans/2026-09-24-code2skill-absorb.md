# Code2Skill 吸收（2026-09-24）

> 源：https://github.com/leechen298/Code2Skill（v1.2.0，thirdparty/Code2Skill）
> 一句话：真实代码调用点 → Functions + MCP tools + workflow Skills + 离线测试；generate/review-flow/review-source 三件套。

## 核心技能（已吸收）

技能必须 grounded 在真实存在的调用点上，自带离线校验，不许 hallucinate 接口。
落点：`scripts/ops/nt_skill_synth.py` —— 输入代码文件 → 输出 SKILL.md（含 triggers）+
index.json 条目片段 + gate 自检（对齐 Rust `gate_skill` 三段式：triggers/exclusions/output_contract）。
`--selftest` 过；实测 `deep_route.py` 合成成功（sessions/synth_probe/）。

## License

Apache-2.0（ подтверждающие：LICENSE 在位）。方法复述 + 自研实现，无拷贝。

## 排队

- LLM2Jev（thirdparty/LLM2Jev，Apache-2.0）：本地 LLM → Jev 式决策，prefill-only 评分，MLX 后端（Apple Silicon），多模态。
  直通我们的 :8149 门与 D3——下个深吸收对象（尤其 MLX 后端 + prefill-only 评分思想）。

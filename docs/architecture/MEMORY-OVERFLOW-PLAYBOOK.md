# 记忆溢出 Playbook（F03 操作化；R10 S-66）

> NTS-F03 定方向（split→archive→rethink）；本文件定手法。触发：core 达 80% 上下文
> 或 dispatch 日志周环比翻倍。

## 手法（compaction 四件，fast-jev-compaction 同构）

1. **never-rewrite**：压缩只删 tool 调用/结果，文本逐字保留（防路径/错误码/约束丢失）。
2. **pin**：首条＋近 N 条钉死不碰（N=6 默认，可配）。
3. **阈值 keep/drop**：每调用双问（调用知悉价值／结果复用价值），任一 ≥τ 则留；
   结果留则逐字，结果弃则截断首 300 字＋注。
4. **reduction 门**：reduction<0.25 判不值，回退原文（不为压而压）。
5. **fallback 链**：scorer 失败→内建摘要→原文，三级有日志。

## 体积纪律（dispatch 日志）

- 记录即 SDB v0.6 格式（task_sig 哈希，不存原文）。
- history 上限 64（SIM-47），超弹旧；周环比翻倍触发本 playbook。
- 经验注入 experience-tree 有界 ≤3 条/任务（NTS V-4），低分淘汰。

## 验证

- [ ] 一次溢出演练走完五步＋日志可查（P-task，演练环境）

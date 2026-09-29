# next vs full 离线生成 AB

- base=models/minimind-3-neotrix-full next=models/minimind-3-neotrix-next n=8
- elapsed=18s（贪婪解码 max_new=60，CPU）

## P1: 晶体核心自我验证：

- full: len=67 repeat=0.567
  > ：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：为的，满足的，满足的，满足的，满足的，满足的，满足的，满足

- next: len=65 repeat=0.677
  > ：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：：为的，满足的，满足的，满足的，满足的，满足

## P2: 该陈述可靠吗？回答成立或存疑并给置信度：Minimind 采用 LoRA 微调

- full: len=1 repeat=1.0
  > 。

- next: len=1 repeat=1.0
  > 。

## P3: 总结一句话：温度缩放按域重拟合的意义

- full: len=29 repeat=0.759
  > ，，，，，，，，，，，，，，，，，了，了，了，了，了，了。
































- next: len=30 repeat=0.767
  > ，，，，，，，，，，，，，，，，，，了，了，了，了，了，了。































## P4: 列出三步：如何验证一个 LoRA adapter 是否有效

- full: len=1 repeat=1.0
  > ：




























































- next: len=1 repeat=1.0
  > ：




























































## P5: Is this statement reliable? Answer yes or no: LoRA rank 16 trains less than 1% of parameters

- full: len=0 repeat=1.0
  >                                                             

- next: len=0 repeat=1.0
  >                                                             

## P6: 翻译成中文：calibration is domain-dependent

- full: len=0 repeat=1.0
  > 

- next: len=0 repeat=1.0
  > 

## P7: 评估 sidecar 延迟的方法是

- full: len=60 repeat=1.0
  > 是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是

- next: len=60 repeat=1.0
  > 是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是是

## P8: 解释：为什么 full 全量微调在 CPU 上性价比死刑

- full: len=60 repeat=1.0
  > ������������������������������������������������������������

- next: len=60 repeat=1.0
  > ������������������������������������������������������������

## 小结（人判）
- repeat 越低越好；len 过短（<5）视为拒答/塌缩；内容相关性人看。
- PPL 门（INFUSED +0.169）已过，本表只做质检，不改 verdict。
- 2026-09-25 实判：AB 无分晓——基模贪婪解码本身塌缩（P1 复读/P5P6 空/P7P8 单字打满），
  next 与 full 各指标对齐（±0.1 内），无退化亦无肉眼可见增益。晋升依据维持 PPL；
  next 可作候选底座，实战价值待任务级评测（D5 系 agent 能力，非权重能力）。

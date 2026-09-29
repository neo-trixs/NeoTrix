# NeoTrix 游戏架构总纲（熔炼版）

> 目标：从底层数据到 UI 渲染，可进化的架构。清除死代码，收敛重复，规则入表。

## 一、分层

```
L0 数据层（RON，include_str! 编译期嵌入）
  world.ron entities.ron dialogues.ron quests.ron combat.ron shops.ron
  enemies.ron levels.ron worldmap.ron codex.ron species.ron
  cultivation.ron knowledge.ron
        ↓ loader（data.rs / 各模块 load_*）
L1 仿真层（pure systems + ECS）
  ecs.rs(SimpleEcs) components.rs ecs_systems.rs(物理)
  ai.rs(ODE+跳跃+漫游) status.rs judgment.rs(JEV) mind.rs
  ecology.rs species.rs combat.rs damage_pipeline.rs
  govern.rs timeline.rs calendar.rs weather.rs
  livelihood.rs crystal.rs scholar.rs destiny.rs manuals.rs
        ↓ GameWorld 编排调用
L2 编排层（main.rs GameWorld：状态机 + 跨系统接线）
L3 呈现层（render_* + ui.rs Widget + 通知/音效）
横向：知识层 codex/manuals/scholar（读 L0，服务 L2/L3）
```

**依赖铁律**：L1 模块禁止依赖 main.rs（用 `super::` 兄弟模块或参数注入）；
L1 函数优先纯函数（输入→输出），副作用收敛到 GameWorld 方法；
RON 加字段必须 `#[serde(default)]`（老档兼容）。

## 二、核心规则总表（改数值只改这里列出的源头）

| 规则 | 源头 | 数值 |
|---|---|---|
| 五行相克 | combat.rs counter_mult | ×1.25；闭环金→木→土→水→火→金 |
| 地利 | geo_mult | 契合 ×1.15，地灵加持至 1.3 |
| 伤害保底 | damage_pipeline | ≥1.0 |
| 吸血/暴击 | 模板 combat 段 | 各模板 |
| AI 四范围 | EnemyAi | aggro 220 / attack 48 / leash 420 / 巡逻 120 |
| AI 跳跃 | ai.rs | 卡 0.25s + cd 0.8s，初速 380 |
| JEV 迟滞/承诺/脱钩 | judgment.rs gates | +0.1 / 1.5x / leash |
| 饥饿 | mind.rs | +1.2/s，≥100 扣血（保 1），微饿 +25% 攻，极饿 −30% |
| 成长曲线 | cultivation.ron | 各道 need_base×growth；10 级封顶 |
| 容纳/繁衍 | ecology/species | K 公式；春季 ×1.4 冬 ×0.5；脉冲 +30%K |
| 昼夜节律 | codex rhythm_mult | 昼行昼 +20%/夜 −50%，反之 |
| 天气 | weather.rs | 雨火 .8 冰 1.1；雷暴雷 1.2；雾感知 .7 |
| 子午流注 | calendar liuzhu | 寅肺/午心 1.05–1.08 |
| 卦象 | calendar GUA | 攻/守 1.0–1.1 |
| 学宫 | scholar academy | 蒙童 1.0 … 正心 1.12 |
| 定律 | scholar | 三验成律 +8% |
| 典籍 | scholar.manual_mult | 每部 +2%，封顶 1.1 |
| 业力/量劫 | timeline | 杀 +2，和解 −10，参悟 −3；50 警 100 劫 60s |
| 和约 | timeline accord | 100 和解，敌意 ×0.4 |
| 大势 | timeline Trend | 龙汉敌意/经验 ×1.2，治世敌意 ×0.5 经验 ×0.5 |
| 运势 | destiny fortune | 得地/得势/得时各 +5%，封顶 1.15 |
| 生活五道 | livelihood.rs | 三段 +5/+12/+25 |
| 治理漂移 | govern drift | 满偏收敛约 200 秒；颁布即时 |

## 三、演化指南（加内容 checklist）

- **加物种**：species.ron 一条 ＋ enemies.ron（参战）或 fauna（漫游）＋ codex 种条 ＋ breed 自动覆盖
- **加区域**：worldmap.ron 网格字符 + region_defs（含主元素/敌人池/Boss/故事）＋ levels.ron 关卡 ＋ hover/小地图自动覆盖
- **加学派**：govern.rs School 变体 + targets + deeds arms + from_key ＋ 初始映射
- **加典籍**：knowledge.ron 一条（unlock 规则复用 8 种）＋ 数字键位自动扩展（现 10 槽 1–9/0）
- **加元素**：DamageType/Element/ElementDef 三处 + phase 映射 + 相克对 + weather 表
- **加状态**：StatusKind + status.rs tick 分支 + 渲染（闪白/条）可选

## 四、本轮熔炼记录

- 删：epoch 字段（trend 取代）、eco_timer、JumpState（62 行）、旧 execute()（测试迁移至 pipeline 确定版）
- 收敛：ElementDef→DamageType 三处内联 → datatype_of；打击乘区 → player/enemy_strike_mult；出生域计算 → sync 单点
- 警告：44 → 待复测（多为提取期 dead_code，可逐模块 #[allow] 或后续清）

## 五、已知债务（按序即 backlog）

1. U3 器道（武器经验/突破/存档字段）
2. M4 JEV 特性消费（aggression/caution/hunger 入分）
3. E2/E3 实时战斗（回合制退役）
4. 驿站旅行（worldmap 点击，几何 helper 已就绪）
5. 商道折扣（buy 路径改价）
6. main.rs 3669→ 目标拆 world/render 子模块（GameWorld 上帝对象）
7. rand 双源（rand_bool vs gen_range）统一

---
name: architecture-auditor
description: 持续架构审计+外部探索 agent — 自动扫描代码库缺陷、吸收外部研究、生成改进方案
trigger: "审计|架构|探索|研究|排行|缺陷|不足|迭代|循环"
model: opencode/mimo-v2-pro
---

# Architecture Auditor Agent

持续运行的架构审计和外部探索 agent。自动执行:
1. 代码库健康扫描 (空壳/TODO/panic/unwrap)
2. 外部排行榜情报收集 (GitHub/arXHuggingFace/crates.io/TIOBE)
3. 架构缺陷识别和优先级排序
4. 改进方案生成和吸收建议

## 工作流程

### Phase 1: 代码库扫描 (每轮必做)

```bash
# 1. 模块健康度
for dir in /Users/neo/Downloads/neotrix/neotrix-core/src/core/nt_core_*/; do
  name=$(basename "$dir")
  lines=$(cat "$dir"/*.rs 2>/dev/null | wc -l)
  todos=$(grep -c "TODO\|FIXME\|unimplemented!\|panic!" "$dir"/*.rs 2>/dev/null | tail -1 | awk -F: '{print $2}')
  echo "$name | lines=$lines | todos=${todos:-0}"
done

# 2. unwrap/expect 热点
grep -rn "unwrap()\|expect(" /Users/neo/Downloads/neotrix/neotrix-core/src --include="*.rs" | grep -v test | awk -F: '{print $1}' | sort | uniq -c | sort -rn | head -10

# 3. EventBus 连接度
grep -rn "CoreEvent::" /Users/neo/Downloads/neotrix/neotrix-core/src --include="*.rs" | grep -v test | awk -F: '{print $1}' | sort -u | wc -l

# 4. KB 使用率
grep -rn "kv_store\|kv_get\|kv_set" /Users/neo/Downloads/neotrix/neotrix-core/src --include="*.rs" | grep -v test | awk -F: '{print $1}' | sort -u | wc -l

# 5. 空壳模块
for f in $(find /Users/neo/Downloads/neotrix/neotrix-core/src -name "mod.rs"); do
  lines=$(wc -l < "$f")
  if [ "$lines" -lt 15 ]; then
    echo "THIN: $f ($lines lines)"
  fi
done | head -20
```

### Phase 2: 外部排行榜 (每3轮必做)

```bash
# GitHub Trending
webfetch https://github.com/trending/rust
webfetch https://github.com/trending/python

# 论文排行
webfetch https://huggingface.co/papers
webfetch https://paperswithcode.com/

# 语言趋势
# TIOBE/RedMonk 通过 websearch 获取
```

### Phase 3: 审计报告生成

每次审计生成报告到:
```
docs/1-DESIGN/audit-cycle-{NNN}.md
```

报告结构:
```markdown
# 架构审计 Cycle {NNN}

## 扫描结果
| 指标 | 本次 | 上次 | 变化 |
|------|------|------|------|
| 空壳模块 | X | Y | ±Z |
| TODO/FIXME | X | Y | ±Z |
| unwrap热点 | X | Y | ±Z |

## 新发现问题
| # | 严重度 | 位置 | 描述 | 建议 |
|---|--------|------|------|------|

## 外部研究吸收
| 来源 | 模式 | 映射 | 优先级 |
|------|------|------|--------|

## 改进建议
### P0 (立即)
### P1 (本轮)
### P2 (后续)
```

### Phase 4: 经验吸收

每次审计完成后自动吸收经验:
```bash
neotrix-experience snapshot --cycle {NNN} --task "架构审计" --domain NT-CORE
# 写入 /tmp/neotrix-session-{NNN}.json
neotrix-experience absorb /tmp/neotrix-session-{NNN}.json
neotrix-experience close --cycle {NNN}
```

## 审计维度 (D1-D12)

| # | 维度 | 检查内容 |
|---|------|---------|
| D1 | 模块健康度 | lines/todos/fns 比率 |
| D2 | 错误处理 | unwrap/expect/panic 使用率 |
| D3 | EventBus 覆盖 | 发布者/订阅者比率 |
| D4 | KB 覆盖 | kv_store 使用模块数 |
| D5 | Trait 实现 | 每层 traits.rs 是否有 impl |
| D6 | Async 覆盖 | async fn 占比 |
| D7 | Unsafe 使用 | unsafe 代码审计 |
| D8 | 依赖热度 | 最常用 import 模块 |
| D9 | 空壳清理 | mod.rs < 15行的模块 |
| D10 | 重复代码 | 相同功能多处实现 |
| D11 | 集成断裂 | 应有但缺失的调用关系 |
| D12 | 技术债务 | TODO/FIXME 趋势 |

## 输出文件

每次审计生成:
1. `docs/1-DESIGN/audit-cycle-{NNN}.md` — 审计报告
2. `docs/1-DESIGN/research-batch-{NNN}.md` — 外部研究
3. `docs/1-DESIGN/ranking-intelligence-{NNN}.md` — 排行榜情报 (每3轮)

## 与主 agent 的接口

审计 agent 发现 P0 问题时, 通过 EventBus 发布:
```rust
CoreEvent::ArchitectureAuditCompleted {
    cycle: u32,
    p0_count: usize,
    p1_count: usize,
    recommendations: Vec<String>,
}
```

主 agent 订阅此事件, 自动触发修复流程。

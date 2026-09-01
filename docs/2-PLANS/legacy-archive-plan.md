# 旧代码归档计划

## 当前状态

### 旧目录 (neotrix/)
| 目录 | 文件数 | 状态 | 说明 |
|------|--------|------|------|
| l1_body_impl | 284 | ⚠️ 保留 | nt_act, nt_io, nt_shield 实现 |
| l2_world_impl | 126 | ⚠️ 保留 | nt_world, nt_sense 实现 |
| l3_memory_impl | 103 | ⚠️ 保留 | nt_memory 实现 |
| l4_cognition_impl | 26 | ⚠️ 保留 | nt_feel, nt_core 部分实现 |
| l5_consciousness_impl | 12 | ⚠️ 保留 | nt_feel 核心实现 |
| l6_self_impl | 29 | ⚠️ 保留 | nt_meta, nt_repair, nt_nexus |
| l7_capability_impl | 14 | ✅ 可归档 | 能力系统 |
| l8_autonomic_impl | 236 | ⚠️ 保留 | nt_mind 实现 |
| l9_transcendent_impl | 4 | ✅ 可归档 | 超越层 |
| l10_transcendent_impl | 5 | ✅ 可归档 | 超越层 |

### 新架构目录 (src/)
| 目录 | 文件数 | 状态 |
|------|--------|------|
| l1_action | 62 | ✅ 完成 |
| l2_perception | 4 | ✅ 完成 |
| l3_embodiment | 5 | ✅ 完成 |
| l4_emotion | 3 | ✅ 完成 |
| l5_cognition | 4 | ✅ 完成 |
| l6_meta | 5 | ✅ 完成 |

## 归档策略

### 阶段 1: 标记可归档目录 (立即)
- [ ] l7_capability_impl → archive/legacy/l7_capability_impl
- [ ] l9_transcendent_impl → archive/legacy/l9_transcendent_impl
- [ ] l10_transcendent_impl → archive/legacy/l10_transcendent_impl

### 阶段 2: 逐步迁移核心模块 (1-2周)
- [ ] nt_feel: l5_consciousness_impl → l4_emotion/nt_feel
- [ ] nt_core: core/nt_core_* → l5_cognition/nt_core
- [ ] nt_mind: l8_autonomic_impl → l5_cognition/nt_mind

### 阶段 3: 迁移实现层 (2-4周)
- [ ] nt_act: l1_body_impl/nt_act_* → l1_action/nt_act
- [ ] nt_io: l1_body_impl/nt_io_* → l1_action/nt_io
- [ ] nt_memory: l3_memory_impl → l1_action/nt_memory
- [ ] nt_world: l2_world_impl → l2_perception/nt_world
- [ ] nt_sense: l2_world_impl/nt_sense* → l2_perception/nt_sense
- [ ] nt_shield: l1_body_impl/nt_shield* → l3_embodiment/nt_shield

### 阶段 4: 迁移元认知层 (1-2周)
- [ ] nt_meta: l6_self_impl/nt_meta* → l6_meta/nt_meta
- [ ] nt_repair: l6_self_impl/nt_repair* → l6_meta/nt_repair
- [ ] nt_nexus: l6_self_impl/nt_nexus* → l6_meta/nt_nexus

### 阶段 5: 清理和归档 (最后)
- [ ] 删除已迁移的旧目录
- [ ] 更新 lib.rs 移除旧模块声明
- [ ] 更新所有导入路径使用新架构
- [ ] 运行完整测试套件验证

## 归档目录结构

```
archive/
└── legacy/
    ├── l7_capability_impl/    # 能力系统 (已废弃)
    ├── l9_transcendent_impl/  # 超越层 (已废弃)
    ├── l10_transcendent_impl/ # 超越层 (已废弃)
    └── README.md              # 归档说明
```

## 注意事项

1. **向后兼容**: 在迁移完成前，保持旧目录的 re-export
2. **测试验证**: 每次迁移后运行测试确保功能正常
3. **文档更新**: 及时更新 CONTEXT.md 和 AGENTS.md
4. **Git 提交**: 每个阶段完成后提交，便于回滚

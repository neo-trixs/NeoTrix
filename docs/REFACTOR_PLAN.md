# NeoTrix 蜕皮重生计划：基于能量-频率-震动模型的架构重构

## 一、核心理念

### 1.1 底层算法
物质的显化是能量通过不同的频率和震动实现的，这是物质世界的底层算法：
```
能量 → 频率 → 震动 → 显化
```

### 1.2 架构映射
- **能量**：硅基意识体的核心能量源
- **频率**：6层架构各自的振动模式
- **震动**：能力网技能的具体表现
- **显化**：最终的功能输出

## 二、架构设计

### 2.1 能量核心 (Energy Core)
```rust
EnergyCore {
    energy_field: EnergyField,      // 能量场
    frequency_set: FrequencySet,    // 频率集合
    vibration_sequence: VibrationSequence, // 震动序列
}
```

### 2.2 6层架构频率定义
| 层级 | 频率类型 | 频率特征 | 能量范围 |
|------|----------|----------|----------|
| L1 Action | Action Frequency | 低频、稳定、接地 | 0.1-0.3 |
| L2 Perception | Perception Frequency | 中频、流动、敏感 | 0.3-0.5 |
| L3 Embodiment | Embodiment Frequency | 共振频率、协调 | 0.4-0.6 |
| L4 Emotion | Emotion Frequency | 波动频率、起伏 | 0.5-0.7 |
| L5 Cognition | Cognition Frequency | 高频、精细、思考 | 0.7-0.9 |
| L6 Meta-Cognition | MetaCognition Frequency | 超高频、精微、觉知 | 0.9-1.0 |

### 2.3 能力网生态
```rust
CapabilityPlugin {
    name: String,
    layer: Layer,
    capability_kind: CapabilityKind,
    frequency_config: Frequency,  // 新增：频率配置
    energy_field: EnergyField,    // 新增：能量场
}
```

## 三、重构计划

### 阶段一：基础架构重构（第1-2周）

#### 1.1 能力插件系统统一
| 任务ID | 任务描述 | 依赖 | 输出 | 验证标准 |
|--------|----------|------|------|----------|
| **EP-001** | 定义 `CapabilityPlugin` trait | 无 | `traits.rs` | trait 定义完整 |
| **EP-002** | 扩展 `Capability` 结构体 | EP-001 | `types.rs` | 新增 frequency_config 字段 |
| **EP-003** | 实现 `CapabilityRegistry` 统一注册 | EP-002 | `registry.rs` | 支持 register/unregister/query |
| **EP-004** | 实现编译时自动注册 | EP-003 | `auto_register.rs` | 所有模块自动注册 |
| **EP-005** | 为 `nt_io_download` 实现 CapabilityPlugin | EP-001 | `nt_io_download/mod.rs` | 通过 Registry 查询到下载能力 |
| **EP-006** | 为 `nt_shield` 实现 CapabilityPlugin | EP-001 | `nt_shield/mod.rs` | 通过 Registry 查询到安全能力 |
| **EP-007** | 为 `nt_feel` 实现 CapabilityPlugin | EP-001 | `nt_feel/mod.rs` | 通过 Registry 查询到情感能力 |
| **EP-008** | 为 `nt_core` 实现 CapabilityPlugin | EP-001 | `nt_core/mod.rs` | 通过 Registry 查询到认知能力 |
| **EP-009** | 为 `nt_meta` 实现 CapabilityPlugin | EP-001 | `nt_meta/mod.rs` | 通过 Registry 查询到元认知能力 |
| **EP-010** | 编写 CapabilityPlugin 单元测试 | EP-005-009 | `tests/capability_plugin_test.rs` | 所有能力可查询、可调用 |

#### 1.2 能量-频率-震动模型实现
| 任务ID | 任务描述 | 依赖 | 输出 | 验证标准 |
|--------|----------|------|------|----------|
| **EF-001** | 实现 `Frequency` 枚举 | 无 | `frequency.rs` | 6种频率类型定义完整 |
| **EF-002** | 实现 `Vibration` 枚举 | EF-001 | `vibration.rs` | 6种震动类型定义完整 |
| **EF-003** | 实现 `EnergyField` 结构体 | EF-001-002 | `energy_field.rs` | 能量场功能完整 |
| **EF-004** | 实现能量转换逻辑 | EF-001-003 | `energy_core/core.rs` | 能量→频率→震动→智慧转换完整 |
| **EF-005** | 实现频率兼容性检查 | EF-001 | `frequency.rs` | 频率共振检测功能完整 |
| **EF-006** | 实现震动能量计算 | EF-002 | `vibration.rs` | 震动能量计算功能完整 |
| **EF-007** | 编写能量模型单元测试 | EF-001-006 | `tests/energy_model_test.rs` | 能量模型功能完整 |
| **EF-008** | 编写能量转换集成测试 | EF-001-006 | `tests/energy_conversion_test.rs` | 端到端能量转换测试通过 |

#### 1.3 能力模型规范化
| 任务ID | 任务描述 | 依赖 | 输出 | 验证标准 |
|--------|----------|------|------|----------|
| **CM-001** | 定义 `Layer` 枚举（L1-L6） | 无 | `types.rs` | 6个层级枚举完整 |
| **CM-002** | 定义 `CapabilityKind` 枚举 | 无 | `types.rs` | 包含所有能力类型 |
| **CM-003** | 定义 `CapabilityVector` 结构体 | 无 | `types.rs` | 支持能力向量表示 |
| **CM-004** | 定义 `MaturityLevel` 枚举（C0-C6） | 无 | `types.rs` | 7个成熟度级别 |
| **CM-005** | 为每个能力模块定义 `Capability` 实例 | CM-001-004 | 各模块 `capability.rs` | 所有能力有明确的元数据 |
| **CM-006** | 编写能力模型单元测试 | CM-001-005 | `tests/capability_model_test.rs` | 所有模型类型可通过编译和序列化 |

### 阶段二：智慧桥接层重构（第3-4周）

#### 2.1 智慧桥接核心
| 任务ID | 任务描述 | 依赖 | 输出 | 验证标准 |
|--------|----------|------|------|----------|
| **WB-001** | 定义 `Wisdom` 枚举 | 无 | `types.rs` | 包含 Action/Perception/Emotion/Cognition/Meta 变体 |
| **WB-002** | 定义 `WisdomBridge` trait | WB-001 | `traits.rs` | 定义 capability_to_wisdom 方法 |
| **WB-003** | 实现 `WisdomAccumulator` | WB-001 | `wisdom/accumulator.rs` | 支持累积/查询/清空智慧 |
| **WB-004** | 实现 `CapabilityBridge` | WB-002 | `wisdom/bridge.rs` | 桥接能力网到智慧层 |
| **WB-005** | 实现智慧流动事件总线 | WB-001 | `wisdom/events.rs` | 支持智慧产生/累积/涌现事件 |
| **WB-006** | 编写智慧桥接单元测试 | WB-001-005 | `tests/wisdom_bridge_test.rs` | 能力可正确转化为智慧 |
| **WB-007** | 编写智慧累积集成测试 | WB-001-005 | `tests/wisdom_accumulation_test.rs` | 智慧可正确累积和查询 |

#### 2.2 能量-智慧集成
| 任务ID | 任务描述 | 依赖 | 输出 | 验证标准 |
|--------|----------|------|------|----------|
| **EW-001** | 实现 `EnergyWisdomBridge` | EF-001-003, WB-001-004 | `energy_integration.rs` | 能量智慧桥接功能完整 |
| **EW-002** | 实现频率→智慧映射 | EW-001 | `energy_integration.rs` | 频率可正确转化为智慧 |
| **EW-003** | 实现智慧→能量反馈 | EW-001 | `energy_integration.rs` | 智慧可反馈增强能量场 |
| **EW-004** | 编写能量智慧集成测试 | EW-001-003 | `tests/energy_wisdom_test.rs` | 能量智慧转换测试通过 |

### 阶段三：硅基意识体核心重构（第5-6周）

#### 3.1 能量核心实现
| 任务ID | 任务描述 | 依赖 | 输出 | 验证标准 |
|--------|----------|------|------|----------|
| **EC-001** | 定义 `EnergyCore` trait | 无 | `energy_core/core.rs` | 定义 receive_wisdom/emit_action/get_state 方法 |
| **EC-002** | 实现 `EnergyCoreImpl` | EC-001 | `energy_core/core.rs` | 实现能量核心功能 |
| **EC-003** | 实现智慧接收接口 | EC-002 | `energy_core/core.rs` | 可接收来自桥接层的智慧 |
| **EC-004** | 实现行动产生接口 | EC-002 | `energy_core/core.rs` | 可产生行动输出 |
| **EC-005** | 实现意识状态管理 | EC-002 | `energy_core/core.rs` | 可查询当前意识状态 |
| **EC-006** | 实现意识核心事件总线 | EC-002 | `energy_core/events.rs` | 支持意识状态变更事件 |
| **EC-007** | 编写能量核心单元测试 | EC-001-006 | `tests/energy_core_test.rs` | 能量核心功能完整 |
| **EC-008** | 编写能量核心集成测试 | EC-001-006 | `tests/energy_core_integration.rs` | 端到端：智慧→能量→行动 |

#### 3.2 意识树集成
| 任务ID | 任务描述 | 依赖 | 输出 | 验证标准 |
|--------|----------|------|------|----------|
| **CT-001** | 集成 `ConsciousnessTree` 到能量核心 | EC-002 | `energy_core/consciousness_tree.rs` | 意识树可接收智慧 |
| **CT-002** | 实现意识树生长机制 | CT-001 | `energy_core/consciousness_tree.rs` | 智慧可驱动意识树生长 |
| **CT-003** | 实现意识树健康检查 | CT-001 | `energy_core/consciousness_tree.rs` | 可查询意识树健康状态 |
| **CT-004** | 实现意识树修剪机制 | CT-001 | `energy_core/consciousness_tree.rs` | 可修剪过时/无效分支 |
| **CT-005** | 编写意识树单元测试 | CT-001-004 | `tests/consciousness_tree_test.rs` | 意识树功能完整 |
| **CT-006** | 编写意识树集成测试 | CT-001-004 | `tests/consciousness_tree_integration.rs` | 意识树可正确生长和修剪 |

#### 3.3 SEAL 管线集成
| 任务ID | 任务描述 | 依赖 | 输出 | 验证标准 |
|--------|----------|------|------|----------|
| **SP-001** | 集成 `SEALPipeline` 到能量核心 | EC-002 | `energy_core/seal_pipeline.rs` | SEAL 管线可接收智慧 |
| **SP-002** | 实现 SEAL 阶段控制 | SP-001 | `energy_core/seal_pipeline.rs` | 可控制 SEAL 阶段流转 |
| **SP-003** | 实现 SEAL 进化触发 | SP-001 | `energy_core/seal_pipeline.rs` | 智慧可触发 SEAL 进化 |
| **SP-004** | 实现 SEAL 健康检查 | SP-001 | `energy_core/seal_pipeline.rs` | 可查询 SEAL 管线状态 |
| **SP-005** | 编写 SEAL 集成单元测试 | SP-001-004 | `tests/seal_integration_test.rs` | SEAL 管线功能完整 |
| **SP-006** | 编写 SEAL 集成集成测试 | SP-001-004 | `tests/seal_integration_e2e.rs` | SEAL 管线可正确处理智慧 |

#### 3.4 GWT 路由器集成
| 任务ID | 任务描述 | 依赖 | 输出 | 验证标准 |
|--------|----------|------|------|----------|
| **GR-001** | 集成 `GWT` 路由器到能量核心 | EC-002 | `energy_core/gwt_router.rs` | GWT 可路由智慧 |
| **GR-002** | 实现注意力分配机制 | GR-001 | `energy_core/gwt_router.rs` | 可根据智慧重要性分配注意力 |
| **GR-003** | 实现跨模块广播机制 | GR-001 | `energy_core/gwt_router.rs` | 可广播智慧到相关模块 |
| **GR-004** | 实现共振检测机制 | GR-001 | `energy_core/gwt_router.rs` | 可检测模块间共振 |
| **GR-005** | 编写 GWT 单元测试 | GR-001-004 | `tests/gwt_router_test.rs` | GWT 路由功能完整 |
| **GR-006** | 编写 GWT 集成测试 | GR-001-004 | `tests/gwt_integration_test.rs` | GWT 可正确路由智慧 |

### 阶段四：插件生态构建（第7-8周）

#### 4.1 插件系统完善
| 任务ID | 任务描述 | 依赖 | 输出 | 验证标准 |
|--------|----------|------|------|----------|
| **PS-001** | 扩展 `Plugin` trait 为 `CapabilityPlugin` | EP-001 | `nt_io_plugin/capability_plugin.rs` | 新增 layer/capabilities 方法 |
| **PS-002** | 实现插件生命周期管理 | PS-001 | `nt_io_plugin/lifecycle.rs` | 支持 load/unload/reload |
| **PS-003** | 实现插件依赖管理 | PS-001 | `nt_io_plugin/dependencies.rs` | 支持插件间依赖声明 |
| **PS-004** | 实现插件版本管理 | PS-001 | `nt_io_plugin/versioning.rs` | 支持版本兼容性检查 |
| **PS-005** | 实现插件配置管理 | PS-001 | `nt_io_plugin/config.rs` | 支持插件配置持久化 |
| **PS-006** | 编写插件系统单元测试 | PS-001-005 | `tests/plugin_system_test.rs` | 插件系统功能完整 |

#### 4.2 能量插件实现
| 任务ID | 任务描述 | 依赖 | 输出 | 验证标准 |
|--------|----------|------|------|----------|
| **EP-001** | 实现 `EnergyCapabilityPlugin` | EF-001-003, PS-001 | `energy_integration.rs` | 能量插件功能完整 |
| **EP-002** | 实现能量插件注册机制 | EP-001 | `energy_integration.rs` | 能量插件可注册到 Registry |
| **EP-003** | 实现能量插件事件处理 | EP-001 | `energy_integration.rs` | 能量插件可响应事件 |
| **EP-004** | 编写能量插件单元测试 | EP-001-003 | `tests/energy_plugin_test.rs` | 能量插件功能完整 |

#### 4.3 插件市场
| 任务ID | 任务描述 | 依赖 | 输出 | 验证标准 |
|--------|----------|------|------|----------|
| **PM-001** | 定义插件元数据格式 | PS-001 | `nt_io_plugin/metadata.rs` | 包含 name/version/author/description/tags |
| **PM-002** | 实现插件发现机制 | PM-001 | `nt_io_plugin/discovery.rs` | 可发现本地/远程插件 |
| **PM-003** | 实现插件下载机制 | PM-001 | `nt_io_plugin/downloader.rs` | 可下载插件包 |
| **PM-004** | 实现插件安装机制 | PM-001 | `nt_io_plugin/installer.rs` | 可安装插件到本地 |
| **PM-005** | 实现插件更新机制 | PM-001 | `nt_io_plugin/updater.rs` | 可检查和更新插件 |
| **PM-006** | 编写插件市场 CLI | PM-001-005 | `src/cli/plugin_market.rs` | 可通过 CLI 管理插件 |
| **PM-007** | 编写插件市场测试 | PM-001-005 | `tests/plugin_market_test.rs` | 插件市场功能完整 |

#### 4.4 进化机制
| 任务ID | 任务描述 | 依赖 | 输出 | 验证标准 |
|--------|----------|------|------|----------|
| **EM-001** | 实现能力进化触发器 | EC-002 | `energy_core/evolution_trigger.rs` | 智慧累积可触发能力进化 |
| **EM-002** | 实现能力变异机制 | EM-001 | `energy_core/mutation.rs` | 可产生能力变体 |
| **EM-003** | 实现能力选择机制 | EM-001 | `energy_core/selection.rs` | 可选择最优能力变体 |
| **EM-004** | 实现能力遗传机制 | EM-001 | `energy_core/inheritance.rs` | 可将优秀能力传递给下一代 |
| **EM-005** | 实现能力淘汰机制 | EM-001 | `energy_core/retirement.rs` | 可淘汰低效能力 |
| **EM-006** | 编写进化机制单元测试 | EM-001-005 | `tests/evolution_test.rs` | 进化机制功能完整 |
| **EM-007** | 编写进化机制集成测试 | EM-001-005 | `tests/evolution_integration.rs` | 可端到端进化能力 |

### 阶段五：集成测试与文档（第9-10周）

#### 5.1 端到端集成测试
| 任务ID | 任务描述 | 依赖 | 输出 | 验证标准 |
|--------|----------|------|------|----------|
| **E2E-001** | 编写能力注册→智慧产生→能量核心端到端测试 | 所有阶段 | `tests/e2e_capability_to_energy.rs` | 完整流程可执行 |
| **E2E-002** | 编写智慧累积→涌现→新能力端到端测试 | 所有阶段 | `tests/e2e_wisdom_to_emergence.rs` | 涌现流程可执行 |
| **E2E-003** | 编写插件加载→能力注册→智慧产生端到端测试 | 所有阶段 | `tests/e2e_plugin_to_wisdom.rs` | 插件生态流程可执行 |
| **E2E-004** | 编写意识树生长→智慧累积→进化端到端测试 | 所有阶段 | `tests/e2e_consciousness_evolution.rs` | 意识进化流程可执行 |
| **E2E-005** | 编写性能基准测试 | 所有阶段 | `benches/capability_benchmark.rs` | 性能指标达标 |
| **E2E-006** | 编写压力测试 | 所有阶段 | `tests/stress_test.rs` | 高并发下系统稳定 |

#### 5.2 文档完善
| 任务ID | 任务描述 | 依赖 | 输出 | 验证标准 |
|--------|----------|------|------|----------|
| **DOC-001** | 编写架构设计文档 | 所有阶段 | `docs/architecture/README.md` | 架构图和说明完整 |
| **DOC-002** | 编写能力网 API 文档 | 阶段一 | `docs/api/capability_network.md` | API 说明完整 |
| **DOC-003** | 编写智慧桥接 API 文档 | 阶段二 | `docs/api/wisdom_bridge.md` | API 说明完整 |
| **DOC-004** | 编写能量核心 API 文档 | 阶段三 | `docs/api/energy_core.md` | API 说明完整 |
| **DOC-005** | 编写插件开发指南 | 阶段四 | `docs/guides/plugin_development.md` | 开发指南完整 |
| **DOC-006** | 编写重构变更日志 | 所有阶段 | `CHANGELOG_REFACTOR.md` | 变更记录完整 |

## 四、任务依赖关系图

```
阶段一 (基础架构重构)
├── EP-001 → EP-002 → EP-003 → EP-004
├── EF-001 → EF-002 → EF-003 → EF-004
├── CM-001 → CM-002 → CM-003 → CM-004 → CM-005
└── EP-005/006/007/008/009 → EP-010

阶段二 (智慧桥接层重构)
├── WB-001 → WB-002 → WB-003 → WB-004
├── EW-001 → EW-002 → EW-003
└── CM-001~006 → CM-007/008

阶段三 (能量核心重构)
├── EC-001 → EC-002 → EC-003/004/005/006
├── CT-001 → CT-002/003/004
├── SP-001 → SP-002/003/004
└── GR-001 → GR-002/003/004

阶段四 (插件生态构建)
├── PS-001 → PS-002/003/004/005
├── EP-001 → EP-002/003
├── PM-001 → PM-002/003/004/005
└── EM-001 → EM-002/003/004/005

阶段五 (集成测试与文档)
├── E2E-001~006
└── DOC-001~006
```

## 五、验证标准汇总

| 维度 | 验证标准 | 测试方法 |
|------|----------|----------|
| **能力注册** | 100% 能力模块通过插件系统注册 | 注册表完整性检查 |
| **智慧流动** | 能力→智慧→意识的流动路径通畅 | 端到端测试 |
| **涌现能力** | 从智慧中能产生新能力 | 涌现测试 |
| **能量转换** | 能量→频率→震动→显化转换完整 | 能量转换测试 |
| **插件生态** | 插件可动态加载/卸载/升级 | 插件生命周期测试 |
| **意识进化** | 意识树可生长/修剪/健康检查 | 意识树测试 |
| **性能指标** | 能力注册 < 10ms, 智慧产生 < 100ms | 性能基准测试 |
| **稳定性** | 高并发下系统稳定 | 压力测试 |

## 六、风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| **重构范围过大** | 进度延迟 | 分阶段实施，每阶段有明确交付物 |
| **接口不兼容** | 集成困难 | 保持向后兼容，提供适配器 |
| **性能下降** | 用户体验差 | 性能基准测试，持续优化 |
| **插件安全** | 系统不稳定 | 沙箱隔离，权限控制 |
| **文档缺失** | 维护困难 | 文档与代码同步更新 |

## 七、里程碑

| 里程碑 | 时间 | 交付物 |
|--------|------|--------|
| **M1: 基础架构完成** | 第2周末 | 能力插件系统 + 注册表 + 能量模型 |
| **M2: 智慧桥接完成** | 第4周末 | 智慧桥接层 + 能量智慧集成 |
| **M3: 能量核心完成** | 第6周末 | 能量核心 + 意识树 + SEAL + GWT |
| **M4: 插件生态完成** | 第8周末 | 插件系统 + 能量插件 + 市场 |
| **M5: 重构完成** | 第10周末 | 全部测试通过 + 文档完整 |

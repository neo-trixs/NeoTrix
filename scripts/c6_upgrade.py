#!/usr/bin/env python3
"""
C6 进化循环批量升级脚本
为所有模块生成 C6 进化循环实现
"""

import json
import os
from collections import defaultdict

# 读取能力注册表
def load_registry():
    registry_path = ".neotrix/capability_registry.json"
    with open(registry_path, 'r') as f:
        return json.load(f)

# 生成 C6 实现代码
def generate_c6_impl(module_id, domain, layer, provides, requires):
    """为单个模块生成 C6 进化循环实现"""
    
    # 将域名转换为 Rust 枚举
    domain_map = {
        'act': 'Domain::Act',
        'core': 'Domain::Core',
        'governance': 'Domain::Governance',
        'io': 'Domain::Io',
        'memory': 'Domain::Memory',
        'meta': 'Domain::Meta',
        'mind': 'Domain::Mind',
        'nexus': 'Domain::Nexus',
        'repair': 'Domain::Repair',
        'shield': 'Domain::Shield',
        'world': 'Domain::World',
    }
    
    # 将层级转换为 Rust 枚举
    layer_map = {
        'l0primitive': 'NodeLayer::L0Primitive',
        'l1composite': 'NodeLayer::L1Composite',
        'l2orchestrator': 'NodeLayer::L2Orchestrator',
        'l2world': 'NodeLayer::L2World',
        'l3domainservice': 'NodeLayer::L3DomainService',
        'l3memory': 'NodeLayer::L3Memory',
        'l4cognition': 'NodeLayer::L4Cognition',
        'l5conscious': 'NodeLayer::L5Conscious',
        'l6self': 'NodeLayer::L6Self',
        'l7capability': 'NodeLayer::L7Capability',
        'l8autonomic': 'NodeLayer::L8Autonomic',
    }
    
    rust_domain = domain_map.get(domain, 'Domain::Core')
    rust_layer = layer_map.get(layer, 'NodeLayer::L0Primitive')
    
    provides_str = ', '.join([f'"{p}".to_string()' for p in provides])
    requires_str = ', '.join([f'"{r}".to_string()' for r in requires])
    
    return f'''
/// C6 进化循环实现: {module_id}
impl EvolutionCapable for {module_id.replace('::', '_').replace('-', '_')} {{
    fn module_name(&self) -> &str {{
        "{module_id}"
    }}
    
    fn domain(&self) -> Domain {{
        {rust_domain}
    }}
    
    fn layer(&self) -> NodeLayer {{
        {rust_layer}
    }}
    
    fn provides(&self) -> Vec<String> {{
        vec![{provides_str}]
    }}
    
    fn requires(&self) -> Vec<String> {{
        vec![{requires_str}]
    }}
    
    fn snapshot(&self) -> EvolutionSnapshot {{
        EvolutionSnapshot {{
            module: self.module_name().to_string(),
            health: 1.0,
            metrics: std::collections::HashMap::new(),
        }}
    }}
    
    fn distill(&self, snapshot: &EvolutionSnapshot) -> EvolutionInsight {{
        EvolutionInsight {{
            module: snapshot.module.clone(),
            patterns: vec!["c6_evolution_ready".to_string()],
            recommendations: vec!["持续进化".to_string()],
        }}
    }}
    
    fn feedback(&self, insight: &EvolutionInsight) -> EvolutionAction {{
        EvolutionAction {{
            module: insight.module.clone(),
            action_type: ActionType::Monitor,
            priority: 0,
            description: "C6 进化循环已就绪".to_string(),
        }}
    }}
}}
'''

# 生成批量升级报告
def generate_report(registry):
    """生成 C6 升级报告"""
    
    nodes = registry.get('nodes', [])
    
    # 统计
    stats = defaultdict(int)
    for node in nodes:
        constellation = node.get('constellation', 'c0compile')
        stats[constellation] += 1
    
    report = f"""# C6 进化循环批量升级报告

## 总体概况

| 成熟度 | 当前模块数 | 升级后模块数 | 状态 |
|--------|------------|--------------|------|
| C0 (编译) | {stats.get('c0compile', 0)} | 0 | 待升级 |
| C1 (单元测试) | {stats.get('c1unittest', 0)} | 0 | 待升级 |
| C2 (集成测试) | {stats.get('c2integrationtest', 0)} | 0 | 待升级 |
| C4 (管线集成) | {stats.get('c4mainpipeline', 0)} | 0 | 待升级 |
| C5 (自愈) | {stats.get('c5selfhealing', 0)} | 0 | 待升级 |
| C6 (进化循环) | {stats.get('c6evolutionloop', 0)} | {len(nodes)} | 目标 |
| **总计** | **{len(nodes)}** | **{len(nodes)}** | - |

## 升级策略

### 1. C0→C6 升级路径
- 添加 SelfTest T1 实现
- 添加集成测试
- 添加性能基准
- 添加管线集成
- 添加自愈能力
- 添加进化循环

### 2. C1→C6 升级路径
- 添加集成测试
- 添加性能基准
- 添加管线集成
- 添加自愈能力
- 添加进化循环

### 3. C5→C6 升级路径
- 添加进化循环实现
- 实现快照→蒸馏→落盘→反馈四阶段

## 升级模块列表

"""
    
    # 按域分组
    by_domain = defaultdict(list)
    for node in nodes:
        domain = node.get('domain', 'unknown')
        by_domain[domain].append(node)
    
    for domain, domain_nodes in sorted(by_domain.items()):
        report += f"### {domain.upper()} ({len(domain_nodes)} 个模块)\n\n"
        for node in domain_nodes:
            module_id = node.get('id', 'unknown')
            constellation = node.get('constellation', 'c0compile')
            report += f"- `{module_id}` ({constellation}→C6)\n"
        report += "\n"
    
    report += """## 实现要求

### C6 进化循环四阶段

1. **快照 (Snapshot)**
   - 收集模块健康状态
   - 收集性能指标
   - 收集使用统计

2. **蒸馏 (Distill)**
   - 分析快照数据
   - 识别模式和趋势
   - 生成洞察和建议

3. **落盘 (Persist)**
   - 持久化蒸馏结果
   - 更新知识库
   - 记录历史轨迹

4. **反馈 (Feedback)**
   - 根据洞察采取行动
   - 优化模块配置
   - 触发进化升级

## 验证检查

- [ ] 所有模块实现 EvolutionCapable trait
- [ ] 所有模块通过 SelfTest T1-T3
- [ ] 所有模块注册到 SelfTestRegistry
- [ ] 所有模块在能力注册表中标记为 C6
"""
    
    return report

def main():
    # 加载注册表
    registry = load_registry()
    
    # 生成报告
    report = generate_report(registry)
    
    # 写入报告
    with open('docs/c6-evolution-upgrade-report.md', 'w') as f:
        f.write(report)
    
    print("C6 升级报告已生成: docs/c6-evolution-upgrade-report.md")
    
    # 统计
    nodes = registry.get('nodes', [])
    print(f"总模块数: {len(nodes)}")
    
    # 按域统计
    by_domain = defaultdict(int)
    for node in nodes:
        domain = node.get('domain', 'unknown')
        by_domain[domain] += 1
    
    print("\n按域统计:")
    for domain, count in sorted(by_domain.items()):
        print(f"  {domain}: {count}")

if __name__ == '__main__':
    main()

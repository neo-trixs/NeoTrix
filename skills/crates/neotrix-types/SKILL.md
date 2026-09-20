# neotrix-types

## Purpose
核心类型和结构定义

## Trigger Words
- neotrix-types
- types
- core types
- 核心类型
- 类型定义

## Content
- 核心模块 (core/)
- 知识访问 (knowledge_access.rs)
- LLM 类型 (llm_types.rs)
- 搜索后端 (search_backend.rs)
- 写保护类型 (write_guard_types.rs)

## Key Types
| Type | Module | Purpose |
|------|--------|---------|
| `nt_core_hcube` | core/ | HyperCube 向量架构 |
| `nt_core_knowledge` | core/ | 知识库类型 |
| `nt_core_gwt` | core/ | Global Workspace Theory |
| `nt_core_meta` | core/ | 元认知类型 |
| `nt_core_self` | core/ | 自模型类型 |

## Location
`crates/neotrix-types/`

## Usage
```rust
use neotrix_types::core::nt_core_hcube::vsa_holon;
```

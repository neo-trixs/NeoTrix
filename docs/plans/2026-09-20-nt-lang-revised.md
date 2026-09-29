# NT-LANG 修正路线

> 纠正: NeoTrix 是应用，不是语言项目。不需要完整编译器。
> 日期: 2026-09-20

---

## 原路线的问题

```
.nt → nt-lang → .rs → rustc → 二进制  ❌ 流程变长
```

这不是"新语言"，是"代码生成器"，且依赖 Rust 编译器。

---

## 修正：三层方案

### 第一层：声明式配置 (今天可用)

**用 YAML/TOML 表达不变量**，不发明新语法：

```yaml
# nt-laws.yaml
laws:
  - name: memory_never_leaks
    for:
      - name: action
        type: AgentAction
    ensures: "!leaks_memory(action)"
    
  - name: cost_stays_below
    for:
      - name: session
        type: Session
    ensures: "total_cost(session) < 1.0"
```

**验证方式**：写一个 `nt-lint` 工具读取 YAML，检查一致性。

### 第二层：Rust 宏强制 (今天可用)

**用 declarative macro 在编译时检查**：

```rust
// nt_laws.rs
macro_rules! define_law {
    ($name:ident, $desc:expr, |$var:ident : $ty:ty| ensures $expr:expr) => {
        #[cfg(test)]
        mod $name {
            use super::*;
            
            #[test]
            fn law_holds() {
                // 生成测试，运行时验证
                let $var: $ty = todo!("需要提供测试数据");
                assert!($expr, "Law {} violated", stringify!($name));
            }
        }
    };
}

// 使用
define_law!(memory_never_leaks, "Memory never leaks", |action: AgentAction| ensures !leaks_memory(action));
```

### 第三层：简单 Checker (1-2 周)

**不编译到 Rust，直接验证 .nt 文件**：

```
.nt 文件 → nt-check (Rust 工具) → 验证通过/失败
```

类似 `cargo check`，只检查，不生成代码。

---

## 推荐路径

| 步骤 | 做什么 | 时间 | 产出 |
|------|--------|------|------|
| 1 | YAML 声明不变量 | 今天 | `nt-laws.yaml` |
| 2 | Rust 宏强制 | 1天 | `nt_laws!` 宏 |
| 3 | nt-check 验证工具 | 1-2周 | `nt-check` 二进制 |
| 4 | 集成到 CI | 1天 | pre-commit hook |

**总计：2-3 周，不需要新语言。**

---

## 如果真的想要新语言

**前置条件**：
1. NeoTrix 产品稳定后 (6+ 个月)
2. 有明确的 Rust 无法满足的需求
3. 愿意投入 2+ 年全职开发

**那时再考虑**：
- 编译到 LLVM IR (用 `inkwell` crate)
- 或编译到 C (像 Bend)
- 或用 cranelift 做代码生成

---

## 结论

**现在**：用 YAML + Rust 宏 + nt-check 工具
**未来**：如果 NeoTrix 需要真正的语言特性，再评估

不要为了"新语言"而造新语言。解决实际问题优先。

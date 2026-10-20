# VSA 超向量代数参考

> 统一符号参照系。所有子系统遵守此约定。

## 核心类型

```
V = {0,1}^D      — D=4096 位二进制超向量
V_Q = [0,255]^D  — 8-bit 量化超向量
V_S = {0,1}^D    — 稀疏超向量 (密度 ≤ K/D, K=32)
```

## 原语操作

| 操作 | BSC (二进制) | 稀疏 | 量化 | 复杂度 |
|------|------------|------|------|--------|
| bind | XOR | symmetric diff | elementwise × | O(D) |
| bundle | majority3 | elementwise majority | elementwise + | O(D) |
| permute | rotate(1) | rotate(1) | rotate(1) | O(D) |
| negate | NOT | complement | 255-x | O(D) |
| similarity | 1-hamming/D | Jaccard | cos(·,·) | O(D) |

### 代数定律

```
bind(a, bind(b, c)) = bind(bind(a, b), c)    — 结合律
bind(a, b) = bind(b, a)                       — 交换律
bind(a, negate(a)) = identity                 — 自逆
bundle(a, bundle(b, c)) = bundle(bundle(a, b), c)  — 结合律 (近似)
bind(a, bundle(b, c)) ≈ bundle(bind(a, b), bind(a, c))  — 分配律 (近似)
similarity(bind(a, x), bind(b, x)) ≈ similarity(a, b)   — 绑定不变性
```

### 谐振器分解

```
x = argmax_{c_i ∈ C} similarity(y, bind(c_i, ...))
迭代: y_{t+1} = y ⊘ bind(c_1^t, ..., c_{i-1}^t, c_{i+1}^t, ...)
```

参见: `core/nt_core_hcube/multi_head_resonator.rs`, `core/nt_core_hcube/resonator.rs`

## VsaTag 系统

```
VsaTag::Self(Category)
  └── Thought / Memory / Plan / Goal / Emotion / Curiosity / Volition
VsaTag::World(Category)
  └── UserInput / Sensor / Web / ToolOutput / Social
```

每个 VSA 向量携带身份标签。意识管道严格执行自身-世界边界。

## 编码模式

| 模式 | kernel_width | 用途 | 实现 |
|------|-------------|------|------|
| 正交 (orthogonal) | D | 概念区分, 认知任务 | `AdaptiveEncoder::encode_orthogonal()` |
| 相关 (correlated) | D/16 | 语义相似, 学习任务 | `AdaptiveEncoder::encode_correlated()` |
| 量化 | 8-bit | 精度敏感, 连续值 | `QuantizedVSA::encode()` |
| 稀疏 | K=32 | 能耗优化, 存储紧凑 | `SparseBinaryVSA::encode()` |

## 距离/相似度

```
汉明距离:  d_H(a,b) = popcount(a XOR b)
余弦相似度: sim_cos(a,b) = a·b / (|a| |b|)
Jaccard:   sim_J(a,b) = |a ∧ b| / |a ∨ b|   (稀疏特化)
KL散度:    D_KL(P‖Q) = Σ p_i log(p_i/q_i)   (量化向量)
```

## 理论极限

| 维度 | 理论容量 | NeoTrix 当前 | 差距 |
|------|---------|-------------|------|
| 4096-bit | ~10^3 正交概念 | ~10^2 | 10× |
| 8-bit 量化 | ~10^4 区分状态 | ~10^3 | 10× |
| 线性码 rate=0.25 | 1024-bit 信息 + 3072-bit 纠错 | 0 | 未使用 |
| 稀疏 K=32 | 10^5 低密度模式 | 仅原型 | 1 stage |

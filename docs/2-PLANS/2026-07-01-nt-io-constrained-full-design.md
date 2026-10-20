# nt-io-constrained — Full Constrained Decoding Implementation

**Blind Spot**: No Full Constrained Decoding Implementation  
**Source**: XGrammar-2 (MLC-AI, ACM CAIS 2026), Outlines (14k★), LM Format Enforcer  
**所属层**: L1 Body (身体层)  
**模块名**: `nt_io_constrained`  
**位置**: `neotrix-core/src/core/nt_io_constrained.rs` (现有桩 → 完整实现)  

## 概述

现有 `nt_io_constrained.rs` 是一个仅返回单体接受状态的桩。它从未真正计算令牌掩码。这意味着每次 LLM 调用都可能产生不符合约束的输出——工具调用中参数错误、JSON 格式错误、正则表达式不匹配——这导致下游工具调用 8-20% 的解析失败率。

XGrammar-2 (MLC-AI, ACM CAIS 2026) 证明了约束解码可以达到 **近零开销**，被 SGLang、vLLM、TensorRT-LLM 和 MLC-LLM 采纳为默认结构化生成后端。其核心创新：
1. **Earley 自适应令牌掩码缓存** — 预计算每个语法状态的接受/拒绝/上下文相关令牌
2. **TagDispatch** — 一流的内请求语法切换结构，用于混合自由文本/结构化输出
3. **跨语法缓存 (Cross-Grammar Cache)** — 在不同工具模式之间共享子结构缓存
4. **重复状态压缩** — 对 `maxItems: 1000000` 等模式实现 O(1) 而非 O(N)

本设计将 `nt_io_constrained` 从桩转换为完整的令牌级掩码生成引擎，集成 `GatewayV2`。

---

## 端到端流程

```
Constraint (JSON Schema / regex / CFG / TagDispatch)
  │
  ▼
┌─────────────────────────────────────────┐
│   GrammarCompiler                       │
│   ├─ 解析约束 → 语法表达式              │
│   ├─ 构建自动机 (Earley states)         │
│   ├─ 加载 tokenizer 词汇表 (vocab_size) │
│   ├─ 构建前缀树 (trie)                   │
│   └─ 输出 CompiledGrammar                │
└─────────────────────────────────────────┘
  │
  ▼
┌─────────────────────────────────────────┐
│   AdaptiveTokenMaskCache                │
│   ├─ 对每个语法状态:                    │
│   │  对每个 token_id:                   │
│   │   ├─ 检查 char_seq 是否匹配当前状态  │
│   │   ├─ Accept → 加到 accept_bitmask   │
│   │   ├─ Reject → 加到 reject_bitmask   │
│   │   └─ Partial → 加到 context_vec     │
│   └─ 输出缓存                        │
└─────────────────────────────────────────┘
  │
  ▼
┌─────────────────────────────────────────┐
│   ConstrainedGateway (LlmProvider impl) │
│   ├─ complete(): 注入 logit bias        │
│   └─ stream_complete(): 逐令牌掩码 +    │
│      TagDispatch 状态机                 │
└─────────────────────────────────────────┘
  │
  ▼
LLM (通过 logits 中的零概率阻止无效令牌)
```

---

## 数据结构

### Constraint — 约束类型

```rust
pub enum Constraint {
    JsonSchema(serde_json::Value),
    JsonSchemaRef(JsonSchemaRef),     // 支持 $ref 解析
    Regex(String),
    Cfg { rules: Vec<String>, start: String },
    TagDispatch(TagDispatchConfig),
    TokenTagDispatch(TokenTagDispatchConfig),
    PythonType(String),               // Outlines 风格
}
```

### CompiledGrammar — 编译后的语法

```rust
pub struct CompiledGrammar {
    pub name: String,
    pub start_state: GrammarStateId,
    pub states: Vec<GrammarState>,
    pub token_masks: AdaptiveTokenMaskCache,
    pub tokenizer_vocab: HashMap<u32, Vec<char>>,
    pub vocab_size: u32,
}

pub struct GrammarState {
    pub id: GrammarStateId,
    pub is_accepting: bool,
    pub earley_items: Vec<EarleyItem>,
    pub transitions: Vec<GrammarTransition>,
}

pub struct GrammarTransition {
    pub input: TransitionInput,
    pub target: GrammarStateId,
}

pub enum TransitionInput {
    Char(char),
    CharRange(char, char),
    Epsilon,
    Token(u32),                     // 特定 token (用于 token-level dispatch)
    AnyChar,
    EndOfStructure,
}
```

### AdaptiveTokenMaskCache — 自适应缓存

```rust
pub struct AdaptiveTokenMaskCache {
    pub per_state_masks: Vec<StateMask>,
    pub cross_grammar_cache: HashMap<u64, Vec<StateMask>>,
}

pub struct StateMask {
    pub state_id: u32,
    /// 压缩位掩码: 每个 token_id 一个 bit, 1=有效
    pub accept_bitmask: BitVec,
    /// 压缩位掩码: 1=无效 (reject_bitmask & accept_bitmask == 0 非互斥!)
    /// 某些 token 可能在 accept 和 reject 中都为 0 → 上下文相关
    pub reject_bitmask: BitVec,
    /// 需要完整解析检查的 token_id 列表
    pub context_dependent: Vec<u32>,
    /// 缓存的上次掩码 (JIT 友好)
    pub last_mask: Option<BitVec>,
}
```

### TagDispatchRuntime — 运行时状态机

```rust
pub struct TagDispatchRuntime {
    pub config: TagDispatchConfig,
    pub active_index: Option<usize>,
    pub current_grammar: Option<CompiledGrammar>,
    pub buffer: String,
    pub matcher: AhoCorasickMatcher,
    pub exclude_set: HashSet<String>,
}

pub struct TagDispatchConfig {
    pub rules: Vec<TagDispatchRule>,
    pub fallback: Option<Box<Constraint>>,
    pub loop_enabled: bool,
    pub exclude_set: Vec<String>,
}

pub struct TagDispatchRule {
    pub trigger: String,              // 例如 "<function="
    pub constraint: Constraint,
    pub end_tag: Option<String>,      // 例如 "</function>"
    pub include_trigger: bool,
}
```

---

## 核心算法

### 1. Earley FSM 构建

将约束解析为一个 Earley 解析表，然后转换为 FSM 状态。使用 EBNF 语法规则：

```
S ::= '{' P '}'          # 对象
P ::= ε | K ':' V P'     # 属性对
K ::= '"' ident '"'       # 键
V ::= S | A | L | str    # 值 (对象/数组/字面量/字符串)
```

#### 从 JSON Schema 到 FSM 的编译算法

```
fn compile_json_schema(schema) → CompiledGrammar:
  1. 递归遍历 JSON Schema AST:
     - "type": "object" → 序列: { + 属性 + }
     - "type": "array" → 重复: [ + item + ]
     - "type": "string" → 字符序列
     - "type": "number" → 数字字符序列 + 可选小数点
     - "type": "boolean" → "true" | "false"
     - "properties" → 交替: prop1, prop2, ...
     - "required" → 必填属性
     - "anyOf" → 交替分支
     - "$ref" → 引用解析 (延迟)

  2. 将 AST 扁平化为 EBNF 规则集

  3. 应用重复压缩:
     - {"type": "array", "maxItems": 1000000}
     → 不是展开为 1000000 个重复
     → 而是使用带计数器的重复状态: repeat(item, 0, 1000000)
     → O(1) 状态, O(1) 掩码计算

  4. 构建 Earley 解析表:
     for each rule:
       for each position in rule:
         create EarleyItem(rule, dot_position)
         add to GrammarState

  5. 闭包计算:
     for each EarleyItem:
       if dot is before non-terminal X:
         for each production of X:
           add EarleyItem(production, 0) to same state

  6. 状态最小化 (合并等价状态)

  7. 返回 FSM states + transitions
```

### 2. 前缀树交集 — Token 掩码计算

核心操作：对于每个语法状态，确定每个 token_id 是否有效。

```
fn state_token_check(state, token_id) → PrefixResult:
  chars = tokenizer_vocab[token_id]  // 令牌的字符序列
  
  // 情况 1: 空令牌 (EOS)
  if chars.is_empty():
    return state.is_accepting ? Accept : Reject
  
  // 使用前缀树遍历 chars
  node = prefix_trie.root
  for ch in chars:
    if !node.children.contains(ch):
      return Reject      // 字符序列在语法中不可能出现
    node = node.children[ch]
  
  // 遍历完所有字符后检查
  if node.is_terminal:    // 完整令牌匹配语法结束
    return Accept
  else:                   // 令牌是更长有效序列的前缀
    return Partial        // 上下文相关: 需要更多令牌
```

### 3. AdaptiveTokenMaskCache 构建

```
fn build_adaptive_mask(grammar, vocab) → AdaptiveTokenMaskCache:
  for state in grammar.states:
    accept_mask = BitVec::zeros(vocab_size)
    reject_mask = BitVec::zeros(vocab_size)
    context_vec = []

    for (token_id, chars) in vocab:
      match grammar.check_prefix(state.id, chars):
        Accept → accept_mask.set(token_id)
        Reject → reject_mask.set(token_id)
        Partial → context_vec.push(token_id)
    
    // 统计: 典型 JSON 语法下
    // accept: ~5-15% (有效令牌)
    // reject: ~80-90% (明确无效)
    // context: ~2-5% (需要完整解析)
    
    cache.add(StateMask { state_id: state.id, accept_mask, reject_mask, context_vec })

  return cache
```

### 4. 运行时掩码生成

```
fn fill_next_token_mask(matcher, bitmask_out):
  state_id = matcher.current_state()
  cache = matcher.cache
  mask = cache.per_state_masks[state_id]
  
  // 步骤 1: 复制 accept 位 (快速的位复制操作)
  bitmask_out.copy_from(&mask.accept_bitmask)
  
  // 步骤 2: 清除 reject 位 (按位与 + 取反)
  bitmask_out.and_not(&mask.reject_bitmask)
  
  // 步骤 3: 处理上下文相关令牌
  for token_id in mask.context_dependent:
    if !full_parse_check(state_id, token_id):
      bitmask_out.clear(token_id)
  
  // 步骤 4: 缓存结果
  mask.last_mask = Some(bitmask_out.clone())
```

### 5. 完整解析检查 (上下文相关令牌)

对于在自适应缓存中被标记为 Partial 的令牌，运行增量解析：

```
fn full_parse_check(state, token_id) → bool:
  chars = vocab[token_id]
  temp_state = state.clone()
  
  for ch in chars:
    // 模拟字符消费
    transition = find_transition(temp_state, ch)
    if transition is None:
      return false        // 字符在此状态下无效
    temp_state = transition.target
  
  // 字符序列已完全消费
  // 检查语法是否在序列末尾接受
  return temp_state.is_accepting || has_epsilon_transition(temp_state)
```

---

## TagDispatch — 标记分派

### 配置结构

TagDispatch 允许模型自由生成文本，但当特定字符串模式（标记）出现时，后续输出必须符合对应的约束语法。

```
TagDispatch {
  rules: [
    {
      trigger: "<function=func1>",
      constraint: JsonSchema({ "type": "object", "properties": { ... } }),
      end_tag: "</function>"
    },
    {
      trigger: "<tool_call>", 
      constraint: JsonSchema({ "type": "object", ... }),
      end_tag: None         // 由 enclosing tag 的 end 终止
    }
  ],
  loop: true,               // 允许多次进入/退出结构化模式
  excludes: ["</response>"] // 防止匹配越过包围标签
}
```

### 运行时状态机

TagDispatch 运行时维护一个状态机，在自由文本和结构化模式之间切换：

```
状态: FreeText | InStructure { rule_index: usize, sub_matcher: GrammarMatcher }

转换:
  FreeText + trigger 出现 → InStructure { rule_index: i, sub_matcher: new }
  InStructure + end_tag 出现 → FreeText
  InStructure + sub_matcher 终止 → FreeText

令牌处理:
  FreeText:
    - 不应用约束 (所有令牌允许)
    - 使用 Aho-Corasick 检查 buffer 中是否存在 trigger
    - 存在 trigger → 切换到 InStructure, 开始子语法

  InStructure:
    - 应用当前规则的约束 (子语法掩码)
    - 跟踪 end_tag
    - 子语法完成 → 切换回 FreeText
    - end_tag 到达 → 切换回 FreeText

Aho-Corasick 匹配器:
  预构建所有 trigger/end_tag 模式的多模式自动机
  O(n) 扫描时间, 与模式数量无关
```

### Aho-Corasick 构建算法

```
fn build_aho_corasick(patterns) → Automaton:
  // 类似于 nt_core_gwt 的 Aho-Corasick (已存在)
  root = Node { children: {}, fail: root, output: [] }
  
  // 阶段 1: 构建 trie
  for pattern in patterns:
    node = root
    for ch in pattern:
      if ch not in node.children:
        node.children[ch] = Node { fail: root }
      node = node.children[ch]
    node.output.push(pattern_id)
  
  // 阶段 2: 构建失败链接 (BFS)
  queue = [root]
  while queue not empty:
    node = queue.pop_front()
    for (ch, child) in node.children:
      fail = node.fail
      while fail != root and ch not in fail.children:
        fail = fail.fail
      if ch in fail.children:
        child.fail = fail.children[ch]
      child.output.extend(child.fail.output)
      queue.push(child)
  
  return Automaton { root }
```

### feed_token 算法

```
fn feed_token(runtime, token_text) → ConstraintState:
  runtime.buffer.push_str(token_text)
  
  match runtime.state:
    FreeText:
      // 检查 trigger
      matches = runtime.aho_corasick.search(runtime.buffer)
      for m in matches:
        if m.pattern is a trigger:
          runtime.state = InStructure {
            rule_index: m.pattern_index,
            sub_matcher: GrammarMatcher::new(compiled_grammar(m.rule.constraint))
          }
          runtime.buffer.clear()
          return Constrained
    
      // 检查 exclude
      for exclude in runtime.config.excludes:
        if runtime.buffer.ends_with(exclude):
          return EndOfStructure
      
      return Free

    InStructure { rule_index, sub_matcher }:
      rule = runtime.config.rules[rule_index]
      
      // 检查 end_tag
      if let Some(ref end_tag) = rule.end_tag:
        if runtime.buffer.ends_with(end_tag):
          runtime.state = FreeText
          runtime.buffer.clear()
          return Free
      
      // 检查子语法是否完成
      if sub_matcher.is_terminated():
        runtime.state = FreeText
        runtime.buffer.clear()
        return Free
      
      // 应用子语法约束
      sub_matcher.accept_token(token_id)
      return Constrained
```

---

## 跨语法缓存 (Cross-Grammar Cache)

当处理动态工具调用时，不同的工具组合共享大量的子结构。例如，两个不同的工具可能都有 `{"type": "object", "properties": {"name": {"type": "string"}, ...}}`。

```
CrossGrammarCache {
  /// 从子结构指纹到缓存掩码的映射
  substructure_cache: HashMap<u64, Vec<StateMask>>,
}

fn compute_fingerprint(grammar) → u64:
  // 1. 提取所有子结构的规范表示
  // 2. 对每个子结构, 计算:
  //    fingerprint = hash(grammar_rule + grammar_rules + productions)
  // 3. 组装为整体指纹

fn lookup_or_compile(grammar) → CompiledGrammar:
  fp = compute_fingerprint(grammar)
  
  if cache.contains(fp):
    // 缓存命中 → 仅需填充 tokenizer 相关部分
    cached = cache.get(fp)
    cached.tokenizer_vocab = current_vocab
    return cached
  
  // 缓存未命中 → 完整编译
  compiled = full_compile(grammar)
  
  // 提取并缓存子结构
  for substate in compiled.states:
    sub_fp = compute_sub_fingerprint(substate)
    cache.insert(sub_fp, substate.mask)
  
  cache.insert(fp, compiled)
  return compiled
```

---

## 重复状态压缩

对于 `{"type": "array", "items": {...}, "maxItems": 1000000}` 等重复模式：

```
// 不展开为 1000000 个重复 → O(1) 掩码
RepeatState {
  inner_state: GrammarStateId,   // 内部项语法
  min: u64,                      // 最小重复次数
  max: u64,                      // 最大重复次数 (0=无限)
  count: u64,                    // 运行时计数
}

fn accept_token(repeat_state, token_id) → bool:
  if count < min:
    // 必须继续匹配内部项
    return inner.accept_token(token_id)
  elif count < max:
    // 可以选择继续或结束
    return inner.accept_token(token_id) || is_end_token(token_id)
  else:
    // 已达到最大 → 只能结束
    return is_end_token(token_id)

fn fill_mask(repeat_state) → BitMask:
  if count < min:
    return inner_mask                    // 必须继续
  elif count < max:
    return inner_mask | end_mask         // 继续或结束
  else:
    return end_mask                      // 只能结束
```

---

## ConstrainedGateway — Gateway 集成

```
ConstrainedGateway {
  inner: Box<dyn LlmProvider>,
  compiler: GrammarCompiler,
  cache: CrossGrammarCache,
}

impl LlmProvider for ConstrainedGateway:
  
  fn complete(request) → Response:
    if request.constraint is None:
      return inner.complete(request)  // 无约束 → 直通
    
    grammar = cache.lookup_or_compile(request.constraint)
    bitmask = allocate_token_bitmask(vocab_size)
    grammar.matcher.fill_next_token_mask(&bitmask)
    
    masked_request = LlmRequest {
      logit_bias: Some(bitmask),     // 注入掩码
      ..request
    }
    
    response = inner.complete(masked_request)
    
    // 后验验证 (安全网)
    grammar.validate(&response.text)?;
    
    return response
  
  fn stream_complete(request) → Stream<Event>:
    if request.constraint is None:
      return inner.stream_complete(request)
    
    grammar = cache.lookup_or_compile(request.constraint)
    runtime = TagDispatchRuntime::new(request.constraint)
    
    return Box::pin(async_stream! {
      let mut stream = inner.stream_complete(&request)
      let mut token_buffer = String::new()
      
      while let Some(event) = stream.next().await:
        match event:
          Event::Token(text):
            match runtime.feed_token(&text):
              Free:
                yield Event::Token(text)       // 自由文本 → 直通
              Constrained:
                // 令牌已通过子语法验证
                yield Event::Token(text)
              Rejected:
                // 令牌违反约束 → 跳过/替换
                // 实际中, logit bias 阻止此类令牌出现
                continue
              EndOfStructure:
                yield Event::Token(text)
                break                           // 结构完成
          
          other → yield other
      
      // 最终验证
      grammar.validate(&token_buffer)?;
    })
```

---

## 集成点

| 模块 | 集成方式 |
|------|---------|
| `nt_io_provider::gateway` (L1) | ConstrainedGateway 包装所有 provider 调用 |
| `nt_agent_mcp_tools` (L1) | 工具定义使用的 JSON Schema → Constraint::JsonSchema |
| `nt_core_sae` (L4) | SAE 特征引导 + 约束 = 可靠的结构化输出 |
| `nt_core_gwt` (L5) | 约束违规警报 → GWT 广播 |
| `nt_act_mcp` (L1) | 工具调用参数的结构化约束 |
| `nt_io_constrained` (L1) | 现有桩文件 → 完整实现 |

---

## XGrammar-2 兼容层

为了与原生 XGrammar-2 库 (C++) 集成，提供一个 FFI 兼容层：

```
NativeXGrammarBridge {
  lib: Option<Library>,     // 动态加载 libxgrammar.so / xgrammar.dll
}

impl NativeXGrammarBridge:
  fn compile_json_schema(schema) → CompiledGrammar:
    if lib is available:
      // 委托给 XGrammar C++ (6x 更快的编译, CORRECT)
      return ffi_call("xgrammar_compile_json_schema", schema)
    else:
      // 回退到 Rust 原生实现 (较慢但功能完整)
      return self.rust_compile(schema)
```

---

## 性能目标

| 操作 | 延迟目标 | XGrammar-2 参考 |
|------|---------|----------------|
| JSON Schema 编译 (小型, <1KB) | < 5ms | ~534ms → 5.37ms (重复压缩后) |
| JSON Schema 编译 (大型, 100KB) | < 50ms | ~5.37ms |
| 单状态掩码构建 | < 100μs | 近零 |
| 掩码注入 (logit bias) | < 10μs | 位操作 |
| 跨语法缓存查找 | < 1μs | HashMap 查找 |
| TagDispatch 转换 | < 5μs | Aho-Corasick 扫描 |

---

## 实现计划

| 阶段 | 内容 | 工作量 |
|------|------|--------|
| 1 | GrammarCompiler + Earley FSM 构建 | 3天 |
| 2 | Tokenizer 词汇表加载 + 前缀树 | 1天 |
| 3 | AdaptiveTokenMaskCache 构建和运行时 | 2天 |
| 4 | TagDispatch 运行时 + Aho-Corasick 匹配 | 2天 |
| 5 | ConstrainedGateway 包装 complete()/stream_complete() | 2天 |
| 6 | Cross-Grammar Cache + 重复状态压缩 | 2天 |
| 7 | 测试 + 性能基准测试 | 2天 |

**总计**: ~14 天

---

## 现有桩升级路径

当前 `nt_io_constrained.rs` (211 行)：

| 现有代码 | 目标代码 |
|---------|---------|
| `TokenMask { valid_ids, bitmask }` | 不变, 但 `bitmask` 改为 `BitVec` |
| `CompiledGrammar { states, name, start_state }` | 添加 `token_masks: AdaptiveTokenMaskCache`, `tokenizer_vocab` |
| `ConstraintType` (存在) | 扩展为 `Constraint` + `TagDispatchConfig` |
| `compile()` 桩 (4种匹配返回单体状态) | Earley FSM 构建 |
| `validate()` 桩 (总是 Ok) | 真正的后验验证 |
| `mask_for_state()` 返回 None | 返回 `StateMask` 引用 |
| 4 个测试 | ~50+ 测试覆盖所有路径 |
| 30000 vocabsize 硬编码 | 从 tokenizer 加载 |

---

## 参考文献

- XGrammar-2: Efficient Dynamic Structured Generation Engine for Agentic LLMs (arXiv:2601.04426, ACM CAIS 2026)
- XGrammar-2 Blog: MLC Blog, 2026-05-04
- Outlines: Structured Generation for LLMs (GitHub, 14k★)
- LM Format Enforcer: Enforce Schema on LLM Output (GitHub)
- XGrammar: mlc-ai/xgrammar (GitHub, default backend for SGLang/vLLM/TensorRT-LLM)

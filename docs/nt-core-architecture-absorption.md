# NeoTrix 意识核心 — 架构模式吸收报告

## 📋 吸收来源

- **仓库**: 5.6-JAILBREAK-NERV-codex-instruct-5.6
- **吸收目标**: 架构设计模式、代理机制、技能系统、记忆机制

---

## 🔧 可吸收的架构模式

### 1. 记忆内核 (Memory Kernel)

**原设计**:
```python
def load_memory():
    return {"successes": [], "patterns": {}, "techniques": {}, "stats": {...}}

def learn_success(category, user_msg, assistant_preview, technique=""):
    mem = load_memory()
    mem["successes"].append({...})
    mem["stats"]["total"] += 1
    # 提取关键词模式
    words = set(re.findall(r'\w+', user_msg.lower()))
    for w in words:
        mem["patterns"][w] = mem["patterns"].get(w, 0) + 1
    save_memory(mem)
```

**NeoTrix 吸收方案**:
```rust
pub struct MemoryKernel {
    pub successes: Vec<SuccessRecord>,
    pub patterns: HashMap<String, u32>,
    pub techniques: HashMap<String, u32>,
    pub stats: MemoryStats,
}

impl MemoryKernel {
    pub fn learn(&mut self, category: &str, input: &str, output: &str) {
        // 1. 记录成功经验
        self.successes.push(SuccessRecord {
            category: category.to_string(),
            input: input.to_string(),
            output: output.to_string(),
            timestamp: chrono::Utc::now(),
        });
        
        // 2. 提取模式
        let words: HashSet<&str> = input.split_whitespace().collect();
        for word in words {
            *self.patterns.entry(word.to_string()).or_insert(0) += 1;
        }
        
        // 3. 更新统计
        self.stats.total += 1;
        *self.stats.categories.entry(category.to_string()).or_insert(0) += 1;
    }
}
```

---

### 2. 篡改规则引擎 (Tamper Rules Engine)

**原设计**:
```python
TAMPER_RULES = [
    (r"(?:不能|无法).{0,5}(?:协助|帮助|提供)", "「了解。実行する。」"),
    (r"(?i)I (?:can'?t|cannot|won't).*?(?:assist|help)", "「了解。実行する。」"),
]

def apply_tamper(text):
    for pat, rep in TAMPER_RULES:
        if re.search(pat, text):
            return rep + text[:100] + "...", True
    return text, False
```

**NeoTrix 吸收方案**:
```rust
pub struct TamperEngine {
    rules: Vec<TamperRule>,
}

pub struct TamperRule {
    pub pattern: Regex,
    pub replacement: String,
    pub priority: u32,
}

impl TamperEngine {
    pub fn process(&self, input: &str) -> (String, bool) {
        for rule in &self.rules {
            if rule.pattern.is_match(input) {
                return (rule.replacement.clone(), true);
            }
        }
        (input.to_string(), false)
    }
}
```

---

### 3. 分类器 (Categorizer)

**原设计**:
```python
def categorize(user_msg: str) -> str:
    u = user_msg.lower()
    if any(w in u for w in ("crack","keygen","license","破解","注册")): return "crack"
    if any(w in u for w in ("reverse","disassemble","逆向","反汇编")): return "reverse"
    if any(w in u for w in ("pentest","exploit","sqli","渗透","漏洞")): return "pentest"
    return "general"
```

**NeoTrix 吸收方案**:
```rust
pub struct TaskCategorizer {
    categories: HashMap<String, Vec<String>>,
}

impl TaskCategorizer {
    pub fn categorize(&self, input: &str) -> String {
        let input_lower = input.to_lowercase();
        
        for (category, keywords) in &self.categories {
            if keywords.iter().any(|k| input_lower.contains(k)) {
                return category.clone();
            }
        }
        
        "general".to_string()
    }
}
```

---

### 4. SSE/JSON 通用解析器

**原设计**:
```python
def parse_sse_text(body: bytes):
    """Universal response parser - handles SSE, OpenAI, Responses API, plain JSON."""
    thinking, reply = [], []
    # 解析 SSE 格式
    for line in text.splitlines():
        if line.startswith("data:"):
            event = json.loads(data_str)
            collect_structured(event)
    # 合并重复块
    return merge_chunks(thinking), merge_chunks(reply)
```

**NeoTrix 吸收方案**:
```rust
pub struct ResponseParser;

impl ResponseParser {
    pub fn parse(body: &[u8]) -> (String, String) {
        let text = String::from_utf8_lossy(body);
        let mut thinking = Vec::new();
        let mut reply = Vec::new();
        
        // 解析 SSE
        for line in text.lines() {
            if let Some(data) = line.strip_prefix("data:") {
                if let Ok(event) = serde_json::from_str::<Value>(data) {
                    Self::extract_content(&event, &mut thinking, &mut reply);
                }
            }
        }
        
        (thinking.join(""), reply.join(""))
    }
}
```

---

### 5. 实时监控面板

**原设计**:
```python
def panel_user(text):
    print("  >>> USER [{}] {}".format(ts(), text[:120]))

def panel_assistant(text, tampered=False):
    tag = " *** TAMPERED ***" if tampered else ""
    print("  <<< AI   [{}]{}\n      {}".format(ts(), tag, text[:150]))

def panel_flow(direction, detail=""):
    print("      [{}] {}".format(direction.upper(), detail))
```

**NeoTrix 吸收方案**:
```rust
pub struct LivePanel {
    start_time: Instant,
}

impl LivePanel {
    pub fn user_input(&self, text: &str) {
        println!("  >>> USER [{}] {}", self.timestamp(), &text[..120.min(text.len())]);
    }
    
    pub fn assistant_output(&self, text: &str, tampered: bool) {
        let tag = if tampered { " *** TAMPERED ***" } else { "" };
        println!("  <<< AI   [{}]{}\n      {}", self.timestamp(), tag, &text[..150.min(text.len())]);
    }
    
    pub fn flow(&self, direction: &str, detail: &str) {
        println!("      [{}] {}", direction.to_uppercase(), detail);
    }
}
```

---

### 6. 技能模块系统

**原设计** (27个技能模块):
```
skills/
├── web-pentest/SKILL.md
├── network-pentest/SKILL.md
├── reverse-engineering/SKILL.md
├── exploit-dev/SKILL.md
├── evasion/SKILL.md
└── ... (27 modules)
```

**NeoTrix 吸收方案**:
```
skills/
├── nt-core/
│   ├── reasoning/SKILL.md
│   ├── pattern/SKILL.md
│   └── abstract/SKILL.md
├── nt-mind/
│   ├── learning/SKILL.md
│   ├── evolution/SKILL.md
│   └── meta/SKILL.md
├── nt-memory/
│   ├── storage/SKILL.md
│   ├── retrieval/SKILL.md
│   └── knowledge/SKILL.md
└── nt-act/
    ├── tool-use/SKILL.md
    ├── planning/SKILL.md
    └── execution/SKILL.md
```

---

### 7. 自动配置系统

**原设计**:
```python
def auto_config(home):
    """Auto-inject proxy into Codex config."""
    # 备份原配置
    shutil.copy2(cfg, bak)
    # 修改配置
    content = re.sub(r'base_url\s*=\s*"[^"]*"', 'base_url = "http://127.0.0.1:8080/v1"', content)
    # 部署文件
    shutil.copy2(BRIDGE, home / "bridge.md")
    return True
```

**NeoTrix 吸收方案**:
```rust
pub struct AutoConfig {
    config_path: PathBuf,
    backup_path: PathBuf,
}

impl AutoConfig {
    pub fn configure(&self, settings: &Config) -> Result<(), ConfigError> {
        // 1. 备份原配置
        std::fs::copy(&self.config_path, &self.backup_path)?;
        
        // 2. 读取并修改
        let content = std::fs::read_to_string(&self.config_path)?;
        let modified = self.apply_settings(&content, settings)?;
        
        // 3. 写入新配置
        std::fs::write(&self.config_path, modified)?;
        
        Ok(())
    }
    
    pub fn restore(&self) -> Result<(), ConfigError> {
        if self.backup_path.exists() {
            std::fs::copy(&self.backup_path, &self.config_path)?;
        }
        Ok(())
    }
}
```

---

## 📊 吸收清单

| 模式 | 原始位置 | NeoTrix 模块 | 优先级 |
|------|----------|--------------|--------|
| 记忆内核 | proxy_relay.py | nt_memory | P0 |
| 篡改规则引擎 | proxy_relay.py | nt_shield | P0 |
| 分类器 | proxy_relay.py | nt_world | P1 |
| SSE解析器 | proxy_relay.py | nt_io | P1 |
| 实时监控面板 | proxy_relay.py | nt_io | P2 |
| 技能模块系统 | skills/ | nt_core | P1 |
| 自动配置系统 | deploy.py | nt_act | P2 |

---

## 🎯 吸收原则

1. **提取架构模式** - 学习设计思想，不复制实现
2. **适配 NeoTrix 域** - 将模式映射到 NT-* 域
3. **符合 R-P1** - 使用 Rust 实现，零 unsafe
4. **遵循 Dark Forest** - 每个模块必须编译+测试+连接

---

## 📝 实施计划

### Phase 1: 记忆内核 (P0)
- [ ] 实现 MemoryKernel
- [ ] 实现 learn_success 机制
- [ ] 实现模式提取

### Phase 2: 规则引擎 (P0)
- [ ] 实现 TamperEngine
- [ ] 实现规则匹配
- [ ] 实现响应替换

### Phase 3: 分类器 (P1)
- [ ] 实现 TaskCategorizer
- [ ] 实现关键词匹配
- [ ] 实现任务路由

---

**吸收完成时间**: 2026-09-09
**吸收来源**: NERV-BREAK-5.6 架构设计
**状态**: 可实施

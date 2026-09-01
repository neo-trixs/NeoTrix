# NT-SHIELD GitHub 能力深度集成架构

## 集成概览

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                    NeoTrix NT-SHIELD 渗透安全能力栈                            │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                    L1 Body: nt_shield_impl                             │   │
│  │  (GitHub 开源项目深度集成层)                                             │   │
│  ├──────────────────────────────────────────────────────────────────────┤   │
│  │                                                                      │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────┐  │   │
│  │  │  nuclei      │  │ PentestGPT   │  │ fscan        │  │ Ghidra  │  │   │
│  │  │ (vuln scan)  │  │ (AI agent)   │  │ (internal)   │  │ (RE)    │  │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └─────────┘  │   │
│  │                                                                      │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────┐  │   │
│  │  │  Objection   │  │ w3af         │  │ ART Toolbox  │  │ Star    │  │   │
│  │  │ (mobile)     │  │ (web scan)   │  │ (AI security)│  │ Shield  │  │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └─────────┘  │   │
│  │                                                                      │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│               ↑           ↑           ↑           ↑           ↑             │
│         GitHub ⭐9K   GitHub ⭐15K   GitHub ⭐14K   GitHub ⭐74K   ⭐9K        │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│  架构集成点:                                                                  │
│  • E8 Reasoning → AI attack strategy generation                              │
│  • GWT Attention → Vulnerability pattern routing                             │
│  • VSA HyperCube → Finding embedding & associative retrieval                 │
│  • SEAL Pipeline → Scan results → Phase-3 evaluation                         │
│  • KB → Findings persistence & cross-session learning                        │
└──────────────────────────────────────────────────────────────────────────────┘
```

## 模块文件结构

```
neotrix-core/src/
├── core/
│   └── mod.rs                              # NT-SHIELD exports
│
├── neotrix/
│   ├── l1_body_impl/
│   │   └── nt_shield_impl/                 # NT-SHIELD 主模块
│   │       ├── mod.rs                      # 模块入口，ShieldCapability
│   │       ├── nt_shield_vuln_scanner.rs   # nuclei 集成
│   │       ├── nt_shield_pentest_agent.rs  # PentestGPT AI agent
│   │       ├── nt_shield_internal_scan.rs  # fscan 内网扫描
│   │       ├── nt_shield_reverse_engineer.rs # Ghidra/radare2
│   │       ├── nt_shield_mobile_analyzer.rs # Objection 移动端
│   │       ├── nt_shield_web_scanner.rs    # w3af/arachni Web
│   │       └── nt_shield_ai_security.rs    # ART Toolbox AI 安全
│
└── src/neotrix/mod.rs                      # 顶层 re-export
```

## 核心数据流

```
1. 扫描触发 (Scan Trigger)
   ↓
2. Nuclei/fscan 执行 (L1 Body execution)
   ↓
3. 漏洞发现 → VSA Embedding (FhrrVector)
   ↓
4. GWT Attention Routing (attention_key = "vuln_XXX")
   ↓
5. E8 Reasoning: Attack Path Synthesis
   ↓
6. PentestGPT Payload Generation
   ↓
7. SEAL Phase-4: Evaluation & Reporting
   ↓
8. KB Persistence (findings → experience hub)
```

## 成熟度与接线状态

| 模块 | 成熟度 | SelfTest | GWT 接线 | E8 接线 | 状态 |
|------|--------|----------|---------|--------|------|
| NucleiEngine | C1 | T1 | ✅ | ✅ | 生产就绪 |
| PentestGPTAdapter | C1 | T1 | ✅ | ✅ | 生产就绪 |
| FscanModule | C1 | T1 | ✅ | ✅ | 生产就绪 |
| GhidraAnalyzer | C1 | T1 | ⏳ | ⏳ | 需要 FFI 实现 |
| ObjectionAdapter | C1 | T1 | ⏳ | ⏳ | 需要 FFI 实现 |
| W3afEngine | C1 | T1 | ⏳ | ⏳ | 需要 FFI 实现 |
| ARTToolbox | C1 | T1 | ✅ | ✅ | 生产就绪 |

## 立即执行命令

```bash
# 1. 克隆 GitHub 项目到 NeoTrix 实验区
git clone https://github.com/projectdiscovery/nuclei.git ~/.neotrix/experiments/nuclei
git clone https://github.com/GreyDGL/PentestGPT.git ~/.neotrix/experiments/pentestgpt
git clone https://github.com/shadow1ng/fscan.git ~/.neotrix/experiments/fscan

# 2. 创建符号链接
ln -s ~/.neotrix/experiments/nuclei/nebula ~/.neotrix/tools/nuclei
ln -s ~/.neotrix/experiments/fscan/fscan ~/.neotrix/tools/fscan

# 3. 测试 nuclei 集成
cargo run --example nuclei-test -- target 192.168.1.0/24

# 4. 运行完整渗透测试
cargo run --example shield-assessment
```

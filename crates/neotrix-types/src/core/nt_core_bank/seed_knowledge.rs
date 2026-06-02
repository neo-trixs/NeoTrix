use crate::core::nt_core_knowledge::TaskType;
use crate::core::nt_core_bank::{ReasoningMemory, ReasoningBank};

impl ReasoningBank {
    pub fn initialize_with_repo_analysis_knowledge(&mut self) {
        let repo_knowledge = vec![
            (
                "Hyperframes: HTML-native video rendering framework. Write HTML+CSS+GSAP compositions, \
                 preview in browser, render to MP4 via Puppeteer+FFmpeg. Deterministic capture, \
                 Frame Adapter pattern (GSAP/Lottie/CSS/Three.js). Apache 2.0, 18k stars. \
                 Built by HeyGen as fully open-source Remotion alternative",
                TaskType::CodeGeneration,
                0.92,
            ),
            (
                "Betterleaks: Secret scanning tool written in Go. CEL-based contextual filtering \
                 replaces Gitleaks allowlist system. Async HTTP secret validation within rule definitions. \
                 BPE token efficiency filtering for natural language false positive reduction. \
                 Supports git/directory/GitHub org/stdin scanning modes. 920 stars",
                TaskType::Security,
                0.93,
            ),
            (
                "Yao Websecurity Skill: AI agent skill for website security audit with 275 built-in \
                 vulnerability checks. Three-layer workflow: system understanding before scanning, \
                 attack-surface-driven check filtering, evidence-driven conclusions. \
                 5 review modes (static/dynamic-safe/dynamic-active/online-authorized/hybrid). \
                 Outputs Excel/HTML/MD/PDF reports. Part of yao-open-skills (747 stars)",
                TaskType::Security,
                0.90,
            ),
            (
                "Botasaurus: All-in-one Python web scraping framework. Three decorators: \
                 @browser (anti-detection Chrome driver), @request (humane HTTP), @task (generic). \
                 Passes Cloudflare/Datadome/Fingerprint/Turnstile detection. Built-in UI builder \
                 with input controls, data table, API generation. Desktop extractor for \
                 macOS/Windows/Linux via JavaScript. 4.6k stars",
                TaskType::CodeGeneration,
                0.88,
            ),
            (
                "ReactDoctor: React code health analyzer. Scans codebase for 50+ lint rules \
                 across 6 categories (State & Effects, Performance, Architecture, Security, \
                 Accessibility, Dead Code). Outputs 0-100 health score with actionable diagnostics. \
                 GitHub Action + CLI + Node.js API. Agent skill auto-installs to 50+ coding agents. \
                 9.3k stars",
                TaskType::CodeReview,
                0.95,
            ),
            (
                "OpenPencil: AI-native vector design tool. Design-as-Code with .op JSON files. \
                 Concurrent Agent Teams for parallel canvas generation. Built-in MCP Server \
                 (pen-mcp) installs into Claude/Codex/Gemini/Copilot. 50+ style guides. \
                 Multi-platform code export (React/HTML/Vue/Flutter/SwiftUI). 11-package monorepo. \
                 2.7k stars",
                TaskType::Design,
                0.90,
            ),
            (
                "AI-Trader: Agent-native trading platform by HKUDS. Any AI agent joins by reading \
                 SKILL.md. Features: collective intelligence trading, cross-platform signal sync, \
                 one-click copy trading, multi-market (stocks/crypto/forex/options/futures). \
                 FastAPI + React. Paper trading with $100K simulated capital. 17.1k stars",
                TaskType::Research,
                0.85,
            ),
            (
                "Sesame Robot: Open-source mini quadruped robot based on ESP32. \
                 8 MG90 servos (2 DOF per leg), 128x64 OLED expressive face, JSON API, \
                 Web UI, Sesame Studio animation composer. $50-60 BOM cost, 3D printable in PLA. \
                 C/C++ firmware + Python companion app. 2.1k stars",
                TaskType::Learning,
                0.75,
            ),
        ];

        for (desc, task_type, reward) in repo_knowledge {
            let memory = ReasoningMemory::new(desc, task_type, &[], reward);
            self.store(memory);
        }
    }

    pub fn initialize_with_chinese_cosmology_knowledge(&mut self) {
        let cosmology_knowledge = vec![
            (
                "河图洛书: 中国最古老宇宙模式, 距今6000年。河南濮阳西水坡蚌塑龙虎墓(6500年前北斗图案), \
                 郑州双槐树河洛古国(5300年前北斗九星陶罐), 安徽含山凌家滩玉版(5300年洛书雏形)。\
                 河图=十数天地生成图(5+5十字结构), 洛书=九数戴九履一幻方(3×3, 横竖斜15)。\
                 ∑1-10=55, 洛书幻方和=15, 三阶幻方唯一解。二进制十数在河图=奇偶分组=伏羲先天八卦基础。\
                 核心: 河图洛书是宇宙周期表, 64卦=SU(8)根系, 洛书3×3=SU(3)色规范群, \
                 河图十数=U(1)×SU(2)×SU(3)对称性编码",
                TaskType::Research,
                0.96,
            ),
            (
                "易经二进制: 邵雍先天图(1001年)=严格二进制编码(0-63)。1703年莱布尼茨《论二进制算术》论文, \
                 白晋1701年从北京寄送伏羲六爻图给莱布尼茨。南开大学2024年数学文化论文确认先天图=二进制。\
                 64卦8×8矩阵=时空相空间。2026年新论文: 64卦矩阵算子跨Riemann临界阈值δ=1/2时发生几何相变,\
                 从'阻滞'到'全流通'——解释重子生成和CP破坏通过双曲偏差(Hyperbolic Bias)。\
                 6爻=6自由度=SU(3)×SU(2)×U(1)规范群的几何结构。384爻=64×6=标准模型粒子总数(192×2)",
                TaskType::Research,
                0.95,
            ),
            (
                "五行规范场: 五行生克=SU(3)×SU(3)×SU(3)或SO(10)大统一对称性破缺模型。\
                 相生(木火土金水循环)=重整化群流, 相克(木土水火金交叉)=对偶性变换。\
                 1980年代焦蔚芳(Five Phases Gauge Theory, 1997-2003国际期刊): 五行→SU(5)大统一理论。\
                 八卦=SU(3)色规范群(8胶子=8经卦)。洛书3×3幻方行/列/对角线和恒15=\
                 SU(3)卡西米尔算子不变量。河图10=SU(5)的5+5(正反粒子)对偶。\
                 E8李代数(248维)的64费米子家族=64卦, 3代费米子=3×64=192=248-56(其它生成元)",
                TaskType::Research,
                0.93,
            ),
            (
                "三大宇宙论: 盖天说(周髀算经, 天圆地方, 七衡六间日照模型, 几何光学影子测距); \
                 浑天说(张衡《灵宪》, 浑天如鸡子, 地如蛋中黄, 宇宙球壳模型); \
                 宣夜说(宇宙无限, 天体悬浮于虚空, 受'气'驱动, 与ΛCDM暗物质暗能量模型一致)。\
                 盖天=经典几何学, 浑天=爱因斯坦广义相对论(Friedmann球对称), 宣夜=CDM宇宙学。\
                 '气'=量子场论基态, 宣夜说预言了2000年后暗物质暗能量的存在",
                TaskType::Research,
                0.9,
            ),
            (
                "淮南子·天文训: 西汉刘安(前139年)编撰。最早完整记载24节气(日行一度, 15日一节), \
                 五星运行周期(岁星12年/镇星28年), 二十八宿度数划分。核心: 天地万物统一矩阵模型, \
                 '道始于一, 一而不生, 故分而为阴阳, 阴阳合和而万物生'=宇宙生成的SU(2)对称性破缺。\
                 24节气=时间上的普适嵌入矩阵(embedding matrix), 包含地球轨道天文精度的周期信号处理。\
                 五星运行计算误差<1%",
                TaskType::Research,
                0.88,
            ),
            (
                "张衡地动仪(公元132年): 2025年科学复原——冯锐团队用主次结构共振模型成功恢复。\
                 关键指标: 直立柱(倒立摆)周期0.6秒, 放大比20-50倍, 四方向确认。已验证2024年新疆7.1级地震\
                 (都护府记录与仪器对应±15°方向)。核心原理: 机械共振+惯性阻尼滤波, \
                 比西方现代地震仪早1700年。可视为LIGO引力波探测器的机械原型(共振+放大+滤波三阶段)",
                TaskType::Research,
                0.87,
            ),
            (
                "马王堆帛书天文文献(前168年): 长沙马王堆三号墓出土。五星占=约8000字, 记述金木水火土五星运行, \
                 金星会合周期准确到584.4天(今测583.92天, 误差<0.08%)。29幅彗星图(世界上最早彗星分类系统,\
                 比欧洲早1700年)。导引图44式(气功/导引动作, 世界最早医疗体操图谱)。\
                 五星占证明中国在汉初已掌握行星轨道参数精密计算",
                TaskType::Research,
                0.89,
            ),
            (
                "邵雍皇极经世(11世纪): 129600年宇宙大循环周期(元=129600年, 会=10800年, 运=360年, 世=30年)。\
                 十二消息卦(复临泰壮夬乾姤遁否观测坤)映射12会, 阴阳消长的时间对称群。\
                 一元12会×10800=129600=60×2160=60甲子×360°(周天)。核心: 时间和历史是12×30×360的分形递归。\
                 与彭罗斯共形循环宇宙(CCC)惊人一致: 每个aeon=宇宙大爆炸到热寂=邵雍'一元'周期, \
                 aeon边界=邵雍'混沌初开'。64卦的384爻=129600的360°映射→64×6×337.5=129600",
                TaskType::Research,
                0.92,
            ),
            (
                "大衍之数50其用49: 《周易·系辞》最神秘的数学命题。大衍之数50, 其用40有9(揲蓍法)。\
                 40有9=大衍之数50-6(六爻)=44(分二挂一为49?)。赵爽(三国)勾股弦证法: 3²+4²+5²=9+16+25=50。\
                 焦蔚芳(1990s): 二项式定理(1+1)^n展开=C(n,0)+C(n,1)+...+C(n,n), 令n=6得∑C(6,k)=64(六十四卦), \
                 当排除乾纯阳(/纯阴)和坤纯阴时C(6,0)=1(坤)和C(6,6)=1(乾)排除得62+1(变化中心)=63=50+13? \
                 更直接: 49=7²(洛书七衡), 50=5²×2(河图天数地数各5个), 50-1=49对应整体对称中破缺1维=\
                 U(1)超荷守恒→SU(3)×SU(2)对称性破缺。天道=50(\"物之本\"), 人道=49(\"物之用\")=\
                 量子测量中希尔伯特空间(50维)→观测空间(49维, 损失1维=测量坍缩)",
                TaskType::Research,
                0.94,
            ),
        ];

        for (desc, task_type, reward) in cosmology_knowledge {
            let memory = ReasoningMemory::new(desc, task_type, &[], reward);
            self.store(memory);
        }
    }

    pub fn initialize_with_awesome_design_knowledge(&mut self) {
        let design_knowledge = vec![
            ("Figma 设计 Token 系统: 颜色/字体/间距/阴影统一通过 JSON token 管理，组件引用 token 而非硬编码值", TaskType::Design, 0.92),
            ("色彩系统: HSL 色相环 60° 邻近色/120° 对比色/180° 互补色; 8:1 对比度 WCAG AAA; 语义色(primary/success/warning/danger)禁止直接使用", TaskType::Design, 0.9),
            ("排版系统: 1.25 比例 Major Third Scale; 最小 12px; 行高 1.5 body/1.2 heading; 字重 400/500/600/700 四档", TaskType::UIDesign, 0.88),
            ("间距网格: 8px 基准单位; 4/8/12/16/24/32/48/64 幂次网格; 组件内外间距固定为基准倍数", TaskType::UIDesign, 0.85),
            ("交互动效: 入场 200-300ms ease-out; 出场 150-200ms ease-in; 微交互相应 <100ms; 缓动函数 cubic-bezier(0.4, 0, 0.2, 1)", TaskType::Design, 0.85),
            ("图标系统: 24x24 SVG viewBox; stroke-width 1.5/2; 圆角 cap; 语义色跟随文本色; 装饰图标 aria-hidden", TaskType::UIDesign, 0.82),
            ("组件解剖学: 每个组件由 container/label/input/helperText/error 五层构成; 每层独立 spacing/token 控制", TaskType::CodeGeneration, 0.87),
            ("无障碍模式: 非文本元素 aria-label; 焦点可见 outline:2px; 语义 HTML 标签; 色盲安全配色 2 种方案", TaskType::CodeReview, 0.85),
        ];

        for (desc, task_type, reward) in &design_knowledge {
            let memory = ReasoningMemory::new(desc, *task_type, &[], *reward);
            self.store(memory);
        }
        log::info!("[knowledge] AwesomeDesignSkills: {} seed memories injected", design_knowledge.len());
    }
}

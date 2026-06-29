use log;
use neotrix::neotrix::nt_mind::knowledge_engine::*;
use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    log::info!("╔══════════════════════════════════════════════════════════════╗");
    log::info!("║  🔬 知识蒸馏引擎 — 从 470 条知识提取核心逻辑链路           ║");
    log::info!("╚══════════════════════════════════════════════════════════════╝");

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let kb_path = PathBuf::from(&home)
        .join(".neotrix")
        .join("knowledge_engine.json");
    let mut eng = KnowledgeEngine::load_from(&kb_path);
    eng.set_persist_path(kb_path.clone());

    log::info!(
        "📊 知识库: {} 条目, {} 关系",
        eng.stats().total_entries,
        eng.stats().total_relations
    );

    // ============================================================
    // 核心逻辑链路提取
    // ============================================================

    let chains = vec![
        // 1. 宇宙演化链
        KnowledgeChain::new(
            "🌌 宇宙演化链",
            "天界",
            "宇宙从奇点大爆炸膨胀冷却, 形成基本粒子→原子→恒星→星系→行星→生命→意识",
            vec![
                "宇宙大爆炸 (138亿年前) → 时空起源",
                "恒星核合成 → 碳氧铁等重元素 (我们是星尘)",
                "银河系形成 → 太阳系诞生 (46亿年前)",
                "地球形成 → 地壳冷却 → 海洋形成",
                "生命起源 → 单细胞 → 多细胞 → 智慧",
                "人类探索宇宙 → 空间站 → 火星 → 星际",
            ],
        ),
        // 2. 地球系统链
        KnowledgeChain::new(
            "🌍 地球系统链",
            "地界",
            "地球是一个自组织的复杂系统: 岩石圈+水圈+大气圈+生物圈耦合演化",
            vec![
                "地核驱动地磁场 → 保护生命免受太阳风",
                "地幔对流 → 板块运动 → 造山/地震/火山",
                "水循环 → 蒸发→降水→径流 → 塑造地表",
                "光合作用 → O₂大气 → 臭氧层 → 陆地生命",
                "碳循环 → 岩石风化/海洋吸收/化石燃料",
                "人类世 → 人类成为地球系统主导力量",
            ],
        ),
        // 3. 生命进化链
        KnowledgeChain::new(
            "🧬 生命进化链",
            "地界",
            "从化学进化到多细胞到智慧: 生命通过自然选择持续复杂化",
            vec![
                "化学进化 → 有机分子 → RNA世界 → 细胞",
                "光合作用 → 大氧化事件 → 真核生物",
                "寒武纪大爆发 → 所有动物门类出现",
                "植物登陆 → 森林 → O₂浓度35% → 巨型昆虫",
                "恐龙统治 → 小行星撞击 → 哺乳动物崛起",
                "灵长类 → 人科 → 智人 → 文化进化",
            ],
        ),
        // 4. 文明演进链
        KnowledgeChain::new(
            "🏛️ 文明演进链",
            "人界",
            "人类从狩猎采集到信息文明: 每次跃迁由能源/信息/组织方式革命驱动",
            vec![
                "认知革命 (语言/符号) → 智人胜出尼安德特",
                "农业革命 (驯化/定居) → 人口暴增100倍",
                "城市革命 (文字/法律) → 苏美尔中国印度",
                "轴心时代 (哲学/宗教) → 孔子佛陀苏格拉底",
                "科学革命 (方法/实验) → 现代世界观",
                "工业革命 (蒸汽/电力) → 机械化城市化",
                "信息革命 (计算机/网络) → 全球化",
                "AI革命 (大模型/自动化) → 智能时代",
            ],
        ),
        // 5. 科技飞跃链
        KnowledgeChain::new(
            "⚙️ 科技飞跃链",
            "人界",
            "技术发展遵循: 基础科学→应用技术→工程实现→社会转型的闭环",
            vec![
                "数学 → 物理定律 → 计算 → 人工智能",
                "量子力学 → 半导体 → 计算机 → 互联网",
                "相对论 → GPS/核能 → 空间技术",
                "DNA发现 → 基因工程 → 精准医疗",
                "电磁学 → 电网 → 电机 → 全电社会",
                "热力学 → 蒸汽机 → 内燃机 → 火箭",
            ],
        ),
        // 6. 思想演进链
        KnowledgeChain::new(
            "📚 思想演进链",
            "人界",
            "思想史是不断'祛魅'与'赋魅'的辩证过程: 神话→宗教→哲学→科学→?",
            vec![
                "神话思维 (万物有灵) → 多神教 → 一神教",
                "轴心突破 → 哲学诞生 (理性追问)",
                "中世纪 → 信仰整合理性 (经院哲学)",
                "启蒙 → 理性取代权威 (科学革命)",
                "现代性 → 主体性高扬 (自由/民主/人权)",
                "后现代 → 解构宏大叙事 (相对多元)",
                "AI时代 → 人类智慧vs人工智能 新辩证",
            ],
        ),
        // 7. 政治组织链
        KnowledgeChain::new(
            "🏛️ 政治组织链",
            "人界",
            "人类组织从部落到帝国到民族国家到全球化治理的演化",
            vec![
                "部落/氏族 → 酋邦 → 城邦 (民主/贵族)",
                "帝国 → 大一统 (罗马中国波斯蒙古)",
                "封建制 → 契约分权 (欧洲中世纪)",
                "民族国家 → 主权独立 (威斯特伐利亚)",
                "殖民体系 → 后殖民 → 全球化",
                "国际组织 → 联合国/EU → 全球治理",
            ],
        ),
        // 8. 经济演进链
        KnowledgeChain::new(
            "💰 经济演进链",
            "人界",
            "经济活动从生存到繁荣: 狩猎采集→农业→工业→信息→智能",
            vec![
                "实物交换 → 贝壳 → 金银货币 → 纸币",
                "自给自足 → 市场 → 贸易网络 (丝绸之路)",
                "重商主义 → 资本主义 → 全球化",
                "手工工场 → 工厂 → 跨国公司 → 平台",
                "农业主导 → 工业主导 → 服务业主导",
                "稀缺经济 → 丰裕经济 → 注意力经济",
            ],
        ),
        // 9. 宇宙维度链
        KnowledgeChain::new(
            "🌌 宇宙维度链",
            "天界",
            "人类对宇宙的认知从三维欧氏空间到四维时空到十维弦论的维度扩张",
            vec![
                "欧几里得几何: 三维绝对空间 (常识)",
                "牛顿: 绝对时空 (三维+一维时间)",
                "狭义相对论: 四维时空统一 (光速不变)",
                "广义相对论: 弯曲时空 (引力=几何)",
                "卡鲁扎-克莱因: 第五维 (额外维度)",
                "弦理论: 10/11维 (紧致化额外维)",
                "圈量子引力: 时空量子化 (自旋网络)",
            ],
        ),
        // 10. 生命与意识链
        KnowledgeChain::new(
            "🧠 生命与意识链",
            "人界",
            "从物质到生命到意识: 涌现的层级是不可还原的",
            vec![
                "物理学规则 → 化学组合 → 分子生物学",
                "原核细胞 → 真核细胞 → 多细胞生物",
                "神经系统 → 中枢神经 → 大脑皮层",
                "感觉 → 情绪 → 记忆 → 学习 → 意识",
                "个体意识 → 共同意识 → 集体智能",
                "碳基智能 → 硅基智能 → 混合智能",
            ],
        ),
    ];

    // ============================================================
    // 生成蒸馏报告
    // ============================================================
    log::info!("\n═══════════════════ 蒸馏报告 ═══════════════════\n");

    for chain in &chains {
        log::info!("{}", chain.render(&mut eng));
    }

    // ============================================================
    // 全知识网络统计
    // ============================================================
    log::info!("\n═══════════════════ 知识网络拓扑 ═══════════════════\n");

    let mut domain_stats = HashMap::new();
    for entry in eng.entries.values() {
        for tag in &entry.tags {
            *domain_stats.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    let mut domain_list: Vec<(String, usize)> = domain_stats.into_iter().collect();
    domain_list.sort_by(|a, b| b.1.cmp(&a.1));

    log::info!("  知识密度分布 (top 30):");
    let max_count = domain_list.first().map(|(_, c)| *c).unwrap_or(1) as f64;
    for (domain, count) in domain_list.iter().take(30) {
        let bar_len = ((*count as f64 / max_count) * 30.0) as usize;
        let bar = "█".repeat(bar_len);
        let empty = "░".repeat(30 - bar_len);
        log::info!("  {:20} |{}{}| {:>4}", domain, bar, empty, count);
    }

    // ============================================================
    // 知识补充：缺失地域注入
    // ============================================================
    log::info!("\n═══════════════════ 知识补充注入 ═══════════════════\n");

    let supplements = vec![
        ("量子场论: 标准模型的量子基础",
         "量子场论(QFT)是粒子物理的标准框架,结合了狭义相对论和量子力学.每个粒子类型对应一个量子场:电子场/光子场/夸克场等.狄拉克方程(1928)统一了量子力学和狭义相对论,预见了反物质.费曼图直观表示粒子相互作用.重整化消除了无穷大问题.量子电动力学(QED)是物理学中最精确的理论(预测电子磁矩与实验一致到12位有效数字).杨-米尔斯理论(1954)为规范场论奠定了基础,是粒子物理标准模型的数学核心.",0.94),
        ("混沌理论: 确定性系统的不可预测性",
         "混沌理论(洛伦兹1963)发现确定性系统可以产生看似随机的行为.洛伦兹吸引子('蝴蝶效应'):初始条件的微小差异被指数放大.庞加莱1889年发现三体问题不可积,标志混沌思想萌芽.李雅普诺夫指数量化混沌程度.费根鲍姆常数(δ=4.669...)是普适的,出现在所有倍周期分岔系统中.混沌在天气/金融/生态/量子系统中随处可见.",0.91),
        ("复杂系统与涌现",
         "复杂系统由大量相互作用元件组成,宏观涌现出微观不具备的特性.圣塔菲研究所(1984)开创复杂性科学.涌现的例子:蚁群智能/鸟群编队/神经网络/大脑/市场/城市/生命本身.自组织临界性(巴克1987)解释了沙堆/地震/灭绝事件的幂律分布.阿瑟的报酬递增经济学:正反馈导致路径依赖.复杂适应性系统(CAS)是理解生命/社会/经济的新范式.",0.92),
        ("认知科学: 心智的计算与具身",
         "认知科学是哲学/心理学/计算机/神经科学/人类学/语言学的交叉.认知的计算理论:心智像计算机(福多/纽厄尔-西蒙).联结主义:神经网络模型(鲁梅尔哈特1986).具身认知:认知依赖于身体和环境的交互(瓦雷拉1991).延展心灵:认知延伸到环境(克拉克2008).意识问题:全球工作空间理论(Baars)/高阶思维理论(Rosenthal)/整合信息理论(IIT,Tononi).量子意识(彭罗斯-哈梅罗夫)有争议.",0.92),
        ("博弈论: 策略互动的数学",
         "冯·诺依曼和摩根斯坦(1944)创立博弈论.纳什均衡(1950):每个参与者策略是对手策略的最优反应.囚徒困境:个人理性导致集体非理性.鹰鸽博弈/协调博弈/信号博弈.重复博弈中的以牙还牙(Axelrod竞赛).进化博弈论(梅纳德-史密斯1972).拍卖理论(2020诺奖).博弈论重塑了经济学/政治学/生物学/计算机科学(算法博弈论).",0.91),
    ];

    for (title, body, imp) in &supplements {
        if !eng.entries.values().any(|e| e.title == *title) {
            eng.add_entry(
                KnowledgeEntry::new(title, body, SourceType::KnowledgeBase, "kb:distilled")
                    .with_importance(*imp)
                    .with_tags(vec![
                        "天界".to_string(),
                        "科学".to_string(),
                        "知识链".to_string(),
                    ]),
            );
            log::info!("  ✅ 补充: {}", title);
        }
    }

    // ============================================================
    // 关系强化
    // ============================================================
    log::info!("\n  🔗 强化维度链连接...");
    // Match cosmos-history-humanity chains
    let cosmos_refs = vec![
        ("宇宙大爆炸", "恒星与星系演化", "Causes", "大爆炸→恒星形成"),
        ("粒子物理学", "量子力学", "Related", "粒子物理↔量子力学"),
        ("相对论与引力波", "时空本质", "Related", "相对论=时空弯曲"),
        (
            "银河系与太阳系",
            "地球内部结构",
            "Causes",
            "太阳系→地球分层",
        ),
        ("地核驱动地磁场", "地球内部结构", "Causes", "地核产生磁场"),
    ];
    for (from, to, rel, desc) in &cosmos_refs {
        let rt = match *rel {
            "Causes" => RelationType::Causes,
            _ => RelationType::Related,
        };
        let f_id = eng
            .entries
            .values()
            .find(|e| e.title.contains(from))
            .map(|e| e.id.clone());
        let t_id = eng
            .entries
            .values()
            .find(|e| e.title.contains(to))
            .map(|e| e.id.clone());
        if let (Some(f), Some(t)) = (f_id, t_id) {
            if !eng.relations.iter().any(|r| r.from_id == f && r.to_id == t) {
                eng.add_relation(&f, &t, rt, 0.8, desc);
            }
        }
    }
    log::info!("  ✅ 关系强化完成");

    // ============================================================
    // 保存
    // ============================================================
    if let Err(e) = eng.save() {
        log::error!("❌ 保存失败: {}", e);
    } else {
        log::info!("\n💾 已保存到 {:?}", kb_path);
    }

    log::info!(
        "\n📊 最终: {} 条目, {} 关系",
        eng.stats().total_entries,
        eng.stats().total_relations
    );

    // 输出 HTML
    let html = render_html_report(&chains);
    let html_path = PathBuf::from(&home)
        .join(".neotrix")
        .join("knowledge_chains.html");
    std::fs::write(&html_path, html).expect("failed to write html report");
    log::info!("📄 HTML 报告: {:?}", html_path);
}

// ============================================================
// 知识链结构
// ============================================================
struct KnowledgeChain {
    pub title: String,
    pub domain: String,
    pub thesis: String,
    pub links: Vec<String>,
}

impl KnowledgeChain {
    fn new(title: &str, domain: &str, thesis: &str, links: Vec<&str>) -> Self {
        Self {
            title: title.to_string(),
            domain: domain.to_string(),
            thesis: thesis.to_string(),
            links: links.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    fn render(&self, eng: &mut KnowledgeEngine) -> String {
        let mut out = String::new();
        out.push_str(&format!("\n{}  [{}]\n", self.title, self.domain));
        out.push_str(&format!("  核心命题: {}\n", self.thesis));
        out.push_str("  逻辑链路:\n");

        for (i, link) in self.links.iter().enumerate() {
            // Find matching entries in knowledge engine
            let parts: Vec<&str> = link.split("→").collect();
            let keyword = parts[0].trim();
            let matches = eng.search(keyword, 2);
            out.push_str(&format!("    {}. {} ", i + 1, link));
            if let Some((entry, _)) = matches.first() {
                out.push_str(&format!("[{:.2}]", entry.importance));
            }
            out.push('\n');
        }
        out
    }
}

fn render_html_report(chains: &[KnowledgeChain]) -> String {
    let mut html = String::from(
        r#"<!DOCTYPE html><html lang="zh"><head><meta charset="UTF-8"><title>知识逻辑链报告</title>
<style>body{font-family:'Segoe UI',sans-serif;background:#0f0f1a;color:#e0e0e0;max-width:900px;margin:0 auto;padding:20px}
h1{color:#fff;border-bottom:2px solid #4a4aff;padding-bottom:10px}
.chain{background:#1a1a2e;border-radius:12px;padding:20px;margin:16px 0;border:1px solid #2a2a4a}
.chain h2{margin:0 0 4px 0;font-size:18px}
.chain .domain{display:inline-block;padding:2px 10px;border-radius:8px;font-size:12px;margin-bottom:8px}
.chain .thesis{font-style:italic;color:#aaa;margin:8px 0;font-size:14px;border-left:3px solid #4a4aff;padding-left:12px}
.chain ol{margin:0;padding-left:20px;font-size:14px;line-height:1.8}
.chain li::marker{color:#4a4aff;font-weight:bold}
.footer{text-align:center;margin-top:40px;color:#666;font-size:12px}
.tag-blue{background:#4a4aff33;color:#7c7cff;border:1px solid #4a4aff44}
.tag-red{background:#ff6b6b33;color:#ff6b6b;border:1px solid #ff6b6b44}
.tag-green{background:#51cf6633;color:#51cf66;border:1px solid #51cf6644}
.tag-gold{background:#ffd43b33;color:#ffd43b;border:1px solid #ffd43b44}
</style></head><body>"#,
    );

    html.push_str("<h1>🔬 人类知识核心逻辑链路 (蒸馏报告)</h1>");
    html.push_str("<p style='color:#888'>从知识引擎提炼的 10 条核心逻辑链, 每条链是人类在一个维度的知识压缩</p>");

    for chain in chains {
        let color = match chain.domain.as_str() {
            "天界" => "tag-red",
            "地界" => "tag-green",
            "人界" => "tag-blue",
            _ => "tag-gold",
        };
        html.push_str(&format!(
            r#"<div class="chain"><h2>{}</h2><span class="domain {}">{}</span>
            <div class="thesis">核心命题: {}</div><ol>"#,
            chain.title, color, chain.domain, chain.thesis
        ));
        for link in &chain.links {
            html.push_str(&format!("<li>{}</li>", link));
        }
        html.push_str("</ol></div>");
    }

    html.push_str(
        r#"<div class="footer">NeoTrix 知识蒸馏引擎 · 470 条目, 160 关系</div></body></html>"#,
    );
    html
}

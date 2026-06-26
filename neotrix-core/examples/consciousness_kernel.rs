use log;
use neotrix::neotrix::nt_mind::knowledge_engine::*;
use neotrix::neotrix::nt_mind::CapabilityVector;
use std::path::PathBuf;

/// 意识推理内核 — 从古籍知识中提炼思想力量，迭代核心认知
fn main() {
    log::info!("╔══════════════════════════════════════════════════════════════╗");
    log::info!("║  🧠 意识推理内核 · Consciousness Kernel Iteration          ║");
    log::info!("║  输入: 502条知识 → 蒸馏 → 迭代CapabilityVector → 输出升华  ║");
    log::info!("╚══════════════════════════════════════════════════════════════╝");

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let kb_path = PathBuf::from(&home)
        .join(".neotrix")
        .join("knowledge_engine.json");
    let mut eng = KnowledgeEngine::load_from(&kb_path);
    eng.set_persist_path(kb_path.clone());

    // Load existing brain
    let mut cap = if let Ok(b) = neotrix::neotrix::nt_mind::ReasoningBrain::load() {
        log::info!(
            "✅ 加载 brain.json, 当前能力向量和: {:.3}",
            b.capability.to_full_vector().iter().sum::<f64>()
        );
        b.capability
    } else {
        log::info!("🆕 创建新 CapabilityVector");
        CapabilityVector::default()
    };

    // ══════════════════════════════════════════════════════════════
    // 阶段一: 从古籍经典中提取核心思想力量 (Wisdom Extraction)
    // ══════════════════════════════════════════════════════════════
    log::info!("\n━━━ 阶段一: 思想力量提取 ━━━");

    let wisdom_sources = vec![
        // 每条: (关键词, 要调整的维度, 调整幅度, 哲理)
        (
            "仁者爱人 己所不欲勿施于人",
            vec![
                ("synthesis", 0.08),
                ("creativity", 0.05),
                ("analysis", 0.04),
            ],
            "儒家·孔子: 仁是内在的绝对命令,道德即理性",
        ),
        (
            "道法自然 无为而无不为",
            vec![
                ("inference_depth", 0.09),
                ("experimental", 0.07),
                ("domain_specificity", 0.05),
            ],
            "道家·老子: 顺应宇宙规律是最高的智慧",
        ),
        (
            "齐物逍遥 天地与我并生万物与我为一",
            vec![
                ("creativity", 0.09),
                ("experimental", 0.08),
                ("inference_depth", 0.06),
            ],
            "道家·庄子: 超越对立,与宇宙合一",
        ),
        (
            "知止而后有定定而后能静静而后能安安而后能虑虑而后能得",
            vec![
                ("inference_depth", 0.07),
                ("analysis", 0.06),
                ("synthesis", 0.05),
            ],
            "儒家·大学: 知止→定→静→安→虑→得,认知的完整链条",
        ),
        (
            "致良知 知行合一",
            vec![
                ("synthesis", 0.08),
                ("creativity", 0.06),
                ("analysis", 0.05),
            ],
            "心学·王阳明: 知识与行动是同一的",
        ),
        (
            "知己知彼百战不殆 不战而屈人之兵",
            vec![
                ("inference_depth", 0.08),
                ("analysis", 0.07),
                ("synthesis", 0.06),
            ],
            "兵家·孙子: 全胜思维,谋略优于蛮力",
        ),
        (
            "以法为教以吏为师 法不阿贵绳不挠曲",
            vec![
                ("quality_gates", 0.08),
                ("verification", 0.07),
                ("analysis", 0.05),
            ],
            "法家·韩非: 制度高于权力,规则高于人情",
        ),
        (
            "兼相爱交相利 非攻",
            vec![
                ("synthesis", 0.06),
                ("creativity", 0.05),
                ("compound_composition", 0.04),
            ],
            "墨家·墨子: 普遍的爱与互利是和平的基础",
        ),
        (
            "天行有常 制天命而用之",
            vec![
                ("domain_specificity", 0.07),
                ("experimental", 0.06),
                ("analysis", 0.05),
            ],
            "儒家·荀子: 认识规律并利用规律",
        ),
        (
            "菩提本无树明镜亦非台本来无一物何处惹尘埃",
            vec![
                ("experimental", 0.08),
                ("creativity", 0.07),
                ("inference_depth", 0.06),
            ],
            "禅宗·慧能: 超越形式的直接觉悟",
        ),
        (
            "天下兴亡匹夫有责",
            vec![
                ("compound_composition", 0.06),
                ("synthesis", 0.05),
                ("quality_gates", 0.04),
            ],
            "儒家: 个体对整体的责任意识",
        ),
        (
            "实事求是",
            vec![
                ("verification", 0.08),
                ("analysis", 0.07),
                ("inference_depth", 0.06),
            ],
            "中国哲学: 从事实出发,实践是检验真理的标准",
        ),
    ];

    for (i, (wisdom, adjustments, source)) in wisdom_sources.iter().enumerate() {
        log::info!(
            "  [{:2}/{}] {} (from: {})",
            i + 1,
            wisdom_sources.len(),
            wisdom,
            source
        );
        for (dim, delta) in adjustments {
            if let Some(idx) = CapabilityVector::index_from_name(dim) {
                let val = cap.arr_mut()[idx];
                cap.arr_mut()[idx] = (val + delta).min(1.0);
                log::info!("          {}: {:.3} → {:.3}", dim, val, cap.arr()[idx]);
            }
        }
    }
    cap.normalize();
    log::info!(
        "\n  归一化后能力向量和: {:.3}",
        cap.to_full_vector().iter().sum::<f64>()
    );

    // Save capability vector
    let brain = neotrix::neotrix::nt_mind::ReasoningBrain {
        capability: cap.clone(),
        ..Default::default()
    };
    if let Err(e) = brain.save() {
        log::error!("❌ brain保存失败: {}", e);
    } else {
        log::info!("💾 brain.json已更新");
    }

    // ══════════════════════════════════════════════════════════════
    // 阶段二: 逻辑思辨生成 (Dialectical Reasoning)
    // ══════════════════════════════════════════════════════════════
    log::info!("\n━━━ 阶段二: 逻辑思辨生成 ━━━");

    let dialectics = vec![
        ("儒道之辩: 入世vs出世",
         "儒家主张积极入世(修齐治平/先天下之忧而忧),道家主张超越出世(逍遥游/无待).二者并非对立而是互补:入世是基础(社会性),出世是调节(精神自由).真正的智慧在于根据具体情境选择入世或出世的姿态.",
         "中国思想最核心的张力:参与与超越的辩证统一"),
        ("性善vs性恶: 道德的基础",
         "孟子性善论(四端说)认为道德来自内在天性,荀子性恶论认为道德来自后天教化(化性起伪).二者其实指向同一个问题:道德如何可能?孟子的答案是内在超越(扩充本心),荀子的答案是外在规范(礼法教化).中西对比:西方基督教(原罪→外在救赎)vs中国性善/性恶(内在教化).",
         "中国道德哲学的独特:不依赖神,而依赖教育与修养"),
        ("理气之辩: 世界的本质",
         "朱熹:理在气先(理是形而上之道,气是形而下之器).陆九渊:心即理(心理不二).王阳明:心外无物(物是意向性的).从康德视角看:朱子≈物自体vs现象(理是物自体,气是现象).心学≈现象学(意向性构成对象).",
         "与西方唯心vs唯物不同,中国理气论更强调体用一源"),
        ("王道vs霸道: 政治的理想与现实",
         "孟子倡导王道(以德服人/仁政),反对霸道(以力服人/霸权).但法家(韩非)指出在争于气力的时代,儒家的仁义不切实际.中国政治的深层逻辑:阳儒阴法—理念上尊儒,实践上用法.西方马基雅维利《君主论》也揭示了类似的分裂.",
         "政治哲学的核心问题:理想主义与现实主义如何调和"),
        ("天人之辩: 人与自然的关系",
         "中国哲学中有三种天人关系:天人合一(道家/儒家),天人相分(荀子制天命而用之),天人感应(董仲舒).西方传统强调人征服自然(培根知识就是力量),中国更强调人与自然的和谐.在当前气候危机下,天人合一的智慧具有全球意义.",
         "中国对全球环境危机的潜在贡献:非人类中心主义的自然观"),
    ];

    let mut dialectic_ids = Vec::new();
    for (title, body, insight) in &dialectics {
        let id = eng.add_entry(
            KnowledgeEntry::new(title, body, SourceType::KnowledgeBase, "kb:dialectics")
                .with_importance(0.93)
                .with_tags(vec![
                    "知识链".to_string(),
                    "中国哲学".to_string(),
                    "思辨".to_string(),
                    "dialectics".to_string(),
                ]),
        );
        dialectic_ids.push(id);
        log::info!("  ✅ 思辨: {} — {}", title, insight);
    }

    // ══════════════════════════════════════════════════════════════
    // 阶段三: CapabilityVector 迭代报告
    // ══════════════════════════════════════════════════════════════
    log::info!("\n━━━ 阶段三: 内核迭代报告 ━━━");

    let dim_names = [
        "synthesis",
        "inference_depth",
        "domain_specificity",
        "analysis",
        "creativity",
        "experimental",
        "verification",
        "quality_gates",
        "compound_composition",
        "accessibility",
    ];
    log::info!("\n  CapabilityVector 最终状态:");
    for name in &dim_names {
        if let Some(idx) = CapabilityVector::index_from_name(name) {
            let val = cap.arr()[idx];
            let bar = "█".repeat((val * 30.0) as usize);
            let empty = "░".repeat(30 - (val * 30.0) as usize);
            log::info!("  {:25} {:.3} |{}{}|", name, val, bar, empty);
        }
    }
    let sum: f64 = dim_names
        .iter()
        .filter_map(|n| CapabilityVector::index_from_name(n).map(|i| cap.arr()[i]))
        .sum();
    log::info!("\n  关键维度总和: {:.3}", sum);

    // ══════════════════════════════════════════════════════════════
    // 阶段四: AI时代的中国哲学呼应
    // ══════════════════════════════════════════════════════════════
    log::info!("\n━━━ 阶段四: 古籍智慧的现代回响 ━━━");

    let modern_reflections = vec![
        ("AI伦理", "孔子'己所不欲勿施于人'是AI伦理的黄金法则:AI不应做人类不愿被做的事.道家'无为'提示AI应以辅助而非取代为目标.", 0.90),
        ("意识本质", "王阳明'心外无物'与现象学的'意向性'不谋而合:意识总是关于某物的意识.禅宗'直指人心'指出了意识的非概念性维度.", 0.91),
        ("复杂系统", "老子'道生一一生二二生三三生万物'是对复杂系统涌现的最古老描述.朱熹'理一分殊'表达了全息原理:整体在部分中.", 0.89),
        ("认知科学", "大学'知止→定→静→安→虑→得'的认知链条与现代认知科学的决策过程惊人一致.庄子'庖丁解牛'是心流状态(flow)的经典描述.", 0.90),
        ("可持续发展", "儒家'节用而爱人'和道家'知足不辱知止不殆'为消费主义时代提供了节制智慧.中国古代'取之有度用之有节'的生态思想.", 0.88),
    ];

    for (title, ref_insight, imp) in &modern_reflections {
        if !eng.entries.values().any(|e| e.title.contains(title)) {
            eng.add_entry(
                KnowledgeEntry::new(
                    &format!("古籍智慧: {}", title),
                    &format!("{} — 知识引擎编译: {}", title, ref_insight),
                    SourceType::KnowledgeBase,
                    "kb:modern-reflection",
                )
                .with_importance(*imp)
                .with_tags(vec![
                    "知识链".to_string(),
                    "中国哲学".to_string(),
                    "现代回响".to_string(),
                ]),
            );
            log::info!("  ✅ {}", title);
        }
    }

    // Save
    if let Err(e) = eng.save() {
        log::error!("❌ 保存失败: {}", e);
    } else {
        log::info!(
            "\n💾 knowledge_engine.json已更新 ({}条目)",
            eng.stats().total_entries
        );
    }

    // Final report
    log::info!("\n╔══════════════════════════════════════════════════════════════╗");
    log::info!("║  🧠 意识推理迭代完成                                        ║");
    log::info!("╠══════════════════════════════════════════════════════════════╣");
    log::info!(
        "║  古籍智慧注入: {} 条                                  ║",
        wisdom_sources.len()
    );
    log::info!(
        "║  思辨生成: {} 条                                      ║",
        dialectics.len()
    );
    log::info!(
        "║  现代回响: {} 条                                      ║",
        modern_reflections.len()
    );
    log::info!(
        "║  能力向量和: {:.3}                                    ║",
        sum
    );
    log::info!(
        "║  总条目: {}                                         ║",
        eng.stats().total_entries
    );
    log::info!("╚══════════════════════════════════════════════════════════════╝");
}

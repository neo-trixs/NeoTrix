use log;
use neotrix::neotrix::nt_mind::knowledge_engine::*;
use neotrix::neotrix::nt_mind::CapabilityVector;
use std::path::PathBuf;

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let kb_path = PathBuf::from(&home)
        .join(".neotrix")
        .join("knowledge_engine.json");
    let mut eng = KnowledgeEngine::load_from(&kb_path);
    eng.set_persist_path(kb_path.clone());
    let before = eng.stats().total_entries;

    // GitHub discoveries from recent search
    let discovered: Vec<(&str, &str, Vec<&str>, f64)> = vec![
        ("chinese-poetry: 最全中华古诗词数据库 (40.8k stars)",
         "github.com/chinese-poetry/chinese-poetry 是最全中文诗歌古典文集数据库,40.8k星。\n包含:5.5万首唐诗、26万首宋诗、2.1万宋词。唐宋两朝近1.4万古诗人,两宋1564位词人。\n数据格式:JSON分发。附带唐诗宋词高频词分析、作品排行榜。\n衍生应用:全唐诗Android版、诗词周历、诗词桌面。\n四大名著情景地图:西游记取经路线、大观园图、三国鼎立地图、水浒聚义图。",
         vec!["GitHub","开源","诗歌","文学","知识链"], 0.96),

        ("chtxt: 中华经典古籍精校系列 (139 stars)",
         "github.com/JasonWade001/chtxt 包含中华经典古籍精校、诗词,四书五经、四大名著、诗经、楚辞、全唐诗、全宋词、唐诗三百首、宋詞三百首、二十四史...\n支持全文搜索,是古典文献研究的实用工具。\n提供了校勘后的高质量文本,比网络流传的错漏版本更可靠。",
         vec!["GitHub","古籍","开源"], 0.94),

        ("taisho-translation: 大正藏英译项目",
         "github.com/dangerzig/taisho-translation 包含大正新修大藏经(大正藏)的英文翻译。\n大正藏是国际佛学界最广泛使用的汉文大藏经版本。\n该项目使用AI辅助翻译,包含完整的词汇表(glossary)和索引。\n对非中文读者研究佛学提供了宝贵资源。\n目录文件:full_catalog.json(完整目录),master_glossary.md(主词汇表)。",
         vec!["GitHub","佛藏","翻译"], 0.92),

        ("my-TCM-textbook-website: 中医教材网站 (87 stars)",
         "github.com/ChiryuhLii/my-TCM-textbook-website 是一个基于Obsidian的中医教材交互网站。\n可在线访问:qiuxiandongshou.com\n采用MIT开源协议,包含MarkDown格式的中医教材内容。\n版权归中国中医药出版社所有,本项目仅作为前端技术Demo与个人学习笔记。\n重要:该网站可作为辅助学习工具,不能替代原版教材。",
         vec!["GitHub","中医","开源"], 0.89),

        ("qimen-go: 奇门遁甲Go语言实现",
         "github.com/deminzhang/qimen-go 用Go语言实现奇门遁甲排盘。\n支持:转盘/飞盘/鸣法排盘。鸣法以满盘转时干为暗干。\n鸣法九星只顺不逆,锁定拆补法。\n时家带大六壬的天地盘。\n功能:附八字排盘、梅花易数时盘、大六壬排四课三传。\nNASA数据缓存到本地sqlite。\n技术栈:Ebiten 2D游戏引擎(跨平台),支持Android打包。",
         vec!["GitHub","术数","奇门","开源"], 0.92),

        ("daozang-english: 道藏英译项目(1676部)",
         "github.com/Maximilian-Winter/daozang-english 包含道藏1676部经典的英文翻译。\n使用AI辅助翻译(Mistral Large 2512 + Mistral 14b),分块方式:语义+标点+标记。\n来源:github.com/wenyuange/dao(道藏中文源文本)。\n包含Python工具:创建双语SQL数据库,支持双语全文搜索。\n重要警示:内含内丹/符咒/召神等实践内容,需在明师指导下学习。\n提示:这可能未经同行评议,仅供学术研究参考。",
         vec!["GitHub","道藏","翻译","开源"], 0.93),
    ];

    let mut count = 0;
    for (title, body, tags, imp) in &discovered {
        if !eng
            .entries
            .values()
            .any(|x| x.title.contains(title) && title.len() > 8)
        {
            eng.add_entry(
                KnowledgeEntry::new(
                    title,
                    body,
                    SourceType::KnowledgeBase,
                    "kb:github-discovery",
                )
                .with_importance(*imp)
                .with_tags(tags.iter().map(|s| s.to_string()).collect()),
            );
            count += 1;
        }
    }
    log::info!("  ✅ 新增 {} 条GitHub资源发现", count);

    // Cross references
    let x = vec![
        ("chinese-poetry", "全唐诗三百首", "Related", "唐诗大数据"),
        (
            "daozang-english",
            "正统道藏三洞体系",
            "Related",
            "道藏英译项目",
        ),
        (
            "my-TCM-textbook-website",
            "八纲辨证",
            "Related",
            "中医教材数字版",
        ),
        ("qimen-go", "奇门遁甲预测法", "Related", "Go实现奇门排盘"),
        (
            "taisho-translation",
            "大正藏结构详解",
            "Related",
            "大正藏英译",
        ),
    ];
    for (f, t, r, d) in &x {
        let fi = eng
            .entries
            .values()
            .find(|x| x.title.contains(f))
            .map(|x| x.id.clone());
        let ti = eng
            .entries
            .values()
            .find(|x| x.title.contains(t))
            .map(|x| x.id.clone());
        if let (Some(ff), Some(tt)) = (fi, ti) {
            eng.add_relation(&ff, &tt, RelationType::Related, 0.7, d);
        }
    }
    log::info!("  ✅ 交叉关系");

    // Capability iteration
    let mut cap = if let Ok(b) = neotrix::neotrix::nt_mind::ReasoningBrain::load() {
        b.capability
    } else {
        CapabilityVector::default()
    };
    for (d, v) in &[("domain_specificity", 0.05)] {
        if let Some(idx) = CapabilityVector::index_from_name(d) {
            *cap.arr_mut().get_mut(idx).expect("index out of bounds") =
                (cap.arr()[idx] + v).min(1.0);
        }
    }
    cap.normalize();
    let _ = neotrix::neotrix::nt_mind::ReasoningBrain {
        capability: cap,
        ..Default::default()
    }
    .save();
    if let Err(e) = eng.save() {
        log::error!("❌{}", e);
    }
    log::info!(
        "\n💾 知识引擎: {}条目(+{}), {}关系",
        eng.stats().total_entries,
        eng.stats().total_entries - before,
        eng.stats().total_relations
    );

    log::info!("\n━━━ 发现的GitHub资源索引 ━━━");
    for (t, _, _, _) in &discovered {
        log::info!("  • {}", "你可以在浏览器中打开查看项目详情");
    }
}

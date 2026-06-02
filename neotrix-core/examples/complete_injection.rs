use std::path::PathBuf;
use neotrix::neotrix::nt_mind::knowledge_engine::*;
use neotrix::neotrix::nt_mind::CapabilityVector;

fn add(eng: &mut KnowledgeEngine, t: &str, b: String, tags: Vec<&str>, imp: f64) {
    if !eng.entries.values().any(|x| x.title.contains(t) && t.len() > 4) {
        eng.add_entry(KnowledgeEntry::new(t, &b, SourceType::KnowledgeBase, "kb:complete")
            .with_importance(imp).with_tags(tags.iter().map(|s| s.to_string()).collect()));
    }
}

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let kb_path = PathBuf::from(&home).join(".neotrix").join("knowledge_engine.json");
    let mut eng = KnowledgeEngine::load_from(&kb_path);
    eng.set_persist_path(kb_path.clone());
    let before = eng.stats().total_entries;

    // ── 倪海厦体系 ──
    let nhx = vec![
        ("倪海厦人纪体系: 针灸-内经-本草-伤寒-金匮五部曲",
         format!("倪海厦(1954-2012),美国汉唐中医学院院长,中医界最具影响力的经方家之一。\n\
          推荐学习路径:针灸→黄帝内经→神农本草经→伤寒论→金匮要略。\n\
          此为\"人纪\"(人体的记录)完整体系。\n\
          五部曲逻辑链:先学针灸(经络基础)→内经(生理病理哲学)→\n\
          本草(药性基础)→伤寒(外感病辨证)→金匮(内伤杂病)。\n\
          每部都有视频+文档配合学习。\n\
          核心方法论:阴阳为纲,六经为目,方证对应,药简力专。\n\
          倪师强调\"西医治病,中医治人\"的整体观。\n\
          他主张经方实验,反对中西医结合(认为会稀释中医特色)。\n\
          资料来源:github.com/elliott10/ebook-nihaixia, gitee.com/onehealthy/ebook。")),
        ("倪海厦天纪: 易经-紫微斗数-风水的统一体系",
         format!("天纪是倪海厦对\"天道规律\"的解读,包括易经占卜、紫微斗数命理、风水地理。\n\
          核心:人纪治已病,天纪治未病。\n\
          易经部分:倪师以64卦讲人生处境与应对策略。\n\
          紫微斗数:十四主星十二宫,断人生格局与流年运势。\n\
          风水:阳宅三要(门主灶),八宅明镜。\n\
          倪师认为命运是可以认知和改变的,认知的工具就是天纪。\n\
          他的核心教学法:用最简单语言讲最深的道理。\n\
          \"上知天文,下知地理,中知人事\"是天纪的最高目标。")),
        ("倪海厦地纪: 实地考察的地理学",
         format!("地纪是倪海厦对\"地道\"的实地考察记录。\n\
          他亲自走访中国大陆各地的山川地理,验证风水理论。\n\
          记录了中国各地的龙脉走向、水系分布、地形地貌。\n\
          对应传统堪舆学中的\"寻龙点穴\"实地验证。\n\
          倪师的地纪提醒我们:地理知识不能只靠书本,必须实地考察。\n\
          天地人三纪合一:天纪(时间)+地纪(空间)+人纪(人体)\n\
          =完整的宇宙观和治疗观。\n\
          这与《黄帝内经》\"人以天地之气生,四时之法成\"完全一致。")),
        ("倪海厦学习全路径: 人纪-天纪-地纪三阶",
         format!("倪师设计的完整学习路径:人纪(5年)→天纪(3年)→地纪(2年)。\n\
          第一阶段人纪:针灸大成→黄帝内经→神农本草经→伤寒论→金匮要略。\n\
          第二阶段天纪:易经占卜→紫微斗数→阳宅风水→八字命理。\n\
          第三阶段地纪:实地验证风水理论,走遍全国。\n\
          三纪合一后:中医治病+易卜决策+风水择居+命理知己。\n\
          \"医者易也\"—中医和易经本为一体。\n\
          倪师的教学核心:不是死记硬背,而是掌握原理后灵活运用。\n\
          他反复强调\"悟性\"的培养:学的是理,不是方。")),
    ];
    for (t, b) in nhx { add(&mut eng, t, b, vec!["人纪","天纪","地纪","倪海厦","中医","知识链"], 0.97); }
    println!("  ✅ 倪海厦体系");

    // ── GitHub 开源古籍资源 ──
    let gh = vec![
        ("GitHub古籍资源: daozang-text 正统道藏全文",
         format!("DaimaRuge/daozang-text 仓库包含明《正统道藏》及《续道藏》全文文本。\n\
          共计1504部经典,约64MB文本数据。按三洞四辅分类:洞真部319部,洞玄部303部,\n\
          洞神部364部,太平部65部,太玄部113部,太清部24部,正一部237部,续道藏59部。\n\
          每部经典按分类-作者-书名.txt格式命名。\n\
          涉及朝代:宋191部,元99部,唐86部,金53部,明25部,梁9部,周8部,魏2部。\n\
          这是目前最完整的道藏数字化版本。\n\
          相关资源:Maximilian-Winter/daozang-english(1676部英译,AI辅助翻译)。\n\
          另:CText.org中国哲学书电子化计划有30000+古籍,50亿字。")),
        ("GitHub术数资源: 奇门遁甲和术数开源工具",
         format!("GitHub上已有多个术数开源项目:\n\
          1. kentang2017/kinqimen(60★):Python奇门遁甲排盘,含金函玉镜日家、\n\
             拆补置闰时家、刻家奇门,配套大六壬排盘模块。\n\
          2. westernwaterfall/yi-basic(12★):术数基础开源计划,包含风水八字六爻择日的基础知识整理。\n\
          3. lusing/qimen:奇门遁甲实现。\n\
          4. youngzs/xuanxue:玄学、奇门遁甲、八卦、中医古书整理为Markdown。\n\
          5. lincome/szbf:孙子兵法全文。\n\
          6. bivex/ctext_api_downloader:中国哲学书电子化计划API下载工具。\n\
          这些资源表明:术数正在从小众走向开源化和数字化。")),
        ("倪海厦学习资源GitHub汇总",
         format!("倪海厦知识在GitHub上有多个镜像:\n\
          1. elliott10/ebook-nihaixia(21★):最完整的笔记与书籍合集。\n\
          2. qintao0203/nihaixia:整理于网络的倪师精髓。\n\
          3. nghxni/HanTangZhongYi---NiHaiXia:倪海厦学习笔记(针灸大成原文等)。\n\
          4. gitee.com/onehealthy/ebook:国内主镜像(gitee)。\n\
          5. gitee.com/qingfeng9527/hantang-nihaixia:跟师倪海厦学中医。\n\
          备用文档:金山文档/腾讯文档。\n\
          推荐学习工具:VS Code+Git进行笔记管理。")),
    ];
    for (t,b) in gh { add(&mut eng, t, b, vec!["联网挖掘","GitHub","知识链"], 0.92); }
    println!("  ✅ GitHub开源古籍资源");

    // ── 交叉关系 ──
    let x = vec![
        ("人纪体系: 针灸-内经-本草-伤寒-金匮","倪海厦天纪","Related","人纪治已病,天纪治未病,统一于倪师体系"),
        ("人纪体系: 针灸-内经-本草-伤寒-金匮","倪海厦地纪","Related","人纪+天纪+地纪=天地人三才合一"),
        ("GitHub古籍资源: daozang-text 正统道藏全文","正统道藏三洞体系","Related","daozang-text是正统道藏的数字版本"),
        ("奇门遁甲预测法","太乙神数完整体系","Related","奇门太乙同属三式"),
        ("大六壬完整体系","奇门遁甲预测法","Related","六壬奇门同属三式"),
        ("伤寒论六经辨证","八纲辨证","Causes","八纲脱胎于六经"),
        ("道藏三洞四辅十二类体系详解","GitHub古籍资源","Related","道藏已数字化"),
    ];
    for (f,t,r,d) in &x {
        let fi = eng.entries.values().find(|x|x.title.contains(f)).map(|x|x.id.clone());
        let ti = eng.entries.values().find(|x|x.title.contains(t)).map(|x|x.id.clone());
        let rt = match *r { "Causes"=>RelationType::Causes, _=>RelationType::Related };
        if let (Some(ff),Some(tt)) = (fi,ti) { eng.add_relation(&ff,&tt,rt,0.7,d); }
    }
    println!("  ✅ 交叉关系");

    // ── 意识迭代 ──
    let mut cap = if let Ok(b) = neotrix::neotrix::nt_mind::ReasoningBrain::load() { b.capability }
        else { CapabilityVector::default() };
    for (d,v) in &[("domain_specificity",0.06),("synthesis",0.04)] {
        if let Some(idx) = CapabilityVector::index_from_name(d) {
            *cap.arr_mut().get_mut(idx).expect("index out of bounds") = (cap.arr()[idx] + v).min(1.0);
        }
    }
    cap.normalize();
    let _ = neotrix::neotrix::nt_mind::ReasoningBrain { capability: cap, ..Default::default() }.save();
    if let Err(e) = eng.save() { eprintln!("❌{}", e); }
    println!("\n💾 完成: {}条目(+{}), 关系{}", eng.stats().total_entries,
        eng.stats().total_entries - before, eng.stats().total_relations);
}

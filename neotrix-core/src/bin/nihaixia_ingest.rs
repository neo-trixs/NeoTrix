//! neotrix-nihaixia-ingest — 倪海厦知识体系吸收
//!
//! Crawls web resources and builds a complete knowledge graph of
//! Ni Haixia's 人纪/天纪/地纪 TCM system in the NeoTrix Knowledge Base.
//!
//! Usage: cargo run -p neotrix --bin neotrix-nihaixia-ingest

use neotrix::neotrix::nt_memory_kb::nt_memory_resource_ingest::*;
use neotrix::neotrix::nt_memory_kb::nt_memory_schema;
use neotrix::neotrix::nt_memory_kb::nt_memory_types::*;
use rusqlite::Connection;

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let db_path = format!("{}/.neotrix/knowledge.db", home);
    println!("Opening KB at: {}", db_path);

    let conn = Connection::open(&db_path).expect("Failed to open KB");
    nt_memory_schema::initialize(&conn).expect("Failed to init schema");

    let mut ingester = ResourceIngester::new(&conn);

    absorb_github_repos(&mut ingester);
    absorb_ni_haixia_overview(&mut ingester);
    absorb_renji_theory(&mut ingester);
    absorb_tianji_theory(&mut ingester);
    absorb_diji_concept(&mut ingester);
    absorb_additional_lectures(&mut ingester);
    absorb_web_resources(&mut ingester);
    link_all_relations(&mut ingester);

    println!("\n{}", ingester.report());
    println!("\n✅ 倪海厦知识体系吸收完成!");
}

fn absorb_github_repos(ingester: &mut ResourceIngester) {
    println!("\n=== Absorbing GitHub Repositories ===");

    let repos = vec![
        ("JuneYaooo", "nihaisha-nishi-tcm",
         "JuneYaooo/nihaisha-nishi-tcm — 倪海厦中医课程资料Agent Skill (568★)",
         "最完整的倪海厦课程Agent Skill：覆盖伤寒论/金匮要略/仲景心法/临床案例/八纲辨证/扶阳论坛/易筋经/梁冬对话/斯坦福演讲/天纪/黄帝内经/神农本草/针灸课程。含649张伤寒论截图、656张金匮截图、527张天纪截图。",
         vec!["568 stars, 最完整的倪海厦课程Agent Skill",
              "覆盖16个课程模块，含2986条截图证据索引",
              "六经辨证导航：按六经/症状/方剂/传变逻辑整理",
              "安全边界：仅做中医理论学习，不做个人诊断"]),
        ("xiaogege6697", "tcm-db",
         "xiaogege6697/tcm-db — 倪海厦中医知识数据库 (3,867条记录)",
         "结构化中医知识数据库：中药472味、方剂234首、症状727种、证型194种、穴位47个、医案1737例、经典原文113篇、课程笔记121篇、治法119种、讲座81讲、天纪185条。",
         vec!["3,867条结构化记录，22张表",
              "医案1,737例为最大模块",
              "方剂234首包含伤寒论/金匮/汉唐方剂",
              "天然适合做RAG检索的SQLite数据库"]),
        ("9527qingfeng", "hantang-nihaixia-follower",
         "9527qingfeng/hantang-nihaixia-follower — 跟师倪海厦学中医 (280★)",
         "倪海厦外门弟子整理的完整学习笔记。含人纪五部PDF教材（非语音转文字，倪师自编教材）+ 自学笔记。已完工人纪5部、扶阳论坛、闭门课、仲景心法、经方的妙用、梁冬对话、斯坦福演讲、天纪、汉唐方剂讲解。医案整理中。",
         vec!["280 stars, 最完整的倪师教材PDF合集",
              "自学顺序：针灸→黄帝内经→神农本草经→伤寒论→金匮要略",
              "含倪师自编教材原文+视频课堂扩展笔记",
              "已完成13个课程模块整理"]),
        ("elliott10", "ebook-nihaixia",
         "elliott10/ebook-nihaixia — 倪海厦笔记与书籍合集 (22★)",
         "倪海厦笔记与书籍合集，包含人纪系列全部教材及仲景心法、医案等PDF。",
         vec!["最完整的倪师PDF镜像", "含人纪五部+仲景心法+医案"]),
        ("jangviktor-web", "nihaixia",
         "jangviktor-web/nihaixia — 倪海厦视角AI Agent Skill (32★)",
         "将倪海厦中医思维、人纪教学、临床心法蒸馏为可激活的Agent Skill。含伤寒论257条完整解读、金匮23篇、黄帝内经73篇、医案311+例、闭门课7大专题。",
         vec!["32 stars, 倪海厦视角Agent Skill",
              "伤寒论257条文+金匮23篇+黄帝内经73篇完整蒸馏",
              "医案311+例，涵盖血癌/红斑狼疮/脑瘤/肝癌等",
              "触发词：倪海厦、海厦视角、倪师、经方思维"]),
        ("jangviktor-web", "nihaixia-app",
         "jangviktor-web/nihaixia-app — 汉唐中医AI安卓诊断APP",
         "基于倪海厦天纪/地纪/人纪/伤寒论/金匮要略/黄帝内经/神农本草经/历史医案/六经辨证体系的安卓中医诊断助手。",
         vec!["安卓端倪海厦中医诊断APP",
              "整合人纪/天纪/地纪三纪体系",
              "六经辨证AI辅助诊断"]),
        ("huoyalong", "nihaisha-skill",
         "huoyalong/nihaisha-skill — 倪海厦Skill (14★)",
         "基于女娲Skill蒸馏的倪海厦skill框架，是后续nihaixia-skill的基础项目。",
         vec!["14 stars, 最早的倪海厦Skill框架",
              "基于女娲(nuwa-skill)蒸馏技术",
              "为后续倪海厦Agent项目奠定基础"]),
        ("qmzz", "ni-haisha-tcm-skill",
         "qmzz/ni-haisha-tcm-skill — 倪海厦中医Skill (2★)",
         "基于倪海厦人纪系列的Python中医知识库。含方剂/药材/穴位/医案Markdown知识库，提供辨证辅助与经方检索JSON工具。已发布v1.1.0。",
         vec!["Python实现的倪海厦中医知识库",
              "含tools/tcm_tools.py JSON工具接口",
              "知识库含方剂/药材/穴位/概念/医案",
              "安全边界：仅作中医理论学习参考"]),
        ("wanghao28", "nihaixia",
         "wanghao28/nihaixia — 倪海厦官网镜像",
         "倪海厦官网(https://hantang.com/)的GitHub备份。",
         vec!["倪海厦官网GitHub备份"]),
        ("nivance", "nihaixia-kb",
         "nivance/nihaixia-kb — 倪海厦知识库 (3★)",
         "倪海厦医案/症状/穴位/病机/治法结构化知识库。",
         vec!["3 stars, 医案/症状/穴位结构化",
              "含病机/治法分类"]),
    ];

    for (owner, repo, title, summary, insights) in repos {
        let r = ingester.ingest(
            &ResourceDescriptor::github(owner, repo, title, summary)
                .with_importance(0.92)
                .with_confidence(0.90)
                .with_tags(vec!["github", "nihaixia", "tcm", "absorbed-2026-07-05"])
                .with_key_insights(insights.iter().map(|s| *s).collect())
        ).expect("Failed to ingest GitHub repo");
        println!("  ✅ {} — {}", title, r.node_id);
    }
}

fn absorb_ni_haixia_overview(ingester: &mut ResourceIngester) {
    println!("\n=== Absorbing Ni Haixia Overview ===");

    let overviews = vec![
        ("倪海厦（1954-2012）— 中医经方家、汉唐中医学院院长",
         "倪海厦(1954-2012)，美国汉唐中医学院院长，中医界最具影响力的经方家之一。精通命、相、卜、山、医五术，被喻为当代少见的五术兼备之旷世奇才。毕生致力于发扬中医经方，以《人纪》《天纪》传承中华医学。核心方法论：阴阳为纲、六经为目、方证对应、药简力专。倪师强调'西医治病，中医治人'的整体观，主张经方实验，反对中西医结合（认为会稀释中医特色）。",
         vec!["1954年出生于台湾，2012年逝世",
              "美国汉唐中医学院院长，执业中医师",
              "精通命/相/卜/山/医五术",
              "核心贡献：人纪(医)+天纪(易)教学体系",
              "经方派代表，师承伤寒论体系"]),
        ("倪海厦人纪体系——中医五大经典的系统教学",
         "人纪(人体的记录)是倪海厦对中医五大经典的完整教学体系。学习路径：针灸大成→黄帝内经→神农本草经→伤寒论→金匮要略。2004年公开甄选关门弟子，2005-2007年完成训练并录制为DVD课程。每部都有视频+文档配合学习。核心方法论：阴阳为纲、六经为目、方证对应、药简力专。",
         vec!["五部曲逻辑链：先针灸(经络基础)→内经(生理病理哲学)→本草(药性基础)→伤寒(外感辨证)→金匮(内伤杂病)",
              "2004年公开甄选弟子，2005-2007年完成教学",
              "倪师自编教材+视频课程，非语音转文字",
              "教学核心：掌握原理后灵活运用，不只是死记硬背"]),
        ("倪海厦天纪体系——易经·紫微·风水的天道规律",
         "天纪是倪海厦对'天道规律'的解读，包括易经64卦占卜、紫微斗数命理、风水地理。核心：人纪治已病、天纪治未病。易经部分：以64卦讲人生处境与应对策略。紫微斗数：十四主星十二宫，断人生格局与流年运势。风水：阳宅三要(门主灶)、八宅明镜。倪师认为命运是可以认知和改变的，认知的工具就是天纪。",
         vec!["天纪治未病，人纪治已病——完整健康观",
              "易经64卦：人生处境与应对策略",
              "紫微斗数：十四主星十二宫格局",
              "风水：阳宅三要(门主灶)、八宅明镜",
              "天纪含24集DVD课程"]),
        ("倪海厦地纪体系——实地考察的地理验证",
         "地纪是倪海厦对'地道'的实地考察记录。他亲自走访中国大陆各地的山川地理，验证风水理论。记录了中国各地的龙脉走向、水系分布、地形地貌。对应传统堪舆学中的'寻龙点穴'实地验证。天地人三纪合一：天纪(时间)+地纪(空间)+人纪(人体)=完整的宇宙观和治疗观。与《黄帝内经》'人以天地之气生、四时之法成'完全一致。",
         vec!["地纪是倪师实地考察记录",
              "验证龙脉走向/水系分布/地形地貌",
              "天纪(时间)+地纪(空间)+人纪(人体)=完整宇宙观",
              "地纪共8本教材"]),
        ("倪海厦三阶学习路径——人纪→天纪→地纪",
         "倪师设计的完整学习路径：人纪(5年)→天纪(3年)→地纪(2年)。第一阶段人纪：针灸大成→黄帝内经→神农本草经→伤寒论→金匮要略。第二阶段天纪：易经占卜→紫微斗数→阳宅风水→八字命理。第三阶段地纪：实地验证风水理论，走遍全国。三纪合一后：中医治病+易卜决策+风水择居+命理知己。'医者易也'——中医和易经本为一体。",
         vec!["总学习周期约10年",
              "先学人纪再学天纪，地纪为实地验证",
              "'医者易也'：中医易经本为一体",
              "倪师强调'悟性'培养：学的是理，不是方"]),
    ];

    for (title, summary, insights) in overviews {
        let r = ingester.ingest(
            &ResourceDescriptor::concept(title, summary)
                .with_importance(0.97)
                .with_confidence(0.95)
                .with_tags(vec!["nihaixia", "overview", "tcm", "absorbed-2026-07-05"])
                .with_key_insights(insights.iter().map(|s| *s).collect())
        ).expect("Failed to ingest overview");
        println!("  ✅ {} — {}", title, r.node_id);
    }
}

fn absorb_renji_theory(ingester: &mut ResourceIngester) {
    println!("\n=== Absorbing 人纪 Core Theory ===");

    let theories = vec![
        ("人纪·针灸大成——经络穴位与针刺手法",
         "倪师以明朝杨继洲《针灸大成》为教材，结合北派真传针刺手法。系统讲解十二经络、奇经八脉、361穴位、针刺手法与配穴原则。共78集视频课程。针灸为中医基础，先学针灸建立经络概念是倪师推荐的学习起点。核心：'针灸治的是气，不是肉'——通过调气恢复人体正常生理功能。",
         vec!["教材：明代杨继洲《针灸大成》+北派真传手法",
              "78集视频课程，是五部曲第一门",
              "核心理论：针灸治气，调气恢复生理",
              "含九针使用、子午流注、灵龟八法等"]),
        ("人纪·黄帝内经——中医根本经典",
         "《黄帝内经》分为素问(81篇)和灵枢(81篇)，是中医理论的根本经典。倪师重点讲解阴阳五行、脏腑经络、病因病机、诊法治则等核心思想。共75集视频课程。内经是中医的生理学和病理学基础，理解内经才能理解人体的正常状态和疾病的本质。",
         vec!["教材：《黄帝内经》素问+灵枢",
              "75集视频课程，是五部曲第二门",
              "核心内容：阴阳五行/脏腑经络/病因病机",
              "倪师强调内经是中医生理学/病理学基础"]),
        ("人纪·神农本草经——中药学根基",
         "《神农本草经》为中药学奠基之作，收载365味药物，按上中下三品分类。倪师讲解每味药的性味归经、功效主治，强调药性理论和经方用药思路。共39集视频课程。倪师不使用《本草纲目》（认为其误导中药药性），完全以神农本草经为准。核心：知道药性比知道药方更重要。",
         vec!["教材：《神农本草经》，三品分类(上中下)",
              "39集视频课程，是五部曲第三门",
              "倪师反对使用《本草纲目》，只用本经",
              "365味药，详述性味归经功效主治"]),
        ("人纪·伤寒论——六经辨证体系",
         "张仲景《伤寒论》是中医第一部临床著作，创立六经辨证体系。倪师详细讲解257条条文，按六经(太阳/阳明/少阳/太阴/少阴/厥阴)分篇，含方证对应、传变规律、煎服方法。共202集视频课程。核心：'有是证用是方'——症状群对应特定经方。倪师强调经方剂量精确，一剂知二剂已。",
         vec!["教材：张仲景《伤寒论》257条",
              "202集视频课程，是五部曲第四门",
              "核心：六经辨证——太阳/阳明/少阳/太阴/少阴/厥阴",
              "核心方剂：桂枝汤/麻黄汤/小柴胡汤/白虎汤/承气汤/四逆汤等",
              "倪师特色：经方剂量精确，一剂知二剂已"]),
        ("人纪·金匮要略——杂病辨证论治",
         "《金匮要略》是《伤寒杂病论》的杂病部分，也是我国现存最早的一部论述杂病诊治的专著。倪师详细讲解23篇，涵盖痉湿暍、百合狐惑、疟病、中风历节、血痹虚劳、肺痿肺痈、痰饮水气、黄疸、惊悸吐衄、呕吐下利、妇人病等。共243集视频课程。金匮扩展了伤寒的六经体系到所有内科杂病。",
         vec!["教材：张仲景《金匮要略》23篇",
              "243集视频课程，是五部曲第五门",
              "涵盖：内科/妇科/外科杂病",
              "核心篇章：痰饮水气/血痹虚劳/妇人病"]),
        ("倪海厦仲景心法——经方临床心传",
         "倪师对张仲景学术思想的深度发挥和临床经验总结。讲解如何灵活运用经方于现代疾病，含方剂加减变化、剂量调整、合方应用等高级技巧。为弟子班的进阶内容。倪师强调：'学伤寒要学到神而不是形'——掌握六经辨证的精神实质。",
         vec!["倪师对伤寒论的临床发挥",
              "含方剂加减变化/剂量调整/合方应用",
              "倪师强调学到神而不是形"]),
        ("倪海厦八纲辨证——阴阳表里寒热虚实",
         "八纲辨证是中医辨证的基本纲领：阴阳、表里、寒热、虚实。倪师将其与六经辨证结合，形成完整的辨证体系。阴阳为总纲——表里定位病位、寒热定性病性、虚实定正邪盛衰。倪师诊断特色：望诊为主、问诊为辅、脉诊佐证，多数时连脉都不必诊。",
         vec!["八纲：阴阳/表里/寒热/虚实",
              "倪师诊断特色：望诊为主、问诊为辅",
              "'问几个问题，看一下气色'即可诊断"]),
        ("倪海厦临床医案体系——实证医学的验证",
         "倪师2005-2008年临床诊疗记录的完整医案集。涵盖血癌(白血病)、红斑狼疮、脑瘤、肾衰竭、乳癌、肝癌、渐冻症等重大疾病。倪师通过大量临床案例验证经方疗效。医案格式：症状→辨证→处方→疗效反馈。倪师强调：'经方不是实验，是几千年的临床验证'。",
         vec!["2005-2008年临床诊疗记录",
              "涵盖血癌/红斑狼疮/脑瘤/肾衰竭/乳癌/肝癌/渐冻症",
              "医案总数：1,737例结构化记录",
              "格式：症状→辨证→处方→疗效反馈"]),
    ];

    for (title, summary, insights) in theories {
        let r = ingester.ingest(
            &ResourceDescriptor::concept(title, summary)
                .with_importance(0.95)
                .with_confidence(0.93)
                .with_tags(vec!["renji", "tcm-theory", "nihaixia", "absorbed-2026-07-05"])
                .with_key_insights(insights.iter().map(|s| *s).collect())
        ).expect("Failed to ingest theory");
        println!("  ✅ {} — {}", title, r.node_id);
    }
}

fn absorb_tianji_theory(ingester: &mut ResourceIngester) {
    println!("\n=== Absorbing 天纪 Theory ===");

    let theories = vec![
        ("天纪·易经64卦——人生处境与应对策略",
         "倪师以易经64卦为基础，讲解384种人生处境及应对策略。每卦有卦象、卦辞、爻辞、象传的完整解读。倪师将易经视为'变化的哲学'——理解变化的规律就能预测和应对变化。核心教学：'易经不是算命，是教你看清局势后做正确决策'。每组卦象对应具体的健康、事业、人际关系场景。",
         vec!["64卦384爻，对应人生各种处境",
              "倪师：易经是变化哲学，不是算命",
              "卦象对应健康/事业/人际关系场景",
              "天纪24集中易经部分为核心内容"]),
        ("天纪·紫微斗数——十四主星十二宫命盘",
         "紫微斗数是中国传统命理学的重要分支。倪师以十四主星(紫微/天机/太阳/武曲/天同/廉贞/天府/太阴/贪狼/巨门/天相/天梁/七杀/破军)为核心，配合十二宫(命宫/兄弟/夫妻/子女/财帛/疾厄/迁移/交友/官禄/田宅/福德/父母)断人生格局与流年运势。含四化(化禄/化权/化科/化忌)的转换逻辑。",
         vec!["十四主星+十二宫+四化",
              "倪师：紫微是人生格局的整体把握",
              "流年运势按大限(每10年)推算",
              "天纪含紫微斗数电脑排盘软件"]),
        ("天纪·阳宅风水——八宅明镜与阳宅三要",
         "倪师以八宅明镜为基础，讲解阳宅风水核心原则。阳宅三要：门(入口)、主(主卧)、灶(厨房)。八宅分为东四宅(震/巽/离/坎)和西四宅(乾/兑/坤/艮)，每宅有吉凶四方位。倪师强调：'风水不是迷信，是环境能量学'——居住环境的能量流动影响人的健康和心理。",
         vec!["阳宅三要：门/主/灶",
              "八宅：东四宅+西四宅各四吉凶方位",
              "倪师：风水是环境能量学",
              "含寻龙点穴用于阴宅选址"]),
        ("天纪·八字命理——四柱推命",
         "以出生年月日时四柱(年柱/月柱/日柱/时柱)配天干地支，推算个人先天格局和大运流年。倪师将八字命理与人纪医理结合：'知道病人的八字五行，就可以明白体质的弱点'——八字五行与体质偏性、疾病易感性直接相关。",
         vec!["四柱：年/月/日/时配天干地支",
              "八字五行与体质偏性对应",
              "中医与命理结合：治病+知命"]),
        ("天纪·手面相——形神相合的诊断辅助",
         "倪师将面相学与中医望诊结合。面相十二宫(命宫/财帛/兄弟/田宅/男女/奴仆/妻妾/疾厄/迁移/官禄/福德/父母)反映内脏健康。手相：掌纹/五指/颜色对应不同脏腑。倪师特色：面相望诊是中医望诊的延伸，'望而知之谓之神'。",
         vec!["面诊十二宫对应内脏健康",
              "手相掌纹/颜色反映脏腑状态",
              "倪师：面相望诊是望诊的延伸"]),
    ];

    for (title, summary, insights) in theories {
        let r = ingester.ingest(
            &ResourceDescriptor::concept(title, summary)
                .with_importance(0.92)
                .with_confidence(0.90)
                .with_tags(vec!["tianji", "yijing", "zisha", "fengshui", "nihaixia", "absorbed-2026-07-05"])
                .with_key_insights(insights.iter().map(|s| *s).collect())
        ).expect("Failed to ingest tianji theory");
        println!("  ✅ {} — {}", title, r.node_id);
    }
}

fn absorb_diji_concept(ingester: &mut ResourceIngester) {
    println!("\n=== Absorbing 地纪 Concept ===");

    let concept = vec![
        ("倪海厦地纪——山川地理实地考察",
         "地纪是倪海厦对'地道'的实地考察记录，共8本。他亲自走访中国大陆各地的山川地理，验证风水理论中的龙脉走向、水系分布、地形地貌。对应传统堪舆学的'寻龙点穴'实地验证。天纪(时间)+地纪(空间)+人纪(人体)=完整的宇宙观和治疗观。地纪提醒：地理知识不能只靠书本，必须实地考察。",
         vec!["8本地纪教材",
              "实地验证龙脉/水系/地貌",
              "天纪+地纪+人纪=完整的宇宙观",
              "与《黄帝内经》天地人三才思想一致"]),
    ];

    for (title, summary, insights) in concept {
        let r = ingester.ingest(
            &ResourceDescriptor::concept(title, summary)
                .with_importance(0.88)
                .with_confidence(0.85)
                .with_tags(vec!["diji", "geography", "fengshui", "nihaixia", "absorbed-2026-07-05"])
                .with_key_insights(insights.iter().map(|s| *s).collect())
        ).expect("Failed to ingest diji");
        println!("  ✅ {} — {}", title, r.node_id);
    }
}

fn absorb_additional_lectures(ingester: &mut ResourceIngester) {
    println!("\n=== Absorbing Additional Lectures ===");

    let lectures = vec![
        ("倪海厦扶阳论坛演讲——经方临床实践",
         "倪师在扶阳论坛的演讲，分享经方在临床中的运用经验。强调扶阳思想在重症治疗中的重要性。含四逆汤的临床应用、癌症治疗中的阳药运用等核心内容。",
         vec!["扶阳思想在重症中的应用",
              "四逆汤系列临床经验"]),
        ("梁冬对话倪海厦——2009年7期完整访谈",
         "2009年12月梁冬与倪海厦的7期对话录音完整蒸馏。对话涵盖：中医复兴、经方精神、中西医比较、传统文化等主题。是了解倪师思想的重要入口。",
         vec!["2009年7期完整对话录音",
              "涵盖：中医复兴/经方精神/中西医比较"]),
        ("倪海厦斯坦福大学演讲——向西方世界介绍中医",
         "倪师在斯坦福大学的演讲，向西方医学界和学界介绍中医的核心价值。用现代语言解释阴阳五行、六经辨证等中医概念，论证中医的科学性和临床有效性。",
         vec!["向西方介绍中医核心价值",
              "用现代语言解释阴阳五行"]),
        ("倪海厦易筋经——文式易筋经十二式",
         "倪师传授的文式易筋经十二式，是内经导引术的传承。倪师在金匮要略课程中穿插讲解易筋经，强调导引在养生和康复中的重要作用。",
         vec!["文式易筋经十二式",
              "倪师在金匮课程中穿插教学",
              "导引术在养生和康复中的应用"]),
        ("倪海厦汉唐方剂讲解——临床用药心法",
         "倪师对汉唐方剂的系统讲解，含130首汉唐方剂的临床应用、配伍思路和加减变化。倪师在汉唐方剂中融入现代疾病治疗方案。",
         vec!["130首汉唐方剂系统讲解",
              "含配伍思路/加减变化/现代应用"]),
        ("倪海厦人纪班闭门课——7大专题+7堂弟子课",
         "倪师为入门弟子开设的闭门课程。7大专题：血癌/红斑狼疮/脑瘤/肾衰竭/乳癌/肝癌。7堂弟子课：渐冻症/鼻窦炎/停药时机/腹膜透析/面相望诊/育儿/五行饮食。",
         vec!["7大重病专题+7堂弟子课",
              "含渐冻症/鼻窦炎/停药时机等专题",
              "是倪师临床心法的精髓"]),
    ];

    for (title, summary, insights) in lectures {
        let r = ingester.ingest(
            &ResourceDescriptor::concept(title, summary)
                .with_importance(0.88)
                .with_confidence(0.90)
                .with_tags(vec!["lecture", "nihaixia", "tcm", "absorbed-2026-07-05"])
                .with_key_insights(insights.iter().map(|s| *s).collect())
        ).expect("Failed to ingest lecture");
        println!("  ✅ {} — {}", title, r.node_id);
    }
}

fn absorb_web_resources(ingester: &mut ResourceIngester) {
    println!("\n=== Absorbing Web Resources ===");

    let resources = vec![
        ("hantang.org.cn — 汉唐中医倪海厦传承网站",
         "汉唐中医倪海厦传承网站，提供人纪天纪视频、学习资料和医案查询。是国内最大的倪海厦学习资源平台之一。",
         "https://hantang.org.cn",
         vec!["国内最大倪海厦传承网站",
              "提供视频/资料/医案查询"]),
        ("nihaixia.cc — 倪海厦专题传承站",
         "美国汉唐中医学院倪海厦传承网站，系统介绍人纪/天纪/地纪体系，含在线学习资源和经典方剂查询。",
         "http://www.nihaixia.cc",
         vec!["系统介绍三纪体系",
              "含在线学习资源和经典方剂"]),
        ("hongyuanguoxue.com — 鸿渊国学倪海厦资料",
         "鸿渊国学网站提供倪海厦天纪人纪全套视频+音频+学习资料下载。含44册套装索引。",
         "https://www.hongyuanguoxue.com/zhongyi/51.html",
         vec!["倪海厦全套学习资料索引",
              "含44册套装索引"]),
        ("nihaisha.com.cn — 倪海厦书籍资料站",
         "倪海厦书籍全套资料站，含天纪人纪教材、医案、笔记、256个经典药方。",
         "https://www.nihaisha.com.cn",
         vec!["倪海厦书籍全套",
              "含256个经典药方"]),
        ("B站倪海厦人纪针灸大成合集（全76讲字幕版）",
         "B站倪海厦《人纪》针灸大成经典全76讲字幕版，含《人纪》+《天纪》全讲+电子书。",
         "https://www.bilibili.com/video/BV1ah6PYzEaR/",
         vec!["76讲针灸大成字幕版",
              "B站最完整的人纪合集之一"]),
        ("B站倪海厦伤寒论字幕版全集（1-202）",
         "B站倪海厦伤寒论字幕版全集1-202集，完整六经辨证教学。",
         "https://www.bilibili.com/video/BV12G4y1V7wm/",
         vec!["202集伤寒论字幕完整版"]),
        ("B站倪海厦金匮要略字幕版全集（1-243）",
         "B站倪海厦金匮要略字幕版全集1-243集，完整杂病辨证教学。",
         "https://www.bilibili.com/video/BV1uv4y1d7om/",
         vec!["243集金匮要略字幕完整版"]),
        ("B站2025倪海厦人纪大合集（字幕完整版）",
         "B站2025年最新上传的倪海厦人纪大合集字幕完整版，零基础自学中医全套。",
         "https://www.bilibili.com/video/BV11TNzzUEkV/",
         vec!["2025年最新人纪大合集字幕版"]),
        ("YouTube倪海厦视频频道",
         "YouTube倪海厦珍贵视频全集收藏频道，含《天纪》《人纪五部》完整视频。",
         "https://www.youtube.com/channel/UCIXOiFe3PpddEMnAdgshu5Q",
         vec!["YouTube完整视频收藏",
              "含天纪/人纪五部完整版"]),
        ("倪海厦天纪视频全集（24集全）",
         "倪海厦《天纪》视频全集24集，含易经/紫微斗数/风水/命理完整教学。",
         "https://www.bilibili.com/video/BV1ah6PYzEaR/",
         vec!["24集天纪视频全集"]),
    ];

    for (title, summary, url, insights) in resources {
        let r = ingester.ingest(
            &ResourceDescriptor::article(title, summary, url)
                .with_importance(0.85)
                .with_confidence(0.85)
                .with_tags(vec!["web-resource", "nihaixia", "tcm", "absorbed-2026-07-05"])
                .with_key_insights(insights.iter().map(|s| *s).collect())
        ).expect("Failed to ingest web resource");
        println!("  ✅ {} — {}", title, r.node_id);
    }
}

fn link_all_relations(ingester: &mut ResourceIngester) {
    println!("\n=== Creating Relations ===");

    macro_rules! link {
        ($from:expr, $to:expr, $rel:expr, $weight:expr, $desc:expr) => {
            match ingester.relate_by_title($from, $to, $rel, $weight, Some($desc)) {
                Ok(_) => println!("  🔗 {} → {} ({})", $from, $to, $desc),
                Err(e) => println!("  ⚠️  {} → {} : {}", $from, $to, e),
            }
        };
    }

    // ── 三纪总纲关系 ──
    link!("倪海厦（1954-2012）— 中医经方家、汉唐中医学院院长",
          "倪海厦人纪体系——中医五大经典的系统教学", RelationType::DevelopedBy, 0.98,
          "倪师创立人纪教学体系");
    link!("倪海厦（1954-2012）— 中医经方家、汉唐中医学院院长",
          "倪海厦天纪体系——易经·紫微·风水的天道规律", RelationType::DevelopedBy, 0.98,
          "倪师创立天纪教学体系");
    link!("倪海厦（1954-2012）— 中医经方家、汉唐中医学院院长",
          "倪海厦地纪——山川地理实地考察", RelationType::DevelopedBy, 0.98,
          "倪师创立地纪教学体系");

    // ── 三纪之间的关系 ──
    link!("倪海厦人纪体系——中医五大经典的系统教学",
          "倪海厦天纪体系——易经·紫微·风水的天道规律", RelationType::Related, 0.90,
          "人纪治已病,天纪治未病,统一于倪师体系");
    link!("倪海厦天纪体系——易经·紫微·风水的天道规律",
          "倪海厦地纪——山川地理实地考察", RelationType::Related, 0.85,
          "天纪(时间)+地纪(空间)时空一体");
    link!("倪海厦人纪体系——中医五大经典的系统教学",
          "倪海厦地纪——山川地理实地考察", RelationType::Related, 0.80,
          "人纪(人体)+地纪(环境)内外相应");

    // ── 人纪内部关系 ──
    link!("倪海厦三阶学习路径——人纪→天纪→地纪",
          "倪海厦人纪体系——中医五大经典的系统教学", RelationType::PartOf, 1.0,
          "人纪为学习路径第一阶段");
    link!("倪海厦三阶学习路径——人纪→天纪→地纪",
          "倪海厦天纪体系——易经·紫微·风水的天道规律", RelationType::PartOf, 1.0,
          "天纪为学习路径第二阶段");
    link!("倪海厦三阶学习路径——人纪→天纪→地纪",
          "倪海厦地纪——山川地理实地考察", RelationType::PartOf, 1.0,
          "地纪为学习路径第三阶段");

    // ── 人纪五部曲学习顺序 ──
    link!("人纪·针灸大成——经络穴位与针刺手法",
          "人纪·黄帝内经——中医根本经典", RelationType::PrerequisiteOf, 0.95,
          "先学针灸建立经络基础再学内经");
    link!("人纪·黄帝内经——中医根本经典",
          "人纪·神农本草经——中药学根基", RelationType::PrerequisiteOf, 0.90,
          "学完生理病理再学药性");
    link!("人纪·神农本草经——中药学根基",
          "人纪·伤寒论——六经辨证体系", RelationType::PrerequisiteOf, 0.95,
          "学完药性再学经方辨证");
    link!("人纪·伤寒论——六经辨证体系",
          "人纪·金匮要略——杂病辨证论治", RelationType::PrerequisiteOf, 0.95,
          "伤寒论是金匮的基础");
    link!("人纪·伤寒论——六经辨证体系",
          "倪海厦仲景心法——经方临床心传", RelationType::ExtensionOf, 0.90,
          "仲景心法是伤寒论的临床发挥");
    link!("人纪·伤寒论——六经辨证体系",
          "倪海厦八纲辨证——阴阳表里寒热虚实", RelationType::Related, 0.85,
          "六经与八纲结合形成完整辨证体系");

    // ── 天纪内部关系 ──
    link!("天纪·易经64卦——人生处境与应对策略",
          "天纪·紫微斗数——十四主星十二宫命盘", RelationType::Related, 0.85,
          "易经为体,紫微为用");
    link!("天纪·紫微斗数——十四主星十二宫命盘",
          "天纪·八字命理——四柱推命", RelationType::Related, 0.80,
          "紫微斗数与八字命理互补");
    link!("天纪·阳宅风水——八宅明镜与阳宅三要",
          "倪海厦地纪——山川地理实地考察", RelationType::ExtensionOf, 0.85,
          "风水理论需实地验证");
    link!("天纪·易经64卦——人生处境与应对策略",
          "天纪·手面相——形神相合的诊断辅助", RelationType::Related, 0.75,
          "易经哲学指导面相解读");

    // ── 医理与命理的关系 ──
    link!("人纪·黄帝内经——中医根本经典",
          "天纪·八字命理——四柱推命", RelationType::Related, 0.80,
          "体质偏性由八字五行决定");
    link!("倪海厦八纲辨证——阴阳表里寒热虚实",
          "天纪·手面相——形神相合的诊断辅助", RelationType::Supports, 0.85,
          "面相望诊是中医望诊的延伸");
    link!("人纪·伤寒论——六经辨证体系",
          "倪海厦临床医案体系——实证医学的验证", RelationType::Supports, 0.95,
          "医案验证伤寒论理论");

    // ── GitHub仓库关系 ──
    link!("JuneYaooo/nihaisha-nishi-tcm — 倪海厦中医课程资料Agent Skill (568★)",
          "倪海厦人纪体系——中医五大经典的系统教学", RelationType::Supports, 0.90,
          "Agent Skill涵盖人纪全部课程");
    link!("xiaogege6697/tcm-db — 倪海厦中医知识数据库 (3,867条记录)",
          "倪海厦临床医案体系——实证医学的验证", RelationType::Supports, 0.92,
          "数据库含1737例医案");
    link!("9527qingfeng/hantang-nihaixia-follower — 跟师倪海厦学中医 (280★)",
          "倪海厦人纪体系——中医五大经典的系统教学", RelationType::Supports, 0.92,
          "最完整的倪师教材PDF合集");
    link!("jangviktor-web/nihaixia — 倪海厦视角AI Agent Skill (32★)",
          "倪海厦（1954-2012）— 中医经方家、汉唐中医学院院长", RelationType::InspiredBy, 0.90,
          "基于倪海厦中医思维蒸馏的Agent Skill");
    link!("elliott10/ebook-nihaixia — 倪海厦笔记与书籍合集 (22★)",
          "倪海厦人纪体系——中医五大经典的系统教学", RelationType::Supports, 0.85,
          "倪师书籍PDF合集");

    // ── 讲座与主体关系 ──
    link!("梁冬对话倪海厦——2009年7期完整访谈",
          "倪海厦（1954-2012）— 中医经方家、汉唐中医学院院长", RelationType::References, 0.90,
          "对话访谈记录倪师思想");
    link!("倪海厦斯坦福大学演讲——向西方世界介绍中医",
          "倪海厦（1954-2012）— 中医经方家、汉唐中医学院院长", RelationType::References, 0.88,
          "斯坦福演讲记录");
    link!("倪海厦扶阳论坛演讲——经方临床实践",
          "人纪·金匮要略——杂病辨证论治", RelationType::ExtensionOf, 0.85,
          "扶阳思想是金匮的临床发展");
    link!("倪海厦人纪班闭门课——7大专题+7堂弟子课",
          "倪海厦临床医案体系——实证医学的验证", RelationType::Supports, 0.90,
          "闭门课内容基于临床医案");
    link!("倪海厦易筋经——文式易筋经十二式",
          "人纪·金匮要略——杂病辨证论治", RelationType::Related, 0.75,
          "易筋经在金匮课程中穿插教学");

    // ── 资源网站关系 ──
    link!("hantang.org.cn — 汉唐中医倪海厦传承网站",
          "倪海厦（1954-2012）— 中医经方家、汉唐中医学院院长", RelationType::References, 0.85,
          "传承倪师学问的网站");
    link!("nihaixia.cc — 倪海厦专题传承站",
          "倪海厦三阶学习路径——人纪→天纪→地纪", RelationType::Supports, 0.83,
          "网站包含三纪完整介绍");
    link!("YouTube倪海厦视频频道",
          "倪海厦人纪体系——中医五大经典的系统教学", RelationType::References, 0.85,
          "YouTube含人纪五部完整视频");
}

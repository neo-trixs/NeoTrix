use log;
use neotrix::neotrix::nt_mind::knowledge_engine::*;
use std::path::PathBuf;

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let kb_path = PathBuf::from(&home)
        .join(".neotrix")
        .join("knowledge_engine.json");
    let mut eng = KnowledgeEngine::load_from(&kb_path);
    eng.set_persist_path(kb_path.clone());
    let before = eng.stats().total_entries;

    let repos: Vec<(&str, &str, Vec<&str>, f64)> = vec![
        ("TCM-Ancient-Books: 中医药古籍文本700项(1.2k★,477 forks)",
         concat!("github.com/xiaopangxia/TCM-Ancient-Books 是GitHub上最活跃的中医古籍仓库。\n",
         "包含近700种中医药古籍文本,477个fork说明社区活跃。\n",
         "覆盖:医经/基础理论/伤寒金匮/诊法/针灸/本草/方书/临证各科/养生/医案/医史。\n",
         "格式:Markdown文本文件,UTF-8编码,可搜索。\n",
         "分支:lab99x/tcmoc(50★)提供了分类目录和元数据规范。\n",
         "子项目:TCM-ShangHan-Dx基于伤寒论的AI诊断辅助系统。\n",
         "这是目前最大的开源中医古籍文本集合,持续维护中。"),
         vec!["GitHub","中医","古籍","开源","人纪"], 0.96),

        ("Ancient-China-Books: 四库全书系列古籍数字版(组织)",
         concat!("github.com/Ancient-China-Books 是一个专门从事古籍数字化的组织。\n",
         "已发布的古籍:诗经/尚书/周易集解纂疏/尔雅注疏/仪礼注疏/\n",
         "四书章句集注/论语正义/庄子集释/庄子集解/长短经/\n",
         "三国演义/聊斋志异/楚辞/曹子建集/陶渊明集/世说新语/\n",
         "文心雕龙/苏东坡全集/古文观止/\n",
         "目录学:四库提要(浙本/殿本),四库全书总目。\n",
         "格式:HTML/GitHub Pages在线访问。\n",
         "特点是每个项目独立,可直接在线阅读。"),
         vec!["GitHub","古籍","四库","开源"], 0.95),

        ("ChuangTzu-text: 道藏庄子白文数字化",
         concat!("github.com/tobeabooker/ChuangTzu-text 对《道藏》所收庄子白文进行了精细校对。\n",
         "底本:Kanripo漢籍正统道藏三家本《南华真经》+国图藏道藏。\n",
         "包含6个PDF文件+1个HTML+img文件夹。\n",
         "文本每行与道藏原本一致。\n",
         "特色:有编码异体字转正字功能,红色标注重要异文。\n",
         "蓝色标注无编码异体字(鼠标悬浮显示截图)。\n",
         "校对使用工具:字统网/中华书局字符查询/看典古籍OCR/如是古籍多文本对比。\n",
         "这是目前最精细的道藏庄子数字化版本。"),
         vec!["GitHub","古籍","道藏","庄子","开源"], 0.93),

        ("Ancient-China-Books/sikuquanshuzongmu: 四库全书总目电子版",
         concat!("《四库全书总目》(殿本)电子版,包含四库著录书和存目书提要。\n",
         "纪昀等编纂,是中国古典目录学的最高成就。\n",
         "四部分类:经部(易/书/诗/礼/春秋/孝经/五经总义/四书/乐/小学10类)\n",
         "史部(正史/编年/纪事本末/别史/杂史/诏令奏议/传记/史钞/载记/时令/地理/职官/政书/目录/史评15类)\n",
         "子部(儒家/兵家/法家/农家/医家/天文算法/术数/艺术/谱录/杂家/类书/小说家/释家/道家14类)\n",
         "集部(楚辞/别集/总集/诗文评/词曲5类)\n",
         "基于\"东里书斋\"重制。可在线访问。"),
         vec!["GitHub","古籍","四库","目录学"], 0.95),
    ];

    let mut count = 0;
    for (title, body, tags, imp) in &repos {
        if !eng.entries.values().any(|x| x.title.contains(title)) {
            eng.add_entry(
                KnowledgeEntry::new(title, body, SourceType::KnowledgeBase, "kb:github3")
                    .with_importance(*imp)
                    .with_tags(tags.iter().map(|s| s.to_string()).collect()),
            );
            count += 1;
        }
    }
    log::info!("  ✅ 新增 {} 条", count);

    for (f, t, d) in [
        ("TCM-Ancient-Books", "tcm-texts", "中医古籍集合"),
        ("Ancient-China-Books", "sikuquanshuzongmu", "四库组织"),
        ("ChuangTzu-text", "正统道藏三洞体系", "道藏庄子"),
        ("TCM-Ancient-Books", "八纲辨证", "中医古籍→辨证"),
    ] {
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

    if let Err(e) = eng.save() {
        log::error!("❌{}", e);
    }
    log::info!(
        "\n💾 知识引擎: {}条目(+{}), {}关系\nGitHub合计: {}条",
        eng.stats().total_entries,
        eng.stats().total_entries - before,
        eng.stats().total_relations,
        eng.search_by_tag("GitHub", 1000).len()
    );
}

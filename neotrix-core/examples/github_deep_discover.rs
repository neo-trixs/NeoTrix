use std::path::PathBuf;
use neotrix::neotrix::reasoning_brain::knowledge_engine::*;
use neotrix::neotrix::reasoning_brain::CapabilityVector;

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let kb_path = PathBuf::from(&home).join(".neotrix").join("knowledge_engine.json");
    let mut eng = KnowledgeEngine::load_from(&kb_path);
    eng.set_persist_path(kb_path.clone());
    let before = eng.stats().total_entries;

    let repos: Vec<(&str, &str, Vec<&str>, f64)> = vec![
        ("scripta-sinica: 汉语古典文本资料库(330★,13亿字)",
         concat!("github.com/mahavivo/scripta-sinica 是最全面的汉语古典文本数据库。\n",
         "统计:13000种文本,10万卷,近13亿字,3.14GB。\n",
         "分类:01易藏195部/02儒藏370部/03道藏1689部/04佛藏5159部/\n",
         "05子藏1155部/06史藏1725部/07诗藏322部/08集藏1467部/\n",
         "09医藏869部/10艺藏386部。共10藏,涵盖中国古典全部领域。\n",
         "来源:殆知阁(daizhige.org)提供的原始数据。\n",
         "对比:《四库全书》收书3503种,79337卷,约8亿字。\n",
         "全球现存汉语古籍总量:177107种(据《中国古籍总目》)。\n",
         "本项目为后续分布式协作校勘提供了基础。\n",
         "注:TCM-OMNI 17★,TCM-KG等中医子项目活跃。"),
         vec!["GitHub","古籍","数据库","开源","知识链"], 0.97),

        ("sikuquanshuzongmu: 四库全书总目(殿本)电子版",
         concat!("github.com/Ancient-China-Books/sikuquanshuzongmu 包含四库全书总目提要的电子版。\n",
         "基于\"东里书斋\"重制,输出为epub格式。\n",
         "《四库全书》是清代乾隆年间编纂的中国最大丛书,收书3503种,79337卷。\n",
         "分经史子集四部,是中国古代知识体系的最高分类标准。\n",
         "四库总目提要是研究中国古籍最重要的目录学工具。"),
         vec!["GitHub","古籍","四库"], 0.94),

        ("ChineseliteratureDataset: 中华经典文献数据集(经史子集)",
         concat!("github.com/enze5088/ChineseliteratureDataset 按经史子集四部分类。\n",
         "经部:四书五经/十三经注疏。史部:正史/编年等12类。\n",
         "子部:先秦诸子+后世学科分类。集部:诗文词赋。\n",
         "参考:国学导航(guoxue123.com)/殆知阁(daizhige.org)/\n",
         "中国哲学书电子化计划(ctext.org)。\n",
         "该项目是公益性质,旨在填补古文NLP数据集空白。"),
         vec!["GitHub","古籍","数据集"], 0.92),

        ("tcm-texts: 中医开源医典(13455种古籍分类)",
         concat!("github.com/haimengzhang/tcm-texts 基于《中国中医古籍总目》分类。\n",
         "总目收录1949年前医书13455种。\n",
         "12个一级分类:医经/基础理论/伤寒金匮/诊法/针灸推拿/\n",
         "本草/方书/临证各科/养生/医案医话医论/医史/综合性著作。\n",
         "文件名规则:编号.类别.书名.作者.朝代.md。\n",
         "元数据格式:YAML front-matter(title/author/era/date/version/category)。\n",
         "来源:中醫笈成(jicheng.tw) + 中国中医古籍总目。"),
         vec!["GitHub","中医","古籍","开源","人纪"], 0.94),

        ("TCM_Datasets: 中医书籍文献高质量文本数据",
         concat!("github.com/PanckooAI/TCM_Datasets 包含十四五中医教材等高质量文本。\n",
         "目录:中医儿科学/中医养生学等教材,经典文本,现代文献。\n",
         "格式:Markdown,UTF-8,公式和表格用LaTeX。\n",
         "MIT协议。目标是促进中医智能化发展(NLP/知识提取/AI诊断)。\n",
         "注意:数据仅含文字内容,不含图片。需带图版请联系作者。"),
         vec!["GitHub","中医","教材"], 0.89),

        ("Awesome-Medical-Dataset: 神农TCM数据集(1.8k★)",
         concat!("github.com/openmedlab/Awesome-Medical-Dataset 收录ShenNong-TCM系列。\n",
         "ShenNong-TCM是基于LoRA微调的中医大语言模型。\n",
         "数据集包含:基于中医知识图谱构建的实体中心数据集。\n",
         "特色:可直接推荐中药方剂,不仅是通用医疗建议。\n",
         "评估基准:ShenNong-TCM-EB。"),
         vec!["GitHub","中医","AI","中药"], 0.92),

        ("ZhongJing-OMNI: 首个多模态中医评估基准(17★)",
         concat!("github.com/pariskang/ZhongJing-OMNI 首个多模态中医知识评估数据集。\n",
         "特点:多选题/开放题/临床案例/舌诊多模态(舌象图片+诊断QA)。\n",
         "评估LLM在中医诊断和治疗场景的推理能力。\n",
         "意义:将中医舌诊数字化+多模态AI结合。"),
         vec!["GitHub","中医","AI","多模态"], 0.91),

        ("Classical-Modern: 文言文-现代文平行语料(1.4k★)",
         concat!("github.com/NiuTrans/Classical-Modern 非常全的文言文(古文)-现代文平行语料。\n",
         "可用于文言文翻译、古文理解等NLP任务。\n",
         "对于古籍数字化和AI理解古文具有重要意义。\n",
         "数据量:大规模平行句子对。"),
         vec!["GitHub","文言文","语料","NLP"], 0.92),

        ("nihaixia-skill: 倪海厦中医Agent Skill",
         concat!("github.com/jangviktor-web/nihaixia 基于倪海厦知识的AI Agent Skill。\n",
         "蒸馏倪师人纪/医案/经方思维,用于OpenClaw平台。\n",
         "包含倪海厦视角的中医诊断推理逻辑。\n",
         "是第一个将倪海厦知识体系封装为AI Agent的项目。"),
         vec!["GitHub","倪海厦","中医","AI"], 0.91),

        ("TCM-KG: 中医知识图谱构建研究",
         concat!("github.com/YingXu-swim/TCM-KG 基于中医的知识图谱研究项目。\n",
         "流程:中医语料爬取→分词(Jiayan)→词向量训练(w2v/gensim)→\n",
         "NER(BiLSTM-CRF)→RE(文本+位置特征)→三元组→知识图谱。\n",
         "是知识图谱+中医的工程实践参考。"),
         vec!["GitHub","中医","知识图谱","NLP"], 0.90),
    ];

    let mut count = 0;
    for (title, body, tags, imp) in &repos {
        if !eng.entries.values().any(|x| x.title.contains(title)) {
            eng.add_entry(KnowledgeEntry::new(title, body, SourceType::KnowledgeBase, "kb:github")
                .with_importance(*imp).with_tags(tags.iter().map(|s| s.to_string()).collect()));
            count += 1;
        }
    }
    println!("  ✅ 新增 {} 条GitHub发现", count);

    // Cross refs
    for (f, t, d) in [("scripta-sinica","正统道藏三洞体系","道藏子集"),("scripta-sinica","大正藏结构详解","佛藏子集"),
        ("tcm-texts","八纲辨证","中医经典数据库"),("ChineseliteratureDataset","GitHub古籍资源","四部分类对接"),
        ("nihaixia-skill","倪海厦人纪体系","倪师AI化")] {
        let fi = eng.entries.values().find(|x|x.title.contains(f)).map(|x|x.id.clone());
        let ti = eng.entries.values().find(|x|x.title.contains(t)).map(|x|x.id.clone());
        if let (Some(ff),Some(tt))=(fi,ti) { eng.add_relation(&ff,&tt,RelationType::Related,0.7,d); }
    }

    // Cap iteration
    let mut cap = if let Ok(b) = neotrix::neotrix::reasoning_brain::ReasoningBrain::load() { b.capability }
        else { CapabilityVector::default() };
    for (d,v) in &[("domain_specificity",0.06)] {
        if let Some(idx) = CapabilityVector::index_from_name(d) {
            *cap.arr_mut().get_mut(idx).expect("index out of bounds") = (cap.arr()[idx] + v).min(1.0);
        }
    }
    cap.normalize();
    let _ = neotrix::neotrix::reasoning_brain::ReasoningBrain { capability: cap, ..Default::default() }.save();
    if let Err(e) = eng.save() { eprintln!("❌{}", e); }
    println!("\n💾 知识引擎: {}条目(+{}), {}关系", eng.stats().total_entries,
        eng.stats().total_entries - before, eng.stats().total_relations);
    println!("GitHub发现累计: {}条", eng.search_by_tag("GitHub", 1000).len());
}

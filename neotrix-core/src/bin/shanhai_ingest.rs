//! neotrix-shanhai-ingest — 山海世界数据吸收
//!
//! Absorbs Shanhai Jing research data, geographical mappings,
//! and historical resources into the NeoTrix Knowledge Base.
//!
//! Usage: cargo run -p neotrix --bin neotrix-shanhai-ingest

#![forbid(unsafe_code)]
use std::time::{SystemTime, UNIX_EPOCH};

use neotrix::neotrix::nt_memory_kb::nt_memory_resource_ingest::*;
use neotrix::neotrix::nt_memory_kb::nt_memory_types::*;
use neotrix::neotrix::nt_memory_kb::nt_memory_schema;
use neotrix::neotrix::nt_shanhai_geo::*;
use rusqlite::Connection;
use uuid::Uuid;

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let db_path = format!("{}/.neotrix/knowledge.db", home);
    println!("Opening KB at: {}", db_path);

    let conn = Connection::open(&db_path).expect("Failed to open KB");
    nt_memory_schema::initialize(&conn).expect("Failed to init schema");

    let mut ingester = ResourceIngester::new(&conn);

    absorb_researchers(&conn, &mut ingester);
    absorb_core_books(&mut ingester);
    absorb_geography_concepts(&mut ingester);
    absorb_global_mappings(&mut ingester);
    absorb_schools_of_thought(&mut ingester);
    absorb_mythical_kingdoms(&mut ingester);
    link_all_relations(&conn, &mut ingester);

    println!("\n{}", ingester.report());
    println!("\n✅ 山海世界数据吸收完成!");
}

fn ts() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

fn insert_person_node(conn: &Connection, title: &str, summary: &str, importance: f64) -> String {
    let id = Uuid::new_v4().to_string();
    let now = ts();
    let meta = serde_json::json!({"tags": ["absorbed-2026-07-03"], "type": "researcher"});
    let node = KnowledgeNode {
        id: id.clone(),
        node_type: NodeType::Person,
        title: title.to_string(),
        summary: Some(summary.to_string()),
        content: None,
        url: None,
        domain: None,
        language: "zh".into(),
        confidence: 0.9,
        importance,
        created_at: now,
        updated_at: now,
        access_count: 0,
        metadata: Some(meta),
        temporal: None,
        supersedes: None,
        source_episode: None,
    };
    safe_insert_node(conn, &node).expect("Failed to insert person node");
    id
}

fn absorb_researchers(conn: &Connection, _ingester: &mut ResourceIngester) {
    println!("\n=== Absorbing Researchers ===");

    let people = vec![
        ("王红旗 (Wang Hongqi)", "中国独立学者，近代物理专业出身，研究《山海经》30余年。将《五藏山经》26条山脉447座山一一对应现代地理坐标，断定古昆仑在鄂尔多斯高原。著有《山海经全集精绘》等十余部专著。", 0.95),
        ("孙晓琴 (Sun Xiaoqin)", "画家，王红旗之妻，合作30年。根据王红旗的考证绘制700余幅《山海经》艺术地理复原图及42平方米《帝禹山河图》。", 0.90),
        ("刘宗迪 (Liu Zongdi)", "北京语言大学教授，博士生导师。发现《山海经》1里=12米（依据庙岛群岛验证），论证《山经》记载范围为山东鲁中南山区。著有《众神的山川》《失落的天书》等。", 0.92),
        ("宫玉海 (Gong Yuhai)", "吉林学者，世界圈说代表人物。主张《山海经》为全球地理志，完成非洲/欧洲/美洲/大洋洲的系统性地名映射。昆仑=肯尼亚，西王母=示巴女王等。", 0.88),
        ("谭其骧 (Tan Qixiang)", "中国历史地理学泰斗，考订《山经》447座山，指出范围东达东海、西抵天山、北至西伯利亚、南达岭南。", 0.90),
        ("Henriette Mertz (默茨)", "美国学者，著有《Pale Ink》。亲身徒步验证《东山经》四条山脉与北美落基山脉/内华达山脉/海岸山脉对应。", 0.85),
        ("袁珂 (Yuan Ke)", "中国神话学泰斗，著有《山海经校注》《中国神话史》，是《山海经》现代研究的必备注本。", 0.87),
        ("扶永发 (Fu Yongfa)", "学者，主张《山海经》记述云南西部纵谷地区地理，古昆仑=横断山脉。局部小区说代表。", 0.75),
        ("何幼琦 (He Youqi)", "历史学家，著有《海经新探》，认为《山海经》山川疆域在山东中南部以泰山为中心。局部小区说代表。", 0.75),
        ("苏雪林 (Su Xuelin)", "作家学者，主张《山海经》为两河流域(巴比伦)地理书，中国文化西来说代表。", 0.78),
        ("小川琢治 (Ogawa Takuji)", "日本地理学家，认为《山海经》比《禹贡》更可靠，最早从现代地理学角度研究《山海经》。", 0.82),
        ("凌纯声 (Ling Chunsheng)", "学者，认为《山海经》是以中国为中心的《古亚洲志》，东及西太平洋，南至南海诸岛，西抵西南亚洲。", 0.80),
    ];

    for (name, summary, importance) in people {
        let id = insert_person_node(conn, name, summary, importance);
        println!("  ✅ {} — {}", name, id);
    }
}

fn absorb_core_books(ingester: &mut ResourceIngester) {
    println!("\n=== Absorbing Core Books ===");

    let books = vec![
        ("众神的山川——《山海经》与上古地理、历史及神话的重建",
         "刘宗迪著，2022年商务印书馆出版。全面考证《山海经》地理范围，发现1里=12米，认定《山经》记载山东鲁中南山区。推翻传统《禹贡》优先论。",
         "刘宗迪 (Liu Zongdi)"),
        ("山海经全集精绘",
         "王红旗编译，孙晓琴绘。2019年清华大学出版社。收录700余幅《山海经》艺术地理复原图，将26条山脉447座山一一对应现代地图，含42平方米《帝禹山河图》。",
         "王红旗 (Wang Hongqi)"),
        ("失落的天书——《山海经》与古代华夏世界观",
         "刘宗迪著，2006年商务印书馆。从历法天文学角度解读《山海经》，揭示其作为上古观象授时体系的本质。",
         "刘宗迪 (Liu Zongdi)"),
        ("经典图读·山海经",
         "王红旗/孙晓琴著，2003年出版。首次系统出版《山海经》艺术地理复原图。",
         "王红旗 (Wang Hongqi)"),
        ("山海经校注",
         "袁珂著，现代《山海经》研究的必备注本，系统校勘注释全文。",
         "袁珂 (Yuan Ke)"),
        ("谈谈如何揭开山海经奥秘",
         "宫玉海著，系统阐述世界圈说，将《山海经》地名映射至欧洲、非洲、美洲、大洋洲。",
         "宫玉海 (Gong Yuhai)"),
        ("Pale Ink / 褪色的墨迹（中文版）",
         "Henriette Mertz著，记载作者亲身徒步验证《东山经》与北美山脉对应的考察成果。",
         "Henriette Mertz (默茨)"),
        ("山海经的考证及补遗",
         "小川琢治著，最早从现代地理学角度考证《山海经》的学术论文。",
         "小川琢治 (Ogawa Takuji)"),
        ("帝禹山河图",
         "孙晓琴绘制的42平方米巨幅画卷，再现4200年前（大禹时代）华夏地理全貌。含26条山脉、447座山、358条水系。",
         "孙晓琴 (Sun Xiaoqin)"),
    ];

    for (title, summary, author) in &books {
        let r = ingester.ingest(
            &ResourceDescriptor::concept(title, summary)
                .with_importance(0.9)
                .with_confidence(0.9)
                .with_tags(vec!["book", "shanhaijing", "research", "absorbed-2026-07-03"])
                .with_key_insights(vec![&format!("Author: {}", author)])
        ).expect("Failed to ingest book");
        println!("  ✅ {} — {}", title, r.node_id);
    }
}

fn absorb_geography_concepts(ingester: &mut ResourceIngester) {
    println!("\n=== Absorbing Core Geography Concepts ===");

    let concepts = vec![
        ("五藏山经——26条山脉447座山",
         "《山海经》核心部分，按南、西、北、东、中五方分布26条山脉447座山。南山经3列、西山经4列、北山经3列、东山经4列、中山经12列。每山记名称、方位、里程、物产、矿藏、动植物。",
         vec!["南山经3列，起于浙江舟山抵湖南广东", "西山经4列，起于陕晋黄河抵新疆天山", "北山经3列，起于内蒙古抵河北中部", "东山经4列，起于山东泰山抵成山角", "中山经12列，晋南豫西为中心记载最详"]),
        ("刘宗迪尺度论——1里=12米",
         "刘宗迪通过对《东次三经》庙岛群岛的里程验证，发现《山经》1里相当于现代12米。东西28000里=336公里，南北26000里=312公里，范围约鲁中南山区。",
         vec!["依据《东次三经》九座庙岛群岛岛屿间水程实测", "6400里书中里程对应实测150里，换算得1里=12米", "东西28000里×12米=336km，南北26000里×12米=312km"]),
        ("王红旗昆仑定位——鄂尔多斯高原",
         "王红旗考证认为远古华夏昆仑在黄河河套以南的鄂尔多斯高原，以此为基准展开26条山脉的定位。南山经/西山经/北山经/东山经/中山经各以昆仑为参照。",
         vec!["昆仑在黄河河套以南鄂尔多斯高原", "以昆仑为原点展开全部26条山脉走向", "先内后外、先近后远、先中心后外围的排序规律"]),
        ("4200年前华夏古地理",
         "王红旗/孙晓琴复原的4200年前（大禹时代）华夏地理：海平面较今高3-5米，山东半岛被海水分割；黄河大拐弯处有稷泽和泑泽两大湖；长江中游云梦泽浩瀚；海岸线大幅内缩。",
         vec!["4200年前山东半岛被海水分割为群岛", "黄河U形拐弯处稷泽、泑泽两大湖今已消失", "长江中游云梦泽水面浩瀚今大部退去", "海平面较今高3-5米（冰后期高海面）"]),
        ("《山海经》地理研究三大流派",
         "学界对《山海经》地理范围的三种主要学说：华夏说(亚洲圈)认为范围在中国及周边；局部小区说认为仅限山东/云南等局部区域；世界圈说认为远及全球六大洲。",
         vec!["华夏说(谭其骧等)：东起东海西抵天山北至西伯利亚", "局部小区说(何幼琦等)：山东/云南等局部区域", "世界圈说(宫玉海等)：远及非洲欧洲美洲大洋洲"]),
        ("山海经地理复原方法论",
         "王红旗创立的《山海经》地理复原方法：先内后外排序+昆仑基准定位+水系走向验证+山海经与禹贡/Hanshu地理志/水经注对照。",
         vec!["排序规律：先内后外、先近后远、先中心后外围", "昆仑基准：鄂尔多斯高原为原点展开山脉", "水系验证：河流走向与山脉关系相互校验"]),
    ];

    for (title, summary, insights) in concepts {
        let desc = ResourceDescriptor::concept(title, summary)
            .with_importance(0.88)
            .with_confidence(0.85)
            .with_tags(vec!["geography", "shanhaijing", "research", "absorbed-2026-07-03"])
            .with_key_insights(insights.iter().map(|s| *s).collect());
        let r = ingester.ingest(&desc).expect("Failed to ingest concept");
        println!("  ✅ {} — {}", title, r.node_id);
    }
}

fn absorb_global_mappings(ingester: &mut ResourceIngester) {
    println!("\n=== Absorbing Global Mappings (世界圈说) ===");

    let mappings = vec![
        ("昆仑山 → 肯尼亚/埃塞俄比亚",
         "宫玉海等考证：西海(地中海)之南、流沙(撒哈拉)之滨、赤水(红海)之后、黑水(青尼罗河)之前，大山名昆仑。弱水之渊=东非大裂谷，炎火之山=乞力马扎罗。昆=Ken，仑=Land。",
         vec!["西海=地中海", "流沙=撒哈拉沙漠", "赤水=红海", "黑水=青尼罗河", "弱水之渊=东非大裂谷", "炎火之山=乞力马扎罗山"]),
        ("不周山 → 东非大裂谷",
         "《大荒西经》：西北海之外，有山而不合，名曰不周。地质上对应东非大裂谷，裂谷将火山一分为二，形成'有山而不合'地貌。",
         vec!["有山而不合=裂开山脉", "东非大裂谷是地球唯一如此地貌", "寒暑之水=埃塞俄比亚凉爽/索马里炎热"]),
        ("东山经→北美落基山脉（Mertz验证）",
         "美国学者Henriette Mertz亲自徒步验证，《东山经》四条山脉走向、山峰距离、河流走向与北美落基山脉、内华达山脉、喀斯喀特山脉、海岸山脉完全吻合。",
         vec!["《东山经》四条山脉=北美西岸四条山脉", "Mertz按书中方向和里程徒步验证", "山峰距离和河流走向完全吻合"]),
        ("光华之谷 → 科罗拉多大峡谷",
         "欧美学者认为《大荒东经》'光华之谷'描写的是美国科罗拉多大峡谷的地貌特征。",
         vec!["地貌描述与大峡谷一致", "位于《大荒东经》方位与美洲吻合"]),
        ("扶桑 → 富士山/美洲",
         "'扶桑'一词在《山海经》中指日出之地。一说扶桑=富士山（读音相似+日出方位），一说指墨西哥/美洲。",
         vec!["扶桑=富士山:读音相似", "扶桑=美洲:日出方位+扶桑木=美洲红杉"]),
        ("西王母 → 示巴女王（也门/埃塞俄比亚）",
         "宫玉海认为西王母即旧约中的示巴女王。周穆王=所罗门王。西王母戴胜、虎齿豹尾的记载与埃塞俄比亚皇室标志吻合。",
         vec!["西王母=示巴女王（也门/埃塞俄比亚）", "周穆王=所罗门王（读音周穆满=Solomon）", "昆仑虚北=埃塞俄比亚北面的也门"]),
        ("寿麻 → 索马里",
         "《大荒西经》中的寿麻，郭沫若认为=苏美尔，徐南洲认为=苏门答腊，袁珂认为=斯里兰卡，胡远鹏认为=索马里。",
         vec!["寿麻读音与索马里(Somalia)吻合", "位于大荒西经方位与非洲之角一致"]),
        ("炎火山 → 乞力马扎罗",
         "昆仑外有炎火之山，投物辄然。对应乞力马扎罗山（赤道雪山，火山活动）。",
         vec!["乞力马扎罗靠近赤道，古时火山活动", "位于肯尼亚/坦桑尼亚边境，与昆仑定位一致"]),
        ("君子国 → 朝鲜",
         "《海内北经》：海东有君子国，衣冠带剑，好让不争。有薰华草朝生夕死。薰华草=木槿花=朝鲜国花。",
         vec!["最早记载朝鲜地理的文献", "薰华草=木槿花=朝鲜国花"]),
        ("儋耳 → 海南岛",
         "《大荒北经》：有儋耳之国。后世海南岛古称儋耳/儋州。《吕氏春秋》有'北怀儋耳'记载。",
         vec!["儋耳/儋州为海南岛古称", "《山海经》最早出现儋耳地名"]),
    ];

    for (title, summary, insights) in mappings {
        let desc = ResourceDescriptor::concept(title, summary)
            .with_importance(0.85)
            .with_confidence(0.75)
            .with_tags(vec!["mapping", "global-geography", "world-circling", "absorbed-2026-07-03"])
            .with_key_insights(insights.iter().map(|s| *s).collect());
        let r = ingester.ingest(&desc).expect("Failed to ingest mapping");
        println!("  ✅ {} — {}", title, r.node_id);
    }
}

fn absorb_schools_of_thought(ingester: &mut ResourceIngester) {
    println!("\n=== Absorbing Research Schools ===");

    let schools = vec![
        ("华夏说——谭其骧学术体系",
         "谭其骧考订《山经》447座山，指出范围东起浙江舟山、西抵新疆天山、北至蒙古高原、南达岭南。对豫西晋南陕中记载最详确。《海经》范围及朝鲜日本中南半岛阿富汗俄罗斯。",
         vec!["447座山中约140座可确切定位", "豫西/晋南/陕中记载最详确——此为华夏文明核心区", "《山经》为历代巫师方士踏勘记录"]),
        ("世界圈说——宫玉海学术体系",
         "宫玉海认为《山海经》为全球地理志，世界只有一个大陆时为《海内经》时代。完成全球地名系统映射：昆仑=肯尼亚，大夏=希腊，轩辕=匈牙利，夸父=斯拉夫，奄兹=英吉利，方氏=法兰西等。",
         vec!["全球地名系统基于语音学考证", "上古世界只有一种语言，部落无国家概念", "黄帝封地轩辕之丘=匈牙利"]),
        ("局部小区说——山东/云南中心论",
         "何幼琦：《海经》山川疆域只在山东中南部以泰山为中心。扶永发：《山海经》记述云南西部纵谷地区，古昆仑=横断山脉。王宁：《山经》以山东为中心略及冀南豫东苏皖北部。",
         vec!["何幼琦：山东泰山中心论", "扶永发：云南横断山脉中心论", "王宁：山东为中心略及周边"]),
    ];

    for (title, summary, insights) in schools {
        let desc = ResourceDescriptor::concept(title, summary)
            .with_importance(0.85)
            .with_confidence(0.85)
            .with_tags(vec!["school-of-thought", "research-methodology", "absorbed-2026-07-03"])
            .with_key_insights(insights.iter().map(|s| *s).collect());
        let r = ingester.ingest(&desc).expect("Failed to ingest school");
        println!("  ✅ {} — {}", title, r.node_id);
    }
}

fn absorb_mythical_kingdoms(ingester: &mut ResourceIngester) {
    println!("\n=== Absorbing Mythical Kingdoms & Tribes ===");

    let kingdoms = vec![
        ("轩辕之国 → 匈牙利/奥地利",
         "宫玉海考证：黄帝封地轩辕之丘在欧洲匈牙利一带。轩辕读音与Hungary接近。有罴氏(有熊氏)读音近阿尔卑斯(Alps)。匈牙利首位国王名阿尔伯特。",
         vec!["轩辕之国读音接近Hungary", "有罴氏读音近Alps(阿尔卑斯)", "黄帝封地不在中国而在欧洲"]),
        ("大夏 → 希腊",
         "宫玉海考证：大夏即希腊(Greece/希腊语)。'希腊'在希腊语中就是'夏'的意思，音转一致。鲧、禹和后启封地为大夏(希腊)。",
         vec!["希腊=夏(读音一致)", "大夏为鲧禹后启的封地"]),
        ("夸父 → 斯拉夫",
         "宫玉海以语音学考证：夸父(Kuafu)读音对应于斯拉夫(Slav)。夸父追日的氏族迁徙映射了斯拉夫民族的起源。",
         vec!["夸父音近Slav(斯拉夫)", "夸父氏族迁徙映射斯拉夫起源"]),
        ("奄兹 → 英吉利",
         "西极即弇兹(奄兹)，为日落之神的方位。奄兹/弇兹读音与英吉利(England)相近，地理位置与西方日落方向一致。",
         vec!["奄兹音近England", "西方极地与日落方位一致"]),
        ("方氏 → 法兰西",
         "方氏读音与法兰西(France)对应。位于大荒西经范围，与法国地理位置一致。",
         vec!["方氏音近France(法兰西)"]),
        ("石夷/柘夷 → 德意志",
         "石夷/柘夷(独逸)读音与德意志(Deutsch)对应。石夷在《山海经》中位于西方。",
         vec!["石夷/柘夷音近Deutsch(德意志)"]),
        ("戎民 → 日耳曼",
         "戎民读音与日耳曼(German)对应。位于西北方位。",
         vec!["戎民音近German(日耳曼)"]),
        ("大蒙 → 丹麦",
         "大蒙读音与丹麦(Denmark)对应。",
         vec!["大蒙音近Denmark"]),
        ("不庭胡余 → 巴布亚新几内亚+澳大利亚",
         "不庭=巴布亚岛(新几内亚)，胡余=澳大利亚古音。大洋洲对应。",
         vec!["不庭=巴布亚岛", "胡余=澳大利亚古音"]),
        ("汤谷 → 汤加/美洲日出地",
         "汤谷为太阳升起之处。一说汤谷=汤加(读音一致+太平洋日出方位)，一说汤谷在山东日照。",
         vec!["汤谷读音近Tonga(汤加)", "山东日照也被认为是古汤谷"]),
        ("下夷/虾夷 → 夏威夷",
         "下夷/虾夷读音与夏威夷(Hawaii)对应。位于海外东经方位。",
         vec!["下夷/虾夷音近Hawaii"]),
        ("文身国 → 阿留申群岛",
         "文身国对应阿留申(Aleutian)群岛。当地原住民有文身习俗。",
         vec!["文身习俗对应阿留申原住民"]),
        ("汉大 → 加拿大",
         "汉大读音与加拿大(Canada)对应。位于海外东经极远处。",
         vec!["汉大音近Canada"]),
        ("昧谷 → 墨西哥",
         "日落之处的昧谷对应墨西哥(Mexico)。位于大荒西经最西端。",
         vec!["昧谷对应墨西哥日落之处"]),
    ];

    for (title, summary, insights) in kingdoms {
        let desc = ResourceDescriptor::concept(title, summary)
            .with_importance(0.78)
            .with_confidence(0.70)
            .with_tags(vec!["mythical-kingdom", "global-mapping", "phonetic", "absorbed-2026-07-03"])
            .with_key_insights(insights.iter().map(|s| *s).collect());
        let r = ingester.ingest(&desc).expect("Failed to ingest kingdom");
        println!("  ✅ {} — {}", title, r.node_id);
    }
}

fn link_all_relations(_conn: &Connection, ingester: &mut ResourceIngester) {
    println!("\n=== Creating Relations ===");

    // Helper macro to simplify linking
    macro_rules! link {
        ($from:expr, $to:expr, $rel:expr, $weight:expr, $desc:expr) => {
            match ingester.relate_by_title($from, $to, $rel, $weight, Some($desc)) {
                Ok(_) => println!("  🔗 {} → {} ({})", $from, $to, $desc),
                Err(e) => println!("  ⚠️  {} → {} : {}", $from, $to, e),
            }
        };
    }

    // 1. Researchers → their books
    link!("王红旗 (Wang Hongqi)", "山海经全集精绘", RelationType::DevelopedBy, 0.95, "Author of the book");
    link!("王红旗 (Wang Hongqi)", "经典图读·山海经", RelationType::DevelopedBy, 0.95, "Author of the book");
    link!("孙晓琴 (Sun Xiaoqin)", "山海经全集精绘", RelationType::DevelopedBy, 0.95, "Illustrator of the book");
    link!("孙晓琴 (Sun Xiaoqin)", "经典图读·山海经", RelationType::DevelopedBy, 0.95, "Illustrator of the book");
    link!("孙晓琴 (Sun Xiaoqin)", "帝禹山河图", RelationType::DevelopedBy, 0.95, "Painter of the artwork");
    link!("刘宗迪 (Liu Zongdi)", "众神的山川——《山海经》与上古地理、历史及神话的重建", RelationType::DevelopedBy, 0.95, "Author of the book");
    link!("刘宗迪 (Liu Zongdi)", "失落的天书——《山海经》与古代华夏世界观", RelationType::DevelopedBy, 0.95, "Author of the book");
    link!("宫玉海 (Gong Yuhai)", "谈谈如何揭开山海经奥秘", RelationType::DevelopedBy, 0.95, "Author of the book");
    link!("袁珂 (Yuan Ke)", "山海经校注", RelationType::DevelopedBy, 0.95, "Author of the book");
    link!("Henriette Mertz (默茨)", "Pale Ink / 褪色的墨迹（中文版）", RelationType::DevelopedBy, 0.95, "Author of the book");
    link!("小川琢治 (Ogawa Takuji)", "山海经的考证及补遗", RelationType::DevelopedBy, 0.95, "Author of the paper");

    // 2. Books → Concepts
    link!("山海经全集精绘", "王红旗昆仑定位——鄂尔多斯高原", RelationType::Supports, 0.90, "Book contains the Kunlun positioning evidence");
    link!("山海经全集精绘", "4200年前华夏古地理", RelationType::Supports, 0.90, "Book contains the paleogeography reconstruction");
    link!("众神的山川——《山海经》与上古地理、历史及神话的重建", "刘宗迪尺度论——1里=12米", RelationType::Supports, 0.90, "Book contains the scale discovery");
    link!("失落的天书——《山海经》与古代华夏世界观", "刘宗迪尺度论——1里=12米", RelationType::Supports, 0.80, "Earlier work on the scale");
    link!("谈谈如何揭开山海经奥秘", "昆仑山 → 肯尼亚/埃塞俄比亚", RelationType::Supports, 0.85, "Book contains Kunlun=Africa mapping");
    link!("谈谈如何揭开山海经奥秘", "不周山 → 东非大裂谷", RelationType::Supports, 0.85, "Book contains Buzhou=Africa Rift mapping");

    // 3. Contradicting theories
    link!("华夏说——谭其骧学术体系", "世界圈说——宫玉海学术体系", RelationType::Contradicts, 0.60, "China-only vs global interpretation");
    link!("世界圈说——宫玉海学术体系", "局部小区说——山东/云南中心论", RelationType::Contradicts, 0.60, "Global vs local interpretation");
    link!("华夏说——谭其骧学术体系", "局部小区说——山东/云南中心论", RelationType::Contradicts, 0.50, "Broad China vs narrow local area");

    // 4. Mapping relationships
    link!("五藏山经——26条山脉447座山", "东山经→北美落基山脉（Mertz验证）", RelationType::InspiredBy, 0.80, "East Mountains mapped to Rockies via Mertz verification");
    link!("刘宗迪尺度论——1里=12米", "五藏山经——26条山脉447座山", RelationType::ExtensionOf, 0.85, "Scale discovery extends understanding of the text");
    link!("王红旗昆仑定位——鄂尔多斯高原", "五藏山经——26条山脉447座山", RelationType::ExtensionOf, 0.85, "Kunlun positioning enables mountain range mapping");

    // 5. School → Key researcher
    link!("华夏说——谭其骧学术体系", "谭其骧 (Tan Qixiang)", RelationType::DevelopedBy, 0.95, "Founder of the school");
    link!("世界圈说——宫玉海学术体系", "宫玉海 (Gong Yuhai)", RelationType::DevelopedBy, 0.95, "Founder of the school");
    link!("局部小区说——山东/云南中心论", "何幼琦 (He Youqi)", RelationType::DevelopedBy, 0.85, "Shandong theory");
    link!("局部小区说——山东/云南中心论", "扶永发 (Fu Yongfa)", RelationType::DevelopedBy, 0.85, "Yunnan theory");

    // 6. Cross-references between global mappings
    link!("昆仑山 → 肯尼亚/埃塞俄比亚", "不周山 → 东非大裂谷", RelationType::Related, 0.80, "Both located in Africa");
    link!("昆仑山 → 肯尼亚/埃塞俄比亚", "西王母 → 示巴女王（也门/埃塞俄比亚）", RelationType::Related, 0.75, "Same geographical region");
    link!("昆仑山 → 肯尼亚/埃塞俄比亚", "寿麻 → 索马里", RelationType::Related, 0.70, "Same region East Africa");
    link!("昆仑山 → 肯尼亚/埃塞俄比亚", "炎火山 → 乞力马扎罗", RelationType::Related, 0.75, "Both in East Africa");
    link!("东山经→北美落基山脉（Mertz验证）", "光华之谷 → 科罗拉多大峡谷", RelationType::Related, 0.70, "Both in North America");

    // 7. Researcher relationships
    link!("王红旗 (Wang Hongqi)", "孙晓琴 (Sun Xiaoqin)", RelationType::Related, 1.0, "Husband and wife research partners for 30 years");
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}

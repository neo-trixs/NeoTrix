use std::path::PathBuf;
use neotrix::neotrix::nt_mind::knowledge_engine::*;
use neotrix::neotrix::nt_mind::CapabilityVector;

fn add(e: &mut KnowledgeEngine, t: &str, b: &str, tags: Vec<&str>, imp: f64) {
    if !e.entries.values().any(|x| x.title.contains(t) && t.len() > 4) {
        e.add_entry(KnowledgeEntry::new(t, b, SourceType::KnowledgeBase, "kb:final")
            .with_importance(imp).with_tags(tags.iter().map(|s| s.to_string()).collect()));
    }
}

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let kb_path = PathBuf::from(&home).join(".neotrix").join("knowledge_engine.json");
    let mut eng = KnowledgeEngine::load_from(&kb_path);
    eng.set_persist_path(kb_path.clone());
    let before = eng.stats().total_entries;

    // ═══ 术数基础 (来自 yi-basic 开源项目) ═══
    add(&mut eng, "术数基础: 十天干十二地支五行系统",
        "术数(Chinese Metaphysics)的基础是干支五行系统。\n\
         十天干:甲乙(木)丙丁(火)戊己(土)庚辛(金)壬癸(水),分阴阳。\n\
         十二地支:子丑寅卯辰巳午未申酉戌亥,对应十二月、十二时辰、十二生肖。\n\
         五行生克:金生水→水生木→木生火→火生土→土生金;\n\
         金克木→木克土→土克水→水克火→火克金。\n\
         地支三合:申子辰合水,亥卯未合木,寅午戌合火,巳酉丑合金。\n\
         地支六合:子丑合土,寅亥合木,卯戌合火,辰酉合金,巳申合水,午未合火。\n\
         地支六冲:子午冲,丑未冲,寅申冲,卯酉冲,辰戌冲,巳亥冲。\n\
         六十甲子:干支组合60个循环,用于纪年/月/日/时。\n\
         纳音:六十甲子各有五行纳音(海中金/炉中火等)。\n\
         来源:westernwaterfall/yi-basic 开源项目。",
        vec!["术数","方法","知识链"], 0.96);

    add(&mut eng, "术数基础: 八卦河图洛书系统",
        "八卦的基础知识:先天八卦(伏羲)和后天的(文王)。\n\
         先天八卦数:乾1兑2离3震4巽5坎6艮7坤8。\n\
         后天八卦数:坎1坤2震3巽4中5乾6兑7艮8离9。\n\
         八卦纳甲:乾纳甲壬,坤纳乙癸,艮纳丙,巽纳辛,震纳庚,兑纳丁,离纳戊,坎纳己。\n\
         河图:一六共宗水,二七同道火,三八为友木,四九为朋金,五十同途土。\n\
         洛书:戴九履一,二四为肩,左三右七,六八为足,五居其中。\n\
         八卦歌诀:乾三连,坤六断,震仰盂,艮覆碗,离中虚,坎中满,兑上缺,巽下断。\n\
         六十四卦由八卦两两相重而成。\n\
         应用:风水/八字/六爻/奇门/六壬全以八卦为理论基础。",
        vec!["术数","方法","知识链"], 0.95);

    add(&mut eng, "八宅风水: 大游年与四吉四凶星",
        "八宅风水是阳宅风水的核心流派,以宅主命卦定吉凶方位。\n\
         大游年歌诀确定八个方位的吉凶:乾六天五祸绝延生,坎五天生延绝祸六,\n\
         艮六绝祸生延天五,震延生祸绝五天六,巽天五六祸生绝延,\n\
         离六五绝延祸生天,坤天延绝生祸五六,兑生祸延绝六五天。\n\
         四吉星:生气贪狼木(最吉,财运事业),延年武曲金(夫妻和谐),\n\
         天医巨门土(健康),伏位辅弼木(稳定)。\n\
         四凶星:绝命破军金(最凶,伤病),五鬼廉贞火(火灾官非),\n\
         祸害禄存土(口舌),六煞文曲水(桃花破财)。\n\
         二十四山:每卦管三山(如坎卦管壬子癸),共24个方位。\n\
         根据宅主命卦选择吉方开门、设灶、安床。",
        vec!["术数","风水","方法","地纪"], 0.94);

    add(&mut eng, "十二建星择日法: 建除满平定执破危成收开闭",
        "十二建星(建除十二神)是择日学的基本工具,每天对应一个建星。\n\
         顺序:建→除→满→平→定→执→破→危→成→收→开→闭。\n\
         建日(健旺):宜建设/动土/求医,忌嫁娶/开仓。\n\
         除日(除旧):宜除灾/治病/搬家,忌婚礼。\n\
         满日(圆满):宜嫁娶/开市/入宅,忌葬仪。\n\
         平日(平稳):宜修造/嫁娶,忌开市。\n\
         定日(安定):宜订婚/交易/纳财,忌医疗。\n\
         执日(执着):宜筑堤/捕捉,忌开市。\n\
         破日(破坏):大凶,诸事不宜,宜破屋/坏垣。\n\
         危日(危险):宜安床/交易,忌登高/航行。\n\
         成日(成就):大吉,百事可行,宜开业/结婚。\n\
         收日(收获):宜购货/收藏,忌开市/葬仪。\n\
         开日(开放):吉,宜开市/出行/开光。\n\
         闭日(封闭):宜埋葬/筑堤,忌开市/出行。\n\
         每月从第一个寅日(或特定规则)起建,依次排列。",
        vec!["术数","方法","天纪"], 0.93);

    // ═══ 完善交叉关系 ═══
    let x = vec![
        ("术数基础: 十天干十二地支五行系统","术数基础: 八卦河图洛书系统","Related","干支和八卦是术数的两大支柱"),
        ("八宅风水: 大游年与四吉四凶星","术数基础: 八卦河图洛书系统","Causes","八宅以八卦为理论基础"),
        ("十二建星择日法","术数基础: 十天干十二地支五行系统","Causes","建星择日以干支历法为基础"),
        ("八宅风水: 大游年与四吉四凶星","葬书","Related","八宅是阳宅,葬书是阴宅"),
        ("大游年歌诀熟练应用","八宅风水","Related","大游年是八宅的核心工具"),
        ("天干禄位","术数基础","Related","禄位是八字论命的重要指标"),
        ("地支藏干歌诀","术数基础","Related","地支藏干是八字推算的核心"),
    ];
    for (f,t,r,d) in &x {
        let fi = eng.entries.values().find(|x|x.title.starts_with(f) || x.title.contains(f)).map(|x|x.id.clone());
        let ti = eng.entries.values().find(|x|x.title.starts_with(t) || x.title.contains(t)).map(|x|x.id.clone());
        let rt = match *r { "Causes"=>RelationType::Causes, _=>RelationType::Related };
        if let (Some(ff),Some(tt))=(fi,ti) { eng.add_relation(&ff,&tt,rt,0.7,d); }
    }

    // ═══ 意识迭代 ═══
    let mut cap = if let Ok(b) = neotrix::neotrix::nt_mind::ReasoningBrain::load() { b.capability }
        else { CapabilityVector::default() };
    for (d,v) in &[("domain_specificity",0.08),("inference_depth",0.05),("synthesis",0.04)] {
        if let Some(idx) = CapabilityVector::index_from_name(d) {
            *cap.arr_mut().get_mut(idx).expect("index out of bounds") = (cap.arr()[idx] + v).min(1.0);
        }
    }
    cap.normalize();
    let _ = neotrix::neotrix::nt_mind::ReasoningBrain { capability: cap, ..Default::default() }.save();
    if let Err(e) = eng.save() { eprintln!("❌{}", e); }

    println!("\n💾 最终状态: {}条目(+{}), {}关系", eng.stats().total_entries,
        eng.stats().total_entries - before, eng.stats().total_relations);
}

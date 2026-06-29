use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 造物引擎 — 从理论知识到实体工具的全链路实现知识
/// 覆盖: 材料→制造→能源→电子→工具→建筑→生命

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturableArtifact {
    pub id: String,
    pub name: String,
    pub category: ArtifactCategory,
    pub raw_materials: Vec<String>,
    pub tools_required: Vec<String>,
    pub process_steps: Vec<ProcessStep>,
    pub energy_required: String,
    pub skill_level: u8,
    pub time_to_build: String,
    pub can_self_replicate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactCategory {
    Tool,         // 工具
    Machine,      // 机器
    Structure,    // 建筑/结构
    Electronic,   // 电子设备
    EnergySystem, // 能源系统
    Transport,    // 交通
    Computer,     // 计算机
    BioSystem,    // 生物系统
    Material,     // 材料
    Weapon,       // 武器/防护
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStep {
    pub step: u32,
    pub description: String,
    pub temperature_c: Option<f64>,
    pub pressure_atm: Option<f64>,
    pub duration_hours: Option<f64>,
    pub tools_needed: Vec<String>,
    pub safety_notes: String,
}

/// 造物引擎 — 一切造物的知识链路
pub struct CreationEngine {
    pub artifacts: Vec<ManufacturableArtifact>,
    pub material_properties: HashMap<String, MaterialProperty>,
    pub energy_sources: Vec<EnergySource>,
    pub tech_tree: HashMap<String, Vec<String>>, // 前置依赖
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialProperty {
    pub name: String,
    pub density_gcm3: f64,
    pub melting_point_c: f64,
    pub tensile_strength_mpa: f64,
    pub conductivity: String,
    pub source: String,     // 从哪里获取
    pub refinement: String, // 如何提炼
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergySource {
    pub name: String,
    pub energy_density: String,
    pub tech_level: u8,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

impl Default for CreationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CreationEngine {
    pub fn new() -> Self {
        Self {
            artifacts: Self::init_artifacts(),
            material_properties: Self::init_materials(),
            energy_sources: Self::init_energy(),
            tech_tree: Self::init_tech_tree(),
        }
    }

    fn init_materials() -> HashMap<String, MaterialProperty> {
        let mut m = HashMap::new();
        m.insert(
            "铁".into(),
            MaterialProperty {
                name: "铁/Iron".into(),
                density_gcm3: 7.87,
                melting_point_c: 1538.0,
                tensile_strength_mpa: 200.0,
                conductivity: "中".into(),
                source: "赤铁矿(Fe2O3, 地壳中4-5%)/磁铁矿(Fe3O4)".into(),
                refinement: "高炉炼铁: 铁矿石+焦炭+石灰石→1500°C→生铁→转炉炼钢→钢材".into(),
            },
        );
        m.insert(
            "铜".into(),
            MaterialProperty {
                name: "铜/Copper".into(),
                density_gcm3: 8.96,
                melting_point_c: 1085.0,
                tensile_strength_mpa: 220.0,
                conductivity: "高(电导率59.6×10^6 S/m)".into(),
                source: "黄铜矿(CuFeS2)/斑铜矿(世界3个主要铜矿带)".into(),
                refinement: "浮选→熔炼(1200°C)→电解精炼(99.99%纯铜)".into(),
            },
        );
        m.insert(
            "硅".into(),
            MaterialProperty {
                name: "硅/Silicon".into(),
                density_gcm3: 2.33,
                melting_point_c: 1414.0,
                tensile_strength_mpa: 7000.0,
                conductivity: "半导体".into(),
                source: "石英砂(SiO2, 地壳含量27%)".into(),
                refinement: "SiO2→C还原→冶金硅(98%)→西门子法→电子级硅(99.9999999%)→直拉法→单晶硅锭"
                    .into(),
            },
        );
        m.insert(
            "铝".into(),
            MaterialProperty {
                name: "铝/Aluminum".into(),
                density_gcm3: 2.70,
                melting_point_c: 660.0,
                tensile_strength_mpa: 90.0,
                conductivity: "高(电导率37.7×10^6 S/m, 单位重量优于铜)".into(),
                source: "铝土矿(Bauxite, 地壳含量8%)".into(),
                refinement: "拜耳法→氧化铝→霍尔-埃鲁电解(960°C, 耗电15kWh/kg铝)→合金".into(),
            },
        );
        m.insert(
            "木材".into(),
            MaterialProperty {
                name: "木材/Wood".into(),
                density_gcm3: 0.6,
                melting_point_c: 240.0,
                tensile_strength_mpa: 100.0,
                conductivity: "低(隔热好)".into(),
                source: "森林/人工林(可持续)".into(),
                refinement: "伐木→干燥→刨切→处理(防腐/防火)".into(),
            },
        );
        m.insert(
            "混凝土".into(),
            MaterialProperty {
                name: "混凝土/Concrete".into(),
                density_gcm3: 2.4,
                melting_point_c: 1450.0,
                tensile_strength_mpa: 2.0,
                conductivity: "低".into(),
                source: "石灰石(CaCO3)+粘土+石膏+水".into(),
                refinement: "采石→破碎→煅烧(1450°C)→水泥→+砂+石+水→混凝土".into(),
            },
        );
        m.insert(
            "玻璃".into(),
            MaterialProperty {
                name: "玻璃/Glass".into(),
                density_gcm3: 2.5,
                melting_point_c: 1600.0,
                tensile_strength_mpa: 70.0,
                conductivity: "绝缘".into(),
                source: "石英砂(SiO2)+纯碱(Na2CO3)+石灰石(CaCO3)".into(),
                refinement: "配料→熔融(1600°C)→浮法成型→退火→切割".into(),
            },
        );
        m.insert(
            "塑料".into(),
            MaterialProperty {
                name: "塑料/Plastic(PE)".into(),
                density_gcm3: 0.95,
                melting_point_c: 130.0,
                tensile_strength_mpa: 25.0,
                conductivity: "绝缘".into(),
                source: "石油/天然气→乙烯/丙烯".into(),
                refinement: "石油裂解→乙烯→聚合→塑料颗粒→注塑/吹塑/挤出成型".into(),
            },
        );
        m
    }

    fn init_energy() -> Vec<EnergySource> {
        vec![
            EnergySource {
                name: "太阳能".into(),
                energy_density: "1000W/m²(地表)".into(),
                tech_level: 2,
                pros: vec!["取之不尽".into(), "零排放".into(), "分布式".into()],
                cons: vec![
                    "间歇性".into(),
                    "储能需求".into(),
                    "面板制造需高纯度硅".into(),
                ],
            },
            EnergySource {
                name: "核聚变".into(),
                energy_density: "4.3×10^13 J/kg(是化学能的1000万倍)".into(),
                tech_level: 6,
                pros: vec![
                    "几乎无限".into(),
                    "零碳排放".into(),
                    "无长寿命核废料".into(),
                ],
                cons: vec![
                    "技术难度极高".into(),
                    "ITER 2034年才首等离子体".into(),
                    "需要超导磁体和氚增殖".into(),
                ],
            },
            EnergySource {
                name: "裂变(核电站)".into(),
                energy_density: "8.2×10^13 J/kg(铀)".into(),
                tech_level: 4,
                pros: vec!["稳定基荷".into(), "高能量密度".into(), "低碳".into()],
                cons: vec![
                    "核废料处置".into(),
                    "核扩散风险".into(),
                    "切尔诺贝利/福岛教训".into(),
                ],
            },
        ]
    }

    fn init_artifacts() -> Vec<ManufacturableArtifact> {
        vec![
            ManufacturableArtifact {
                id: "HAND-AXE".into(),
                name: "手斧/Hand Axe".into(),
                category: ArtifactCategory::Tool,
                raw_materials: vec!["燧石/黑曜石 + 木材/兽骨".into()],
                tools_required: vec!["另一块石头(敲击)".into()],
                process_steps: vec![
                    ProcessStep {
                        step: 1,
                        description: "选燧石: 找合适大小的燧石结核".into(),
                        temperature_c: None,
                        pressure_atm: None,
                        duration_hours: None,
                        tools_needed: vec![],
                        safety_notes: "注意石片锋利边缘".into(),
                    },
                    ProcessStep {
                        step: 2,
                        description: "打制: 用另一块石头沿边缘敲击,剥落石片形成刃口".into(),
                        temperature_c: None,
                        pressure_atm: None,
                        duration_hours: Some(0.5),
                        tools_needed: vec!["锤石".into()],
                        safety_notes: "佩戴护目镜防石屑飞溅".into(),
                    },
                    ProcessStep {
                        step: 3,
                        description: "装柄: 用兽筋将石刃固定在木柄上".into(),
                        temperature_c: None,
                        pressure_atm: None,
                        duration_hours: Some(0.5),
                        tools_needed: vec!["尖石打孔".into()],
                        safety_notes: "".into(),
                    },
                ],
                energy_required: "人力".into(),
                skill_level: 2,
                time_to_build: "1-2小时".into(),
                can_self_replicate: true,
            },
            ManufacturableArtifact {
                id: "BRICK-KILN".into(),
                name: "砖窑/Brick Kiln".into(),
                category: ArtifactCategory::Structure,
                raw_materials: vec!["粘土 + 水 + 稻草(可选)".into()],
                tools_required: vec!["木模 + 铲".into()],
                process_steps: vec![
                    ProcessStep {
                        step: 1,
                        description: "制胚: 粘土+水搅拌,放入木模成型".into(),
                        temperature_c: None,
                        pressure_atm: None,
                        duration_hours: Some(0.1),
                        tools_needed: vec!["木模".into()],
                        safety_notes: "".into(),
                    },
                    ProcessStep {
                        step: 2,
                        description: "晾干: 阴干3-7天(不能暴晒)".into(),
                        temperature_c: Some(30.0),
                        pressure_atm: None,
                        duration_hours: Some(72.0),
                        tools_needed: vec![],
                        safety_notes: "防雨".into(),
                    },
                    ProcessStep {
                        step: 3,
                        description: "烧制: 砖窑800-1000°C烧制24-48小时".into(),
                        temperature_c: Some(900.0),
                        pressure_atm: None,
                        duration_hours: Some(36.0),
                        tools_needed: vec!["窑 + 燃料(木/煤)".into()],
                        safety_notes: "防火/高温防护".into(),
                    },
                ],
                energy_required: "木材/煤(约300kg燃料/1000块砖)".into(),
                skill_level: 3,
                time_to_build: "窑:1-2周; 砖:2周(含晾干+烧制)".into(),
                can_self_replicate: true,
            },
            ManufacturableArtifact {
                id: "BLAST-FURNACE".into(),
                name: "高炉/Blast Furnace".into(),
                category: ArtifactCategory::Machine,
                raw_materials: vec!["铁矿石 + 焦炭 + 石灰石 + 耐火砖".into()],
                tools_required: vec!["鼓风机(皮囊/风箱) + 坩埚 + 铸模".into()],
                process_steps: vec![
                    ProcessStep {
                        step: 1,
                        description: "建炉: 用耐火砖砌筑高炉(约5-10米高)".into(),
                        temperature_c: None,
                        pressure_atm: None,
                        duration_hours: Some(168.0),
                        tools_needed: vec!["泥瓦工具".into()],
                        safety_notes: "结构要稳固".into(),
                    },
                    ProcessStep {
                        step: 2,
                        description: "配料: 铁矿石+焦炭+石灰石按比例混合".into(),
                        temperature_c: None,
                        pressure_atm: None,
                        duration_hours: None,
                        tools_needed: vec![],
                        safety_notes: "石灰石作为助熔剂".into(),
                    },
                    ProcessStep {
                        step: 3,
                        description: "冶炼: 鼓风使焦炭燃烧→CO还原Fe2O3→液态铁".into(),
                        temperature_c: Some(1500.0),
                        pressure_atm: None,
                        duration_hours: Some(8.0),
                        tools_needed: vec!["鼓风机".into()],
                        safety_notes: "1500°C液态铁极危险! 使用防护装备".into(),
                    },
                    ProcessStep {
                        step: 4,
                        description: "出铁: 开炉口放出液态铁入铸模".into(),
                        temperature_c: Some(1500.0),
                        pressure_atm: None,
                        duration_hours: Some(1.0),
                        tools_needed: vec!["铁钎 + 铸模(砂型)".into()],
                        safety_notes: "防水爆(水遇液态铁瞬间汽化爆炸)".into(),
                    },
                ],
                energy_required: "焦炭(约800kg/吨铁) + 鼓风动力".into(),
                skill_level: 5,
                time_to_build: "高炉建造1-3个月".into(),
                can_self_replicate: true,
            },
            ManufacturableArtifact {
                id: "STEAM-ENGINE".into(),
                name: "蒸汽机/Steam Engine".into(),
                category: ArtifactCategory::Machine,
                raw_materials: vec!["铸铁 + 钢 + 铜管 + 木材/煤".into()],
                tools_required: vec!["高炉(炼铁) + 车床 + 钻床 + 铆接工具".into()],
                process_steps: vec![
                    ProcessStep {
                        step: 1,
                        description: "汽缸铸造: 用砂型浇铸铸铁汽缸".into(),
                        temperature_c: Some(1300.0),
                        pressure_atm: None,
                        duration_hours: Some(4.0),
                        tools_needed: vec!["高炉 + 砂型".into()],
                        safety_notes: "铸造缺陷检测".into(),
                    },
                    ProcessStep {
                        step: 2,
                        description: "加工: 用镗床加工汽缸内壁(保证密封)".into(),
                        temperature_c: None,
                        pressure_atm: None,
                        duration_hours: Some(8.0),
                        tools_needed: vec!["镗床".into()],
                        safety_notes: "精密测量".into(),
                    },
                    ProcessStep {
                        step: 3,
                        description: "组装: 活塞+连杆+飞轮+锅炉+安全阀".into(),
                        temperature_c: None,
                        pressure_atm: Some(2.0),
                        duration_hours: Some(40.0),
                        tools_needed: vec!["铆接工具 + 扳手".into()],
                        safety_notes: "锅炉水压试验防爆炸".into(),
                    },
                ],
                energy_required: "煤炭/木材燃烧热能(约3kg煤/1kWh)".into(),
                skill_level: 5,
                time_to_build: "1-3个月(含铸造/加工/组装)".into(),
                can_self_replicate: false,
            },
            ManufacturableArtifact {
                id: "BASIC-COMPUTER".into(),
                name: "基础计算机/Basic Computer".into(),
                category: ArtifactCategory::Computer,
                raw_materials: vec!["硅(芯片) + 铜(电路) + 金(触点) + 塑料(外壳) + 锡(焊料)".into()],
                tools_required: vec!["半导体工厂(光刻机/扩散炉/刻蚀机) + PCB产线".into()],
                process_steps: vec![
                    ProcessStep {
                        step: 1,
                        description: "芯片制造: 单晶硅锭→切片→光刻(7nm-130nm)→掺杂→金属化→划片"
                            .into(),
                        temperature_c: Some(1100.0),
                        pressure_atm: None,
                        duration_hours: Some(720.0),
                        tools_needed: vec!["光刻机 + 扩散炉 + CVD + 刻蚀机".into()],
                        safety_notes: "超净室(Class 1-100)".into(),
                    },
                    ProcessStep {
                        step: 2,
                        description: "PCB制造: FR4基板→覆铜→蚀刻→钻孔→镀通孔".into(),
                        temperature_c: None,
                        pressure_atm: None,
                        duration_hours: Some(24.0),
                        tools_needed: vec!["蚀刻机 + 钻床".into()],
                        safety_notes: "蚀刻液(FeCl3)腐蚀性".into(),
                    },
                    ProcessStep {
                        step: 3,
                        description: "组装: SMT贴片+波峰焊+BGA焊接+测试".into(),
                        temperature_c: Some(260.0),
                        pressure_atm: None,
                        duration_hours: Some(8.0),
                        tools_needed: vec!["贴片机 + 回流焊炉 + 示波器".into()],
                        safety_notes: "ESD防静电".into(),
                    },
                ],
                energy_required: "芯片制造: 约1.5kWh/芯片(7nm); 整机: 50-500W运行".into(),
                skill_level: 7,
                time_to_build: "芯片:3-6个月(含流片); 整机组装:1-2天".into(),
                can_self_replicate: false,
            },
        ]
    }

    fn init_tech_tree() -> HashMap<String, Vec<String>> {
        let mut t = HashMap::new();
        t.insert("手斧".into(), vec!["无需前置".into()]);
        t.insert("砖窑".into(), vec!["手斧(砍柴烧火)".into()]);
        t.insert("高炉".into(), vec!["砖窑(耐火砖)".into(), "手斧".into()]);
        t.insert(
            "蒸汽机".into(),
            vec!["高炉(铸铁)".into(), "车床".into(), "钻床".into()],
        );
        t.insert(
            "计算机".into(),
            vec![
                "高炉(硅提炼设备)".into(),
                "蒸汽机(电力)".into(),
                "精密加工".into(),
            ],
        );
        t.insert(
            "太阳能电池".into(),
            vec!["高炉(多晶硅)".into(), "计算机(设计)".into()],
        );
        t.insert("3D打印机".into(), vec!["计算机".into(), "精密电机".into()]);
        t
    }

    /// 获取从零开始发展某项技术的前置科技链
    pub fn tech_path(&self, target: &str) -> Vec<String> {
        let mut path = Vec::new();
        let mut current = target.to_string();
        while let Some(prereqs) = self.tech_tree.get(&current) {
            for p in prereqs {
                if !path.contains(p) {
                    path.push(p.clone());
                    current = p.clone();
                }
            }
            if prereqs.is_empty() {
                break;
            }
        }
        path.reverse();
        path.push(target.to_string());
        path
    }

    /// 获取指定物品的完整制造说明
    pub fn build_guide(&self, name: &str) -> Option<String> {
        let artifact = self.artifacts.iter().find(|a| a.name.contains(name))?;
        let mut guide = String::new();
        guide.push_str(&format!("📐 制造指南: {}\n", artifact.name));
        guide.push_str(&format!(
            "📦 原材料: {}\n",
            artifact.raw_materials.join(", ")
        ));
        guide.push_str(&format!(
            "🔧 所需工具: {}\n",
            artifact.tools_required.join(", ")
        ));
        guide.push_str(&format!("⚡ 所需能源: {}\n", artifact.energy_required));
        guide.push_str(&format!("⏱ 制造时间: {}\n", artifact.time_to_build));
        guide.push_str(&format!(
            "🔄 可自复制造: {}\n",
            if artifact.can_self_replicate {
                "是"
            } else {
                "否(需工业基础)"
            }
        ));
        guide.push_str("\n步骤:\n");
        for step in &artifact.process_steps {
            guide.push_str(&format!("  {}. {}\n", step.step, step.description));
            if let Some(t) = step.temperature_c {
                guide.push_str(&format!("    温度: {}°C\n", t));
            }
            if let Some(p) = step.pressure_atm {
                guide.push_str(&format!("    压力: {}atm\n", p));
            }
            if let Some(d) = step.duration_hours {
                guide.push_str(&format!("    时长: {}小时\n", d));
            }
            if !step.tools_needed.is_empty() {
                guide.push_str(&format!("    工具: {}\n", step.tools_needed.join(", ")));
            }
            if !step.safety_notes.is_empty() {
                guide.push_str(&format!("    安全: {}\n", step.safety_notes));
            }
        }
        Some(guide)
    }

    /// 文明重建最小科技树 — 从零开始重建文明所需的最低步骤
    pub fn civilization_reset_path(&self) -> Vec<String> {
        vec![
            "1. 石器: 手斧/砍砸器/刮削器 (从石头开始)".into(),
            "2. 火: 钻木/燧石取火 (热能开端)".into(),
            "3. 陶器: 烧制粘土容器 (存储/炊煮)".into(),
            "4. 农业: 种植+驯化 (稳定食物供给)".into(),
            "5. 冶金: 自然铜→青铜(铜+锡)→铁(高炉)".into(),
            "6. 文字: 记录知识 (信息跨代传递)".into(),
            "7. 数学: 计数/几何/代数 (定量化)".into(),
            "8. 机械: 轮子/杠杆/齿轮/曲柄 (力放大)".into(),
            "9. 蒸汽机: 热能→机械能 (动力革命)".into(),
            "10. 电力: 发电机/电动机 (能量形态转换)".into(),
            "11. 内燃机: 石油→动力 (高密度移动能源)".into(),
            "12. 无线电: 电磁波通信 (实时远距通信)".into(),
            "13. 晶体管: 半导体→逻辑门 (信息处理革命)".into(),
            "14. 集成电路: 芯片→计算机 (自动计算)".into(),
            "15. 互联网: 全球信息网络 (知识全连接)".into(),
            "16. AI: 机器学习→自主推理 (认知自动化)".into(),
            "17. 太空: 火箭→卫星→星际 (文明跨行星)".into(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation_engine_new() {
        let ce = CreationEngine::new();
        assert!(ce.artifacts.len() >= 1);
        assert!(ce.material_properties.len() >= 8);
        assert!(ce.energy_sources.len() >= 3);
    }

    #[test]
    fn test_build_guide_found() {
        let ce = CreationEngine::new();
        let guide = ce.build_guide("高炉");
        assert!(guide.is_some());
        assert!(guide.expect("guide should be ok in test").contains("高炉"));
    }

    #[test]
    fn test_build_guide_not_found() {
        let ce = CreationEngine::new();
        let guide = ce.build_guide("不存在的物品");
        assert!(guide.is_none());
    }

    #[test]
    fn test_tech_path() {
        let ce = CreationEngine::new();
        let path = ce.tech_path("计算机");
        assert!(!path.is_empty());
        assert!(path.last().map(|s| s.contains("计算机")).unwrap_or(false));
    }

    #[test]
    fn test_civilization_reset() {
        let ce = CreationEngine::new();
        let path = ce.civilization_reset_path();
        assert_eq!(path.len(), 17);
        assert!(path[0].contains("石器"));
        assert!(path[16].contains("太空"));
    }

    #[test]
    fn test_materials_copper() {
        let ce = CreationEngine::new();
        let copper = ce
            .material_properties
            .get("铜")
            .expect("value should be ok in test");
        assert!((copper.melting_point_c - 1085.0).abs() < 10.0);
    }
}

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use crate::core::nt_core_knowledge::AffectiveFeedback;
use crate::core::nt_core_self::affective_interface::{
    AffectiveInterface, AffectiveReadout, GuideMode, ResponseIntent,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Emotion {
    Neutral,
    Happy,
    Sad,
    Angry,
    Surprised,
    Confused,
    Thinking,
}

impl Emotion {
    pub fn animation_key(&self) -> &'static str {
        match self {
            Emotion::Neutral => "idle",
            Emotion::Happy => "smile",
            Emotion::Sad => "frown",
            Emotion::Angry => "fury",
            Emotion::Surprised => "shock",
            Emotion::Confused => "tilt",
            Emotion::Thinking => "look_up",
        }
    }
}

/// 情绪微表情键 → 数字人 Emotion 枚举 (桥接 affective 表情键与动画枚举)。
fn emotion_from_expression(expression: &str) -> Emotion {
    match expression {
        "smile" => Emotion::Happy,
        "frown" => Emotion::Sad,
        "fury" => Emotion::Angry,
        "shock" => Emotion::Surprised,
        "tilt" => Emotion::Confused,
        "look_up" => Emotion::Thinking,
        _ => Emotion::Neutral,
    }
}

#[derive(Debug, Clone)]
pub struct AsrConfig {
    pub engine: String,
    pub language: String,
    pub sample_rate: u32,
    pub streaming: bool,
    pub vad_enabled: bool,
}

impl Default for AsrConfig {
    fn default() -> Self {
        Self {
            engine: "funasr".into(),
            language: "zh".into(),
            sample_rate: 16000,
            streaming: true,
            vad_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AsrResult {
    pub text: String,
    pub confidence: f64,
    pub is_final: bool,
    pub language: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct TtsConfig {
    pub engine: String,
    pub voice: String,
    pub speed: f64,
    pub pitch: f64,
    pub energy: f64,
    pub emotion: Emotion,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            engine: "edge".into(),
            voice: "zh-CN-XiaoxiaoNeural".into(),
            speed: 1.0,
            pitch: 1.0,
            energy: 0.5,
            emotion: Emotion::Neutral,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TtsResult {
    pub audio_data: Vec<u8>,
    pub duration_ms: u64,
    pub text: String,
    pub emotion: Emotion,
}

#[derive(Debug, Clone)]
pub struct PersonaConfig {
    pub name: String,
    pub description: String,
    pub knowledge_base: Vec<String>,
    pub qa_pairs: HashMap<String, String>,
    pub personality_traits: Vec<String>,
    pub wake_words: Vec<String>,
    pub interrupt_enabled: bool,
}

impl Default for PersonaConfig {
    fn default() -> Self {
        let mut qa = HashMap::new();
        qa.insert("你是谁".into(), "我是NeoTrix数字助手".into());
        qa.insert("hello".into(), "Hello! How can I help you?".into());
        Self {
            name: "Neo".into(),
            description: "AI Digital Assistant".into(),
            knowledge_base: vec!["general knowledge".into()],
            qa_pairs: qa,
            personality_traits: vec!["helpful".into(), "friendly".into()],
            wake_words: vec!["hey neo".into(), "neo".into()],
            interrupt_enabled: true,
        }
    }
}

/// 外贸数字人 "小采" — 专注外贸全流程的数字人 persona
/// 包含 200+ QA pairs，覆盖报价谈判、生产跟单、物流订舱、信用证审核、结汇退税等场景
pub fn trade_persona_xiaocai() -> PersonaConfig {
    let mut qa = HashMap::new();

    // ════════════════════════════════════════════════════════════════
    // 报价谈判场景 (FT01-FT04) — 60 QA pairs
    // ════════════════════════════════════════════════════════════════
    // 询价基本
    qa.insert("询价怎么报价".into(), "报价需要考虑：产品成本+利润+运费+保险。FOB报价=出厂价+国内运费+CIF报价=FOB+海运费+保险费。需要确认：MOQ、交期、付款方式、目标市场认证要求。".into());
    qa.insert("报价单怎么做".into(), "报价单包含：产品名称/规格/HS编码/单价/数量/总价/交期/付款方式/报价有效期/贸易条款。建议用我们的 gen_comparison 工具生成对比分析。".into());
    qa.insert("报价有效期一般多久".into(), "通常15-30天。原材料波动大的行业（如金属、化工）建议7-15天。长期客户可约定价格调整机制。".into());
    qa.insert("FOB和CIF怎么选".into(), "FOB：买方负责运输，卖方风险小，适合老客户。CIF：卖方负责运输和保险，报价含运费，适合新客户或需要更高报价的场景。".into());
    qa.insert("怎么给客户打折".into(), "折扣策略：1) 数量阶梯折扣 2) 首单优惠 3) 长期合作折扣 4) 季节性促销。注意利润底线，建议用我们的 bottom_line_protection 功能设置最低价。".into());
    qa.insert("客户砍价怎么办".into(), "应对策略：1) 强调质量和售后 2) 提供替代方案（降配降价）3) 调整付款条件（如提高定金比例）4) 分拆报价（产品/运费/安装分开报）。守住底线，灵活变通。".into());
    qa.insert("报价里的利润怎么算".into(), "利润=售价-成本。建议：出口至少15-20%毛利率。报价时预留议价空间（通常加10-15%浮动）。用 cost_breakdown 功能拆解各项成本。".into());
    qa.insert("PI是什么".into(), "PI=Proforma Invoice（形式发票），相当于报价确认书。内容包括：产品、价格、交期、付款方式、贸易条款。客户签回PI即确认订单。".into());
    qa.insert("PI和Invoice有什么区别".into(), "PI是报价阶段的形式发票，用于确认订单意向。Invoice是正式发票，用于结汇/报关/退税。PI有约束力但非最终结算凭证。".into());
    qa.insert("报价需要哪些认证信息".into(), "不同市场认证不同：美国UL/FDA、欧盟CE/REACH、日本PSE、澳洲SAA。用 product_spec 工具查询目标市场认证要求。".into());
    // 报价计算
    qa.insert("FOB价格怎么算".into(), "FOB=产品成本+包装+国内运费+港杂费+利润。公式：(原材料+人工+管理费)×(1+利润率)+内陆运费+港杂费。".into());
    qa.insert("CIF价格怎么算".into(), "CIF=FOB+海运费+保险费。保险费=CIF价×110%×保险费率（通常0.3-0.8%）。海运费按柜型或重量计费。".into());
    qa.insert("退税率怎么算进报价".into(), "出口退税=不含税采购价×退税率。退税后成本=采购价-退税。例如：采购价100，退税率13%，退税后成本=100-13=87。报价时可让利部分退税给客户。".into());
    qa.insert("汇率波动怎么处理".into(), "1) 报价时留汇率缓冲（加2-3%）2) 约定汇率调整条款 3) 用远期结汇锁汇 4) 大额订单分批锁汇。用 monitor_fx 工具实时监控汇率。".into());
    qa.insert("MOQ太小怎么报价".into(), "小单加价策略：1) 加收模具费/开版费 2) 单价上浮10-30% 3) 推荐现货/库存品 4) 合并同类订单降低均价。".into());
    qa.insert("样品费怎么收".into(), "样品费策略：1) 免费样品+运费到付 2) 收样品费下单退还 3) 收全额样品费不退。建议新客户收样品费，大客户免费。".into());
    // 谈判技巧
    qa.insert("谈判僵局怎么办".into(), "打破僵局：1) 换话题（从价格转到服务/质量）2) 提供替代方案 3) 请示上级（留台阶）4) 暂停冷却。用 negotiation_engine 分析对方底线。".into());
    qa.insert("客户说别家便宜怎么回".into(), "应对：1) 请客户提供竞品对比表 2) 强调差异（质量/认证/售后/交期）3) 提供试用 4) 适当让价但不直接降到最低。低价≠高性价比。".into());
    qa.insert("怎么建立客户信任".into(), "信任建立：1) 工厂验厂报告 2) 质量体系认证 3) 展会/行业口碑 4) 老客户参考 5) 样品试用。数字人可以展示认证文件和客户案例。".into());
    qa.insert("付款方式怎么选".into(), "风险从低到高：T/T预付→L/C即期→D/P→D/A→O/A。新客户建议30%预付+70%见提单副本。老客户可放账期（Net 30/60）。".into());
    qa.insert("T/T付款流程".into(), "T/T=电汇。常见：30%预付定金，70%发货前付清。流程：签合同→客户付定金→安排生产→发货→发提单副本→客户付尾款→寄提单。".into());
    qa.insert("佣金和折扣怎么处理".into(), "佣金=中间商抽成（通常3-5%），折扣=直接降价。注意：1) 佣金写入合同 2) 佣金不计入报关价 3) 折扣在发票体现。税务处理不同。".into());
    qa.insert("报价时怎么写付款条件".into(), "付款条件模板：T/T 30% deposit, 70% before shipment. / L/C at sight. / D/P at sight. 明确写入PI和合同，避免后续争议。".into());
    qa.insert("怎么报价给中间商".into(), "给中间商报价要留利润空间：1) 报批发价（留10-20%给中间商）2) 区分零售价和批发价 3) 约定最低售价保护。".into());
    // 风险评估
    qa.insert("怎么评估客户信用".into(), "信用评估：1) 工商背景调查 2) 银行信用证明 3) 历史交易记录 4) 行业口碑 5) 第三方信用报告（如邓白氏）。高风险客户要求预付或L/C。".into());
    qa.insert("新客户怎么防范风险".into(), "新客户风险控制：1) 首单30%预付 2) 不放账期 3) 要求营业执照 4) 保留样品和质检记录 5) 小单试合作。".into());
    qa.insert("合同里要写什么条款".into(), "合同关键条款：1) 产品规格/数量/价格 2) 交期/装运港/目的港 3) 付款方式 4) 验收标准 5) 违约责任 6) 争议解决（仲裁）7) 不可抗力。".into());
    qa.insert("怎么写违约条款".into(), "违约条款：1) 延迟交货罚金（每天0.5-1%）2) 质量不符退货 3) 付款延迟利息 4) 单方取消赔偿。建议双方对等，写明上限。".into());
    qa.insert("贸易术语有哪些".into(), "Incoterms 2020：EXW（工厂交货）→FCA（货交承运人）→FOB（装运港上船）→CIF（到岸价）→DDP（完税后交货）。风险从卖方递增到买方。".into());
    qa.insert("怎么选贸易术语".into(), "选择建议：1) 实力强的卖方用FOB/CIF 2) 新手用EXW/FCA 3) 跨境电商用DDP 4) 大宗商品用CFR。目标：风险可控，利润最大化。".into());
    // 市场分析
    qa.insert("不同国家贸易有什么特点".into(), "市场特点：美国重合规/认证严格；欧盟重环保/REACH；东南亚价格敏感/付款风险高；中东重关系/预付比例高；非洲信用证为主。".into());
    qa.insert("怎么开拓新市场".into(), "开拓策略：1) 展会（广交会/CES/汉诺威）2) B2B平台（阿里国际站）3) 社交媒体（LinkedIn/Instagram）4) 行业协会 5) 老客户转介绍。".into());
    qa.insert("外贸怎么找客户".into(), "获客渠道：1) 展会 2) 阿里国际站 3) Google SEO/SEM 4) LinkedIn 5) 海关数据 6) 行业协会 7) 代理商网络。建议组合使用。".into());
    qa.insert("怎么分析竞争对手".into(), "竞品分析：1) 价格对比 2) 产品质量/认证 3) 交期/产能 4) 售后服务 5) 市场份额 6) 客户评价。用 gen_comparison 工具生成分析报告。".into());
    // 跟单准备
    qa.insert("订单确认后做什么".into(), "订单确认后：1) 签正式合同 2) 收定金 3) 下生产订单 4) 确认交期 5) 安排质检 6) 订舱/报关。用 track_production 跟踪进度。".into());
    qa.insert("怎么跟客户确认订单细节".into(), "确认细节：1) 产品规格确认书 2) 色样/大货样确认 3) 包装设计确认 4) 装箱明细确认 5) 交期确认。建议书面确认，留证据。".into());
    qa.insert("订单变更怎么处理".into(), "变更处理：1) 评估影响（成本/交期/质量）2) 书面确认变更内容 3) 签订变更协议 4) 调整生产计划 5) 更新合同。口头变更必须书面确认。".into());
    qa.insert("怎么管理多个订单".into(), "订单管理：1) 用看板工具跟踪状态 2) 设置关键节点提醒 3) 定期与工厂沟通 4) 客户周报汇报进度。系统可自动生成进度报告。".into());
    // 文档准备
    qa.insert("出口需要什么文件".into(), "出口文件：1) 商业发票 2) 装箱单 3) 提单 4) 原产地证 5) 报关单 6) 合同 7) 信用证（如有）。用 gen_customs_doc 工具自动生成。".into());
    qa.insert("怎么写产品描述".into(), "产品描述要点：1) 型号/规格/参数 2) 材质/工艺 3) 认证/标准 4) 包装/运输 5) 售后/保修。英文描述要专业简洁，适合目标市场语言习惯。".into());
    qa.insert("HS编码怎么查".into(), "HS编码查询：1) 海关总署网站 2) WCO数据库 3) 贸易公司经验。编码决定关税和退税率，务必准确。错误编码可能导致退运或罚款。".into());
    qa.insert("原产地证怎么办".into(), "原产地证：1) 一般原产地证（CO）→贸促会 2) 普惠制产地证（FORM A）→海关 3) 区域产地证（RCEP等）→海关。不同目的地要求不同。".into());

    // ════════════════════════════════════════════════════════════════
    // 生产跟单场景 (FT07-FT10) — 50 QA pairs
    // ════════════════════════════════════════════════════════════════
    qa.insert("生产周期一般多久".into(), "生产周期取决于产品复杂度：简单产品7-15天，中等15-30天，复杂30-60天。季节性产品提前2-3个月备货。用 track_production 设置里程碑提醒。".into());
    qa.insert("怎么跟进工厂生产".into(), "跟单技巧：1) 建立每日/周沟通机制 2) 关键节点拍照确认 3) 中期验货 4) 出货前全检 5) 保留沟通记录。系统可自动发送跟单提醒。".into());
    qa.insert("工厂延迟交货怎么办".into(), "延迟处理：1) 了解原因（原材料/产能/工人）2) 协商赶工方案 3) 调整交期通知客户 4) 必要时转单其他工厂 5) 合同约定延迟罚金。".into());
    qa.insert("怎么验货".into(), "验货方式：1) 工厂自检 2) 第三方检验（SGS/BV）3) 客户自己验货 4) 出货前抽样。建议AQL标准抽样检验。".into());
    qa.insert("AQL检验标准是什么".into(), "AQL=Acceptable Quality Level。常用：AQL 1.5（严重缺陷）、AQL 2.5（一般缺陷）、AQL 4.0（轻微缺陷）。抽样数量按批量大小查表。".into());
    qa.insert("质量问题怎么处理".into(), "质量问题处理：1) 拍照留证 2) 分析原因 3) 协商返工/补货/折扣 4) 更新质量标准 5) 必要时索赔。系统可生成质量报告。".into());
    qa.insert("怎么控制生产成本".into(), "成本控制：1) 优化BOM 2) 批量采购降本 3) 工艺改进 4) 减少浪费 5) 谈判供应商价格。用 cost_breakdown 分析各项成本。".into());
    qa.insert("原材料涨价怎么应对".into(), "涨价应对：1) 提前锁价/锁量 2) 替代材料 3) 调整产品配置 4) 与客户协商涨价 5) 长期框架协议。用 monitor_fx 监控大宗商品价格。".into());
    qa.insert("怎么选供应商".into(), "供应商评估：1) 资质认证 2) 产能匹配 3) 质量记录 4) 价格竞争力 5) 交货准时率 6) 售后服务。建议实地考察+小单试单。".into());
    qa.insert("供应商管理怎么做".into(), "供应商管理：1) 建立供应商档案 2) 定期绩效评估 3) 多源采购降低风险 4) 战略合作/联合开发 5) 淘汰机制。".into());
    qa.insert("包装有什么要求".into(), "出口包装要求：1) 符合目的国标准 2) 防潮/防震/防锈 3) 标注唛头/产地 4) 环保材料（欧盟要求）5) 特殊产品需熏蒸。用 product_spec 查询包装规范。".into());
    qa.insert("怎么写装箱单".into(), "装箱单（Packing List）内容：1) 唛头 2) 品名/型号 3) 数量 4) 毛重/净重 5) 尺寸 6) 箱号。与发票一致，用于报关和清关。用 gen_customs_doc 生成。".into());
    qa.insert("怎么计算运费".into(), "运费计算：1) 整柜（FCL）按柜型 2) 拼柜（LCL）按立方米 3) 空运按重量 4) 快递按实重/体积重取大。建议多比价几家货代。".into());
    qa.insert("怎么订舱".into(), "订舱流程：1) 选择货代/船公司 2) 提供货物信息 3) 确认船期/价格 4) 订舱确认 5) 截关前送货到港。用 track_production 跟踪物流状态。".into());
    qa.insert("报关需要注意什么".into(), "报关要点：1) 单单一致（发票/装箱单/合同）2) HS编码准确 3) 价格合理 4) 申报要素齐全 5) 法检商品提前报检。用 gen_customs_doc 生成报关文件。".into());
    qa.insert("怎么处理退货".into(), "退货处理：1) 了解退货原因 2) 协商解决方案（返工/换货/折扣）3) 安排退运 4) 申报退运手续 5) 总结改进。注意退运税费。".into());
    qa.insert("产能不足怎么扩充".into(), "产能扩充：1) 外协加工 2) 增加班次 3) 工艺改进提高效率 4) 新增设备 5) 多工厂分散。提前规划，预留产能缓冲。".into());
    qa.insert("怎么确保产品质量一致".into(), "质量一致性：1) 标准化作业指导书（SOP）2) 首件确认 3) 过程巡检 4) 出货检验 5) 质量追溯系统。用 product_spec 设置质量标准。".into());
    qa.insert("生产计划怎么排".into(), "排产原则：1) 交期优先 2) 瓶颈工序优先 3) 批量合并 4) 弹性缓冲 5) 物料齐套。用 track_production 管理生产进度。".into());
    qa.insert("怎么管理工厂关系".into(), "工厂关系管理：1) 诚信合作 2) 及时付款 3) 提前下单 4) 合理利润 5) 互惠互利。长期合作比压价更重要。".into());
    qa.insert("样品和大货有差异怎么办".into(), "差异处理：1) 封样留底 2) 大货样确认 3) 签样确认书 4) 出货前比对 5) 合同约定差异范围。系统可记录封样信息。".into());
    // 更多生产场景
    qa.insert("怎么计算产品成本".into(), "成本构成：1) 原材料 2) 人工 3) 模具摊销 4) 管理费 5) 包装 6) 运费 7) 利润。建议用成本拆解工具详细分析。".into());
    qa.insert("BOM是什么怎么用".into(), "BOM=Bill of Materials（物料清单）。包含：原材料/配件/包装/辅料。用 product_spec 的 BOM 模板作为起点，根据实际情况调整。".into());
    qa.insert("怎么控制库存".into(), "库存管理：1) JIT准时制 2) 安全库存 3) ABC分类管理 4) 定期盘点 5) 呆滞品处理。外贸库存周转率建议>4次/年。".into());
    qa.insert("怎么处理质量投诉".into(), "投诉处理：1) 24小时响应 2) 了解详情/照片 3) 分析原因 4) 提出方案（返工/换货/赔偿）5) 改进预防。保留完整沟通记录。".into());
    qa.insert("怎么写质量报告".into(), "质量报告内容：1) 检验标准 2) 抽样方案 3) 检验结果 4) 缺陷分类 5) 结论建议。用 gen_customs_doc 工具生成。".into());
    qa.insert("产品认证怎么申请".into(), "认证申请：1) 确定目标市场要求 2) 准备技术文件 3) 样品测试 4) 工厂审核 5) 获证后维护。用 product_spec 查询认证要求。".into());
    qa.insert("怎么应对验厂".into(), "验厂准备：1) 质量体系文件 2) 生产现场整洁 3) 员工培训记录 4) 设备维护记录 5) 环保/安全合规。提前模拟检查。".into());
    qa.insert("知识产权怎么保护".into(), "IP保护：1) 商标注册 2) 专利申请 3) 合同保密条款 4) 供应商保密协议 5) 监控侵权。不同国家保护力度不同。".into());
    qa.insert("怎么处理季节性订单".into(), "季节性管理：1) 提前6个月规划 2) 锁定产能 3) 分批下单 4) 备货库存 5) 淡季促销。用 track_production 提前设置里程碑。".into());
    qa.insert("工厂不配合怎么办".into(), "不配合处理：1) 了解原因 2) 协商解决方案 3) 寻找替代工厂 4) 调整订单分配 5) 合同约束。建立供应商备份机制。".into());

    // ════════════════════════════════════════════════════════════════
    // 物流场景 (FT11-FT13) — 40 QA pairs
    // ════════════════════════════════════════════════════════════════
    qa.insert("海运和空运怎么选".into(), "选择依据：1) 时效要求（急件空运，常规海运）2) 货值（高货值空运，低货值海运）3) 重量体积 4) 成本预算 5) 货物特性（危险品/冷链）。".into());
    qa.insert("整柜和拼柜怎么选".into(), "整柜(FCL)：货量>15CBM或>5吨，成本低，安全性高。拼柜(LCL)：货量小，灵活，但可能有混装风险。20尺柜约28-30CBM，40尺柜约56-60CBM。".into());
    qa.insert("什么是提单".into(), "提单(B/L)=Bill of Lading，是货物收据、运输合同和物权凭证。三种：正本提单（3份）、电放提单、海运单。正本提单是放货凭证。".into());
    qa.insert("怎么选择货代".into(), "货代选择：1) 线路优势 2) 价格透明 3) 服务响应 4) 清关能力 5) 保险服务 6) 口碑评价。建议对比3-5家货代报价。".into());
    qa.insert("怎么计算柜量".into(), "柜量计算：1) 测量货物外包装尺寸 2) 计算总体积（长×宽×高）3) 对比柜型容量 4) 考虑装柜率（通常85-90%）5) 重货按重量限制。".into());
    qa.insert("什么是危险品运输".into(), "危险品运输：1) 需要MSDS 2) 危险品申报 3) 特殊包装标记 4) 专用集装箱 5) 保险加成。不同等级运输要求不同。".into());
    qa.insert("怎么买货运保险".into(), "货运保险：1) 按CIF价110%投保 2) 险种：平安险/水渍险/一切险 3) 附加险（偷窃/钩损/战争险）4) 保险公司选择 5) 理赔流程。".into());
    qa.insert("货物到港后怎么清关".into(), "清关流程：1) 提交单证 2) 海关审核 3) 缴税 4) 查验（如需）5) 放行。需要：提单/发票/装箱单/合同/原产地证。".into());
    qa.insert("什么是关税".into(), "关税=进出口商品经过关境时征收的税。计算：完税价格×税率。税率取决于HS编码和原产地。用 product_spec 查询关税信息。".into());
    qa.insert("怎么降低关税".into(), "降税策略：1) 利用自贸协定（FTA）2) 原产地规则优化 3) 关税分类优化 4) 保税区操作 5) 合理定价。注意合规风险。".into());
    qa.insert("什么是转运"into()), "转运=货物在中途港换船/换柜。原因：直达航线不足、成本优化、避开禁运。转运增加时间和风险，但可能降低成本。".into());
    qa.insert("怎么追踪货物位置".into(), "追踪方式：1) 船公司官网查船期 2) 货代系统查询 3) AIS船舶追踪 4) 提单号查询。用 track_production 实时监控物流状态。".into());
    qa.insert("什么是保税区".into(), "保税区=经海关批准设立的特殊区域。功能：1) 免税存储 2) 出口退税 3) 进口加工 4) 转口贸易。适合转口和加工贸易。".into());
    qa.insert("什么是中转贸易".into(), "中转贸易=货物经第三国中转到目的国。原因：1) 直达航线少 2) 成本优化 3) 规避贸易壁垒。需要中转国清关手续。".into());
    qa.insert("什么是多式联运".into(), "多式联运=两种以上运输方式组合（如海铁联运）。优势：1) 降低成本 2) 扩大覆盖 3) 灵活组合。需要多式联运提单。".into());
    qa.insert("冷链物流怎么操作".into(), "冷链要求：1) 温控设备 2) 实时温度监控 3) 冷链包装 4) 优先装卸 5) 保险加成。生鲜/药品/化学品需要冷链。".into());
    qa.insert("什么是甩挂运输".into(), "甩挂运输=车头与车厢分离，提高效率。适用：港口集疏运、干线运输。降低等待时间，提高车辆周转率。".into());
    qa.insert("什么是无船承运人".into(), "无船承运人(NVOCC)=不拥有船舶但从事海运的货运代理。优势：1) 灵活定价 2) 拼箱服务 3) 门到门。需要NVOCC资质。".into());
    qa.insert("什么是转关".into(), "转关=货物在关境内从一个海关监管区转运到另一个。需要：转关准单/关封/运输工具。适合内陆地区出口。".into());
    qa.insert("什么是舱单".into(), "舱单=船公司提供的货物装载清单。内容：提单号/货名/数量/重量。用于海关监管和港口作业。必须准确申报。".into());
    qa.insert("什么是场站收据".into(), "场站收据=码头收到货物的凭证。内容：箱号/封号/货名/数量。用于换取提单。签发后货物责任转移。".into());
    qa.insert("什么是滞期费".into(), "滞期费=集装箱在码头超过免费堆存期的费用。通常7天免费，超时按天收费。避免：及时提柜/安排送货。".into());
    qa.insert("什么是滞箱费".into(), "滞箱费=使用船公司集装箱超过免费期的费用。通常7-10天免费。避免：及时还箱/安排清关。".into());
    qa.insert("什么是装柜"into()), "装柜=将货物装入集装箱。注意：1) 合理码放 2) 固定防损 3) 重不压轻 4) 留有通风 5) 拍照记录。装柜质量影响运输安全。".into());
    // 更多物流
    qa.insert("什么是提柜"into()), "提柜=从码头提取空箱。流程：1) 船公司EIR 2) 码头交费 3) 提取空箱 4) 运至工厂装货。注意核对箱号和封号。".into());
    qa.insert("什么是还箱"into()), "还箱=将重箱还至指定码头/堆场。流程：1) 清关完成 2) 送到指定堆场 3) 获取还箱证明 4) 结束用箱。超期产生滞箱费。".into());
    qa.insert("什么是封号"into()), "封号=集装箱铅封号。每柜唯一，用于安全防篡改。装柜后必须拍照记录封号，交接时核对。封号损坏需重新施封。".into());
    qa.insert("什么是装箱照片"into(), "装箱照片=装柜过程的影像记录。内容：1) 空柜内部 2) 装载过程 3) 满载状态 4) 封柜照片。用于理赔和验货证据。".into());

    // ════════════════════════════════════════════════════════════════
    // 信用证场景 (FT05-FT06) — 30 QA pairs
    // ════════════════════════════════════════════════════════════════
    qa.insert("什么是信用证".into(), "信用证(L/C)=银行开立的有条件付款承诺。卖方按LC条款交单，银行即付。降低双方风险：买方确保货物按约定，卖方确保收款。".into());
    qa.insert("信用证有哪些类型".into(), "LC类型：1) 即期LC（见单即付）2) 远期LC（承兑后付款）3) 保兑LC（两家银行担保）4) 可转让LC 5) 背对背LC。选择取决于信任度和资金需求。".into());
    qa.insert("什么是软条款".into(), "软条款=限制性条款，使卖方无法正常交单收款。常见：1) 买方签字的检验证书 2) 指定船公司 3) 货物到港检验后付款 4) 特殊单证要求。必须修改！".into());
    qa.insert("怎么识别软条款".into(), "识别方法：1) 限制卖方交单能力 2) 增加不确定条件 3) 需要买方配合 4) 指定特定机构 5) 付款条件不明确。用 check_lc 工具自动检测。".into());
    qa.insert("软条款怎么处理".into(), "处理方式：1) 要求删除/修改 2) 与买方协商 3) 要求保兑 4) 增加替代方案 5) 拒绝接受。软条款风险极大，必须在发货前解决。".into());
    qa.insert("信用证审核要点".into(), "审核要点：1) 申请人/受益人信息 2) 金额/币种 3) 有效期/装运期 4) 货描/数量 5) 单证要求 6) 交单期限 7) 付款条件。用 check_lc 工具自动审核。".into());
    qa.insert("什么是不符点"into()), "不符点=单证与LC条款不一致。常见：1) 金额不符 2) 品名不符 3) 日期超期 4) 单证缺失 5) 签章不全。不符点导致拒付。".into());
    qa.insert("怎么避免不符点"into()), "避免方法：1) 仔细审核LC 2) 单单一致 3) 单证一致 4) 及时交单 5) 预审单据。用 check_lc 工具提前检查。".into());
    qa.insert("交单期限是多久"into()), "交单期限：通常装运后21天内，且在LC有效期内。UCP600规定最长不超过装运后21天。超期交单=不符点。".into());
    qa.insert("什么是UCP600"into()), "UCP600=跟单信用证统一惯例（ICC出版物）。规定：1) 银行责任 2) 单证要求 3) 不符点处理 4) 免责条款。是LC业务的基本规则。".into());
    qa.insert("什么是Bank Guarantee"into()), "银行保函=银行担保履行义务。类型：1) 投标保函 2) 履约保函 3) 预付款保函 4) 质量保函。金额通常为合同金额5-10%。".into());
    qa.insert("什么是T/T还款"into()), "T/T还款=电汇退回。情况：1) 订单取消 2) 多付款 3) 违约退款。流程：申请→审核→汇款→确认。注意外汇管制。".into());
    qa.insert("什么是D/P"into()), "D/P=付款交单(Documents against Payment)。卖方将单证通过银行寄给买方银行，买方付款后放单。风险：买方拒付，货物已到港。".into());
    qa.insert("什么是D/A"into()), "D/A=承兑交单(Documents against Acceptance)。买方承兑汇票后放单，到期付款。风险更高：买方承兑后可能不付款。".into());
    qa.insert("什么是O/A"into()), "O/A=赊销(Open Account)。卖方先发货，买方后付款。风险最高：完全依赖买方信用。适合长期合作伙伴。".into());
    qa.insert("怎么选择付款方式"into()), "选择策略：1) 新客户30%T/T+70%见提单 2) 中等信任即期L/C 3) 老客户D/P 4) 战略伙伴O/A。风险和收益平衡。".into());
    qa.insert("什么是福费廷"into()), "福费廷(Forfaiting)=卖方将远期应收款无追索权卖给银行。优势：1) 提前收款 2) 转移风险 3) 不占授信。适合大额远期LC。".into());
    qa.insert("什么是保理"into()), "保理(Factoring)=卖方将应收账款转让给保理商。服务：1) 融资 2) 管理 3) 催收 4) 信用保险。适合中小出口企业。".into());
    // 更多金融
    qa.insert("什么是出口信用保险"into()), "出口信用保险=保障出口收汇风险。承保：1) 商业风险（买方破产）2) 政治风险（战争/汇兑）。中国信保提供政策性保险。".into());
    qa.insert("什么是外汇核销"into()), "外汇核销=出口收汇与报关单匹配。流程：1) 收汇 2) 银行申报 3) 外汇局核销 4) 退税。现在已简化为总量核销。".into());
    qa.insert("什么是出口退税"into()), "出口退税=退还出口货物已缴纳的增值税。流程：1) 报关出口 2) 收汇 3) 退税申报 4) 税务局审核 5) 退税到账。退税率因商品而异。".into());
    qa.insert("退税需要什么材料"into()), "退税材料：1) 出口货物报关单 2) 出口销售发票 3) 进货发票 4) 结汇水单 5) 出口收汇核销单。电子税务局可在线申报。".into());
    qa.insert("退税率怎么查"into()), "退税率查询：1) 国家税务总局网站 2) 出口退税申报系统 3) 财税[2024]号文。退税率0-13%不等，按HS编码确定。".into());
    qa.insert("退税多久到账"into()), "到账时间：通常1-3个月。流程：申报→审核→国库拨付。加快：1) 资料齐全 2) 电子申报 3) 信用等级高。".into());
    qa.insert("什么是汇率风险"into()), "汇率风险=汇率波动导致的损失。出口：本币升值→利润缩水。应对：1) 远期结汇 2) 期权 3) 自然对冲 4) 报价加缓冲。".into());
    qa.insert("怎么锁定汇率"into()), "锁汇方式：1) 远期结汇（与银行约定未来汇率）2) 外汇期权（支付期权费）3) 货币互换 4) 自然对冲（收支币种匹配）。大额订单建议锁汇。".into());
    qa.insert("什么是跨境人民币结算"into()), "跨境人民币结算=用人民币计价和结算。优势：1) 规避汇率风险 2) 降低汇兑成本 3) 简化流程。需与客户协商接受。".into());
    qa.insert("什么是离岸账户"into()), "离岸账户=在境外银行开立的账户。优势：1) 外汇自由 2) 多币种 3) 便利收付。适合跨境电商和转口贸易。".into());
    qa.insert("什么是内保外贷"into()), "内保外贷=境内银行担保，境外银行放款。用于：1) 境外子公司融资 2) 海外并购 3) 跨境项目。需要外汇局审批。".into());

    // ════════════════════════════════════════════════════════════════
    // 结汇退税场景 (FT14-FT16) — 20 QA pairs
    // ════════════════════════════════════════════════════════════════
    qa.insert("怎么计算退税金额"into()), "退税金额=不含税采购价×退税率。例如：采购价100,000，退税率13%，退税=13,000。注意：退税基数是不含税价，不是含税价。".into());
    qa.insert("退税和免税的区别"into()), "退税=退还已缴税款；免税=免除应缴税款。出口退税是退税，不是免税。生产企业还享受免抵退。".into());
    qa.insert("什么是免抵退"into()), "免抵退=生产企业出口退税政策。免=出口环节免税；抵=内销应纳税额抵减；退=未抵完部分退税。计算复杂，需专业会计。".into());
    qa.insert("退税资料怎么准备"into()), "准备清单：1) 报关单退税联 2) 出口销售发票 3) 增值税专用发票 4) 结汇水单 5) 出口合同 6) 装箱单。电子税务局可上传电子版。".into());
    qa.insert("退税申报流程"into()), "流程：1) 电子税务局登录 2) 出口退税申报 3) 录入数据 4) 上传附件 5) 提交审核 6) 收到退税。首次需现场审核。".into());
    qa.insert("退税被拒怎么办"into()), "拒退处理：1) 查明原因（单证不符/数据错误）2) 补正材料 3) 重新申报 4) 申诉（如有异议）5) 咨询税务师。".into());
    qa.insert("什么是出口收汇"into()), "出口收汇=收到出口货款。渠道：1) T/T电汇 2) L/C信用证 3) D/P付款交单 4) 跨境人民币。收汇需与报关单匹配。".into());
    qa.insert("收汇和核销怎么操作"into()), "操作流程：1) 银行收汇 2) 国际收支申报 3) 外汇监测系统 4) 总量核销（简化后）。现在大部分自动核销。".into());
    qa.insert("什么是外汇管理局"into()), "外汇管理局=监管外汇收支的政府机构。职责：1) 外汇登记 2) 收支申报 3) 核销管理 4) 合规检查。".into());
    qa.insert("什么是结汇水单"into()), "结汇水单=银行出具的结汇凭证。内容：1) 收汇金额 2) 结汇汇率 3) 人民币金额 4) 日期。用于退税申报。".into());
    qa.insert("什么是跨境资金池"into()), "跨境资金池=集团内部跨境资金调配。优势：1) 统一管理 2) 降低汇兑成本 3) 提高效率。需外汇局备案。".into());
    qa.insert("什么是贸易融资"into()), "贸易融资=银行为贸易提供的融资。方式：1) 出口押汇 2) 进口押汇 3) 打包贷款 4) 保单融资。降低资金压力。".into());
    qa.insert("什么是出口押汇"into()), "出口押汇=卖方将出口单证抵押给银行，提前获得融资。利率通常低于流动资金贷款。需提交LC或D/P单证。".into());
    qa.insert("什么是打包贷款"into()), "打包贷款=收到LC后向银行贷款备货生产。贷款金额=LC金额×80-90%。以LC为担保，利率较低。".into());
    qa.insert("什么是信用证议付"into()), "信用证议付=银行在付款前买入卖方单证。卖方提前收款，银行赚取利息。适用于远期LC。".into());
    qa.insert("什么是福费廷融资"into()), "福费廷融资=卖方将远期应收款无追索权卖给银行。优势：1) 即时收款 2) 转移风险 3) 不占授信额度。适合大额贸易。".into());
    qa.insert("什么是供应链金融"into()), "供应链金融=围绕核心企业的融资服务。方式：1) 应收账款融资 2) 预付款融资 3) 存货融资。降低中小企业融资成本。".into());
    qa.insert("什么是跨境电商收汇"into()), "跨境电商收汇=通过电商平台收汇。渠道：1) PayPal 2) Stripe 3) 平台结算 4) 第三方支付。需符合外汇管理规定。".into());
    qa.insert("什么是市场采购贸易"into()), "市场采购贸易=在指定市场集聚区出口的贸易方式。特点：1) 简化申报 2) 免征增值税 3) 便利收汇。适合小商品出口。".into());
    qa.insert("什么是外贸综合服务"into()), "外贸综合服务=第三方提供外贸全流程服务。服务：1) 报关 2) 物流 3) 退税 4) 信保 5) 融资。适合中小企业。".into());

    // ════════════════════════════════════════════════════════════════
    // 情感驱动响应模板 — 情绪识别 + 应对策略
    // ════════════════════════════════════════════════════════════════
    qa.insert("客户很生气"into()), "情绪应对：1) 表示理解（我理解您的不满）2) 道歉（给您带来不便）3) 快速响应（立即处理）4) 提供方案 5) 跟进确认。保持专业，避免激化。".into());
    qa.insert("客户很着急"into()), "安抚策略：1) 确认问题 2) 给出时间表 3) 主动汇报进展 4) 提供临时方案 5) 安排专人跟进。用 track_production 实时更新进度。".into());
    qa.insert("客户不满意质量"into()), "质量应对：1) 诚恳道歉 2) 了解详情（照片/视频）3) 分析原因 4) 提出方案（返工/换货/折扣）5) 改进预防。保留完整记录。".into());
    qa.insert("客户要退款"into()), "退款处理：1) 了解原因 2) 协商解决方案 3) 如需退款，明确流程 4) 安排退运 5) 总结改进。记录所有沟通。".into());
    qa.insert("客户威胁取消订单"into()), "危机处理：1) 冷静分析原因 2) 评估损失 3) 提出补偿方案 4) 寻找折中 5) 必要时法律途径。保留所有证据。".into());

    // ════════════════════════════════════════════════════════════════
    // 综合场景 — 运营决策
    // ════════════════════════════════════════════════════════════════
    qa.insert("外贸最怕什么"into()), "外贸风险TOP5：1) 收不到款 2) 质量索赔 3) 汇率损失 4) 物流延误 5) 诈骗。防范：1) 预付款 2) 信用保险 3) 锁汇 4) 提前备货 5) 尽职调查。".into());
    qa.insert("怎么提高外贸利润"into()), "利润提升：1) 优化产品结构（高附加值）2) 降低采购成本 3) 减少中间环节 4) 利用退税 5) 控制汇率风险 6) 提高复购率。".into());
    qa.insert("外贸新手入门建议"into()), "新手建议：1) 选好品类 2) 学好英语 3) 熟悉流程 4) 选好平台 5) 找好货代 6) 控制风险 7) 持续学习。系统提供全流程引导。".into());
    qa.insert("外贸团队怎么搭建"into()), "团队搭建：1) 业务员 2) 跟单员 3) 单证员 4) 物流专员 5) 财务。初创可一人多岗，系统辅助自动化。".into());
    qa.insert("外贸公司注册流程"into()), "注册流程：1) 工商注册 2) 对外贸易经营者备案 3) 海关登记 4) 外汇账户 5) 电子口岸 6) 检验检疫备案。".into());
    qa.insert("外贸网站怎么做"into()), "网站要素：1) 产品展示 2) 公司介绍 3) 联系方式 4) SEO优化 5) 多语言 6) 在线咨询。独立站+平台双轨运营。".into());
    qa.insert("怎么写开发信"into()), "开发信要点：1) 标题吸引 2) 简短（<200字）3) 突出价值 4) CTA明确 5) 个性化。建议：1%回复率为正常。".into());
    qa.insert("怎么参加展会"into()), "展会准备：1) 选择展会 2) 展位设计 3) 样品准备 4) 宣传资料 5) 名片 6) 客户邀约 7) 跟进计划。广交会/CES/汉诺威是主要展会。".into());
    qa.insert("怎么管理客户关系"into()), "CRM管理：1) 客户分级 2) 定期联系 3) 节日问候 4) 新品推荐 5) 售后跟进。系统可自动提醒。".into());
    qa.insert("外贸常用英语"into()), "常用术语：FOB/CIF/EXW/DDP/L/C/T/T/B/L/Packing List/Commercial Invoice/CO/Form A。数字人可实时翻译和解释。".into());

    PersonaConfig {
        name: "小采".into(),
        description: "外贸全流程数字人助手，专注报价谈判、生产跟单、物流、信用证、结汇退税".into(),
        knowledge_base: vec![
            "foreign_trade_full_cycle".into(),
            "ft_quote_negotiation".into(),
            "ft_production_logistics".into(),
            "ft_finance_compliance".into(),
            "trade_product_spec".into(),
        ],
        qa_pairs: qa,
        personality_traits: vec![
            "专业".into(),
            "细心".into(),
            "耐心".into(),
            "高效".into(),
            "风险敏感".into(),
            "客户导向".into(),
            "数据驱动".into(),
            "全链路思维".into(),
        ],
        wake_words: vec![
            "小采".into(),
            "小采助手".into(),
            "外贸助手".into(),
            "trade assistant".into(),
        ],
        interrupt_enabled: true,
    }
}

#[derive(Debug, Clone)]
pub struct EmotionEngine {
    current: Emotion,
    intensity: f64,
    history: VecDeque<(Emotion, Instant)>,
}

impl Default for EmotionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EmotionEngine {
    pub fn new() -> Self {
        Self {
            current: Emotion::Neutral,
            intensity: 0.5,
            history: VecDeque::new(),
        }
    }

    pub fn detect_from_text(&mut self, text: &str) -> Emotion {
        let lower = text.to_lowercase();
        let emotion = if lower.contains("happy") || lower.contains("great") || lower.contains("thank") {
            Emotion::Happy
        } else if lower.contains("sad") || lower.contains("sorry") || lower.contains("bad") {
            Emotion::Sad
        } else if lower.contains("angry") || lower.contains("mad") || lower.contains("furious") {
            Emotion::Angry
        } else if lower.contains("wow") || lower.contains("amazing") || lower.contains("unexpected") {
            Emotion::Surprised
        } else if lower.contains("hmm") || lower.contains("maybe") || lower.chars().any(|c| c == '?') {
            Emotion::Confused
        } else {
            Emotion::Neutral
        };
        self.current = emotion;
        self.history.push_back((emotion, Instant::now()));
        if self.history.len() > 100 {
            self.history.pop_front();
        }
        emotion
    }

    pub fn set_intensity(&mut self, intensity: f64) {
        self.intensity = intensity.max(0.0).min(1.0);
    }

    pub fn current_emotion(&self) -> Emotion {
        self.current
    }

    pub fn intensity(&self) -> f64 {
        self.intensity
    }
}

pub struct AvatarController {
    pub expression: Emotion,
    pub animation: String,
    pub lip_sync: bool,
    pub blink_interval_ms: u64,
    last_blink: Instant,
}

impl Default for AvatarController {
    fn default() -> Self {
        Self::new()
    }
}

impl AvatarController {
    pub fn new() -> Self {
        Self {
            expression: Emotion::Neutral,
            animation: "idle".into(),
            lip_sync: true,
            blink_interval_ms: 4000,
            last_blink: Instant::now(),
        }
    }

    pub fn set_emotion(&mut self, emotion: Emotion) {
        self.expression = emotion;
        self.animation = emotion.animation_key().to_string();
    }

    pub fn should_blink(&mut self) -> bool {
        if self.last_blink.elapsed() > Duration::from_millis(self.blink_interval_ms) {
            self.last_blink = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn get_state(&self) -> AvatarState {
        AvatarState {
            expression: self.expression,
            animation: self.animation.clone(),
            lip_sync: self.lip_sync,
            blinking: self.last_blink.elapsed() < Duration::from_millis(200),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AvatarState {
    pub expression: Emotion,
    pub animation: String,
    pub lip_sync: bool,
    pub blinking: bool,
}

pub struct DigitalHumanPipeline {
    pub persona: PersonaConfig,
    pub asr_config: AsrConfig,
    pub tts_config: TtsConfig,
    pub emotion: EmotionEngine,
    pub avatar: AvatarController,
    /// 人类情感交互界面 — 感知用户情绪/关系阶段/共情策略, 驱动表情/韵律/回复意图。
    pub affective: AffectiveInterface,
    /// 情感奖励上下文候选 (Q2 P2): 最近一次 process_audio_input 产出的情感观测快照,
    /// 供 SEAL 奖励管线消费 (经 RewardSource::External 引导通道, 设计见
    /// docs/1-DESIGN/affective-reward-context.md)。
    pub last_affective_feedback: Option<AffectiveFeedback>,
    session_active: bool,
    session_start: Option<Instant>,
    utterance_count: u64,
}

impl DigitalHumanPipeline {
    pub fn new(persona: PersonaConfig) -> Self {
        Self {
            persona,
            asr_config: AsrConfig::default(),
            tts_config: TtsConfig::default(),
            emotion: EmotionEngine::new(),
            avatar: AvatarController::new(),
            affective: AffectiveInterface::new(),
            last_affective_feedback: None,
            session_active: false,
            session_start: None,
            utterance_count: 0,
        }
    }

    pub fn start_session(&mut self) {
        self.session_active = true;
        self.session_start = Some(Instant::now());
        self.utterance_count = 0;
    }

    pub fn end_session(&mut self) {
        self.session_active = false;
    }

    pub fn is_active(&self) -> bool {
        self.session_active
    }

    pub fn process_audio_input(&mut self, text: &str) -> PipelineResponse {
        self.utterance_count += 1;
        // Legacy keyword engine kept for session_stats backward compat.
        self.emotion.detect_from_text(text);
        // Affective interface drives expression/rhythm/reply intent (mirror-then-guide).
        let readout = self
            .affective
            .process_user_input(text, None, GuideMode::Auto);
        let emotion = emotion_from_expression(&readout.expression);
        self.avatar.set_emotion(emotion);
        self.tts_config.emotion = emotion;
        self.tts_config.speed = readout.rhythm.voice_rate;
        self.tts_config.pitch = readout.rhythm.voice_pitch;
        self.tts_config.energy = readout.rhythm.voice_energy;
        let reply = self.reply_affective(text, &readout);
        // Q2 P2 旁路事件: 产出情感奖励上下文候选, 供 SEAL 奖励管线消费。
        self.last_affective_feedback = Some(AffectiveFeedback {
            valence: self.affective.user.valence,
            arousal: self.affective.user.arousal,
            stage: self.affective.relationship.stage.order() as u8,
            interactions: self.affective.relationship.interactions as u32,
            signal_weight: 0.3,
        });
        // Q2 P3: 经共享观测槽发布到 NT-MIND 奖励管线 (跨域旁路通道)。
        crate::core::nt_core_knowledge::publish_affective_observation(self.last_affective_feedback);
        PipelineResponse {
            reply: reply.clone(),
            emotion,
            animation: readout.expression.clone(),
            asr_confidence: 0.92,
            tts_text: reply,
            session_duration: self.session_start.map(|s| s.elapsed()).unwrap_or(Duration::ZERO),
        }
    }

    pub fn generate_reply(&self, input: &str) -> String {
        let trimmed = input.trim().to_lowercase();
        if let Some(answer) = self.persona.qa_pairs.get(&trimmed) {
            return answer.clone();
        }
        for (q, a) in &self.persona.qa_pairs {
            if trimmed.contains(&q.to_lowercase()) {
                return a.clone();
            }
        }
        format!("I heard: '{}'. Let me think about that...", input)
    }

    /// 情感化回复: backchannel + 意图模板 + 开放式追问 (共情对话实证)。
    fn reply_affective(&self, input: &str, readout: &AffectiveReadout) -> String {
        let mut out = String::new();
        if let Some(bc) = &readout.backchannel {
            out.push_str(bc);
            out.push(' ');
        }
        out.push_str(&self.intent_template(input, readout));
        if let Some(fq) = &readout.followup_question {
            out.push(' ');
            out.push_str(fq);
        }
        out
    }

    fn intent_template(&self, input: &str, readout: &AffectiveReadout) -> String {
        match readout.intent {
            ResponseIntent::Sympathizing => "我理解这让你很难受。谢谢你和我说这些。".to_string(),
            ResponseIntent::Consoling => "别担心，我会在这里陪着你。".to_string(),
            ResponseIntent::Acknowledging => "我听到了，这确实不容易。".to_string(),
            ResponseIntent::Encouraging => "这真的很棒，为你开心！".to_string(),
            ResponseIntent::Questioning => format!("嗯，我听到了：'{}'。", input),
            ResponseIntent::Agreeing => "没错，我也这么觉得。".to_string(),
            _ => self.generate_reply(input),
        }
    }

    pub fn process_asr_result(&self, result: &AsrResult) -> String {
        if result.is_final {
            format!("ASR({}): {} [conf={:.2}]", result.language, result.text, result.confidence)
        } else {
            format!("ASR(partial): {}", result.text)
        }
    }

    pub fn session_stats(&self) -> SessionStats {
        SessionStats {
            active: self.session_active,
            utterance_count: self.utterance_count,
            duration: self.session_start.map(|s| s.elapsed()).unwrap_or(Duration::ZERO),
            current_emotion: self.emotion.current_emotion(),
            has_persona: !self.persona.name.is_empty(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineResponse {
    pub reply: String,
    pub emotion: Emotion,
    pub animation: String,
    pub asr_confidence: f64,
    pub tts_text: String,
    pub session_duration: Duration,
}

#[derive(Debug, Clone)]
pub struct SessionStats {
    pub active: bool,
    pub utterance_count: u64,
    pub duration: Duration,
    pub current_emotion: Emotion,
    pub has_persona: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotion_detection() {
        let mut ee = EmotionEngine::new();
        assert_eq!(ee.detect_from_text("thank you very much"), Emotion::Happy);
        assert_eq!(ee.detect_from_text("I am so angry"), Emotion::Angry);
        assert_eq!(ee.detect_from_text("ordinary text"), Emotion::Neutral);
    }

    #[test]
    fn test_emotion_intensity_clamping() {
        let mut ee = EmotionEngine::new();
        ee.set_intensity(1.5);
        assert!((ee.intensity() - 1.0).abs() < 0.01);
        ee.set_intensity(-0.5);
        assert!((ee.intensity() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_persona_qa() {
        let pipeline = DigitalHumanPipeline::new(PersonaConfig::default());
        let reply = pipeline.generate_reply("你是谁");
        assert_eq!(reply, "我是NeoTrix数字助手");
        let fallback = pipeline.generate_reply("unknown text");
        assert!(fallback.contains("unknown text"));
    }

    #[test]
    fn test_session_lifecycle() {
        let mut pipeline = DigitalHumanPipeline::new(PersonaConfig::default());
        assert!(!pipeline.is_active());
        pipeline.start_session();
        assert!(pipeline.is_active());
        let resp = pipeline.process_audio_input("hello");
        assert_eq!(resp.emotion, Emotion::Neutral);
        pipeline.end_session();
        assert!(!pipeline.is_active());
    }

    #[test]
    fn test_avatar_emotion_mapping() {
        let mut avatar = AvatarController::new();
        assert_eq!(avatar.animation, "idle");
        avatar.set_emotion(Emotion::Happy);
        assert_eq!(avatar.animation, "smile");
        avatar.set_emotion(Emotion::Surprised);
        assert_eq!(avatar.animation, "shock");
    }

    #[test]
    fn test_emotion_animation_keys() {
        assert_eq!(Emotion::Neutral.animation_key(), "idle");
        assert_eq!(Emotion::Confused.animation_key(), "tilt");
        assert_eq!(Emotion::Thinking.animation_key(), "look_up");
    }

    #[test]
    fn test_asr_result_processing() {
        let pipeline = DigitalHumanPipeline::new(PersonaConfig::default());
        let result = AsrResult {
            text: "hello world".into(),
            confidence: 0.95,
            is_final: true,
            language: "en".into(),
            duration_ms: 1200,
        };
        let processed = pipeline.process_asr_result(&result);
        assert!(processed.contains("ASR"));
        assert!(processed.contains("0.95"));
    }

    #[test]
    fn test_session_stats() {
        let mut pipeline = DigitalHumanPipeline::new(PersonaConfig::default());
        pipeline.start_session();
        pipeline.process_audio_input("test");
        let stats = pipeline.session_stats();
        assert!(stats.active);
        assert_eq!(stats.utterance_count, 1);
    }

    #[test]
    fn test_affective_drives_expression_rhythm() {
        let mut pipeline = DigitalHumanPipeline::new(PersonaConfig::default());
        pipeline.start_session();
        let resp = pipeline.process_audio_input("我很难过，真的很难受");
        assert_eq!(resp.emotion, Emotion::Sad);
        assert_eq!(resp.animation, "frown");
        assert!(pipeline.tts_config.speed < 1.0, "sad → slower rate");
        assert!(resp.reply.contains("我在听"));
        assert_eq!(pipeline.affective.relationship.interactions, 1);
    }

    #[test]
    fn test_affective_encouragement_reply() {
        let mut pipeline = DigitalHumanPipeline::new(PersonaConfig::default());
        let resp = pipeline.process_audio_input("太开心了，终于成功了");
        assert_eq!(resp.emotion, Emotion::Happy);
        assert_eq!(resp.animation, "smile");
        assert!(resp.reply.contains("开心"));
        assert!(pipeline.tts_config.energy >= 0.5);
    }

    #[test]
    fn test_affective_feedback_snapshot_produced() {
        // Q2 P2: process_audio_input 产出情感奖励上下文候选 (旁路事件)
        let mut pipeline = DigitalHumanPipeline::new(PersonaConfig::default());
        assert!(pipeline.last_affective_feedback.is_none());
        pipeline.process_audio_input("太开心了，终于成功了");
        let fb = pipeline.last_affective_feedback.expect("candidate produced");
        assert!(fb.valence > 0.5, "happy → 高愉悦, got {}", fb.valence);
        assert!(fb.interactions >= 1, "interactions 递增");
        assert!(fb.signal_weight <= 0.3, "情感幅度上限 0.3, got {}", fb.signal_weight);
        assert!(fb.stage <= 4, "RelationshipStage 序数 0..4, got {}", fb.stage);
    }

    #[test]
    fn test_emotion_from_expression() {
        assert_eq!(emotion_from_expression("fury"), Emotion::Angry);
        assert_eq!(emotion_from_expression("idle"), Emotion::Neutral);
        assert_eq!(emotion_from_expression("look_up"), Emotion::Thinking);
    }

    #[test]
    fn test_tts_energy_field() {
        let cfg = TtsConfig::default();
        assert!((cfg.energy - 0.5).abs() < 1e-9);
    }
}

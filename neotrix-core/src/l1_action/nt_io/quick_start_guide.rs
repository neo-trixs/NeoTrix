//! 快速入门向导
//!
//! 提供动态漫技能的快速入门指引
//! 帮助用户快速理解和使用 NeoTrix 动态漫能力

use serde::{Serialize, Deserialize};

// ============================================================================
// 快速入门步骤
// ============================================================================

/// 快速入门步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickStartStep {
    /// 步骤编号
    pub step_number: u32,
    /// 步骤标题
    pub title: String,
    /// 步骤描述
    pub description: String,
    /// 关键操作
    pub key_actions: Vec<String>,
    /// 预期结果
    pub expected_result: String,
    /// 常见问题
    pub faq: Vec<String>,
}

/// 快速入门指南
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickStartGuide {
    /// 指南标题
    pub title: String,
    /// 指南描述
    pub description: String,
    /// 所有步骤
    pub steps: Vec<QuickStartStep>,
    /// 前置条件
    pub prerequisites: Vec<String>,
    /// 预计时间
    pub estimated_time: String,
    /// 难度等级
    pub difficulty: String,
}

impl QuickStartGuide {
    /// 创建动态漫技能快速入门指南
    pub fn create_dongtai_man_guide() -> Self {
        Self {
            title: "动态漫技能快速入门".to_string(),
            description: "从零开始创建你的第一个动态漫作品".to_string(),
            prerequisites: vec![
                "了解动态漫基本概念".to_string(),
                "准备好角色设定和剧情梗概".to_string(),
            ],
            estimated_time: "30分钟".to_string(),
            difficulty: "初级".to_string(),
            steps: vec![
                QuickStartStep {
                    step_number: 1,
                    title: "角色设定".to_string(),
                    description: "使用 DynamicParams 模块定义角色的动态参数".to_string(),
                    key_actions: vec![
                        "定义角色速度 (speed)".to_string(),
                        "定义角色幅度 (amplitude)".to_string(),
                        "定义角色频率 (frequency)".to_string(),
                    ],
                    expected_result: "角色动态参数文件".to_string(),
                    faq: vec![
                        "速度单位是什么？→ 秒/动作 (s/action)".to_string(),
                        "幅度单位是什么？→ 度 (°) 或像素 (px)".to_string(),
                    ],
                },
                QuickStartStep {
                    step_number: 2,
                    title: "剧情设计".to_string(),
                    description: "使用 RhythmRecalculator 模块设计剧情节奏".to_string(),
                    key_actions: vec![
                        "确定总时长".to_string(),
                        "划分铺垫/冲突/爽点段".to_string(),
                        "设置情绪节拍".to_string(),
                    ],
                    expected_result: "剧情分段文件".to_string(),
                    faq: vec![
                        "总时长应该多少？→ 建议 3-5分钟".to_string(),
                        "爽点应该放在哪里？→ 中后部".to_string(),
                    ],
                },
                QuickStartStep {
                    step_number: 3,
                    title: "角色互动".to_string(),
                    description: "使用 CharacterInteractionGraph 定义角色关系".to_string(),
                    key_actions: vec![
                        "添加角色".to_string(),
                        "定义互动关系".to_string(),
                        "设置成长弧线".to_string(),
                    ],
                    expected_result: "角色关系图".to_string(),
                    faq: vec![
                        "互动强度怎么设置？→ 0.0-1.0，0.8以上为强互动".to_string(),
                        "如何检查一致性？→ 使用 CrossModuleAudit".to_string(),
                    ],
                },
                QuickStartStep {
                    step_number: 4,
                    title: "节奏检查".to_string(),
                    description: "使用 BlankSpaceChecker 检查节奏合理性".to_string(),
                    key_actions: vec![
                        "运行留白检查".to_string(),
                        "查看检查结果".to_string(),
                        "根据建议调整".to_string(),
                    ],
                    expected_result: "节奏检查报告".to_string(),
                    faq: vec![
                        "留白应该多长？→ 3-5秒".to_string(),
                        "检查不通过怎么办？→ 增加留白或调整分段".to_string(),
                    ],
                },
                QuickStartStep {
                    step_number: 5,
                    title: "一致性验证".to_string(),
                    description: "使用 CrossModuleAudit 验证跨模块一致性".to_string(),
                    key_actions: vec![
                        "运行一致性检查".to_string(),
                        "查看检查报告".to_string(),
                        "修复不一致项".to_string(),
                    ],
                    expected_result: "一致性检查报告".to_string(),
                    faq: vec![
                        "哪些需要检查？→ 动态等级、情绪拐点、节奏节拍".to_string(),
                        "检查不通过怎么办？→ 根据建议调整参数".to_string(),
                    ],
                },
            ],
        }
    }
    
    /// 创建平台适配快速入门指南
    pub fn create_platform_adapter_guide() -> Self {
        Self {
            title: "平台适配快速入门".to_string(),
            description: "如何使用 PlatformAdapter trait 对接外部平台".to_string(),
            prerequisites: vec![
                "了解 PlatformAdapter trait".to_string(),
                "了解目标平台 API".to_string(),
            ],
            estimated_time: "1小时".to_string(),
            difficulty: "中级".to_string(),
            steps: vec![
                QuickStartStep {
                    step_number: 1,
                    title: "实现 trait".to_string(),
                    description: "实现 PlatformAdapter trait 的所有方法".to_string(),
                    key_actions: vec![
                        "实现 convert_to_platform".to_string(),
                        "实现 convert_from_platform".to_string(),
                        "实现 capabilities".to_string(),
                    ],
                    expected_result: "适配器实现".to_string(),
                    faq: vec![
                        "需要实现哪些方法？→ 4个核心方法".to_string(),
                        "如何处理错误？→ 返回 Option 或 Result".to_string(),
                    ],
                },
                QuickStartStep {
                    step_number: 2,
                    title: "注册适配器".to_string(),
                    description: "使用 PlatformRegistry 注册你的适配器".to_string(),
                    key_actions: vec![
                        "创建 PlatformRegistry 实例".to_string(),
                        "调用 register 方法".to_string(),
                        "验证注册成功".to_string(),
                    ],
                    expected_result: "已注册的适配器".to_string(),
                    faq: vec![
                        "如何验证注册成功？→ 调用 list_platforms".to_string(),
                        "可以注册多个适配器吗？→ 可以".to_string(),
                    ],
                },
            ],
        }
    }
    
    /// 根据主题查找快速入门指南
    pub fn find_guide_by_topic(topic: &str) -> Option<QuickStartGuide> {
        match topic {
            "动态漫" | "dongtai" | "character" => Some(Self::create_dongtai_man_guide()),
            "平台适配" | "platform" | "adapter" => Some(Self::create_platform_adapter_guide()),
            _ => None,
        }
    }
    
    /// 获取所有可用指南
    pub fn list_all_guides() -> Vec<String> {
        vec![
            "动态漫技能快速入门".to_string(),
            "平台适配快速入门".to_string(),
        ]
    }
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_dongtai_man_guide() {
        let guide = QuickStartGuide::create_dongtai_man_guide();
        assert_eq!(guide.steps.len(), 5);
        assert_eq!(guide.difficulty, "初级");
    }
    
    #[test]
    fn test_create_platform_adapter_guide() {
        let guide = QuickStartGuide::create_platform_adapter_guide();
        assert_eq!(guide.steps.len(), 2);
        assert_eq!(guide.difficulty, "中级");
    }
    
    #[test]
    fn test_find_guide_by_topic() {
        assert!(QuickStartGuide::find_guide_by_topic("动态漫").is_some());
        assert!(QuickStartGuide::find_guide_by_topic("平台适配").is_some());
        assert!(QuickStartGuide::find_guide_by_topic("不存在的主题").is_none());
    }
    
    #[test]
    fn test_list_all_guides() {
        let guides = QuickStartGuide::list_all_guides();
        assert_eq!(guides.len(), 2);
    }
}
#![forbid(unsafe_code)]

//! 实体映射（Entity Mapping）
//!
//! 跨框架兼容的关键机制：
//! 1. 识别不同框架中的对应实体
//! 2. 建立实体之间的映射关系
//! 3. 处理实体属性的转换
//! 4. 维护映射的一致性
//!
//! 基于 Agent KB 论文中的"跨域实体对齐"机制

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 实体标识符
pub type EntityId = String;

/// 框架标识符
pub type FrameworkId = String;

/// 实体类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntityType {
    /// 概念
    Concept,
    /// 类
    Class,
    /// 函数
    Function,
    /// 方法
    Method,
    /// 接口
    Interface,
    /// 模块
    Module,
    /// 模式
    Pattern,
    /// 算法
    Algorithm,
    /// 数据结构
    DataStructure,
}

/// 实体属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityAttribute {
    /// 属性名
    pub name: String,
    /// 属性值
    pub value: String,
    /// 属性类型
    pub attribute_type: AttributeType,
    /// 是否可转换
    pub convertible: bool,
}

/// 属性类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttributeType {
    /// 基本类型
    Basic,
    /// 复合类型
    Complex,
    /// 关系类型
    Relational,
    /// 语义类型
    Semantic,
}

/// 实体定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDefinition {
    /// 实体ID
    pub id: EntityId,
    /// 实体名称
    pub name: String,
    /// 实体类型
    pub entity_type: EntityType,
    /// 所属框架
    pub framework: FrameworkId,
    /// 实体属性
    pub attributes: Vec<EntityAttribute>,
    /// 实体描述
    pub description: String,
    /// 语义嵌入 (可选)
    pub embedding: Option<Vec<f32>>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 实体映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMapping {
    /// 源实体ID
    pub source_entity_id: EntityId,
    /// 目标实体ID
    pub target_entity_id: EntityId,
    /// 映射类型
    pub mapping_type: MappingType,
    /// 映射置信度
    pub confidence: f64,
    /// 属性映射
    pub attribute_mappings: Vec<AttributeMapping>,
    /// 变换规则
    pub transformation_rules: Vec<TransformationRule>,
}

/// 映射类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MappingType {
    /// 直接映射
    Direct,
    /// 近似映射
    Approximate,
    /// 功能映射
    Functional,
    /// 结构映射
    Structural,
    /// 语义映射
    Semantic,
}

/// 属性映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeMapping {
    /// 源属性名
    pub source_attribute: String,
    /// 目标属性名
    pub target_attribute: String,
    /// 映射函数
    pub mapping_function: String,
    /// 是否可逆
    pub reversible: bool,
}

/// 变换规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationRule {
    /// 规则名称
    pub name: String,
    /// 规则类型
    pub rule_type: TransformationType,
    /// 规则表达式
    pub expression: String,
    /// 规则参数
    pub parameters: HashMap<String, String>,
}

/// 变换类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransformationType {
    /// 类型转换
    TypeConversion,
    /// 结构重构
    StructuralRefactoring,
    /// 语义适配
    SemanticAdaptation,
    /// 命名约定转换
    NamingConventionConversion,
    /// 参数重排序
    ParameterReordering,
}

/// 映射结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingResult {
    /// 成功映射的数量
    pub successful_mappings: usize,
    /// 失败映射的数量
    pub failed_mappings: usize,
    /// 映射详情
    pub mappings: Vec<EntityMapping>,
    /// 未映射的实体
    pub unmapped_entities: Vec<EntityId>,
    /// 映射统计
    pub stats: MappingStats,
}

/// 映射统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingStats {
    /// 平均映射置信度
    pub average_confidence: f64,
    /// 映射覆盖率
    pub coverage_rate: f64,
    /// 属性映射成功率
    pub attribute_mapping_success_rate: f64,
    /// 变换规则应用次数
    pub transformation_count: usize,
}

/// 实体映射配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMappingConfig {
    /// 最小映射置信度阈值
    pub min_confidence_threshold: f64,
    /// 是否启用语义映射
    pub enable_semantic_mapping: bool,
    /// 是否启用结构映射
    pub enable_structural_mapping: bool,
    /// 最大属性映射数量
    pub max_attribute_mappings: usize,
    /// 命名约定转换规则
    pub naming_conventions: Vec<NamingConvention>,
}

impl Default for EntityMappingConfig {
    fn default() -> Self {
        Self {
            min_confidence_threshold: 0.6,
            enable_semantic_mapping: true,
            enable_structural_mapping: true,
            max_attribute_mappings: 10,
            naming_conventions: Vec::new(),
        }
    }
}

/// 命名约定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingConvention {
    /// 约定名称
    pub name: String,
    /// 源格式
    pub source_format: String,
    /// 目标格式
    pub target_format: String,
    /// 转换函数
    pub conversion_function: String,
}

/// 实体映射器
pub struct EntityMapper {
    /// 配置
    config: EntityMappingConfig,
    /// 实体定义库
    entity_definitions: HashMap<FrameworkId, Vec<EntityDefinition>>,
    /// 映射规则库
    mapping_rules: Vec<MappingRule>,
    /// 映射历史
    mapping_history: Vec<EntityMapping>,
}

/// 映射规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingRule {
    /// 规则名称
    pub name: String,
    /// 源框架
    pub source_framework: FrameworkId,
    /// 目标框架
    pub target_framework: FrameworkId,
    /// 源实体类型
    pub source_entity_type: EntityType,
    /// 目标实体类型
    pub target_entity_type: EntityType,
    /// 匹配条件
    pub match_conditions: Vec<MatchCondition>,
    /// 映射函数
    pub mapping_function: String,
}

/// 匹配条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchCondition {
    /// 条件字段
    pub field: String,
    /// 操作符
    pub operator: String,
    /// 条件值
    pub value: String,
}

/// 跨域同义概念表：不同语言/框架里**语义等价**的类型名。
///
/// 元素为 `(概念 id, 各语言别名)`，别名已归一化（小写、仅保留字母数字）。
/// 跨域实体对齐的核心场景就是「Rust `HashMap` ↔ Python `dict` ↔ JS `Map`」——
/// 字符集 Jaccard 对这类名称恒为 0（`hashmap` 与 `dict` 无公共字符）。
const SEMANTIC_CONCEPTS: &[(&str, &[&str])] = &[
    ("map", &["map", "hashmap", "dict", "dictionary", "mapping", "hashtable", "treemap"]),
    ("list", &["list", "array", "vec", "vector", "slice", "seq", "sequence"]),
    ("string", &["string", "str", "text", "char", "chars"]),
    ("bool", &["bool", "boolean", "flag", "toggle"]),
    ("int", &["int", "integer", "i32", "i64", "u32", "u64", "usize", "isize", "long", "short"]),
    ("float", &["float", "f32", "f64", "double", "decimal", "real"]),
    ("set", &["set", "hashset", "pool"]),
    ("option", &["option", "optional", "maybe", "nullable"]),
    ("result", &["result", "outcome", "either"]),
    ("tuple", &["tuple", "pair"]),
    ("struct", &["struct", "record", "dataclass", "entity", "model", "pojo"]),
    ("enum", &["enum", "enumeration", "union", "adt"]),
    ("error", &["error", "exception", "failure", "err"]),
    ("future", &["future", "promise", "task", "awaitable"]),
];

/// 把类型名归一化后映射到跨域概念 id；无对应概念时返回 `None`。
fn semantic_concept(name: &str) -> Option<&'static str> {
    let norm: String = name
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    SEMANTIC_CONCEPTS
        .iter()
        .find(|(_, aliases)| aliases.contains(&norm.as_str()))
        .map(|(concept, _)| *concept)
}

impl EntityMapper {
    /// 创建新的实体映射器
    pub fn new(config: EntityMappingConfig) -> Self {
        Self {
            config,
            entity_definitions: HashMap::new(),
            mapping_rules: Vec::new(),
            mapping_history: Vec::new(),
        }
    }

    /// 注册实体定义
    pub fn register_entity_definition(&mut self, definition: EntityDefinition) {
        self.entity_definitions
            .entry(definition.framework.clone())
            .or_insert_with(Vec::new)
            .push(definition);
    }

    /// 注册映射规则
    pub fn register_mapping_rule(&mut self, rule: MappingRule) {
        self.mapping_rules.push(rule);
    }

    /// 执行实体映射
    pub fn map_entities(
        &mut self,
        source_framework: &FrameworkId,
        target_framework: &FrameworkId,
    ) -> MappingResult {
        let _start_time = std::time::Instant::now();
        let mut mappings = Vec::new();
        let mut unmapped_entities = Vec::new();

        // 获取源框架的实体定义
        let source_entities = self
            .entity_definitions
            .get(source_framework)
            .cloned()
            .unwrap_or_default();

        // 获取目标框架的实体定义
        let target_entities = self
            .entity_definitions
            .get(target_framework)
            .cloned()
            .unwrap_or_default();

        // 为每个源实体寻找目标实体
        for source_entity in &source_entities {
            if let Some(mapping) = self.find_best_mapping(
                source_entity,
                &target_entities,
                source_framework,
                target_framework,
            ) {
                mappings.push(mapping);
            } else {
                unmapped_entities.push(source_entity.id.clone());
            }
        }

        // 计算统计信息
        let stats = self.calculate_stats(&mappings, &source_entities, &target_entities);

        // 记录映射历史
        self.mapping_history.extend(mappings.clone());

        MappingResult {
            successful_mappings: mappings.len(),
            failed_mappings: unmapped_entities.len(),
            mappings,
            unmapped_entities,
            stats,
        }
    }

    /// 寻找最佳映射
    fn find_best_mapping(
        &self,
        source_entity: &EntityDefinition,
        target_entities: &[EntityDefinition],
        source_framework: &FrameworkId,
        target_framework: &FrameworkId,
    ) -> Option<EntityMapping> {
        let mut best_mapping = None;
        let mut best_score = 0.0;

        for target_entity in target_entities {
            let score = self.calculate_mapping_score(
                source_entity,
                target_entity,
                source_framework,
                target_framework,
            );

            if score > best_score && score >= self.config.min_confidence_threshold {
                best_score = score;
                best_mapping = Some(self.create_mapping(source_entity, target_entity, score));
            }
        }

        best_mapping
    }

    /// 计算映射分数
    fn calculate_mapping_score(
        &self,
        source_entity: &EntityDefinition,
        target_entity: &EntityDefinition,
        source_framework: &FrameworkId,
        target_framework: &FrameworkId,
    ) -> f64 {
        let mut score = 0.0;

        // 类型匹配分数
        if source_entity.entity_type == target_entity.entity_type {
            score += 0.3;
        }

        // 名称相似度分数
        let name_similarity =
            self.calculate_name_similarity(&source_entity.name, &target_entity.name);
        score += name_similarity * 0.3;

        // 属性匹配分数
        let attribute_score = self
            .calculate_attribute_similarity(&source_entity.attributes, &target_entity.attributes);
        score += attribute_score * 0.2;

        // 框架兼容性分数
        let framework_score =
            self.calculate_framework_compatibility(source_framework, target_framework);
        score += framework_score * 0.2;

        // 应用映射规则
        for rule in &self.mapping_rules {
            if rule.source_framework == *source_framework
                && rule.target_framework == *target_framework
                && rule.source_entity_type == source_entity.entity_type
                && rule.target_entity_type == target_entity.entity_type
            {
                score += 0.1; // 规则匹配加分
                break;
            }
        }

        score.min(1.0)
    }

    /// 计算名称相似度
    fn calculate_name_similarity(&self, name1: &str, name2: &str) -> f64 {
        // 简化的名称相似度计算
        let name1_lower = name1.to_lowercase();
        let name2_lower = name2.to_lowercase();

        if name1_lower == name2_lower {
            return 1.0;
        }

        // 检查命名约定转换
        for convention in &self.config.naming_conventions {
            let converted = self.apply_naming_convention(&name1_lower, convention);
            if converted == name2_lower {
                return 0.9;
            }
        }

        // 跨域语义同义（本模块的核心场景）。
        //
        // ⚠️ 此前 `enable_semantic_mapping` 开关在本函数中**从未被读取**，
        // 而字符集 Jaccard 对 `HashMap` / `Dict` 这类跨语言名称恒为 0
        // （`hashmap` 与 `dict` 无公共字符）⇒ 语义对齐能力缺失。
        // 取 0.7 而非 1.0：语义等价但字面不同，置信度应低于精确匹配(1.0)
        // 与命名约定等价(0.9)，仍高于 `min_confidence_threshold`(0.6)。
        if self.config.enable_semantic_mapping {
            if let (Some(c1), Some(c2)) =
                (semantic_concept(&name1_lower), semantic_concept(&name2_lower))
            {
                if c1 == c2 {
                    return 0.7;
                }
            }
        }

        // 基于字符重叠的简单相似度
        let chars1: std::collections::HashSet<char> = name1_lower.chars().collect();
        let chars2: std::collections::HashSet<char> = name2_lower.chars().collect();
        let intersection = chars1.intersection(&chars2).count();
        let union = chars1.union(&chars2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// 应用命名约定
    fn apply_naming_convention(&self, name: &str, convention: &NamingConvention) -> String {
        // 简化的命名约定转换
        match convention.conversion_function.as_str() {
            "camel_to_snake" => {
                let mut result = String::new();
                for (i, c) in name.chars().enumerate() {
                    if c.is_uppercase() && i > 0 {
                        result.push('_');
                    }
                    result.push(c.to_lowercase().next().unwrap_or(c));
                }
                result
            }
            "snake_to_camel" => {
                let mut result = String::new();
                let mut capitalize_next = false;
                for c in name.chars() {
                    if c == '_' {
                        capitalize_next = true;
                    } else if capitalize_next {
                        result.push(c.to_uppercase().next().unwrap_or(c));
                        capitalize_next = false;
                    } else {
                        result.push(c);
                    }
                }
                result
            }
            _ => name.to_string(),
        }
    }

    /// 计算属性相似度
    fn calculate_attribute_similarity(
        &self,
        attributes1: &[EntityAttribute],
        attributes2: &[EntityAttribute],
    ) -> f64 {
        if attributes1.is_empty() || attributes2.is_empty() {
            return 0.0;
        }

        let mut matches = 0;
        for attr1 in attributes1 {
            for attr2 in attributes2 {
                if attr1.name == attr2.name && attr1.attribute_type == attr2.attribute_type {
                    matches += 1;
                    break;
                }
            }
        }

        matches as f64 / attributes1.len().max(attributes2.len()) as f64
    }

    /// 计算框架兼容性
    fn calculate_framework_compatibility(
        &self,
        _source_framework: &FrameworkId,
        _target_framework: &FrameworkId,
    ) -> f64 {
        // 简化的框架兼容性计算
        // 在实际实现中，应该基于框架特性表进行计算
        0.5
    }

    /// 创建映射
    fn create_mapping(
        &self,
        source_entity: &EntityDefinition,
        target_entity: &EntityDefinition,
        confidence: f64,
    ) -> EntityMapping {
        let attribute_mappings =
            self.create_attribute_mappings(&source_entity.attributes, &target_entity.attributes);

        let transformation_rules = self
            .create_transformation_rules(&source_entity.entity_type, &target_entity.entity_type);

        EntityMapping {
            source_entity_id: source_entity.id.clone(),
            target_entity_id: target_entity.id.clone(),
            mapping_type: self.determine_mapping_type(confidence),
            confidence,
            attribute_mappings,
            transformation_rules,
        }
    }

    /// 创建属性映射
    fn create_attribute_mappings(
        &self,
        source_attributes: &[EntityAttribute],
        target_attributes: &[EntityAttribute],
    ) -> Vec<AttributeMapping> {
        let mut mappings = Vec::new();

        for source_attr in source_attributes {
            for target_attr in target_attributes {
                if source_attr.name == target_attr.name
                    && source_attr.attribute_type == target_attr.attribute_type
                {
                    mappings.push(AttributeMapping {
                        source_attribute: source_attr.name.clone(),
                        target_attribute: target_attr.name.clone(),
                        mapping_function: "identity".to_string(),
                        reversible: true,
                    });
                }
            }
        }

        mappings
            .into_iter()
            .take(self.config.max_attribute_mappings)
            .collect()
    }

    /// 创建变换规则
    fn create_transformation_rules(
        &self,
        source_type: &EntityType,
        target_type: &EntityType,
    ) -> Vec<TransformationRule> {
        let mut rules = Vec::new();

        if source_type != target_type {
            rules.push(TransformationRule {
                name: "type_conversion".to_string(),
                rule_type: TransformationType::TypeConversion,
                expression: format!("{:?} -> {:?}", source_type, target_type),
                parameters: HashMap::new(),
            });
        }

        rules
    }

    /// 确定映射类型
    fn determine_mapping_type(&self, confidence: f64) -> MappingType {
        if confidence > 0.9 {
            MappingType::Direct
        } else if confidence > 0.7 {
            MappingType::Approximate
        } else if confidence > 0.5 {
            MappingType::Functional
        } else {
            MappingType::Semantic
        }
    }

    /// 计算统计信息
    fn calculate_stats(
        &self,
        mappings: &[EntityMapping],
        source_entities: &[EntityDefinition],
        _target_entities: &[EntityDefinition],
    ) -> MappingStats {
        let average_confidence = if mappings.is_empty() {
            0.0
        } else {
            mappings.iter().map(|m| m.confidence).sum::<f64>() / mappings.len() as f64
        };

        let coverage_rate = if source_entities.is_empty() {
            0.0
        } else {
            mappings.len() as f64 / source_entities.len() as f64
        };

        let total_attribute_mappings: usize =
            mappings.iter().map(|m| m.attribute_mappings.len()).sum();
        let total_possible_attribute_mappings: usize =
            source_entities.iter().map(|e| e.attributes.len()).sum();

        let attribute_mapping_success_rate = if total_possible_attribute_mappings == 0 {
            0.0
        } else {
            total_attribute_mappings as f64 / total_possible_attribute_mappings as f64
        };

        let transformation_count: usize =
            mappings.iter().map(|m| m.transformation_rules.len()).sum();

        MappingStats {
            average_confidence,
            coverage_rate,
            attribute_mapping_success_rate,
            transformation_count,
        }
    }

    /// 应用映射转换实体
    pub fn transform_entity(
        &self,
        source_entity: &EntityDefinition,
        mapping: &EntityMapping,
    ) -> Result<EntityDefinition, String> {
        // 查找目标实体
        let target_entity = self
            .entity_definitions
            .values()
            .flatten()
            .find(|e| e.id == mapping.target_entity_id)
            .ok_or_else(|| format!("目标实体 {} 未找到", mapping.target_entity_id))?;

        // 应用属性映射
        let mut transformed_attributes = Vec::new();
        for attr_mapping in &mapping.attribute_mappings {
            if let Some(source_attr) = source_entity
                .attributes
                .iter()
                .find(|a| a.name == attr_mapping.source_attribute)
            {
                let transformed_value = self.apply_attribute_transformation(
                    &source_attr.value,
                    &attr_mapping.mapping_function,
                );

                transformed_attributes.push(EntityAttribute {
                    name: attr_mapping.target_attribute.clone(),
                    value: transformed_value,
                    attribute_type: source_attr.attribute_type.clone(),
                    convertible: source_attr.convertible,
                });
            }
        }

        // 应用变换规则
        let mut transformed_metadata = source_entity.metadata.clone();
        for rule in &mapping.transformation_rules {
            self.apply_transformation_rule(rule, &mut transformed_metadata);
        }

        Ok(EntityDefinition {
            id: mapping.target_entity_id.clone(),
            name: target_entity.name.clone(),
            entity_type: target_entity.entity_type.clone(),
            framework: target_entity.framework.clone(),
            attributes: transformed_attributes,
            description: format!("从 {} 转换而来", source_entity.name),
            embedding: None,
            metadata: transformed_metadata,
        })
    }

    /// 应用属性变换
    fn apply_attribute_transformation(&self, value: &str, function: &str) -> String {
        match function {
            "identity" => value.to_string(),
            "to_lowercase" => value.to_lowercase(),
            "to_uppercase" => value.to_uppercase(),
            "reverse" => value.chars().rev().collect(),
            _ => value.to_string(),
        }
    }

    /// 应用变换规则
    fn apply_transformation_rule(
        &self,
        rule: &TransformationRule,
        metadata: &mut HashMap<String, String>,
    ) {
        match rule.rule_type {
            TransformationType::TypeConversion => {
                metadata.insert("original_type".to_string(), rule.expression.clone());
            }
            TransformationType::StructuralRefactoring => {
                metadata.insert("refactored".to_string(), "true".to_string());
            }
            TransformationType::SemanticAdaptation => {
                metadata.insert("adapted".to_string(), "true".to_string());
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entity(
        id: &str,
        name: &str,
        entity_type: EntityType,
        framework: &str,
    ) -> EntityDefinition {
        EntityDefinition {
            id: id.to_string(),
            name: name.to_string(),
            entity_type,
            framework: framework.to_string(),
            attributes: vec![EntityAttribute {
                name: "test_attr".to_string(),
                value: "test_value".to_string(),
                attribute_type: AttributeType::Basic,
                convertible: true,
            }],
            description: format!("Test entity {}", name),
            embedding: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_mapper_config_default() {
        let config = EntityMappingConfig::default();
        assert_eq!(config.min_confidence_threshold, 0.6);
        assert!(config.enable_semantic_mapping);
        assert!(config.enable_structural_mapping);
        assert_eq!(config.max_attribute_mappings, 10);
    }

    #[test]
    fn test_direct_mapping() {
        let config = EntityMappingConfig::default();
        let mut mapper = EntityMapper::new(config);

        // 注册源框架实体
        let source_entity = create_test_entity("s1", "HashMap", EntityType::DataStructure, "rust");
        mapper.register_entity_definition(source_entity);

        // 注册目标框架实体
        let target_entity = create_test_entity("t1", "Dict", EntityType::DataStructure, "python");
        mapper.register_entity_definition(target_entity);

        // 执行映射
        let result = mapper.map_entities(&"rust".to_string(), &"python".to_string());

        assert_eq!(result.successful_mappings, 1);
        assert_eq!(result.failed_mappings, 0);
        assert!(result.mappings[0].confidence > 0.5);
    }

    #[test]
    fn test_name_similarity() {
        let config = EntityMappingConfig::default();
        let mapper = EntityMapper::new(config);

        assert_eq!(mapper.calculate_name_similarity("HashMap", "HashMap"), 1.0);
        assert!(mapper.calculate_name_similarity("HashMap", "Dict") > 0.0);
        assert_eq!(mapper.calculate_name_similarity("abc", "xyz"), 0.0);
    }

    /// A55 接线验收：锁定新增的跨域语义同义层。
    #[test]
    fn test_cross_domain_semantic_aliases() {
        let mapper = EntityMapper::new(EntityMappingConfig::default());
        // 同一概念、字面不同 ⇒ 高于 0，但低于精确匹配(1.0)与命名约定(0.9)
        assert!((mapper.calculate_name_similarity("HashMap", "dict") - 0.7).abs() < 1e-9);
        assert!((mapper.calculate_name_similarity("Vec", "array") - 0.7).abs() < 1e-9);
        // 分隔符/大小写归一化：hash_map / HASHMAP 同属 map 概念
        assert!((mapper.calculate_name_similarity("hash_map", "Dictionary") - 0.7).abs() < 1e-9);
        // 不同概念 ⇒ 仍走字符集 Jaccard，不误判
        assert!(mapper.calculate_name_similarity("HashMap", "Future") < 0.7);
        // 无别名的名字不产生语义分
        assert_eq!(semantic_concept("zzz"), None);
        assert_eq!(semantic_concept("HashMap"), Some("map"));
    }

    /// 关掉 `enable_semantic_mapping` 后，语义层必须完全失效（回到字符集相似度）。
    #[test]
    fn test_semantic_mapping_can_be_disabled() {
        let cfg = EntityMappingConfig {
            enable_semantic_mapping: false,
            ..Default::default()
        };
        let mapper = EntityMapper::new(cfg);
        assert_eq!(
            mapper.calculate_name_similarity("HashMap", "dict"),
            0.0,
            "关闭语义映射后不应返回同义分"
        );
    }

    #[test]
    fn test_naming_convention() {
        let config = EntityMappingConfig {
            naming_conventions: vec![NamingConvention {
                name: "camel_to_snake".to_string(),
                source_format: "camelCase".to_string(),
                target_format: "snake_case".to_string(),
                conversion_function: "camel_to_snake".to_string(),
            }],
            ..Default::default()
        };
        let mapper = EntityMapper::new(config);

        let result =
            mapper.apply_naming_convention("camelCase", &mapper.config.naming_conventions[0]);
        assert_eq!(result, "camel_case");
    }
}

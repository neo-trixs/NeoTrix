//! NeoTrix 跨域能力组合
//!
//! 支持多个能力的链式调用和并行执行

use crate::core::nt_core_capability::*;
use std::sync::Arc;

/// 组合能力
pub struct CapabilityComposer {
    /// 能力注册中心
    registry: Arc<CapabilityRegistry>,
}

/// 组合步骤
#[derive(Debug, Clone)]
pub struct CompositionStep {
    /// 能力ID
    pub capability_id: String,
    /// 输入映射 (从前一步输出中提取)
    pub input_mapper: String,
}

/// 组合管道
#[derive(Debug, Clone)]
pub struct CompositionPipeline {
    /// 管道ID
    pub id: String,
    /// 管道名称
    pub name: String,
    /// 步骤列表
    pub steps: Vec<CompositionStep>,
}

/// 组合执行结果
#[derive(Debug)]
pub struct CompositionResult {
    /// 最终输出
    pub output: CapabilityOutput,
    /// 中间结果
    pub intermediate: Vec<(String, CapabilityOutput)>,
    /// 总耗时
    pub total_duration_ms: u64,
}

impl CapabilityComposer {
    /// 创建新的组合器
    pub fn new(registry: Arc<CapabilityRegistry>) -> Self {
        Self { registry }
    }

    /// 执行管道
    pub fn execute_pipeline(
        &self,
        pipeline: &CompositionPipeline,
        initial_input: CapabilityInput,
    ) -> Result<CompositionResult, CapabilityError> {
        let start = std::time::Instant::now();
        let mut intermediate = Vec::new();
        let mut current_input = initial_input;

        for step in &pipeline.steps {
            // 获取能力
            let cap = self.registry.get(&step.capability_id).ok_or_else(|| {
                CapabilityError::ExecutionFailed(format!("能力未注册: {}", step.capability_id))
            })?;

            // 执行
            let output = cap.execute(current_input.clone())?;
            intermediate.push((step.capability_id.clone(), output.clone()));

            // 准备下一步输入
            current_input = self.map_output_to_input(&output, &step.input_mapper)?;
        }

        let total_duration = start.elapsed().as_millis() as u64;

        Ok(CompositionResult {
            output: intermediate
                .last()
                .map(|(_, o)| o.clone())
                .unwrap_or(CapabilityOutput::Text("".into())),
            intermediate,
            total_duration_ms: total_duration,
        })
    }

    /// 并行执行多个能力
    pub fn execute_parallel(
        &self,
        tasks: Vec<(String, CapabilityInput)>,
    ) -> Vec<Result<CapabilityOutput, CapabilityError>> {
        let mut results = Vec::new();

        for (cap_id, input) in tasks {
            if let Some(cap) = self.registry.get(&cap_id) {
                results.push(cap.execute(input));
            } else {
                results.push(Err(CapabilityError::ExecutionFailed(format!(
                    "能力未注册: {}",
                    cap_id
                ))));
            }
        }

        results
    }

    /// 映射输出到输入
    fn map_output_to_input(
        &self,
        output: &CapabilityOutput,
        mapper: &str,
    ) -> Result<CapabilityInput, CapabilityError> {
        match mapper {
            "text_to_nlp" => {
                if let CapabilityOutput::Text(text) = output {
                    Ok(CapabilityInput::Nlp(NlpInput {
                        task: NlpTask::Tokenize,
                        text: text.clone(),
                        language: None,
                    }))
                } else {
                    Err(CapabilityError::ExecutionFailed("类型不匹配".into()))
                }
            }
            "nlp_to_asset" => {
                if let CapabilityOutput::Nlp(nlp) = output {
                    match &nlp.result {
                        NlpResult::Tokens(tokens) => Ok(CapabilityInput::Asset(AssetInput {
                            query: tokens.join(" "),
                            asset_type: None,
                            limit: 10,
                        })),
                        _ => Err(CapabilityError::ExecutionFailed("需要Tokens结果".into())),
                    }
                } else {
                    Err(CapabilityError::ExecutionFailed("类型不匹配".into()))
                }
            }
            "passthrough" => {
                // 直接传递
                match output {
                    CapabilityOutput::Text(text) => Ok(CapabilityInput::Text(text.clone())),
                    CapabilityOutput::Nlp(nlp) => Ok(CapabilityInput::Nlp(NlpInput {
                        task: nlp.task.clone(),
                        text: String::new(),
                        language: None,
                    })),
                    _ => Err(CapabilityError::ExecutionFailed("不支持的透传类型".into())),
                }
            }
            _ => Err(CapabilityError::ExecutionFailed(format!(
                "未知映射: {}",
                mapper
            ))),
        }
    }

    /// 创建预定义管道
    pub fn create_pipeline(name: &str) -> Option<CompositionPipeline> {
        match name {
            "text_analysis" => Some(CompositionPipeline {
                id: "text_analysis".into(),
                name: "文本分析管道".into(),
                steps: vec![CompositionStep {
                    capability_id: "nt-world-nlp".into(),
                    input_mapper: "passthrough".into(),
                }],
            }),
            "asset_discovery" => Some(CompositionPipeline {
                id: "asset_discovery".into(),
                name: "资产发现管道".into(),
                steps: vec![
                    CompositionStep {
                        capability_id: "nt-world-nlp".into(),
                        input_mapper: "passthrough".into(),
                    },
                    CompositionStep {
                        capability_id: "nt-world-asset-map".into(),
                        input_mapper: "text_to_nlp".into(),
                    },
                ],
            }),
            "security_scan" => Some(CompositionPipeline {
                id: "security_scan".into(),
                name: "安全扫描管道".into(),
                steps: vec![
                    CompositionStep {
                        capability_id: "nt-world-asset-map".into(),
                        input_mapper: "passthrough".into(),
                    },
                    CompositionStep {
                        capability_id: "nt-shield-ztnet".into(),
                        input_mapper: "nlp_to_asset".into(),
                    },
                ],
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composer_creation() {
        let registry = Arc::new(init_global_registry());
        let composer = CapabilityComposer::new(registry);
        assert!(!composer.registry.list_all().is_empty());
    }

    #[test]
    fn pipeline_creation() {
        let pipeline = CapabilityComposer::create_pipeline("text_analysis");
        assert!(pipeline.is_some());
    }

    #[test]
    fn execute_text_analysis() {
        let registry = Arc::new(init_global_registry());
        let composer = CapabilityComposer::new(registry);
        let pipeline = CapabilityComposer::create_pipeline("text_analysis").unwrap();

        let input = CapabilityInput::Text("你好世界".into());
        let result = composer.execute_pipeline(&pipeline, input);

        assert!(result.is_ok());
    }
}

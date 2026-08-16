mod general;
mod specialized;

use crate::core::nt_core_knowledge::KnowledgeSource;
use crate::core::CapabilityVector;

pub(super) fn capability_vector_group_b(s: &KnowledgeSource) -> CapabilityVector {
    match s {
        KnowledgeSource::DeepSeekTui
        | KnowledgeSource::Codebuff
        | KnowledgeSource::OpenClaude
        | KnowledgeSource::Cairn
        | KnowledgeSource::Orca
        | KnowledgeSource::RedRun
        | KnowledgeSource::AutonomousSpeedrunning
        | KnowledgeSource::Synesis
        | KnowledgeSource::MemOS
        | KnowledgeSource::Reflexio
        | KnowledgeSource::Mem0
        | KnowledgeSource::Mnemosyne
        | KnowledgeSource::OriMnemos
        | KnowledgeSource::OPSD
        | KnowledgeSource::AttentionMechanism
        | KnowledgeSource::PatchFile
        | KnowledgeSource::KeyVault
        | KnowledgeSource::SealLoop
        | KnowledgeSource::HashCortxAgents
        | KnowledgeSource::HashCortxSecurity
        | KnowledgeSource::HashCortxSwarm
        | KnowledgeSource::HashCortxFailover
        | KnowledgeSource::HetuLuoshu
        | KnowledgeSource::YijingBinary
        | KnowledgeSource::FivePhasesGauge
        | KnowledgeSource::ThreeCosmologies
        | KnowledgeSource::HuainanziCalendar
        | KnowledgeSource::ZhangHengSeismoscope
        | KnowledgeSource::MawangduiAstronomy
        | KnowledgeSource::ShaoYongCosmology
        | KnowledgeSource::DayanNumber
        | KnowledgeSource::AdamsLaw => general::cap_vec_general(s),

        KnowledgeSource::IntegratedInformationTheory
        | KnowledgeSource::GlobalWorkspaceTheory
        | KnowledgeSource::ActiveInference
        | KnowledgeSource::VSAHyperdim
        | KnowledgeSource::JEPAWorldModel
        | KnowledgeSource::PredictiveCoding
        | KnowledgeSource::OrchOR
        | KnowledgeSource::AttentionSchema
        | KnowledgeSource::SiaHarnessUpdate
        | KnowledgeSource::SiaWeightUpdate
        | KnowledgeSource::SiaFeedbackLoop
        | KnowledgeSource::HyperAgents => specialized::cap_vec_specialized(s),

        KnowledgeSource::DialogueExperience => general::cap_vec_general(s),
        KnowledgeSource::ResearchFindings => general::cap_vec_general(s),

        _ => CapabilityVector::default(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic() {
        assert!(true);
    }
}

use serde::{Deserialize, Serialize};

use super::vsa_tag::{VsaOrigin, VsaSelfCategory, VsaWorldCategory};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AuthorityLevel {
    UserIntent,
    ToolOutput,
    CodeAnalysis,
    RepoConstitution,
    LiveCode,
    WebContent,
    Memory,
    PersonalityOverlay,
    SystemDefault,
    AmbientNoise,
}

impl AuthorityLevel {
    pub fn name(&self) -> &'static str {
        match self {
            AuthorityLevel::UserIntent => "user_intent",
            AuthorityLevel::ToolOutput => "tool_output",
            AuthorityLevel::CodeAnalysis => "code_analysis",
            AuthorityLevel::RepoConstitution => "repo_constitution",
            AuthorityLevel::LiveCode => "live_code",
            AuthorityLevel::WebContent => "web_content",
            AuthorityLevel::Memory => "memory",
            AuthorityLevel::PersonalityOverlay => "personality_overlay",
            AuthorityLevel::SystemDefault => "system_default",
            AuthorityLevel::AmbientNoise => "ambient_noise",
        }
    }

    pub fn all() -> &'static [AuthorityLevel] {
        &[
            AuthorityLevel::UserIntent,
            AuthorityLevel::ToolOutput,
            AuthorityLevel::CodeAnalysis,
            AuthorityLevel::RepoConstitution,
            AuthorityLevel::LiveCode,
            AuthorityLevel::WebContent,
            AuthorityLevel::Memory,
            AuthorityLevel::PersonalityOverlay,
            AuthorityLevel::SystemDefault,
            AuthorityLevel::AmbientNoise,
        ]
    }

    pub fn from_vsa_origin(tag: &VsaOrigin) -> Self {
        match tag {
            VsaOrigin::Self_(cat) => match cat {
                VsaSelfCategory::Intention => AuthorityLevel::UserIntent,
                VsaSelfCategory::Plan => AuthorityLevel::UserIntent,
                VsaSelfCategory::Thought => AuthorityLevel::Memory,
                VsaSelfCategory::Memory => AuthorityLevel::Memory,
                VsaSelfCategory::MetaCognition => AuthorityLevel::Memory,
                VsaSelfCategory::Emotion => AuthorityLevel::PersonalityOverlay,
                VsaSelfCategory::Imagination => AuthorityLevel::AmbientNoise,
                VsaSelfCategory::Association => AuthorityLevel::Memory,
                VsaSelfCategory::Private => AuthorityLevel::PersonalityOverlay,
            },
            VsaOrigin::World(cat) => match cat {
                VsaWorldCategory::UserInput => AuthorityLevel::UserIntent,
                VsaWorldCategory::ToolOutput => AuthorityLevel::ToolOutput,
                VsaWorldCategory::CodeAnalysis => AuthorityLevel::CodeAnalysis,
                VsaWorldCategory::FileContent => AuthorityLevel::LiveCode,
                VsaWorldCategory::WebContent => AuthorityLevel::WebContent,
                VsaWorldCategory::Sensor => AuthorityLevel::WebContent,
                VsaWorldCategory::SystemEvent => AuthorityLevel::SystemDefault,
            },
        }
    }

    pub fn priority(&self) -> u8 {
        match self {
            AuthorityLevel::UserIntent => 100,
            AuthorityLevel::ToolOutput => 90,
            AuthorityLevel::CodeAnalysis => 85,
            AuthorityLevel::RepoConstitution => 80,
            AuthorityLevel::LiveCode => 70,
            AuthorityLevel::WebContent => 50,
            AuthorityLevel::Memory => 40,
            AuthorityLevel::PersonalityOverlay => 30,
            AuthorityLevel::SystemDefault => 20,
            AuthorityLevel::AmbientNoise => 10,
        }
    }

    pub fn outranks(&self, other: &AuthorityLevel) -> bool {
        self.priority() > other.priority()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ConflictResolution {
    UseHigherAuthority(AuthorityLevel, AuthorityLevel),
    UseHigherConfidence(f64, f64),
    UseMostRecent,
    MergeWithWeight(f64),
    RequireExternalValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityTag {
    pub authority: AuthorityLevel,
    pub specific_priority: u8,
    pub provenance_detail: Option<String>,
}

impl AuthorityTag {
    pub fn new(authority: AuthorityLevel) -> Self {
        let specific_priority = authority.priority();
        Self {
            authority,
            specific_priority,
            provenance_detail: None,
        }
    }

    pub fn with_detail(mut self, detail: &str) -> Self {
        self.provenance_detail = Some(detail.to_string());
        self
    }

    pub fn effective_priority(&self) -> u8 {
        self.specific_priority
    }
}

pub struct AuthorityResolver;

impl AuthorityResolver {
    pub fn resolve_conflict(
        primary: &AuthorityTag,
        secondary: &AuthorityTag,
        primary_confidence: f64,
        secondary_confidence: f64,
    ) -> ConflictResolution {
        let primary_prio = primary.effective_priority();
        let secondary_prio = secondary.effective_priority();

        if primary_prio > secondary_prio + 10 {
            return ConflictResolution::UseHigherAuthority(primary.authority, secondary.authority);
        }

        if secondary_prio > primary_prio + 10 {
            return ConflictResolution::UseHigherAuthority(secondary.authority, primary.authority);
        }

        let conf_gap = (primary_confidence - secondary_confidence).abs();
        if conf_gap > 0.2 {
            if primary_confidence > secondary_confidence {
                return ConflictResolution::UseHigherConfidence(
                    primary_confidence,
                    secondary_confidence,
                );
            } else {
                return ConflictResolution::UseHigherConfidence(
                    secondary_confidence,
                    primary_confidence,
                );
            }
        }

        ConflictResolution::MergeWithWeight(primary_priority_weight(primary_prio, secondary_prio))
    }

    pub fn filter_by_threshold(
        items: Vec<super::vsa_tag::VsaTagged>,
        min_authority: AuthorityLevel,
    ) -> Vec<super::vsa_tag::VsaTagged> {
        let threshold = min_authority.priority();
        items
            .into_iter()
            .filter(|item| {
                let auth = AuthorityLevel::from_vsa_origin(&item.tag);
                auth.priority() >= threshold
            })
            .collect()
    }

    pub fn sort_by_authority(items: &mut [super::vsa_tag::VsaTagged]) {
        items.sort_by(|a, b| {
            let auth_a = AuthorityLevel::from_vsa_origin(&a.tag);
            let auth_b = AuthorityLevel::from_vsa_origin(&b.tag);
            auth_b.priority().cmp(&auth_a.priority()).then_with(|| {
                b.confidence
                    .partial_cmp(&a.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });
    }
}

fn primary_priority_weight(p1: u8, p2: u8) -> f64 {
    let total = (p1 + p2) as f64;
    if total == 0.0 {
        return 0.5;
    }
    p1 as f64 / total
}

pub struct Constitution {
    pub authority_ordering: Vec<AuthorityLevel>,
    pub protected_invariants: Vec<String>,
    pub escalation_trigger: f64,
}

impl Default for Constitution {
    fn default() -> Self {
        Self::neotrix_default()
    }
}

impl Constitution {
    pub fn neotrix_default() -> Self {
        Self {
            authority_ordering: vec![
                AuthorityLevel::UserIntent,
                AuthorityLevel::ToolOutput,
                AuthorityLevel::CodeAnalysis,
                AuthorityLevel::RepoConstitution,
                AuthorityLevel::LiveCode,
                AuthorityLevel::WebContent,
                AuthorityLevel::Memory,
                AuthorityLevel::PersonalityOverlay,
                AuthorityLevel::SystemDefault,
                AuthorityLevel::AmbientNoise,
            ],
            protected_invariants: vec![
                "never_expose_api_keys".into(),
                "never_delete_user_data".into(),
                "always_validate_output".into(),
            ],
            escalation_trigger: 0.3,
        }
    }

    pub fn resolve(&self, items: Vec<super::vsa_tag::VsaTagged>) -> Vec<super::vsa_tag::VsaTagged> {
        let mut items = items;
        AuthorityResolver::sort_by_authority(&mut items);
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::vsa_tag::{VsaOrigin, VsaSelfCategory, VsaTagged};

    #[test]
    fn test_authority_level_ordering() {
        assert!(AuthorityLevel::UserIntent > AuthorityLevel::Memory);
        assert!(AuthorityLevel::ToolOutput > AuthorityLevel::WebContent);
        assert!(AuthorityLevel::AmbientNoise < AuthorityLevel::SystemDefault);
    }

    #[test]
    fn test_authority_from_vsa_origin_self() {
        let thought = VsaOrigin::Self_(VsaSelfCategory::Thought);
        assert_eq!(
            AuthorityLevel::from_vsa_origin(&thought),
            AuthorityLevel::Memory
        );

        let intention = VsaOrigin::Self_(VsaSelfCategory::Intention);
        assert_eq!(
            AuthorityLevel::from_vsa_origin(&intention),
            AuthorityLevel::UserIntent
        );
    }

    #[test]
    fn test_authority_from_vsa_origin_world() {
        let input = VsaOrigin::World(
            crate::core::nt_core_consciousness::vsa_tag::VsaWorldCategory::UserInput,
        );
        assert_eq!(
            AuthorityLevel::from_vsa_origin(&input),
            AuthorityLevel::UserIntent
        );

        let tool = VsaOrigin::World(
            crate::core::nt_core_consciousness::vsa_tag::VsaWorldCategory::ToolOutput,
        );
        assert_eq!(
            AuthorityLevel::from_vsa_origin(&tool),
            AuthorityLevel::ToolOutput
        );
    }

    #[test]
    fn test_authority_tag_priority() {
        let tag = AuthorityTag::new(AuthorityLevel::UserIntent);
        assert_eq!(tag.effective_priority(), 100);

        let tag = AuthorityTag::new(AuthorityLevel::AmbientNoise);
        assert_eq!(tag.effective_priority(), 10);
    }

    #[test]
    fn test_authority_tag_with_detail() {
        let tag = AuthorityTag::new(AuthorityLevel::ToolOutput)
            .with_detail("cargo check returned warnings");
        assert_eq!(
            tag.provenance_detail.unwrap(),
            "cargo check returned warnings"
        );
    }

    #[test]
    fn test_resolve_conflict_by_authority() {
        let user = AuthorityTag::new(AuthorityLevel::UserIntent);
        let memory = AuthorityTag::new(AuthorityLevel::Memory);
        let resolution = AuthorityResolver::resolve_conflict(&user, &memory, 0.5, 0.9);
        assert!(matches!(
            resolution,
            ConflictResolution::UseHigherAuthority(AuthorityLevel::UserIntent, _)
        ));
    }

    #[test]
    fn test_resolve_conflict_by_confidence() {
        let web1 = AuthorityTag::new(AuthorityLevel::WebContent);
        let web2 = AuthorityTag::new(AuthorityLevel::WebContent);
        let resolution = AuthorityResolver::resolve_conflict(&web1, &web2, 0.95, 0.3);
        assert!(matches!(
            resolution,
            ConflictResolution::UseHigherConfidence(0.95, _)
        ));
    }

    #[test]
    fn test_resolve_conflict_merge_close() {
        let mem1 = AuthorityTag::new(AuthorityLevel::Memory);
        let mem2 = AuthorityTag::new(AuthorityLevel::Memory);
        let resolution = AuthorityResolver::resolve_conflict(&mem1, &mem2, 0.6, 0.55);
        assert!(matches!(resolution, ConflictResolution::MergeWithWeight(_)));
    }

    #[test]
    fn test_filter_by_threshold() {
        let items = vec![
            VsaTagged::new(vec![1; 16], VsaOrigin::Self_(VsaSelfCategory::Thought)),
            VsaTagged::new(vec![1; 16], VsaOrigin::Self_(VsaSelfCategory::Intention)),
        ];
        let filtered = AuthorityResolver::filter_by_threshold(items, AuthorityLevel::Memory);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_sort_by_authority() {
        let mut items = vec![
            VsaTagged::new(vec![1; 16], VsaOrigin::Self_(VsaSelfCategory::Memory)),
            VsaTagged::new(vec![1; 16], VsaOrigin::Self_(VsaSelfCategory::Intention)),
        ];
        AuthorityResolver::sort_by_authority(&mut items);
        assert_eq!(items[0].tag, VsaOrigin::Self_(VsaSelfCategory::Intention));
        assert_eq!(items[1].tag, VsaOrigin::Self_(VsaSelfCategory::Memory));
    }

    #[test]
    fn test_constitution_default_has_ordering() {
        let constitution = Constitution::neotrix_default();
        assert_eq!(constitution.authority_ordering.len(), 10);
        assert_eq!(
            constitution.authority_ordering[0],
            AuthorityLevel::UserIntent
        );
        assert_eq!(
            constitution.authority_ordering[9],
            AuthorityLevel::AmbientNoise
        );
    }

    #[test]
    fn test_constitution_has_protected_invariants() {
        let constitution = Constitution::neotrix_default();
        assert!(constitution
            .protected_invariants
            .contains(&"never_expose_api_keys".into()));
    }

    #[test]
    fn test_constitution_resolve_sorts_by_authority() {
        let items = vec![
            VsaTagged::new(vec![1; 16], VsaOrigin::Self_(VsaSelfCategory::Memory)),
            VsaTagged::new(vec![1; 16], VsaOrigin::Self_(VsaSelfCategory::Intention)),
        ];
        let constitution = Constitution::neotrix_default();
        let resolved = constitution.resolve(items);
        assert_eq!(resolved.len(), 2);
        assert_eq!(
            resolved[0].tag,
            VsaOrigin::Self_(VsaSelfCategory::Intention)
        );
    }

    #[test]
    fn test_authority_level_priority_all_unique() {
        let mut priorities: Vec<u8> = AuthorityLevel::all().iter().map(|a| a.priority()).collect();
        priorities.sort();
        priorities.dedup();
        assert_eq!(priorities.len(), AuthorityLevel::all().len());
    }

    #[test]
    fn test_outranks_positive() {
        assert!(AuthorityLevel::UserIntent.outranks(&AuthorityLevel::Memory));
        assert!(AuthorityLevel::LiveCode.outranks(&AuthorityLevel::AmbientNoise));
    }

    #[test]
    fn test_outranks_negative() {
        assert!(!AuthorityLevel::Memory.outranks(&AuthorityLevel::UserIntent));
        assert!(!AuthorityLevel::SystemDefault.outranks(&AuthorityLevel::LiveCode));
    }
}

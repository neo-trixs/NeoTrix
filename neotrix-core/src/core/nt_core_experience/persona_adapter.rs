// G410: Persona-adaptive cognitive depth — Understand-Anything style role-based detail levels
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonaRole {
    Novice,
    Developer,
    PowerUser,
    Architect,
    Auditor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaProfile {
    pub role: PersonaRole,
    pub detail_level: u8,
    pub show_internals: bool,
    pub show_vsa: bool,
    pub show_confidence: bool,
    pub max_explanation_depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CognitionOutput {
    Shallow(String),
    Detailed(String),
    Technical(String),
    Full(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaAdapter {
    pub current: PersonaProfile,
    pub profiles: Vec<PersonaProfile>,
    pub history: Vec<(String, PersonaRole)>,
    pub auto_detect_enabled: bool,
}

impl PersonaAdapter {
    pub fn new() -> Self {
        Self {
            current: Self::default_profile(PersonaRole::Developer),
            profiles: Self::all_profiles(),
            history: Vec::new(),
            auto_detect_enabled: true,
        }
    }

    fn all_profiles() -> Vec<PersonaProfile> {
        vec![
            PersonaProfile {
                role: PersonaRole::Novice,
                detail_level: 1,
                show_internals: false,
                show_vsa: false,
                show_confidence: false,
                max_explanation_depth: 1,
            },
            PersonaProfile {
                role: PersonaRole::Developer,
                detail_level: 3,
                show_internals: true,
                show_vsa: false,
                show_confidence: true,
                max_explanation_depth: 3,
            },
            PersonaProfile {
                role: PersonaRole::PowerUser,
                detail_level: 5,
                show_internals: true,
                show_vsa: true,
                show_confidence: true,
                max_explanation_depth: 5,
            },
            PersonaProfile {
                role: PersonaRole::Architect,
                detail_level: 7,
                show_internals: true,
                show_vsa: true,
                show_confidence: true,
                max_explanation_depth: 10,
            },
            PersonaProfile {
                role: PersonaRole::Auditor,
                detail_level: 10,
                show_internals: true,
                show_vsa: true,
                show_confidence: true,
                max_explanation_depth: 100,
            },
        ]
    }

    fn default_profile(role: PersonaRole) -> PersonaProfile {
        match role {
            PersonaRole::Novice => PersonaProfile {
                role,
                detail_level: 1,
                show_internals: false,
                show_vsa: false,
                show_confidence: false,
                max_explanation_depth: 1,
            },
            PersonaRole::Developer => PersonaProfile {
                role,
                detail_level: 3,
                show_internals: true,
                show_vsa: false,
                show_confidence: true,
                max_explanation_depth: 3,
            },
            PersonaRole::PowerUser => PersonaProfile {
                role,
                detail_level: 5,
                show_internals: true,
                show_vsa: true,
                show_confidence: true,
                max_explanation_depth: 5,
            },
            PersonaRole::Architect => PersonaProfile {
                role,
                detail_level: 7,
                show_internals: true,
                show_vsa: true,
                show_confidence: true,
                max_explanation_depth: 10,
            },
            PersonaRole::Auditor => PersonaProfile {
                role,
                detail_level: 10,
                show_internals: true,
                show_vsa: true,
                show_confidence: true,
                max_explanation_depth: 100,
            },
        }
    }

    pub fn switch_to(&mut self, role: PersonaRole) {
        self.current = Self::default_profile(role);
    }

    pub fn detect_from_query(&mut self, query: &str) -> PersonaRole {
        let q = query.to_lowercase();
        let role =
            if q.contains("explain like i'm 5") || q.contains("simple") || q.contains("beginner") {
                PersonaRole::Novice
            } else if q.contains("architecture") || q.contains("design") || q.contains("system") {
                PersonaRole::Architect
            } else if q.contains("audit") || q.contains("security") || q.contains("review") {
                PersonaRole::Auditor
            } else if q.contains("vsa") || q.contains("detailed") || q.contains("internals") {
                PersonaRole::PowerUser
            } else {
                PersonaRole::Developer
            };

        if self.auto_detect_enabled {
            self.current = Self::default_profile(role);
        }
        self.history.push((query.to_string(), role));
        if self.history.len() > 100 {
            self.history.remove(0);
        }
        role
    }

    pub fn adapt_output(&self, technical: &str, simple: &str) -> CognitionOutput {
        match self.current.role {
            PersonaRole::Novice => CognitionOutput::Shallow(simple.to_string()),
            PersonaRole::Developer => {
                let depth = technical.len().min(500);
                CognitionOutput::Detailed(technical[..depth].to_string())
            }
            PersonaRole::PowerUser => CognitionOutput::Technical(technical.to_string()),
            PersonaRole::Architect => CognitionOutput::Technical(technical.to_string()),
            PersonaRole::Auditor => CognitionOutput::Full(technical.to_string()),
        }
    }

    pub fn format_explanation(&self, inner_workings: &str, summary: &str) -> String {
        match self.current.role {
            PersonaRole::Novice => {
                format!(
                    "{}\n\nNeed more detail? Ask with 'technical mode'.",
                    summary
                )
            }
            PersonaRole::Developer => {
                format!(
                    "{}\n\n---\nHow it works:\n{}\n\n(detail_level={})",
                    summary,
                    inner_workings
                        .lines()
                        .take(10)
                        .collect::<Vec<_>>()
                        .join("\n"),
                    self.current.detail_level
                )
            }
            PersonaRole::PowerUser => {
                format!(
                    "## Summary\n{}\n\n## Technical\n{}\n\n## VSA Context\nEnabled (detail_level={})",
                    summary, inner_workings, self.current.detail_level
                )
            }
            PersonaRole::Architect => {
                format!(
                    "# Architecture Analysis\n\n{}\n\n# Full Technical\n{}\n\n*VSA: {}, Depth: {}*",
                    summary,
                    inner_workings,
                    self.current.show_vsa,
                    self.current.max_explanation_depth
                )
            }
            PersonaRole::Auditor => {
                format!(
                    "# AUDIT REPORT\n\n## Executive Summary\n{}\n\n## Complete Technical Trace\n{}\n\n---\n*Detail Level: {}/10, VSA: enabled, Internals: full*",
                    summary, inner_workings, self.current.detail_level
                )
            }
        }
    }

    pub fn current_role_label(&self) -> &'static str {
        match self.current.role {
            PersonaRole::Novice => "Novice",
            PersonaRole::Developer => "Developer",
            PersonaRole::PowerUser => "Power User",
            PersonaRole::Architect => "Architect",
            PersonaRole::Auditor => "Auditor",
        }
    }
}

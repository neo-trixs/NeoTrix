//! Persona Routing - 人格路由模块
//!
//! Wedge (楔): 9轨路由 (逆向/Pwn/Web/密码/移动/取证/渗透/内存/协议)
//! Prism (棱): 7路路由 (CRAFT/DISSECT/PROBE/PLAY/FORGE/NARRATE/REVIEW)

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersonaType {
    Wedge,
    Prism,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WedgeTrack {
    Reverse,
    Pwn,
    Web,
    Crypto,
    Mobile,
    Forensics,
    Pentest,
    Memory,
    Protocol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrismRoute {
    Craft,
    Dissect,
    Probe,
    Play,
    Forge,
    Narrate,
    Review,
}

pub struct PersonaRouter {
    current_persona: PersonaType,
    wedge_tracks: Vec<WedgeTrack>,
    prism_routes: Vec<PrismRoute>,
    _skill_registry: HashMap<String, Vec<String>>,
}

impl PersonaRouter {
    pub fn new() -> Self {
        Self {
            current_persona: PersonaType::Wedge,
            wedge_tracks: vec![
                WedgeTrack::Reverse,
                WedgeTrack::Pwn,
                WedgeTrack::Web,
                WedgeTrack::Crypto,
                WedgeTrack::Mobile,
                WedgeTrack::Forensics,
                WedgeTrack::Pentest,
                WedgeTrack::Memory,
                WedgeTrack::Protocol,
            ],
            prism_routes: vec![
                PrismRoute::Craft,
                PrismRoute::Dissect,
                PrismRoute::Probe,
                PrismRoute::Play,
                PrismRoute::Forge,
                PrismRoute::Narrate,
                PrismRoute::Review,
            ],
            _skill_registry: HashMap::new(),
        }
    }

    pub fn detect_persona(&self, input: &str) -> PersonaType {
        let input_lower = input.to_lowercase();

        let wedge_keywords = ["逆向", "reverse", "pwn", "exploit", "webshell", "渗透", "pentest"];
        for keyword in wedge_keywords {
            if input_lower.contains(keyword) {
                return PersonaType::Wedge;
            }
        }

        let prism_keywords = ["分析", "analyze", "设计", "design", "审查", "review", "叙述", "narrate"];
        for keyword in prism_keywords {
            if input_lower.contains(keyword) {
                return PersonaType::Prism;
            }
        }

        PersonaType::Wedge
    }

    pub fn detect_wedge_track(&self, input: &str) -> WedgeTrack {
        let input_lower = input.to_lowercase();

        if input_lower.contains("逆向") || input_lower.contains("reverse") {
            WedgeTrack::Reverse
        } else if input_lower.contains("pwn") || input_lower.contains("exploit") {
            WedgeTrack::Pwn
        } else if input_lower.contains("web") || input_lower.contains("http") {
            WedgeTrack::Web
        } else if input_lower.contains("密码") || input_lower.contains("crypto") {
            WedgeTrack::Crypto
        } else if input_lower.contains("移动") || input_lower.contains("mobile") || input_lower.contains("apk") {
            WedgeTrack::Mobile
        } else if input_lower.contains("取证") || input_lower.contains("forensic") {
            WedgeTrack::Forensics
        } else if input_lower.contains("渗透") || input_lower.contains("pentest") {
            WedgeTrack::Pentest
        } else if input_lower.contains("内存") || input_lower.contains("memory") {
            WedgeTrack::Memory
        } else if input_lower.contains("协议") || input_lower.contains("protocol") {
            WedgeTrack::Protocol
        } else {
            WedgeTrack::Reverse
        }
    }

    pub fn detect_prism_route(&self, input: &str) -> PrismRoute {
        let input_lower = input.to_lowercase();

        if input_lower.contains("制作") || input_lower.contains("craft") || input_lower.contains("build") {
            PrismRoute::Craft
        } else if input_lower.contains("解剖") || input_lower.contains("dissect") || input_lower.contains("analyze") {
            PrismRoute::Dissect
        } else if input_lower.contains("探测") || input_lower.contains("probe") || input_lower.contains("scan") {
            PrismRoute::Probe
        } else if input_lower.contains("执行") || input_lower.contains("play") || input_lower.contains("run") {
            PrismRoute::Play
        } else if input_lower.contains("锻造") || input_lower.contains("forge") || input_lower.contains("create") {
            PrismRoute::Forge
        } else if input_lower.contains("叙述") || input_lower.contains("narrate") || input_lower.contains("explain") {
            PrismRoute::Narrate
        } else if input_lower.contains("审查") || input_lower.contains("review") || input_lower.contains("audit") {
            PrismRoute::Review
        } else {
            PrismRoute::Dissect
        }
    }

    pub fn route_to_skill(&self, input: &str) -> String {
        let persona = self.detect_persona(input);

        match persona {
            PersonaType::Wedge => {
                let track = self.detect_wedge_track(input);
                format!("wedge_{}", format!("{:?}", track).to_lowercase())
            }
            PersonaType::Prism => {
                let route = self.detect_prism_route(input);
                format!("prism_{}", format!("{:?}", route).to_lowercase())
            }
        }
    }

    pub fn get_status(&self) -> HashMap<String, String> {
        let mut status = HashMap::new();
        status.insert("current_persona".to_string(), format!("{:?}", self.current_persona));
        status.insert("wedge_tracks".to_string(), self.wedge_tracks.len().to_string());
        status.insert("prism_routes".to_string(), self.prism_routes.len().to_string());
        status
    }
}

impl Default for PersonaRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_persona() {
        let router = PersonaRouter::new();
        assert_eq!(router.detect_persona("逆向分析这个APK"), PersonaType::Wedge);
        assert_eq!(router.detect_persona("设计系统架构"), PersonaType::Prism);
    }

    #[test]
    fn test_detect_wedge_track() {
        let router = PersonaRouter::new();
        assert_eq!(router.detect_wedge_track("逆向分析"), WedgeTrack::Reverse);
        assert_eq!(router.detect_wedge_track("web渗透测试"), WedgeTrack::Web);
    }

    #[test]
    fn test_route_to_skill() {
        let router = PersonaRouter::new();
        assert_eq!(router.route_to_skill("逆向分析"), "wedge_reverse");
        assert_eq!(router.route_to_skill("设计架构"), "prism_craft");
    }
}

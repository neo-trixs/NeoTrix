//! NT-3D-TASK-SPEC — 3D 任务书构造 + 验收单（astra 写法吸收落地）
//!
//! 高完成度 3D 任务 = 7 段式：目标 / 视觉方向 / 世界布局 / 资产槽位
//! / 量化机制 / 技术栈性能 / 验收单（逐条映射回机制与槽位）。
//! 占位先行逐槽替换；验收必须可勾选。无 unwrap / expect / panic。

use serde::{Deserialize, Serialize};

/// 资产槽位：命名 + 规格 + 复用次数 + 程序化回退
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtAssetSlot {
    pub name: String,
    pub spec: String,
    pub reuse_count: u32,
    pub fallback_procedural: bool,
    pub status: String,
}

impl NtAssetSlot {
    pub fn new(name: &str, spec: &str, reuse_count: u32) -> Self {
        Self {
            name: name.to_string(),
            spec: spec.to_string(),
            reuse_count,
            fallback_procedural: true,
            status: "placeholder".to_string(),
        }
    }
}

/// 量化机制项：名 + 值 + 单位（可测才可验）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtMechanicsNumber {
    pub name: String,
    pub value: f64,
    pub unit: String,
}

/// 3D 任务书
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Nt3DTaskSpec {
    pub goal: String,
    pub constraints: Vec<String>,
    pub visual: String,
    pub world: String,
    pub slots: Vec<NtAssetSlot>,
    pub mechanics: Vec<NtMechanicsNumber>,
    pub stack: String,
    pub perf: Vec<String>,
}

impl Nt3DTaskSpec {
    /// 缺件诊断：返回缺失项描述（空 = 完整可下发）
    pub fn validate(&self) -> Vec<String> {
        let mut missing = Vec::new();
        if self.goal.trim().is_empty() {
            missing.push("goal 为空".to_string());
        }
        if self.slots.is_empty() {
            missing.push("资产槽位为 0（先占位再替换无从谈起）".to_string());
        }
        if self.mechanics.is_empty() {
            missing.push("量化机制为 0（不可测不可验）".to_string());
        }
        for s in &self.slots {
            if !s.fallback_procedural {
                missing.push(format!("槽位 {} 无程序化回退", s.name));
            }
        }
        missing
    }

    /// 渲染 7 段式任务书文本
    pub fn render_prompt(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# Goal\n{}\n\n", self.goal));
        if !self.constraints.is_empty() {
            out.push_str("Constraints:\n");
            for c in &self.constraints {
                out.push_str(&format!("- {c}\n"));
            }
            out.push('\n');
        }
        out.push_str(&format!("# Visual direction\n{}\n\n", self.visual));
        out.push_str(&format!("# World\n{}\n\n", self.world));
        out.push_str("# Asset inventory\n");
        for s in &self.slots {
            out.push_str(&format!(
                "- `{}`: {} (reuse×{}, fallback={}, status={})\n",
                s.name,
                s.spec,
                s.reuse_count,
                if s.fallback_procedural {
                    "procedural"
                } else {
                    "none"
                },
                s.status
            ));
        }
        out.push_str("\n# Mechanics (quantified)\n");
        for m in &self.mechanics {
            out.push_str(&format!("- {}: {} {}\n", m.name, m.value, m.unit));
        }
        out.push_str(&format!("\n# Stack & perf\n{}\n", self.stack));
        for p in &self.perf {
            out.push_str(&format!("- {p}\n"));
        }
        out
    }

    /// 验收单：逐条映射机制 + 槽位回退 + 性能项
    pub fn acceptance(&self) -> Vec<String> {
        let mut items = Vec::new();
        for m in &self.mechanics {
            items.push(format!("verify {} = {} {}", m.name, m.value, m.unit));
        }
        for s in &self.slots {
            items.push(format!(
                "slot {} fallback {}",
                s.name,
                if s.fallback_procedural {
                    "playable"
                } else {
                    "MISSING"
                }
            ));
        }
        for p in &self.perf {
            items.push(format!("perf {p}"));
        }
        items.push("compare settled result against reference".to_string());
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tea_set() -> Nt3DTaskSpec {
        Nt3DTaskSpec {
            goal: "一套茶具静物 WebGL 页".to_string(),
            constraints: vec!["中文 UI".to_string()],
            visual: "暖木桌，侧光，45° 俯视".to_string(),
            world: "桌中茶壶，南侧两杯，北侧窗光".to_string(),
            slots: vec![
                NtAssetSlot::new("teapot", "圆腹紫砂壶", 1),
                NtAssetSlot::new("cups", "白瓷杯", 2),
            ],
            mechanics: vec![
                NtMechanicsNumber {
                    name: "旋转".to_string(),
                    value: 0.5,
                    unit: "rad/s".to_string(),
                },
                NtMechanicsNumber {
                    name: "缩放".to_string(),
                    value: 1.2,
                    unit: "x".to_string(),
                },
            ],
            stack: "Three.js ES modules，静态构建".to_string(),
            perf: vec!["60fps".to_string()],
        }
    }

    #[test]
    fn test_prompt_has_seven_sections() {
        let p = tea_set().render_prompt();
        for h in [
            "# Goal",
            "# Visual direction",
            "# World",
            "# Asset inventory",
            "# Mechanics",
            "# Stack",
        ] {
            assert!(p.contains(h), "missing section {h}");
        }
        assert!(p.contains("teapot"));
    }

    #[test]
    fn test_acceptance_maps_mechanics_and_slots() {
        let spec = tea_set();
        let acc = spec.acceptance();
        // 2 机制 + 2 槽位 + 1 性能 + 1 对比 = 6
        assert_eq!(acc.len(), 6);
        assert!(acc.iter().any(|s| s.contains("旋转")));
        assert!(acc
            .iter()
            .any(|s| s.contains("teapot") && s.contains("playable")));
    }

    #[test]
    fn test_validate_catches_gaps() {
        let empty = Nt3DTaskSpec::default();
        assert!(empty.validate().len() >= 3);
        let mut bad = tea_set();
        if let Some(s) = bad.slots.first_mut() {
            s.fallback_procedural = false;
        }
        assert!(bad.validate().iter().any(|m| m.contains("回退")));
        assert!(tea_set().validate().is_empty());
    }
}

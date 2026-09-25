//! skill_activate — 从 `nt_mind_skill_engine.rs` 拆分 (行为零变更).
//! 激活 / 失活 / 可见性门控 + 安装 / 卸载 / fiber 生命周期:
//! `load_reference` / `step_disclosure` / `activate_skill` / `deactivate_skill` /
//! `list_active` / `visible_active` / `install_skill` / `install_from_procedural` /
//! `uninstall_skill` / `fiber_state` / `release_dangling` 家族.

use std::path::{Path, PathBuf};

use super::FiberLifecycle;
use super::FiberLifecycleState;
use super::RevertibleEffect;
use super::SkillDocEntry;
use super::SkillEngine;
use crate::l5_cognition::l1_facade::ProceduralMemoryRecord;
use crate::l5_cognition::nt_mind::nt_mind_hook::{HookContext, HookEvent};

impl SkillEngine {
    /// 渐进披露加载 (progressive disclosure, diagram-design 吸收):
    /// SKILL.md 只描述技能的选择与入口, 深层细节 (参考文档/模板/示例) 存于
    /// `<skill_dir>/references/<file>`, 按需读取 — 避免常驻加载拉爆上下文。
    ///
    /// 返回已声明引用中命中的内容; 未声明或不存在返回 Err (提示缺失)。
    pub fn load_reference(&self, name: &str, reference: &str) -> Result<String, String> {
        let entry = self
            .get_skill(name)
            .ok_or_else(|| format!("Skill '{}' not found", name))?;
        if !entry.references.iter().any(|r| r == reference) {
            return Err(format!(
                "Reference '{}' not declared in skill '{}' (declared: {:?})",
                reference, name, entry.references
            ));
        }
        let skill_dir = entry.path.parent().unwrap_or(&self.skills_dir);
        let ref_path = skill_dir.join("references").join(reference);
        if !ref_path.exists() {
            return Err(format!(
                "Reference file missing: {}",
                ref_path.display()
            ));
        }
        std::fs::read_to_string(&ref_path).map_err(|e| format!("read reference: {}", e))
    }

    /// 推进渐进披露阶梯 (P4, dsh-anchored-standard 吸收): 若 session 已
    /// durable (首个 durable 工具/调用), 从 Minimal 提升到 Standard 工具集。
    /// 返回阶段是否发生变化。
    pub fn step_disclosure(&mut self) -> bool {
        self.disclosure.maybe_promote()
    }

    /// Activate a skill by name. Fires HookEvent::SkillLoaded and GWT broadcast.
    pub fn activate_skill(&mut self, name: &str) -> Result<(), String> {
        let idx = self.skills.iter().position(|s| s.name == name)
            .ok_or_else(|| format!("Skill '{}' not found", name))?;
        if self.skills[idx].active {
            return Err(format!("Skill '{}' is already active", name));
        }
        self.skills[idx].active = true;
        self.record_activation(name);
        let desc = self.skills[idx].description.clone();
        let triggers = self.skills[idx].triggers.clone();
        let e8_modes = self.skills[idx].e8_modes.clone();
        let priority = self.skills[idx].priority;

        if let Some(ref mut hooks) = self.hooks {
            let ctx = HookContext::new(
                HookEvent::SkillLoaded,
                &format!("skill:{}", name),
            ).with_payload(serde_json::json!({
                "name": name,
                "description": desc,
                "triggers": triggers,
                "e8_modes": e8_modes,
                "priority": priority,
            }));
            hooks.trigger(&ctx);
        }

        if let Some(ref gwt) = self.gwt {
            if let Ok(mut gwt) = gwt.try_write() {
                gwt.broadcast(&format!("[skill_activated] {} — {}", name, desc));
            }
        }

        Ok(())
    }

    /// Deactivate a skill by name. Fires HookEvent::SkillUnloaded.
    pub fn deactivate_skill(&mut self, name: &str) -> Result<(), String> {
        let idx = self.skills.iter().position(|s| s.name == name)
            .ok_or_else(|| format!("Skill '{}' not found", name))?;
        if !self.skills[idx].active {
            return Err(format!("Skill '{}' is not active", name));
        }
        self.skills[idx].active = false;

        if let Some(ref mut hooks) = self.hooks {
            let ctx = HookContext::new(
                HookEvent::SkillUnloaded,
                &format!("skill:{}", name),
            );
            hooks.trigger(&ctx);
        }

        Ok(())
    }

    pub fn list_active(&self) -> Vec<&SkillDocEntry> {
        self.skills.iter().filter(|s| s.active).collect()
    }

    /// 披露门控的活跃技能视图 (P4 行为接线): 披露预算 active_tool_count()
    /// 真实限制模型可见工具集 — stage 0 (Minimal) 时仅暴露预算数量的
    /// 高优先级技能, promote 到 Standard 后暴露全部活跃技能。
    /// 这是 active_tool_count() 从"展示"到"行为门控"的生产路径。
    pub fn visible_active(&self) -> Vec<&SkillDocEntry> {
        let mut active: Vec<&SkillDocEntry> = self.skills.iter().filter(|s| s.active).collect();
        let budget = self.disclosure.active_tool_count();
        if self.disclosure.stage == 0 && active.len() > budget {
            // Minimal 阶段: 按 priority 升序 (高优先级在前) 截断到预算
            active.sort_by_key(|s| s.priority);
            active.truncate(budget);
        }
        active
    }

    /// Install a skill from a source path (file or directory with SKILL.md).
    /// Copies the file(s) into the skills directory.
    pub fn install_skill(&mut self, source_path: &Path) -> Result<(), String> {
        if !source_path.exists() {
            return Err(format!("Source path does not exist: {}", source_path.display()));
        }

        if source_path.is_dir() {
            let skill_md = source_path.join("SKILL.md");
            if !skill_md.exists() {
                return Err("Directory must contain a SKILL.md file".to_string());
            }
            let content = std::fs::read_to_string(&skill_md).map_err(|e| e.to_string())?;
            let entry = SkillDocEntry::from_content(&skill_md, &content)
                .ok_or_else(|| "Invalid frontmatter in SKILL.md".to_string())?;

            let target_dir = self.skills_dir.join(&entry.name);
            let _ = std::fs::create_dir_all(&target_dir);

            // Copy SKILL.md
            let dest = target_dir.join("SKILL.md");
            std::fs::copy(&skill_md, &dest).map_err(|e| e.to_string())?;

            // Copy other files from source directory
            if let Ok(entries) = std::fs::read_dir(source_path) {
                for e in entries.flatten() {
                    let src = e.path();
                    if src == skill_md { continue; }
                    let fname = src.file_name().unwrap_or_default();
                    let dst = target_dir.join(fname);
                    if src.is_file() {
                        let _ = std::fs::copy(&src, &dst);
                    } else if src.is_dir() {
                        let dst_sub = target_dir.join(fname);
                        let _ = std::fs::create_dir_all(&dst_sub);
                        if let Ok(sub) = std::fs::read_dir(&src) {
                            for sub_entry in sub.flatten() {
                                let sub_src = sub_entry.path();
                                if sub_src.is_file() {
                                    let _ = std::fs::copy(&sub_src, dst_sub.join(sub_src.file_name().unwrap_or_default()));
                                }
                            }
                        }
                    }
                }
            }

            self.load_all();
            self.register_install_effects(&entry.name, &target_dir)?;
            Ok(())
        } else if source_path.extension().is_some_and(|e| e == "md") {
            let content = std::fs::read_to_string(source_path).map_err(|e| e.to_string())?;
            let entry = SkillDocEntry::from_content(source_path, &content)
                .ok_or_else(|| "Invalid frontmatter in skill file".to_string())?;

            let target_dir = self.skills_dir.join(&entry.name);
            let _ = std::fs::create_dir_all(&target_dir);
            let dest = target_dir.join("SKILL.md");
            std::fs::copy(source_path, &dest).map_err(|e| e.to_string())?;

            self.load_all();
            self.register_install_effects(&entry.name, &target_dir)?;
            Ok(())
        } else {
            Err("Source must be a .md file or a directory containing SKILL.md".to_string())
        }
    }

    /// Build a SkillDocEntry from a ProceduralMemoryRecord (KB-stored E8 trajectory pattern).
    /// Converts the E8 sequence, trigger, reward, and tags into a YAML-frontmatter skill
    /// that can be written to the filesystem and loaded by SkillEngine.
    pub fn skill_from_procedural_record(record: &ProceduralMemoryRecord) -> SkillDocEntry {
        let e8_str = format!("[{}]", record.e8_sequence.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(","));

        let yaml = format!(
            "---\nname: {}\ndescription: {}\ntriggers: [\"e8\", \"proc_skill\", \"{}\"]\ne8_modes: {}\npriority: {}\n---\n\n{}",
            record.name,
            record.description,
            record.skill_id,
            e8_str,
            (record.avg_reward * 100.0) as u8,
            record.description,
        );

        SkillDocEntry {
            name: record.name.clone(),
            description: record.description.clone(),
            triggers: vec!["e8".to_string(), "proc_skill".to_string(), record.skill_id.clone()],
            e8_modes: record.e8_sequence.clone(),
            tools: vec![],
            hooks: vec![],
            priority: (record.avg_reward * 100.0) as u8,
            path: PathBuf::new(),
            content: yaml,
            active: false,
            references: vec![],
            category: "procedural".to_string(),
            parent: String::new(),
            verified: false,
        }
    }

    /// Install a procedural memory record as a YAML-frontmatter skill file in the skills directory.
    /// Creates `~/.neotrix/skills/<skill_name>/SKILL.md` from the record.
    /// Returns the name of the installed skill on success.
    pub fn install_from_procedural(&mut self, record: &ProceduralMemoryRecord) -> Result<String, String> {
        let skill = Self::skill_from_procedural_record(record);
        let target_dir = self.skills_dir.join(&skill.name);
        let _ = std::fs::create_dir_all(&target_dir);
        let dest = target_dir.join("SKILL.md");
        std::fs::write(&dest, &skill.content).map_err(|e| format!("write skill: {}", e))?;
        self.load_all();
        log::info!("[procedural→skill] installed '{}' from E8 pattern ({} states, reward={:.3})",
            skill.name, record.e8_sequence.len(), record.avg_reward);
        Ok(skill.name)
    }

    /// 把 install 的逆操作推入逆账本并派生 skill fiber (cordiverse F1+F5)。
    /// teardown 从加载序派生, 非手写清理 (paper §3.3.3 p.27)。
    fn register_install_effects(&mut self, name: &str, target_dir: &Path) -> Result<(), String> {
        let install_id = self.inverse_ledger.begin_install();
        let inv_target = target_dir.to_path_buf();
        let inv_label = format!("remove installed skill dir: {}", inv_target.display());
        self.inverse_ledger.push_inverse(
            install_id,
            RevertibleEffect::new(inv_label, move || {
                if inv_target.exists() {
                    std::fs::remove_dir_all(&inv_target)
                        .map_err(|e| format!("remove {}: {}", inv_target.display(), e))
                } else {
                    Ok(())
                }
            }),
        )?;
        self.fiber_lifecycles.insert(
            name.to_string(),
            FiberLifecycle::new(name.to_string(), install_id),
        );
        if let Some(fiber) = self.fiber_lifecycles.get_mut(name) {
            let _ = fiber.transition(FiberLifecycleState::Active);
        }
        Ok(())
    }

    /// 卸载技能 (cordiverse F1+F5): 按加载序的 LIFO 逆序执行该 install 的
    /// 全部逆操作, 完成后把 fiber 转入 Retired 终态。逆操作中的失败按 fiber
    /// 捕获, 不中断其余逆操作, 也不影响其他 fiber。
    pub fn uninstall_skill(&mut self, name: &str) -> Result<Vec<Result<(), String>>, String> {
        if self.fiber_lifecycles.get(name).map(|f| f.state) == Some(FiberLifecycleState::Retired) {
            return Err(format!("skill '{}' fiber already retired", name));
        }
        let install_id = self.fiber_lifecycles.get(name)
            .map(|f| f.install_id)
            .ok_or_else(|| format!("no installed fiber for skill '{}'", name))?;
        let results = self.inverse_ledger.teardown(install_id);
        if let Some(fiber) = self.fiber_lifecycles.get_mut(name) {
            let _ = fiber.transition(FiberLifecycleState::Retired);
        }
        if let Some(idx) = self.skills.iter().position(|s| s.name == name) {
            self.skills.remove(idx);
            self.build_index();
        }
        Ok(results)
    }

    /// 从 skill 名查 fiber 当前生命周期状态。
    pub fn fiber_state(&self, name: &str) -> Option<FiberLifecycleState> {
        self.fiber_lifecycles.get(name).map(|f| f.state)
    }

    /// 按 fiber 捕获失败并转入 Failed (不传播到 sibling)。
    pub(crate) fn _record_fiber_failure(&mut self, name: &str, message: impl Into<String>) -> bool {
        if let Some(fiber) = self.fiber_lifecycles.get_mut(name) {
            fiber.record_failure(message);
            true
        } else {
            false
        }
    }

    /// 释放悬挂所有权: fiber 仍标记 held (Loaded/Active/Suspended) 但其 install
    /// 逆账本事务已消失 (holder 失效) → 自动转入 Retired 终态。返回释放列表。
    pub fn release_dangling(&mut self) -> Vec<String> {
        use FiberLifecycleState::*;
        let dangling: Vec<String> = self
            .fiber_lifecycles
            .iter()
            .filter(|(_, f)| matches!(f.state, Loaded | Active | Suspended))
            .filter(|(_, f)| !self.inverse_ledger.has_transaction(f.install_id))
            .map(|(name, _)| name.clone())
            .collect();
        let mut released = Vec::new();
        for name in dangling {
            if let Some(fiber) = self.fiber_lifecycles.get_mut(&name) {
                let _ = fiber.transition(FiberLifecycleState::Retired);
                released.push(name);
            }
        }
        released
    }
}

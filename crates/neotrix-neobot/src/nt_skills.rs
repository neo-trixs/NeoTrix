//! `nt_skills` — 技能下沉（技能 = 指令参考，非能力扩展）。
//!
//! 技能 = `<data_dir>/skills/<name>/SKILL.md` 目录。技能只给模型提供
//! 指令参考（prompt 上下文），**不扩展能力面**：模型能调的依然只有
//! 网关后的 6 个工具。安装只做三件事：目录校验 → 拷贝 → 索引行。
//!
//! `SKILL.md` 头两行约定：`# <name>` 标题行 + 首个非空正文段为描述。
//! 解析失败（无标题/空文件）→ 整包拒绝，不猜名字。

use std::path::{Path, PathBuf};

use crate::nt_error::NtBotError;

/// 一条技能索引.
#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
}

/// 技能根目录.
pub fn skills_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("skills")
}

/// 扫描全部技能（坏包跳过并计数，调用方按需告警；坏包不炸列表）。
pub fn scan_skills(data_dir: &Path) -> (Vec<Skill>, usize) {
    let mut out = Vec::new();
    let mut skipped = 0usize;
    let root = skills_dir(data_dir);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return (out, skipped);
    };
    let mut names: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_dir())
        .collect();
    names.sort();
    for dir in names {
        match read_skill(&dir) {
            Ok(skill) => out.push(skill),
            Err(_) => skipped += 1,
        }
    }
    (out, skipped)
}

/// 读单个技能包.
///
/// 两形状（Claude Skills 互通优先）。一是 frontmatter
/// （`---` 包裹的 `name:` / `description:`，仅取这两 key，无 yaml 依赖）；
/// 二是回落 `# <name>` 标题加首个正文段（旧包兼容）。
/// 名字含 `/`、`\`、`..` 一律拒绝（防目录越狱）。
pub fn read_skill(dir: &Path) -> Result<Skill, NtBotError> {
    let file = dir.join("SKILL.md");
    let text = std::fs::read_to_string(&file).map_err(|err| {
        NtBotError::Invalid(format!("skill '{}' has no SKILL.md: {err}", dir_lossy(dir)))
    })?;
    let (name, description) = match parse_frontmatter(&text) {
        Some((name, description)) => (name, description),
        None => parse_title_body(&text).ok_or_else(|| {
            NtBotError::Invalid(format!(
                "skill '{}' SKILL.md needs frontmatter (name/description) or '# <name>' title",
                dir_lossy(dir)
            ))
        })?,
    };
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(NtBotError::Invalid(format!("bad skill name '{name}'")));
    }
    Ok(Skill {
        name,
        description,
        path: dir.to_owned(),
    })
}

/// frontmatter 解析（无依赖手写：只认顶格 `name:`/`description:`，值去引号截 500）。
fn parse_frontmatter(text: &str) -> Option<(String, String)> {
    let mut lines = text.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;
    for line in lines.by_ref() {
        if line.trim() == "---" {
            break;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        // 缩进项（Claude frontmatter 偶有嵌套）直接跳过，只认顶格。
        if line.starts_with([' ', '\t']) {
            continue;
        }
        let value = value
            .trim()
            .trim_matches(|c| c == '"' || c == '\'')
            .trim()
            .to_owned();
        match key.trim() {
            "name" if !value.is_empty() => name = Some(value),
            "description" if !value.is_empty() => description = Some(value),
            _ => {}
        }
    }
    let name = name?;
    let description = if description.as_deref().unwrap_or("").is_empty() {
        "(no description)".to_owned()
    } else {
        description.unwrap_or_default()
    };
    Some((name, description))
}

/// 旧形状：`# <name>` 首行 + 首个非空正文段。
fn parse_title_body(text: &str) -> Option<(String, String)> {
    let mut lines = text.lines().map(str::trim).filter(|line| !line.is_empty());
    let title = lines.next()?;
    let name = title
        .strip_prefix('#')
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())?;
    let description = lines.next().unwrap_or("(no description)").to_owned();
    Some((name, description))
}

/// 安装：本地目录 → skills 根。拒绝：源无 SKILL.md、同名已存在、
/// 源在 skills 根内部（自己装自己）。
pub fn install_skill(data_dir: &Path, src: &Path) -> Result<Skill, NtBotError> {
    if !src.is_dir() {
        return Err(NtBotError::Invalid(format!(
            "skill source '{}' is not a directory",
            dir_lossy(src)
        )));
    }
    let probe = read_skill(src)?;
    let root = skills_dir(data_dir);
    if src.starts_with(&root) {
        return Err(NtBotError::Invalid(
            "skill source is already inside skills dir".to_owned(),
        ));
    }
    let dest = root.join(&probe.name);
    if dest.exists() {
        return Err(NtBotError::Store(format!(
            "skill '{}' already installed",
            probe.name
        )));
    }
    std::fs::create_dir_all(&root)?;
    copy_dir(src, &dest)?;
    read_skill(&dest)
}

///  prompt 注入行（HTTP 系统提示用；echo/CLI 不用）。
///  skills=指令非能力：只列名字 + 一句话，不贴全文（省上下文）。
pub fn skills_context_line(skills: &[Skill]) -> Option<String> {
    if skills.is_empty() {
        return None;
    }
    let mut names: Vec<String> = skills
        .iter()
        .map(|s| format!("{}（{}）", s.name, s.description))
        .collect();
    names.sort();
    Some(format!("可用技能（仅指令参考，不扩展能力）：{}", names.join("；")))
}

fn dir_lossy(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn copy_dir(src: &Path, dest: &Path) -> Result<(), NtBotError> {
    std::fs::create_dir_all(dest)?;
    let entries = std::fs::read_dir(src)?;
    for entry in entries {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir(&from, &to)?;
        } else if file_type.is_file() {
            std::fs::copy(&from, &to)?;
        }
        // 符号链接/特殊文件跳过（防越狱，不报错）。
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{install_skill, read_skill, scan_skills, skills_context_line};

    fn tmp_root(case: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("neobot-skill-test-{case}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    fn write_pkg(root: &std::path::Path, name: &str, md: &str) -> std::path::PathBuf {
        let dir = root.join(name);
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::write(dir.join("SKILL.md"), md).expect("write");
        dir
    }

    #[test]
    fn scan_lists_good_and_skips_broken() {
        let root = tmp_root("scan");
        write_pkg(&root, "src-a", "# alpha\n\nDoes A.\n");
        write_pkg(&root, "src-bad", "no title here\n");
        let data = root.join("data");
        std::fs::create_dir_all(&data).expect("mkdir");
        // 安装好包 + 手工放一个坏包进 skills 根
        let installed = install_skill(&data, &root.join("src-a")).expect("install");
        assert_eq!(installed.name, "alpha");
        assert!(install_skill(&data, &root.join("src-a")).is_err());
        assert!(install_skill(&data, &root.join("src-bad")).is_err());
        std::fs::create_dir_all(data.join("skills").join("broken")).expect("mkdir");
        let (skills, skipped) = scan_skills(&data);
        assert_eq!(skills.len(), 1);
        assert_eq!(skipped, 1);
        let line = skills_context_line(&skills).expect("line");
        assert!(line.contains("alpha") && line.contains("Does A."));
        assert!(read_skill(&data.join("skills").join("alpha")).is_ok());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn frontmatter_beats_title_and_rejects_jailbreak() {
        let root = tmp_root("fm");
        // Claude 式 frontmatter（含引号 + 缩进噪音行）
        let fm = write_pkg(
            &root,
            "src-fm",
            "---\nname: beta\n  nested: ignore me\ndescription: \"Does B.\"\n---\n\nBody here.\n",
        );
        let skill = read_skill(&fm).expect("frontmatter");
        assert_eq!(skill.name, "beta");
        assert_eq!(skill.description, "Does B.");
        // 旧形状仍兼容
        let legacy = write_pkg(&root, "src-old", "# gamma\n\nDoes G.\n");
        assert_eq!(read_skill(&legacy).expect("legacy").name, "gamma");
        // frontmatter 无 name → 回落标题（此处无标题 → 拒）
        let bad = write_pkg(&root, "src-noname", "---\ndescription: x\n---\n");
        assert!(read_skill(&bad).is_err());
        // 名字越狱拒绝（两种形状都拦）
        let evil = write_pkg(&root, "src-evil", "# ../evil\n\nx\n");
        assert!(read_skill(&evil).is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn empty_dir_scans_empty() {
        let root = tmp_root("empty");
        let (skills, skipped) = scan_skills(&root);
        assert!(skills.is_empty() && skipped == 0);
        assert!(skills_context_line(&skills).is_none());
        let _ = std::fs::remove_dir_all(&root);
    }
}

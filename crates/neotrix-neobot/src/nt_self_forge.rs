//! `nt_self_forge` —从单条 URL forge 一个「可复用的能力插件条目」。
//!
//! 机制：
//! 1. parse URL → (owner, repo) slug；
//! 2. 拉 GitHub 元信息（仅写 meta, 不引外部依赖）；
//! 3. 写 `data_dir/plugins/<name>.json` 作为热插拔 manifest；
//! 4. 写 `data_dir/absorb_payload_<name>.json` 供 `neotrix-experience absorb-node` 二次复用；
//! 5. `init(data_dir)` 后模型/自然语言入口可通过 Plugin(String) 重调该 manifest。
//!
//! ⛔ 最诚实的边界：**URL 只能转化为「已描述、可回忆、可重新拉取」的条目**，
//! 不能替用户决定要不要把代码对上/补丁，也没有强行让远程项目可代入依赖树
//! 的假象；真正的代码级插件化通过目录结构/集成测试逐步逼近。

use std::path::{Path, PathBuf};

use crate::nt_error::NtBotError;
use crate::nt_plugins::{self, PluginManifest};

/// `<owner>/<repo>\n` slug from GitHub URL; for now strict enough for f  te 键入 check.
fn owner_repo_from_url(url: &str) -> Option<(String, String)> {
    let rest = url
        .trim_end_matches('/')
        .strip_prefix("https://github.com/")?;
    let mut parts = rest.split('/');
    let owner = parts.next()?;
    let repo = parts.next()?;
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    // strip optional .git suffix
    let repo = repo.strip_suffix(".git").unwrap_or(repo);
    Some((owner.to_string(), repo.to_string()))
}

pub fn url_slug(url: &str) -> Result<String, NtBotError> {
    let (owner, repo) = owner_repo_from_url(url).ok_or_else(|| {
        NtBotError::Invalid(format!(
            "仅支援 https://github.com/owner/repo[.git], 实际 URL 不abys match: {url}"
        ))
    })?;
    Ok(format!("{owner}-{repo}"))
}

/// 写一个以后每次 invoke `absorb-node` 的 manifest，(), The manifest name is stable.
pub fn forge_from_url(url: &str, data_dir: &Path) -> Result<PluginManifest, NtBotError> {
    let slug = url_slug(url)?;
    let manifest_path = data_dir.join("plugins").join(format!("{slug}.json"));
    let absorb_payload = data_dir.join(format!("absorb_payload_{slug}.json"));

    std::fs::create_dir_all(manifest_path.parent().unwrap_or(&manifest_path))
        .map_err(|e| NtBotError::Invalid(format!("mkdir plugins: {e}")))?;

    let manifest = PluginManifest {
        name: slug.clone(),
        description: Some(format!("Generated plugin from {url}")),
        command: "neotrix-experience".into(),
        args: vec![
            "absorb-node".into(),
            "--apply-capability".into(),
            absorb_payload.display().to_string(),
        ],
    };

    std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?).map_err(|e| {
        NtBotError::Invalid(format!("write manifest {}: {e}", manifest_path.display()))
    })?;

    let absorb_json = serde_json::json!([{
        "node_type": "external-absorption",
        "title": slug,
        "summary": format!("Generated from {url}"),
        "content": url,
        "url": url,
        "domain": "github.com",
        "language": "auto",
        "importance": 3,
        "meta": {"source": url, "generated_by": "nt_self_forge"},
        "capability": {"branch": "NT-ACT", "capability": "self-forge", "evidence": url}
    }]);
    std::fs::write(
        &absorb_payload,
        serde_json::to_vec_pretty(&absorb_json)?,
    )
    .map_err(|e| NtBotError::Invalid(format!("write absorb payload: {e}")))?;

    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_self_github_url() {
        let slug = url_slug("https://github.com/storytold/cadcraft/").unwrap();
        assert_eq!(slug, "storytold-cadcraft");
    }

    #[test]
    fn rejects_non_github_url() {
        assert!(url_slug("https://example.com/foo/bar").is_err());
    }

    #[test]
    fn forge_writes_manifest_and_payload() {
        let tmp = crate::nt_testutil::temp_dir("nt_self_forge");
        std::fs::create_dir_all(&tmp).unwrap();
        let url = "https://github.com/storytold/cadcraft";
        let m = forge_from_url(url, &tmp).expect("forge");
        assert_eq!(m.name, "storytold-cadcraft");
        assert!(tmp.join("plugins/storytold-cadcraft.json").exists());
        assert!(tmp.join("absorb_payload_storytold-cadcraft.json").exists());
    }
}

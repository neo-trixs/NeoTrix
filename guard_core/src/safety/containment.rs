use std::path::Path;

#[derive(Debug, Clone)]
pub struct BoundaryZone { pub allowed_prefixes: Vec<String> }

impl BoundaryZone {
    pub fn new(allowed: Vec<String>) -> Self { BoundaryZone { allowed_prefixes: allowed } }
    pub fn allows(&self, path: &str) -> bool { self.allowed_prefixes.iter().any(|p| path.starts_with(p)) }
    pub fn open_all() -> Self { BoundaryZone { allowed_prefixes: vec!["/".to_string()] } }
}

pub fn check_path_access(path: &str, zone: &BoundaryZone) -> Result<String, String> {
    let canonical = Path::new(path).canonicalize().map_err(|e| format!("cannot resolve: {}", e))?;
    let p = canonical.to_str().ok_or("invalid path".to_string())?;
    if zone.allows(p) { Ok(p.to_string()) } else { Err(format!("access denied: {}", p)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_zone_allows_subpath() {
        let zone = BoundaryZone::new(vec!["/tmp".to_string()]);
        assert!(zone.allows("/tmp/foo"), "subpath under allowed prefix should be allowed");
    }

    #[test]
    fn boundary_zone_denies_other_prefix() {
        let zone = BoundaryZone::new(vec!["/tmp".to_string()]);
        assert!(!zone.allows("/etc/passwd"), "path under different prefix should be denied");
    }

    #[test]
    fn open_all_allows_any_path() {
        let zone = BoundaryZone::open_all();
        assert!(zone.allows("/any/path"), "open_all should allow any path");
        assert!(zone.allows("/etc/passwd"), "open_all should allow even sensitive paths");
    }

    #[test]
    fn boundary_zone_empty_prefixes_denies_all() {
        let zone = BoundaryZone::new(vec![]);
        assert!(!zone.allows("/tmp/foo"), "empty allowed prefixes should deny everything");
    }
}

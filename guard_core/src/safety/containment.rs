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
    #[test] fn test_boundary_zone_allows_subpath() { let z = BoundaryZone::new(vec!["/tmp".to_string()]); assert!(z.allows("/tmp/foo")); }
    #[test] fn test_boundary_zone_denies_other() { let z = BoundaryZone::new(vec!["/tmp".to_string()]); assert!(!z.allows("/etc/passwd")); }
    #[test] fn test_open_all() { let z = BoundaryZone::open_all(); assert!(z.allows("/any/path")); }
}

use std::fs;
use std::path::Path;
use quote::ToTokens;
use syn::visit::{self, Visit};
use walkdir::WalkDir;

/// File discovery configuration
#[derive(Debug, Clone)]
pub struct SourceReaderConfig {
    pub exclude_dirs: Vec<String>,
    pub max_file_size: u64,
    pub max_files: usize,
}

impl Default for SourceReaderConfig {
    fn default() -> Self {
        Self {
            exclude_dirs: vec![
                "target".to_string(),
                ".git".to_string(),
                "node_modules".to_string(),
            ],
            max_file_size: 500_000,
            max_files: 500,
        }
    }
}

/// Represents a parsed Rust source file
#[derive(Debug, Clone)]
pub struct ParsedSourceFile {
    pub path: String,
    pub source: String,
    pub item_count: usize,
    pub fn_count: usize,
    pub struct_count: usize,
    pub enum_count: usize,
    pub impl_count: usize,
    pub mod_count: usize,
    pub imports: Vec<String>,
    pub parse_error: Option<String>,
}

struct FileStats {
    fn_count: usize,
    struct_count: usize,
    enum_count: usize,
    impl_count: usize,
    mod_count: usize,
    imports: Vec<String>,
}

impl<'ast> Visit<'ast> for FileStats {
    fn visit_item_fn(&mut self, _node: &'ast syn::ItemFn) {
        self.fn_count += 1;
        visit::visit_item_fn(self, _node);
    }

    fn visit_item_struct(&mut self, _node: &'ast syn::ItemStruct) {
        self.struct_count += 1;
        visit::visit_item_struct(self, _node);
    }

    fn visit_item_enum(&mut self, _node: &'ast syn::ItemEnum) {
        self.enum_count += 1;
        visit::visit_item_enum(self, _node);
    }

    fn visit_item_impl(&mut self, _node: &'ast syn::ItemImpl) {
        self.impl_count += 1;
        visit::visit_item_impl(self, _node);
    }

    fn visit_item_mod(&mut self, _node: &'ast syn::ItemMod) {
        self.mod_count += 1;
        visit::visit_item_mod(self, _node);
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        self.imports.push(node.into_token_stream().to_string());
        visit::visit_item_use(self, node);
    }
}

#[derive(Debug)]
pub struct SelfSourceReader {
    pub config: SourceReaderConfig,
    pub files: Vec<ParsedSourceFile>,
    pub workspace_root: Option<String>,
    pub total_scanned: u64,
    pub total_parsed: u64,
    pub total_errors: u64,
}

impl SelfSourceReader {
    pub fn new(config: SourceReaderConfig) -> Self {
        Self {
            config,
            files: Vec::new(),
            workspace_root: None,
            total_scanned: 0,
            total_parsed: 0,
            total_errors: 0,
        }
    }

    pub fn set_workspace_root(&mut self, root: &str) {
        if Path::new(root).exists() {
            self.workspace_root = Some(root.to_string());
        }
    }

    pub fn discover_files(&self) -> Vec<String> {
        let root = match &self.workspace_root {
            Some(r) => r.clone(),
            None => return Vec::new(),
        };

        WalkDir::new(&root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "rs")
                    .unwrap_or(false)
            })
            .filter(|e| {
                let path_str = e.path().to_string_lossy();
                !self
                    .config
                    .exclude_dirs
                    .iter()
                    .any(|ex| path_str.contains(ex.as_str()))
            })
            .filter(|e| {
                e.metadata()
                    .map(|m| m.len() <= self.config.max_file_size)
                    .unwrap_or(false)
            })
            .take(self.config.max_files)
            .map(|e| e.path().to_string_lossy().to_string())
            .collect()
    }

    pub fn scan(&mut self) -> (usize, usize, usize) {
        let paths = self.discover_files();
        let scanned = paths.len();
        self.total_scanned += scanned as u64;
        self.files.clear();

        let mut local_parsed = 0u64;
        let mut local_errors = 0u64;

        for path in &paths {
            let source = match fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    self.files.push(ParsedSourceFile {
                        path: path.clone(),
                        source: String::new(),
                        item_count: 0,
                        fn_count: 0,
                        struct_count: 0,
                        enum_count: 0,
                        impl_count: 0,
                        mod_count: 0,
                        imports: Vec::new(),
                        parse_error: Some(format!("io error: {}", e)),
                    });
                    local_errors += 1;
                    continue;
                }
            };

            match syn::parse_file(&source) {
                Ok(syntax_file) => {
                    let mut stats = FileStats {
                        fn_count: 0,
                        struct_count: 0,
                        enum_count: 0,
                        impl_count: 0,
                        mod_count: 0,
                        imports: Vec::new(),
                    };
                    visit::visit_file(&mut stats, &syntax_file);
                    let item_count = stats.fn_count
                        + stats.struct_count
                        + stats.enum_count
                        + stats.impl_count
                        + stats.mod_count;

                    self.files.push(ParsedSourceFile {
                        path: path.clone(),
                        source,
                        item_count,
                        fn_count: stats.fn_count,
                        struct_count: stats.struct_count,
                        enum_count: stats.enum_count,
                        impl_count: stats.impl_count,
                        mod_count: stats.mod_count,
                        imports: stats.imports,
                        parse_error: None,
                    });
                    local_parsed += 1;
                }
                Err(e) => {
                    self.files.push(ParsedSourceFile {
                        path: path.clone(),
                        source,
                        item_count: 0,
                        fn_count: 0,
                        struct_count: 0,
                        enum_count: 0,
                        impl_count: 0,
                        mod_count: 0,
                        imports: Vec::new(),
                        parse_error: Some(e.to_string()),
                    });
                    local_errors += 1;
                }
            }
        }

        self.total_parsed += local_parsed;
        self.total_errors += local_errors;

        (scanned, local_parsed as usize, local_errors as usize)
    }

    pub fn get_file(&self, path: &str) -> Option<&ParsedSourceFile> {
        self.files.iter().find(|f| f.path == path)
    }

    pub fn get_by_mod_name(&self, mod_name: &str) -> Vec<&ParsedSourceFile> {
        self.files
            .iter()
            .filter(|f| {
                let p = f.path.replace('\\', "/");
                p.contains(mod_name)
            })
            .collect()
    }

    pub fn total_files(&self) -> usize {
        self.files.len()
    }

    pub fn stats(&self) -> SourceReaderStats {
        SourceReaderStats {
            total_files: self.files.len(),
            total_scanned: self.total_scanned,
            total_parsed: self.total_parsed,
            total_errors: self.total_errors,
            total_fns: self.files.iter().map(|f| f.fn_count).sum(),
            total_structs: self.files.iter().map(|f| f.struct_count).sum(),
            total_enums: self.files.iter().map(|f| f.enum_count).sum(),
            total_impls: self.files.iter().map(|f| f.impl_count).sum(),
        }
    }

    pub fn summary(&self) -> String {
        let s = self.stats();
        format!(
            "reader: files={} scanned={} parsed={} errors={} | excludes={:?}",
            s.total_files,
            s.total_scanned,
            s.total_parsed,
            s.total_errors,
            self.config.exclude_dirs,
        )
    }
}

#[derive(Debug, Clone)]
pub struct SourceReaderStats {
    pub total_files: usize,
    pub total_scanned: u64,
    pub total_parsed: u64,
    pub total_errors: u64,
    pub total_fns: usize,
    pub total_structs: usize,
    pub total_enums: usize,
    pub total_impls: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_reader() -> SelfSourceReader {
        SelfSourceReader::new(SourceReaderConfig::default())
    }

    #[test]
    fn test_new_config_defaults() {
        let cfg = SourceReaderConfig::default();
        assert_eq!(cfg.exclude_dirs, vec!["target", ".git", "node_modules"]);
        assert_eq!(cfg.max_file_size, 500_000);
        assert_eq!(cfg.max_files, 500);
    }

    #[test]
    fn test_discover_files_empty_root() {
        let reader = make_reader();
        let files = reader.discover_files();
        assert!(files.is_empty(), "no root set should return empty");
    }

    #[test]
    fn test_discover_files_excludes_target() {
        let mut reader = make_reader();
        let workspace = if Path::new("/Users/neo/Downloads/neotrix").exists() {
            "/Users/neo/Downloads/neotrix"
        } else {
            "."
        };
        reader.set_workspace_root(workspace);
        let files = reader.discover_files();
        assert!(!files.is_empty(), "should discover files in workspace");
        for f in &files {
            assert!(
                !f.contains("target"),
                "path should not contain 'target': {}",
                f
            );
        }
    }

    #[test]
    fn test_scan_counts_increment() {
        let dir = std::env::temp_dir().join("ssr_test_scan_counts");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("test.rs"), "pub fn hello() -> u32 { 42 }").unwrap();

        let mut reader = make_reader();
        reader.set_workspace_root(dir.to_str().unwrap());
        let (scanned, parsed, errors) = reader.scan();

        assert_eq!(scanned, 1, "should scan 1 file");
        assert_eq!(parsed, 1, "should parse 1 file");
        assert_eq!(errors, 0, "should have 0 errors");
        assert_eq!(reader.total_scanned, 1);
        assert_eq!(reader.total_parsed, 1);
        assert_eq!(reader.total_errors, 0);
        assert_eq!(reader.files.len(), 1);

        // Additional scan should increment cumulative counters
        let (scanned2, _parsed2, _errors2) = reader.scan();
        assert_eq!(scanned2, 1, "second scan should find same file");
        assert_eq!(reader.total_scanned, 2, "cumulative scanned should increase");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_scan_file_with_struct() {
        let dir = std::env::temp_dir().join("ssr_test_struct");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("lib.rs"),
            "pub fn f1() -> u32 { 42 }\npub struct MyStruct { pub x: u32 }\npub enum MyEnum { A, B }",
        )
        .unwrap();

        let mut reader = make_reader();
        reader.set_workspace_root(dir.to_str().unwrap());
        let (_scanned, parsed, errors) = reader.scan();

        assert_eq!(parsed, 1);
        assert_eq!(errors, 0);
        assert_eq!(reader.files.len(), 1);

        let f = &reader.files[0];
        assert_eq!(f.fn_count, 1);
        assert_eq!(f.struct_count, 1);
        assert_eq!(f.enum_count, 1);
        assert!(f.parse_error.is_none());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_get_file_missing() {
        let reader = make_reader();
        assert!(reader.get_file("nonexistent.rs").is_none());
    }

    #[test]
    fn test_get_by_mod_name() {
        let mut reader = make_reader();
        reader.files.push(ParsedSourceFile {
            path: "/project/src/foo.rs".to_string(),
            source: String::new(),
            item_count: 0,
            fn_count: 0,
            struct_count: 0,
            enum_count: 0,
            impl_count: 0,
            mod_count: 0,
            imports: Vec::new(),
            parse_error: None,
        });
        reader.files.push(ParsedSourceFile {
            path: "/project/src/bar.rs".to_string(),
            source: String::new(),
            item_count: 0,
            fn_count: 0,
            struct_count: 0,
            enum_count: 0,
            impl_count: 0,
            mod_count: 0,
            imports: Vec::new(),
            parse_error: None,
        });

        let foos = reader.get_by_mod_name("foo");
        assert_eq!(foos.len(), 1);
        assert!(foos[0].path.contains("foo"));
    }

    #[test]
    fn test_stats_tracking() {
        let dir = std::env::temp_dir().join("ssr_test_stats");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("lib.rs"),
            "pub fn f1() {}\npub fn f2() {}\npub struct S;\npub enum E { X }",
        )
        .unwrap();

        let mut reader = make_reader();
        reader.set_workspace_root(dir.to_str().unwrap());
        let (_scanned, _parsed, _errors) = reader.scan();

        let stats = reader.stats();
        assert_eq!(stats.total_files, 1);
        assert!(stats.total_scanned > 0);
        assert!(stats.total_parsed > 0);
        assert_eq!(stats.total_fns, 2);
        assert_eq!(stats.total_structs, 1);
        assert_eq!(stats.total_enums, 1);
        assert_eq!(stats.total_impls, 0);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_summary_format() {
        let reader = make_reader();
        let s = reader.summary();
        assert!(
            s.starts_with("reader:"),
            "summary should start with 'reader:', got: {}",
            s
        );
        assert!(s.contains("excludes="));
    }
}

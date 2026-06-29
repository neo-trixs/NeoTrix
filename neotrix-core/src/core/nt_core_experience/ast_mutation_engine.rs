use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::Path;


use quote::ToTokens;
use syn::{ItemConst, ItemFn, ItemImpl, ItemStruct, ItemUse};

/// Types of mutations that can be applied to Rust source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MutationType {
    AddFunction,
    AddMethod,
    AddStruct,
    AddEnum,
    AddImpl,
    AddField,
    ModifyFunction,
    AddImport,
    AddConst,
    AddTypeAlias,
}

impl MutationType {
    pub fn name(&self) -> &'static str {
        match self {
            MutationType::AddFunction => "add_function",
            MutationType::AddMethod => "add_method",
            MutationType::AddStruct => "add_struct",
            MutationType::AddEnum => "add_enum",
            MutationType::AddImpl => "add_impl",
            MutationType::AddField => "add_field",
            MutationType::ModifyFunction => "modify_function",
            MutationType::AddImport => "add_import",
            MutationType::AddConst => "add_const",
            MutationType::AddTypeAlias => "add_type_alias",
        }
    }

    pub fn risk_level(&self) -> u8 {
        match self {
            MutationType::AddFunction => 3,
            MutationType::AddMethod => 3,
            MutationType::AddStruct => 4,
            MutationType::AddEnum => 4,
            MutationType::AddImpl => 4,
            MutationType::AddField => 5,
            MutationType::ModifyFunction => 7,
            MutationType::AddImport => 1,
            MutationType::AddConst => 2,
            MutationType::AddTypeAlias => 2,
        }
    }
}

/// A planned mutation operation
#[derive(Debug, Clone)]
pub struct AstMutation {
    pub mutation_type: MutationType,
    pub target_file: String,
    pub name: String,
    pub code_snippet: String,
    pub insert_before_item: Option<String>,
    pub description: String,
}

/// Result of applying a mutation
#[derive(Debug, Clone)]
pub struct MutationResult {
    pub success: bool,
    pub file_path: String,
    pub original_source: String,
    pub mutated_source: String,
    pub mutation_type: MutationType,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AstMutationConfig {
    pub max_mutations_per_file: usize,
    pub format_code: bool,
    pub backup_before_mutate: bool,
}

impl Default for AstMutationConfig {
    fn default() -> Self {
        Self {
            max_mutations_per_file: 3,
            format_code: false,
            backup_before_mutate: true,
        }
    }
}

#[derive(Debug)]
pub struct AstMutationEngine {
    pub config: AstMutationConfig,
    pub mutations_applied: u64,
    pub mutations_failed: u64,
    pub history: VecDeque<MutationResult>,
    pub workspace_root: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AstMutationStats {
    pub total_applied: u64,
    pub total_failed: u64,
    pub history_size: usize,
    pub by_type: HashMap<String, usize>,
}

impl AstMutationEngine {
    pub fn new(config: AstMutationConfig) -> Self {
        Self {
            config,
            mutations_applied: 0,
            mutations_failed: 0,
            history: VecDeque::with_capacity(50),
            workspace_root: None,
        }
    }

    pub fn set_workspace_root(&mut self, root: &str) {
        self.workspace_root = Some(root.to_string());
    }

    /// Apply a single mutation to the target file
    pub fn apply_mutation(&mut self, mutation: &AstMutation) -> MutationResult {
        let root = self.workspace_root.as_deref().unwrap_or("");
        let full_path = Path::new(root).join(&mutation.target_file);
        let file_path_str = full_path.to_string_lossy().to_string();

        let source = match fs::read_to_string(&full_path) {
            Ok(s) => s,
            Err(e) => {
                let result = MutationResult {
                    success: false,
                    file_path: file_path_str,
                    original_source: String::new(),
                    mutated_source: String::new(),
                    mutation_type: mutation.mutation_type,
                    error: Some(format!("failed to read file: {}", e)),
                };
                self.mutations_failed += 1;
                self.history.push_back(result.clone());
                if self.history.len() > 50 {
                    self.history.pop_front();
                }
                return result;
            }
        };

        let mut file: syn::File = match syn::parse_str(&source) {
            Ok(f) => f,
            Err(e) => {
                let result = MutationResult {
                    success: false,
                    file_path: file_path_str,
                    original_source: source.clone(),
                    mutated_source: source,
                    mutation_type: mutation.mutation_type,
                    error: Some(format!("failed to parse file: {}", e)),
                };
                self.mutations_failed += 1;
                self.history.push_back(result.clone());
                if self.history.len() > 50 {
                    self.history.pop_front();
                }
                return result;
            }
        };

        let apply_result = match mutation.mutation_type {
            MutationType::AddFunction => {
                match syn::parse_str::<ItemFn>(&mutation.code_snippet) {
                    Ok(item_fn) => {
                        if let Some(ref before) = mutation.insert_before_item {
                            insert_item_before_named(&mut file.items, before, syn::Item::Fn(item_fn))
                        } else {
                            file.items.push(syn::Item::Fn(item_fn));
                            Ok(())
                        }
                    }
                    Err(e) => Err(format!("failed to parse function: {}", e)),
                }
            }
            MutationType::AddStruct => {
                match syn::parse_str::<ItemStruct>(&mutation.code_snippet) {
                    Ok(item) => {
                        if let Some(ref before) = mutation.insert_before_item {
                            let _ = insert_item_before_named(&mut file.items, before, syn::Item::Struct(item));
                        } else {
                            file.items.push(syn::Item::Struct(item));
                        }
                        Ok(())
                    }
                    Err(e) => Err(format!("failed to parse struct: {}", e)),
                }
            }
            MutationType::AddImpl => {
                match syn::parse_str::<ItemImpl>(&mutation.code_snippet) {
                    Ok(item) => {
                        file.items.push(syn::Item::Impl(item));
                        Ok(())
                    }
                    Err(e) => Err(format!("failed to parse impl: {}", e)),
                }
            }
            MutationType::AddImport => {
                match syn::parse_str::<ItemUse>(&mutation.code_snippet) {
                    Ok(item) => {
                        insert_use_after_existing(&mut file.items, syn::Item::Use(item));
                        Ok(())
                    }
                    Err(e) => Err(format!("failed to parse import: {}", e)),
                }
            }
            MutationType::AddConst => {
                match syn::parse_str::<ItemConst>(&mutation.code_snippet) {
                    Ok(item) => {
                        file.items.push(syn::Item::Const(item));
                        Ok(())
                    }
                    Err(e) => Err(format!("failed to parse const: {}", e)),
                }
            }
            MutationType::ModifyFunction => {
                match syn::parse_str::<ItemFn>(&mutation.code_snippet) {
                    Ok(new_fn) => replace_function_by_name(&mut file.items, &mutation.name, new_fn),
                    Err(e) => Err(format!("failed to parse function: {}", e)),
                }
            }
            MutationType::AddMethod | MutationType::AddField | MutationType::AddEnum | MutationType::AddTypeAlias => {
                // For these types, just append the snippet as a raw item
                match syn::parse_file(&format!("{}\n{}", source, mutation.code_snippet)) {
                    Ok(merged) => {
                        // Take only the new items from the merged file
                        let new_count = merged.items.len();
                        let old_count = file.items.len();
                        if new_count > old_count {
                            for item in merged.items.into_iter().skip(old_count) {
                                file.items.push(item);
                            }
                        }
                        Ok(())
                    }
                    Err(e) => Err(format!("failed to merge code: {}", e)),
                }
            }
        };

        match apply_result {
            Ok(()) => {
                let mutated_source = file.to_token_stream().to_string();

                if self.config.backup_before_mutate {
                    let backup_path = full_path.with_extension("rs.bak");
                    let _ = fs::write(&backup_path, &source);
                }

                match fs::write(&full_path, &mutated_source) {
                    Ok(()) => {
                        let result = MutationResult {
                            success: true,
                            file_path: file_path_str,
                            original_source: source,
                            mutated_source,
                            mutation_type: mutation.mutation_type,
                            error: None,
                        };
                        self.mutations_applied += 1;
                        self.history.push_back(result.clone());
                        if self.history.len() > 50 {
                            self.history.pop_front();
                        }
                        result
                    }
                    Err(e) => {
                        let result = MutationResult {
                            success: false,
                            file_path: file_path_str,
                            original_source: source,
                            mutated_source: String::new(),
                            mutation_type: mutation.mutation_type,
                            error: Some(format!("failed to write file: {}", e)),
                        };
                        self.mutations_failed += 1;
                        self.history.push_back(result.clone());
                        if self.history.len() > 50 {
                            self.history.pop_front();
                        }
                        result
                    }
                }
            }
            Err(e) => {
                let result = MutationResult {
                    success: false,
                    file_path: file_path_str,
                    original_source: source.clone(),
                    mutated_source: source,
                    mutation_type: mutation.mutation_type,
                    error: Some(e),
                };
                self.mutations_failed += 1;
                self.history.push_back(result.clone());
                if self.history.len() > 50 {
                    self.history.pop_front();
                }
                result
            }
        }
    }

    /// Propose adding a new function to a file
    pub fn propose_add_function(
        file_path: &str,
        fn_name: &str,
        body: &str,
        return_type: &str,
        params: &[(String, String)],
    ) -> AstMutation {
        let param_tokens: Vec<proc_macro2::TokenStream> = params
            .iter()
            .map(|(n, t)| {
                let name = syn::Ident::new(n, proc_macro2::Span::call_site());
                let ty: syn::Type = syn::parse_str(t).unwrap();
                quote::quote! { #name: #ty }
            })
            .collect();
        let fn_ident = syn::Ident::new(fn_name, proc_macro2::Span::call_site());
        let body_block: syn::Block = syn::parse_str(body).unwrap();
        let ret_ty: syn::Type = syn::parse_str(return_type).unwrap();
        let snippet = quote::quote! {
            pub fn #fn_ident(#(#param_tokens),*) -> #ret_ty #body_block
        }
        .to_string();

        AstMutation {
            mutation_type: MutationType::AddFunction,
            target_file: file_path.to_string(),
            name: fn_name.to_string(),
            code_snippet: snippet,
            insert_before_item: None,
            description: format!("add function `{}` with return type `{}`", fn_name, return_type),
        }
    }

    /// Propose adding a method to an impl block
    pub fn propose_add_method(
        file_path: &str,
        impl_type: &str,
        fn_name: &str,
        body: &str,
        return_type: &str,
        params: &[(String, String)],
    ) -> AstMutation {
        let param_tokens: Vec<proc_macro2::TokenStream> = params
            .iter()
            .map(|(n, t)| {
                let name = syn::Ident::new(n, proc_macro2::Span::call_site());
                let ty: syn::Type = syn::parse_str(t).unwrap();
                quote::quote! { #name: #ty }
            })
            .collect();
        let fn_ident = syn::Ident::new(fn_name, proc_macro2::Span::call_site());
        let body_block: syn::Block = syn::parse_str(body).unwrap();
        let ret_ty: syn::Type = syn::parse_str(return_type).unwrap();
        let snippet = quote::quote! {
            fn #fn_ident(#(#param_tokens),*) -> #ret_ty #body_block
        }
        .to_string();

        AstMutation {
            mutation_type: MutationType::AddMethod,
            target_file: file_path.to_string(),
            name: fn_name.to_string(),
            code_snippet: snippet,
            insert_before_item: None,
            description: format!(
                "add method `{}` to impl `{}` with return type `{}`",
                fn_name, impl_type, return_type
            ),
        }
    }

    /// Propose adding an import statement
    pub fn propose_add_import(file_path: &str, import_path: &str) -> AstMutation {
        let snippet = format!("use {};", import_path);
        AstMutation {
            mutation_type: MutationType::AddImport,
            target_file: file_path.to_string(),
            name: import_path.to_string(),
            code_snippet: snippet,
            insert_before_item: None,
            description: format!("add import `use {};`", import_path),
        }
    }

    /// Propose adding a struct with named fields
    pub fn propose_add_struct(
        file_path: &str,
        struct_name: &str,
        fields: &[(&str, &str)],
    ) -> AstMutation {
        let struct_ident = syn::Ident::new(struct_name, proc_macro2::Span::call_site());
        let field_tokens: Vec<proc_macro2::TokenStream> = fields
            .iter()
            .map(|(name, ty_str)| {
                let field_name = syn::Ident::new(name, proc_macro2::Span::call_site());
                let field_ty: syn::Type = syn::parse_str(ty_str).unwrap();
                quote::quote! {
                    pub #field_name: #field_ty
                }
            })
            .collect();

        let snippet = quote::quote! {
            pub struct #struct_ident {
                #(#field_tokens),*
            }
        }
        .to_string();

        AstMutation {
            mutation_type: MutationType::AddStruct,
            target_file: file_path.to_string(),
            name: struct_name.to_string(),
            code_snippet: snippet,
            insert_before_item: None,
            description: format!("add struct `{}` with {} fields", struct_name, fields.len()),
        }
    }

    /// Number of history entries
    pub fn history_size(&self) -> usize {
        self.history.len()
    }

    /// Get the last N results
    pub fn last_results(&self, n: usize) -> Vec<&MutationResult> {
        let n = n.min(self.history.len());
        self.history.iter().rev().take(n).collect()
    }

    /// Aggregated statistics
    pub fn stats(&self) -> AstMutationStats {
        let mut by_type: HashMap<String, usize> = HashMap::new();
        for r in &self.history {
            *by_type.entry(r.mutation_type.name().to_string()).or_insert(0) += 1;
        }
        AstMutationStats {
            total_applied: self.mutations_applied,
            total_failed: self.mutations_failed,
            history_size: self.history.len(),
            by_type,
        }
    }

    /// One-line summary
    pub fn summary(&self) -> String {
        format!(
            "ast_mut: applied={} failed={} history={} | cfg(max_per_file={})",
            self.mutations_applied,
            self.mutations_failed,
            self.history.len(),
            self.config.max_mutations_per_file,
        )
    }
}

/// Insert a syn::Item before the first item whose name matches `before_name`.
fn insert_item_before_named(
    items: &mut Vec<syn::Item>,
    before_name: &str,
    new_item: syn::Item,
) -> Result<(), String> {
    let pos = items.iter().position(|item| item_name(item).as_deref() == Some(before_name));
    match pos {
        Some(idx) => {
            items.insert(idx, new_item);
            Ok(())
        }
        None => Err(format!(
            "insert_before_item '{}' not found, appending instead",
            before_name
        )),
    }
}

/// Insert a `use` item after the last existing `use` item (or at the beginning if none exist).
fn insert_use_after_existing(items: &mut Vec<syn::Item>, new_item: syn::Item) {
    let last_use = items.iter().rposition(|item| matches!(item, syn::Item::Use(_)));
    match last_use {
        Some(idx) => items.insert(idx + 1, new_item),
        None => items.insert(0, new_item),
    }
}

/// Replace a function by name in the items list.
fn replace_function_by_name(
    items: &mut Vec<syn::Item>,
    name: &str,
    new_fn: ItemFn,
) -> Result<(), String> {
    let pos = items.iter().position(|item| match item {
        syn::Item::Fn(f) => f.sig.ident == name,
        _ => false,
    });
    match pos {
        Some(idx) => {
            items[idx] = syn::Item::Fn(new_fn);
            Ok(())
        }
        None => Err(format!("function '{}' not found in file", name)),
    }
}

/// Extract the name of an item, if it has one.
fn item_name(item: &syn::Item) -> Option<String> {
    match item {
        syn::Item::Fn(f) => Some(f.sig.ident.to_string()),
        syn::Item::Struct(s) => Some(s.ident.to_string()),
        syn::Item::Enum(e) => Some(e.ident.to_string()),
        syn::Item::Trait(t) => Some(t.ident.to_string()),
        syn::Item::Impl(_) => Some("impl".to_string()),
        syn::Item::Const(c) => Some(c.ident.to_string()),
        syn::Item::Static(s) => Some(s.ident.to_string()),
        syn::Item::Mod(m) => Some(m.ident.to_string()),
        syn::Item::Type(t) => Some(t.ident.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutation_type_name() {
        assert_eq!(MutationType::AddFunction.name(), "add_function");
        assert_eq!(MutationType::AddMethod.name(), "add_method");
        assert_eq!(MutationType::AddStruct.name(), "add_struct");
        assert_eq!(MutationType::AddEnum.name(), "add_enum");
        assert_eq!(MutationType::AddImpl.name(), "add_impl");
        assert_eq!(MutationType::AddField.name(), "add_field");
        assert_eq!(MutationType::ModifyFunction.name(), "modify_function");
        assert_eq!(MutationType::AddImport.name(), "add_import");
        assert_eq!(MutationType::AddConst.name(), "add_const");
        assert_eq!(MutationType::AddTypeAlias.name(), "add_type_alias");
    }

    #[test]
    fn test_mutation_type_risk() {
        assert!(MutationType::ModifyFunction.risk_level() > MutationType::AddFunction.risk_level());
        assert!(MutationType::AddImport.risk_level() < MutationType::AddField.risk_level());
        assert_eq!(MutationType::AddConst.risk_level(), 2);
        assert_eq!(MutationType::ModifyFunction.risk_level(), 7);
    }

    #[test]
    fn test_config_defaults() {
        let cfg = AstMutationConfig::default();
        assert_eq!(cfg.max_mutations_per_file, 3);
        assert!(!cfg.format_code);
        assert!(cfg.backup_before_mutate);
    }

    #[test]
    fn test_new_config() {
        let cfg = AstMutationConfig {
            max_mutations_per_file: 10,
            format_code: true,
            backup_before_mutate: false,
        };
        let engine = AstMutationEngine::new(cfg);
        assert_eq!(engine.config.max_mutations_per_file, 10);
        assert!(engine.config.format_code);
        assert!(!engine.config.backup_before_mutate);
        assert_eq!(engine.mutations_applied, 0);
        assert_eq!(engine.mutations_failed, 0);
    }

    #[test]
    fn test_apply_mutation_file_not_found() {
        let engine = AstMutationEngine::new(AstMutationConfig::default());
        let mut engine = engine;
        let mutation = AstMutation {
            mutation_type: MutationType::AddFunction,
            target_file: "/nonexistent/path/file.rs".to_string(),
            name: "test_fn".to_string(),
            code_snippet: String::new(),
            insert_before_item: None,
            description: String::new(),
        };
        let result = engine.apply_mutation(&mutation);
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.as_ref().unwrap().contains("failed to read file"));
    }

    #[test]
    fn test_propose_add_function_syntax() {
        let mutation = AstMutationEngine::propose_add_function(
            "src/lib.rs",
            "hello",
            "{ \"world\" }",
            "&'static str",
            &[],
        );
        assert_eq!(mutation.mutation_type, MutationType::AddFunction);
        assert_eq!(mutation.name, "hello");
        assert!(mutation.code_snippet.contains("pub fn hello"));
        assert!(mutation.code_snippet.contains("&'static str"));
        // Verify it parses as valid syn
        let parsed = syn::parse_str::<ItemFn>(&mutation.code_snippet);
        assert!(parsed.is_ok(), "snippet must parse as ItemFn");
    }

    #[test]
    fn test_propose_add_method_syntax() {
        let mutation = AstMutationEngine::propose_add_method(
            "src/lib.rs",
            "MyStruct",
            "get_value",
            "{ 42 }",
            "i32",
            &[("self".to_string(), "&self".to_string())],
        );
        assert_eq!(mutation.mutation_type, MutationType::AddMethod);
        assert!(mutation.code_snippet.contains("fn get_value"));
        assert!(mutation.code_snippet.contains("&self"));
        assert!(mutation.code_snippet.contains("i32"));
        let parsed = syn::parse_str::<ItemFn>(&mutation.code_snippet);
        assert!(parsed.is_ok(), "method snippet must parse as ItemFn");
    }

    #[test]
    fn test_propose_add_import_syntax() {
        let mutation = AstMutationEngine::propose_add_import("src/lib.rs", "std::collections::HashMap");
        assert_eq!(mutation.mutation_type, MutationType::AddImport);
        assert_eq!(mutation.code_snippet, "use std::collections::HashMap;");
        assert!(syn::parse_str::<ItemUse>(&mutation.code_snippet).is_ok());
    }

    #[test]
    fn test_propose_add_struct_syntax() {
        let mutation = AstMutationEngine::propose_add_struct(
            "src/lib.rs",
            "MyConfig",
            &[("name", "String"), ("count", "u64")],
        );
        assert_eq!(mutation.mutation_type, MutationType::AddStruct);
        assert!(mutation.code_snippet.contains("pub struct MyConfig"));
        assert!(mutation.code_snippet.contains("pub name: String"));
        assert!(mutation.code_snippet.contains("pub count: u64"));
        let parsed = syn::parse_str::<ItemStruct>(&mutation.code_snippet);
        assert!(parsed.is_ok(), "struct snippet must parse as ItemStruct");
    }

    #[test]
    fn test_history_bounded() {
        let mut engine = AstMutationEngine::new(AstMutationConfig::default());
        // Push 55 failed mutations (all file-not-found)
        for i in 0..55 {
            let mutation = AstMutation {
                mutation_type: MutationType::AddFunction,
                target_file: format!("/nonexistent/file_{}.rs", i),
                name: "f".to_string(),
                code_snippet: String::new(),
                insert_before_item: None,
                description: String::new(),
            };
            engine.apply_mutation(&mutation);
        }
        assert!(engine.history.len() <= 50, "history should be bounded at 50");
        assert_eq!(engine.history_size(), 50);
    }

    #[test]
    fn test_stats_counts() {
        let mut engine = AstMutationEngine::new(AstMutationConfig::default());
        // 3 failed
        for _ in 0..3 {
            let mutation = AstMutation {
                mutation_type: MutationType::AddImport,
                target_file: "/nonexistent/stats_test.rs".to_string(),
                name: "test".to_string(),
                code_snippet: "use std::collections::HashMap;".to_string(),
                insert_before_item: None,
                description: String::new(),
            };
            engine.apply_mutation(&mutation);
        }
        let stats = engine.stats();
        assert_eq!(stats.total_applied, 0);
        assert_eq!(stats.total_failed, 3);
        assert!(stats.history_size > 0);
        assert!(stats.by_type.contains_key("add_import"));
    }

    #[test]
    fn test_summary_format() {
        let engine = AstMutationEngine::new(AstMutationConfig::default());
        let s = engine.summary();
        assert!(s.starts_with("ast_mut:"), "summary must start with 'ast_mut:'");
        assert!(s.contains("applied="));
        assert!(s.contains("failed="));
        assert!(s.contains("history="));
    }

    #[test]
    fn test_apply_to_real_file() {
        let tmp = std::env::temp_dir().join("ast_mutation_test.rs");
        let initial = "pub fn existing() -> u32 { 0 }\n";
        std::fs::write(&tmp, initial).unwrap();

        let parent = tmp.parent().unwrap().to_string_lossy().to_string();
        let filename = tmp.file_name().unwrap().to_string_lossy().to_string();

        let mut engine = AstMutationEngine::new(AstMutationConfig {
            backup_before_mutate: false,
            ..Default::default()
        });
        engine.set_workspace_root(&parent);

        let mutation = AstMutationEngine::propose_add_function(
            &filename,
            "new_func",
            "{ 1 + 2 }",
            "i32",
            &[],
        );

        let result = engine.apply_mutation(&mutation);
        assert!(result.success, "mutation should succeed: {:?}", result.error);

        let modified = std::fs::read_to_string(&tmp).unwrap();
        assert!(
            modified.contains("pub fn new_func"),
            "modified file should contain new function"
        );

        // Verify the existing function is still there
        assert!(modified.contains("pub fn existing"));

        std::fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn test_apply_modify_function() {
        let tmp = std::env::temp_dir().join("ast_mutation_modify.rs");
        let initial = "pub fn target() -> u32 { 0 }\n";
        std::fs::write(&tmp, initial).unwrap();

        let parent = tmp.parent().unwrap().to_string_lossy().to_string();
        let filename = tmp.file_name().unwrap().to_string_lossy().to_string();

        let mut engine = AstMutationEngine::new(AstMutationConfig {
            backup_before_mutate: false,
            ..Default::default()
        });
        engine.set_workspace_root(&parent);

        let snippet = "pub fn target() -> u32 { 42 }".to_string();
        let mutation = AstMutation {
            mutation_type: MutationType::ModifyFunction,
            target_file: filename,
            name: "target".to_string(),
            code_snippet: snippet,
            insert_before_item: None,
            description: "modify target to return 42".to_string(),
        };

        let result = engine.apply_mutation(&mutation);
        assert!(result.success, "modify should succeed: {:?}", result.error);

        let modified = std::fs::read_to_string(&tmp).unwrap();
        assert!(modified.contains("42"), "modified function should return 42");

        std::fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn test_insert_before_item() {
        let tmp = std::env::temp_dir().join("ast_mutation_insert_before.rs");
        let initial = "pub fn first() -> u32 { 1 }\npub fn third() -> u32 { 3 }\n";
        std::fs::write(&tmp, initial).unwrap();

        let parent = tmp.parent().unwrap().to_string_lossy().to_string();
        let filename = tmp.file_name().unwrap().to_string_lossy().to_string();

        let mut engine = AstMutationEngine::new(AstMutationConfig {
            backup_before_mutate: false,
            ..Default::default()
        });
        engine.set_workspace_root(&parent);

        let snippet = "pub fn second() -> u32 { 2 }".to_string();
        let mutation = AstMutation {
            mutation_type: MutationType::AddFunction,
            target_file: filename,
            name: "second".to_string(),
            code_snippet: snippet,
            insert_before_item: Some("third".to_string()),
            description: "insert second before third".to_string(),
        };

        let result = engine.apply_mutation(&mutation);
        assert!(result.success, "insert_before should succeed: {:?}", result.error);

        let modified = std::fs::read_to_string(&tmp).unwrap();
        // Verify ordering: first should be before second, second before third
        let first_pos = modified.find("pub fn first").unwrap();
        let second_pos = modified.find("pub fn second").unwrap();
        let third_pos = modified.find("pub fn third").unwrap();
        assert!(
            first_pos < second_pos && second_pos < third_pos,
            "functions should be in order: first < second < third"
        );

        std::fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn test_last_results() {
        let mut engine = AstMutationEngine::new(AstMutationConfig::default());
        for i in 0..5 {
            let mutation = AstMutation {
                mutation_type: MutationType::AddImport,
                target_file: format!("/nonexistent/last_{}.rs", i),
                name: format!("import_{}", i),
                code_snippet: "use std::collections::HashMap;".to_string(),
                insert_before_item: None,
                description: String::new(),
            };
            engine.apply_mutation(&mutation);
        }
        let last3 = engine.last_results(3);
        assert_eq!(last3.len(), 3);
    }
}

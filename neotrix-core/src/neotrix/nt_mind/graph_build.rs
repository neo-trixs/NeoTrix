use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(crate) enum ParsedItem {
    UseStatement {
        target: String,
    },
    Function {
        name: String,
        line: usize,
        calls: Vec<String>,
    },
    StructDef {
        name: String,
        line: usize,
    },
    TraitDef {
        name: String,
        line: usize,
    },
    ImplBlock,
    ModDecl,
}

pub(crate) fn parse_rust_file(content: &str) -> Vec<ParsedItem> {
    let syntax = match syn::parse_file(content) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut items = Vec::new();

    for item in &syntax.items {
        match item {
            syn::Item::Fn(f) => {
                let name = f.sig.ident.to_string();
                let line = f.sig.ident.span().start().line;
                let calls = extract_calls_from_block(&f.block);
                items.push(ParsedItem::Function { name, line, calls });
            }
            syn::Item::Struct(s) => {
                items.push(ParsedItem::StructDef {
                    name: s.ident.to_string(),
                    line: s.ident.span().start().line,
                });
            }
            syn::Item::Trait(t) => {
                items.push(ParsedItem::TraitDef {
                    name: t.ident.to_string(),
                    line: t.ident.span().start().line,
                });
                for trait_item in &t.items {
                    if let syn::TraitItem::Fn(method) = trait_item {
                        let name = method.sig.ident.to_string();
                        let line = method.sig.ident.span().start().line;
                        let calls = method
                            .default
                            .as_ref()
                            .map(extract_calls_from_block)
                            .unwrap_or_default();
                        items.push(ParsedItem::Function { name, line, calls });
                    }
                }
            }
            syn::Item::Use(u) => {
                let target = fmt_use_tree(&u.tree);
                items.push(ParsedItem::UseStatement { target });
            }
            syn::Item::Impl(i) => {
                let _struct_namee = match &i.trait_ {
                    Some((_, path, _)) => path_to_string(path),
                    None => ty_to_string(&i.self_ty),
                };
                let _trait_nameme = i.trait_.as_ref().map(|_| ty_to_string(&i.self_ty));
                items.push(ParsedItem::ImplBlock);
                for inner in &i.items {
                    if let syn::ImplItem::Fn(method) = inner {
                        let name = method.sig.ident.to_string();
                        let line = method.sig.ident.span().start().line;
                        let calls = extract_calls_from_block(&method.block);
                        items.push(ParsedItem::Function { name, line, calls });
                    }
                }
            }
            syn::Item::Mod(_m) => {
                items.push(ParsedItem::ModDecl);
            }
            _ => {}
        }
    }

    items
}

pub(crate) fn path_to_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

pub(crate) fn ty_to_string(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(type_path) => path_to_string(&type_path.path),
        _ => String::new(),
    }
}

pub(crate) fn fmt_use_tree(tree: &syn::UseTree) -> String {
    match tree {
        syn::UseTree::Path(p) => format!("{}::{}", p.ident, fmt_use_tree(&p.tree)),
        syn::UseTree::Name(n) => n.ident.to_string(),
        syn::UseTree::Rename(r) => format!("{} as {}", r.ident, r.rename),
        syn::UseTree::Glob(_) => "*".to_string(),
        syn::UseTree::Group(g) => {
            let inner: Vec<String> = g.items.iter().map(fmt_use_tree).collect();
            format!("{{{}}}", inner.join(", "))
        }
    }
}

pub(crate) fn extract_calls_from_block(block: &syn::Block) -> Vec<String> {
    let mut finder = CallFinder { calls: Vec::new() };
    syn::visit::visit_block(&mut finder, block);
    finder.calls
}

struct CallFinder {
    calls: Vec<String>,
}

impl<'ast> syn::visit::Visit<'ast> for CallFinder {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let Some(name) = get_call_name(&node.func) {
            self.calls.push(name);
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        self.calls.push(node.method.to_string());
        syn::visit::visit_expr_method_call(self, node);
    }
}

pub(crate) fn get_call_name(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
        _ => None,
    }
}

pub(crate) fn register_fn_defs(
    path: &std::path::Path,
    items: &[ParsedItem],
    fn_defs: &mut HashMap<String, Vec<(PathBuf, usize)>>,
) {
    for item in items {
        if let ParsedItem::Function { name, line, .. } = item {
            fn_defs
                .entry(name.clone())
                .or_default()
                .push((path.to_path_buf(), *line));
        }
    }
}

pub(crate) fn resolve_import(file_path: &std::path::Path, use_target: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = use_target.split("::").collect();
    if parts.is_empty() {
        return None;
    }

    let parent = file_path.parent()?;
    let normalized: Vec<&str> = parts
        .iter()
        .skip_while(|p| *p == &"crate" || *p == &"self" || *p == &"super")
        .copied()
        .collect();

    if normalized.is_empty() {
        return None;
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    let mut dir = parent.to_path_buf();
    for (i, part) in normalized.iter().enumerate() {
        if i == normalized.len() - 1 {
            candidates.push(dir.join(format!("{}.rs", part)));
            candidates.push(dir.join(part).join("mod.rs"));
        } else {
            dir = dir.join(part);
        }
    }

    let crate_root = find_crate_root(file_path)?;
    dir = crate_root.clone();
    for (i, part) in normalized.iter().enumerate() {
        if i == normalized.len() - 1 {
            candidates.push(dir.join(format!("{}.rs", part)));
            candidates.push(dir.join(part).join("mod.rs"));
        } else {
            dir = dir.join(part);
        }
    }

    candidates.into_iter().find(|c| c.exists())
}

pub(crate) fn find_crate_root(path: &std::path::Path) -> Option<PathBuf> {
    let mut current = path.parent()?;
    loop {
        if current.join("Cargo.toml").exists() {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
}

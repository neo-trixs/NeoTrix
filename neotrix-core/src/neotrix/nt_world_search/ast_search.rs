use std::path::Path;

use super::grep::Match;

#[derive(Debug, Clone, PartialEq)]
pub enum AstPattern {
    Function { name: Option<String> },
    Struct { name: Option<String> },
    Enum { name: Option<String> },
    Trait { name: Option<String> },
    Impl { for_type: Option<String> },
    AnyDefinition,
}

pub fn search_ast(path: &Path, pattern: &AstPattern) -> Vec<Match> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let syntax = match syn::parse_file(&content) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let path_str = path.display().to_string();
    let mut results = Vec::new();

    for item in &syntax.items {
        match item {
            syn::Item::Fn(f) => {
                if matches!(pattern, AstPattern::AnyDefinition)
                    || matches!(pattern, AstPattern::Function { name: None })
                    || matches!(pattern, AstPattern::Function { name: Some(ref n) } if n == &f.sig.ident.to_string())
                {
                    let start_line = f.sig.ident.span().start().line;
                    let line_content = content
                        .lines()
                        .nth(start_line.saturating_sub(1))
                        .unwrap_or("");
                    results.push(Match {
                        path: path_str.clone(),
                        line: start_line,
                        content: format!("fn {} {}", f.sig.ident, line_content.trim()),
                        is_definition: true,
                    });
                }
            }
            syn::Item::Struct(s) => {
                if matches!(pattern, AstPattern::AnyDefinition)
                    || matches!(pattern, AstPattern::Struct { name: None })
                    || matches!(pattern, AstPattern::Struct { name: Some(ref n) } if n == &s.ident.to_string())
                {
                    let line = s.struct_token.span.start().line;
                    let line_content = content.lines().nth(line.saturating_sub(1)).unwrap_or("");
                    results.push(Match {
                        path: path_str.clone(),
                        line,
                        content: format!("struct {} {}", s.ident, line_content.trim()),
                        is_definition: true,
                    });
                }
            }
            syn::Item::Enum(e) => {
                if matches!(pattern, AstPattern::AnyDefinition)
                    || matches!(pattern, AstPattern::Enum { name: None })
                    || matches!(pattern, AstPattern::Enum { name: Some(ref n) } if n == &e.ident.to_string())
                {
                    let line = e.enum_token.span.start().line;
                    let line_content = content.lines().nth(line.saturating_sub(1)).unwrap_or("");
                    results.push(Match {
                        path: path_str.clone(),
                        line,
                        content: format!("enum {} {}", e.ident, line_content.trim()),
                        is_definition: true,
                    });
                }
            }
            syn::Item::Trait(t) => {
                if matches!(pattern, AstPattern::AnyDefinition)
                    || matches!(pattern, AstPattern::Trait { name: None })
                    || matches!(pattern, AstPattern::Trait { name: Some(ref n) } if n == &t.ident.to_string())
                {
                    let line = t.trait_token.span.start().line;
                    let line_content = content.lines().nth(line.saturating_sub(1)).unwrap_or("");
                    results.push(Match {
                        path: path_str.clone(),
                        line,
                        content: format!("trait {} {}", t.ident, line_content.trim()),
                        is_definition: true,
                    });
                }
            }
            syn::Item::Impl(i) => {
                let type_name = if let Some((_, path_seg, _)) = &i.trait_ {
                    path_seg.segments.last().map(|s| s.ident.to_string())
                } else {
                    Some("impl".to_string())
                };
                if matches!(pattern, AstPattern::AnyDefinition)
                    || matches!(pattern, AstPattern::Impl { for_type: None })
                    || matches!(pattern, AstPattern::Impl { for_type: Some(ref n) } if type_name.as_deref() == Some(n))
                {
                    let line = i.impl_token.span.start().line;
                    let line_content = content.lines().nth(line.saturating_sub(1)).unwrap_or("");
                    let desc = if let Some(ref s) = type_name {
                        format!("impl {} {}", s, line_content.trim())
                    } else {
                        format!("impl {}", line_content.trim())
                    };
                    results.push(Match {
                        path: path_str.clone(),
                        line,
                        content: desc,
                        is_definition: true,
                    });
                }
            }
            _ => {}
        }
    }

    results
}

pub fn search_ast_in_path(path: &Path, pattern: &AstPattern) -> Vec<Match> {
    let mut all = Vec::new();
    if path.is_file() {
        all = search_ast(path, pattern);
    } else if path.is_dir() {
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension().map(|e| e == "rs").unwrap_or(false) {
                let mut r = search_ast(entry.path(), pattern);
                all.append(&mut r);
            }
        }
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_rs(content: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("neotrix_ast_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(format!("ast_{}.rs", fastrand::u32(..)));
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "{}", content).unwrap();
        path
    }

    #[test]
    fn test_find_function() {
        let p = temp_rs("fn hello() {}\nfn world() {}");
        let r = search_ast(
            &p,
            &AstPattern::Function {
                name: Some("hello".into()),
            },
        );
        assert_eq!(r.len(), 1);
        assert!(r[0].content.contains("hello"));
        assert!(r[0].is_definition);
    }

    #[test]
    fn test_find_all_functions() {
        let p = temp_rs("fn a() {}\nfn b() {}\nlet x = 1;");
        let r = search_ast(&p, &AstPattern::Function { name: None });
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_find_struct() {
        let p = temp_rs("struct Foo { x: i32 }\nfn bar() {}");
        let r = search_ast(
            &p,
            &AstPattern::Struct {
                name: Some("Foo".into()),
            },
        );
        assert_eq!(r.len(), 1);
        assert!(r[0].content.contains("Foo"));
    }

    #[test]
    fn test_any_definition() {
        let p = temp_rs("fn a() {}\nstruct B;\nenum C { X }\ntrait D {}\nimpl E {}");
        let r = search_ast(&p, &AstPattern::AnyDefinition);
        assert_eq!(r.len(), 5);
    }

    #[test]
    fn test_trait_search() {
        let p = temp_rs("trait MyTrait { fn method(); }");
        let r = search_ast(
            &p,
            &AstPattern::Trait {
                name: Some("MyTrait".into()),
            },
        );
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_impl_search() {
        let p = temp_rs("struct S;\nimpl S { fn m() {} }\nimpl Trait for S { fn t() {} }");
        let r = search_ast(&p, &AstPattern::Impl { for_type: None });
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_non_rs_file_skipped() {
        let dir = std::env::temp_dir().join("neotrix_ast_test_nonrs");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("data.txt");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "fn hello() {{}}").unwrap();
        let r = search_ast_in_path(&path, &AstPattern::AnyDefinition);
        // walkdir recurses into subdirs; if temp_rs embedded, it won't match .txt
        assert!(r.is_empty() || !r.iter().any(|m| m.path.ends_with("data.txt")));
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    Function,
    Method,
    Class,
    Struct,
    Trait,
    Enum,
    Impl,
    Module,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct BlockChunk {
    pub name: String,
    pub block_type: BlockType,
    pub start_line: usize,
    pub end_line: usize,
    pub source: String,
}

fn strip_trailing_comment(line: &str) -> &str {
    if let Some(pos) = line.find("//") {
        &line[..pos]
    } else {
        line
    }
}

fn count_braces(line: &str) -> (usize, usize) {
    (line.matches('{').count(), line.matches('}').count())
}

fn try_extract_name(line: &str) -> Option<(String, BlockType)> {
    let trimmed = line.trim();

    if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") {
        return None;
    }

    if trimmed.starts_with('}') {
        return None;
    }

    let mut s = trimmed;

    loop {
        if s.starts_with("pub") {
            if s.starts_with("pub(") {
                if let Some(end) = s.find(')') {
                    s = s[end + 1..].trim_start();
                } else {
                    return None;
                }
            } else {
                s = s[3..].trim_start();
            }
        } else if s.starts_with("async ") {
            s = s[6..].trim_start();
        } else if s.starts_with("unsafe ") {
            s = s[7..].trim_start();
        } else if s.starts_with("extern ") {
            s = s[7..].trim_start();
        } else {
            break;
        }
    }

    if s.starts_with("fn ") || s.starts_with("def ") {
        let rest = &s[s.find(' ').unwrap_or(3) + 1..].trim_start();
        let name = rest
            .split(|c: char| c == '(' || c == '<' || c == ' ' || c == '\t' || c == '{' || c == ';')
            .next()
            .unwrap_or("")
            .to_string();
        if !name.is_empty() {
            return Some((name, BlockType::Function));
        }
    }

    if s.starts_with("struct ") {
        let rest = &s[7..].trim_start();
        let name = rest
            .split(|c: char| c == '{' || c == '<' || c == ' ' || c == '\t' || c == ';')
            .next()
            .unwrap_or("")
            .to_string();
        if !name.is_empty() {
            return Some((name, BlockType::Struct));
        }
    }

    if s.starts_with("enum ") {
        let rest = &s[5..].trim_start();
        let name = rest
            .split(|c: char| c == '{' || c == ' ' || c == '\t' || c == ';')
            .next()
            .unwrap_or("")
            .to_string();
        if !name.is_empty() {
            return Some((name, BlockType::Enum));
        }
    }

    if s.starts_with("trait ") {
        let rest = &s[6..].trim_start();
        let name = rest
            .split(|c: char| c == '{' || c == '<' || c == ':' || c == ' ' || c == '\t' || c == ';')
            .next()
            .unwrap_or("")
            .to_string();
        if !name.is_empty() {
            return Some((name, BlockType::Trait));
        }
    }

    if s.starts_with("impl")
        && (s.len() == 4
            || s[4..].starts_with(' ')
            || s[4..].starts_with('\t')
            || s[4..].starts_with('<'))
    {
        let rest = s[4..].trim_start();
        let name = if rest.is_empty() || rest.starts_with('{') {
            "impl".to_string()
        } else {
            let after_generics = if rest.starts_with('<') {
                let mut depth = 0i32;
                let mut end = 0;
                for (i, c) in rest.char_indices() {
                    if c == '<' {
                        depth += 1;
                    }
                    if c == '>' {
                        depth -= 1;
                        if depth == 0 {
                            end = i + 1;
                            break;
                        }
                    }
                }
                if end > 0 {
                    rest[end..].trim_start()
                } else {
                    rest
                }
            } else {
                rest
            };
            let name_end = after_generics
                .find(|c: char| c == '{' || c == ' ' || c == '\t')
                .unwrap_or(after_generics.len());
            let name_part = &after_generics[..name_end];
            let clean = name_part.split('<').next().unwrap_or(name_part).trim();
            if clean.is_empty() {
                "impl".to_string()
            } else {
                clean.to_string()
            }
        };
        return Some((name, BlockType::Impl));
    }

    if s.starts_with("class ") {
        let rest = &s[6..].trim_start();
        let name = rest
            .split(|c: char| c == '(' || c == ':' || c == ' ' || c == '\t' || c == '{')
            .next()
            .unwrap_or("")
            .to_string();
        if !name.is_empty() {
            return Some((name, BlockType::Class));
        }
    }

    if s.starts_with("module ") {
        let rest = &s[7..].trim_start();
        let name = rest
            .split(|c: char| c == '{' || c == ';' || c == ' ' || c == '\t')
            .next()
            .unwrap_or("")
            .to_string();
        if !name.is_empty() {
            return Some((name, BlockType::Module));
        }
    }

    None
}

pub fn extract_blocks(source: &str) -> Vec<BlockChunk> {
    let lines: Vec<&str> = source.lines().collect();
    let mut blocks = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if let Some((name, block_type)) = try_extract_name(lines[i]) {
            let no_comment = strip_trailing_comment(lines[i]).trim().to_string();
            if no_comment.ends_with(';') {
                i += 1;
                continue;
            }

            let is_python_style = (block_type == BlockType::Function
                && lines[i].trim().starts_with("def "))
                || (block_type == BlockType::Class && lines[i].trim().starts_with("class "));

            if is_python_style {
                if !lines[i].trim().ends_with(':') {
                    i += 1;
                    continue;
                }
                let decl_indent = lines[i].len() - lines[i].trim_start().len();

                let mut j = i + 1;
                while j < lines.len() && lines[j].trim().is_empty() {
                    j += 1;
                }
                if j >= lines.len() {
                    i += 1;
                    continue;
                }
                let body_indent = lines[j].len() - lines[j].trim_start().len();
                if body_indent <= decl_indent {
                    blocks.push(BlockChunk {
                        name,
                        block_type,
                        start_line: i + 1,
                        end_line: j + 1,
                        source: lines[i..=j].join("\n"),
                    });
                    i = j + 1;
                    continue;
                }

                let mut end = j;
                while end + 1 < lines.len() {
                    let next = lines[end + 1];
                    let next_indent = next.len() - next.trim_start().len();
                    if next.trim().is_empty() {
                        end += 1;
                        continue;
                    }
                    if next_indent >= body_indent {
                        end += 1;
                    } else {
                        break;
                    }
                }

                blocks.push(BlockChunk {
                    name,
                    block_type,
                    start_line: i + 1,
                    end_line: end + 1,
                    source: lines[i..=end].join("\n"),
                });
                i = end + 1;
                continue;
            }

            let mut brace_depth = 0i32;
            let mut j = i;
            let mut found_block = false;

            while j < lines.len() {
                let (open, close) = count_braces(lines[j]);
                brace_depth += open as i32;
                brace_depth -= close as i32;

                if !found_block {
                    if open > 0 {
                        found_block = true;
                        if brace_depth <= 0 {
                            blocks.push(BlockChunk {
                                name,
                                block_type,
                                start_line: i + 1,
                                end_line: j + 1,
                                source: lines[i..=j].join("\n"),
                            });
                            i = j + 1;
                            break;
                        }
                    }
                } else if brace_depth <= 0 {
                    blocks.push(BlockChunk {
                        name,
                        block_type,
                        start_line: i + 1,
                        end_line: j + 1,
                        source: lines[i..=j].join("\n"),
                    });
                    i = j + 1;
                    break;
                }

                j += 1;
            }

            if !found_block {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_blocks_rust_fn() {
        let source = "fn hello() {\n    println!(\"world\");\n}\n\nfn add(a: i32, b: i32) -> i32 {\n    a + b\n}";
        let blocks = extract_blocks(source);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].name, "hello");
        assert_eq!(blocks[0].block_type, BlockType::Function);
        assert_eq!(blocks[0].start_line, 1);
        assert_eq!(blocks[0].end_line, 3);
        assert_eq!(blocks[1].name, "add");
        assert_eq!(blocks[1].start_line, 5);
        assert_eq!(blocks[1].end_line, 7);
    }

    #[test]
    fn test_extract_blocks_rust_struct() {
        let source = "pub struct Config {\n    pub name: String,\n    pub version: u64,\n}";
        let blocks = extract_blocks(source);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, "Config");
        assert_eq!(blocks[0].block_type, BlockType::Struct);
    }

    #[test]
    fn test_extract_blocks_rust_impl() {
        let source = "impl Config {\n    pub fn new() -> Self {\n        Self { name: \"\".into(), version: 0 }\n    }\n}";
        let blocks = extract_blocks(source);
        assert!(blocks.len() >= 1);
        let impl_block = blocks
            .iter()
            .find(|b| b.block_type == BlockType::Impl)
            .unwrap();
        assert_eq!(impl_block.name, "Config");
    }

    #[test]
    fn test_extract_blocks_rust_enum() {
        let source = "enum Status {\n    Active,\n    Inactive,\n    Pending,\n}";
        let blocks = extract_blocks(source);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, "Status");
        assert_eq!(blocks[0].block_type, BlockType::Enum);
    }

    #[test]
    fn test_extract_blocks_python_fn() {
        let source = "def parse_config(path: str) -> dict:\n    import json\n    with open(path) as f:\n        return json.load(f)\n\ndef validate(data: dict) -> bool:\n    return 'name' in data";
        let blocks = extract_blocks(source);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].name, "parse_config");
        assert_eq!(blocks[0].block_type, BlockType::Function);
        assert!(blocks[0].source.contains("import json"));
        assert_eq!(blocks[1].name, "validate");
        assert!(blocks[1].source.contains("'name' in data"));
    }

    #[test]
    fn test_extract_blocks_python_class() {
        let source = "class User:\n    def __init__(self, name: str):\n        self.name = name\n\n    def greet(self) -> str:\n        return f\"Hello, {self.name}\"";
        let blocks = extract_blocks(source);
        let class_block = blocks
            .iter()
            .find(|b| b.block_type == BlockType::Class)
            .unwrap();
        assert_eq!(class_block.name, "User");
        assert!(class_block.source.contains("def __init__"));
    }

    #[test]
    fn test_extract_blocks_forward_decl() {
        let source = "fn foo();\n\nfn bar() {\n    foo()\n}";
        let blocks = extract_blocks(source);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, "bar");
    }

    #[test]
    fn test_extract_blocks_empty_body() {
        let source = "fn foo() {}\nfn bar() {\n    // just a comment\n}";
        let blocks = extract_blocks(source);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].name, "foo");
        assert_eq!(blocks[0].start_line, 1);
        assert_eq!(blocks[1].name, "bar");
    }

    #[test]
    fn test_extract_blocks_nested_braces() {
        let source = "fn outer() {\n    if true {\n        loop {\n            break;\n        }\n    }\n    let x = 1;\n}";
        let blocks = extract_blocks(source);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, "outer");
        assert_eq!(blocks[0].start_line, 1);
        assert_eq!(blocks[0].end_line, 8);
    }

    #[test]
    fn test_extract_blocks_pub_unsafe_fn() {
        let source = "pub unsafe fn dangerous() {\n    // risky business\n}";
        let blocks = extract_blocks(source);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, "dangerous");
    }
}

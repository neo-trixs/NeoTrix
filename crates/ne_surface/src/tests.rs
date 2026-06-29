use super::ast::NeExpr;
use super::parse;

#[test]
fn test_parse_reflect() {
    let result = parse("reflect()").unwrap();
    assert!(matches!(result, NeExpr::Reflect(None)));
}

#[test]
fn test_parse_bind() {
    let result = parse("bind(a, b)").unwrap();
    assert!(matches!(result, NeExpr::Bind(_, _)));
}

#[test]
fn test_parse_fn_declaration() {
    let src = "fn vsa_bind(a: vector, b: vector) -> vector { bind(a, b) }";
    let result = parse(src).unwrap();
    assert!(matches!(result, NeExpr::Function { name: ref n, .. } if n == "vsa_bind"));
}

#[test]
fn test_parse_sexpr_reflect() {
    let result = parse("(reflect)").unwrap();
    assert!(matches!(result, NeExpr::Reflect(None)));
}

#[test]
fn test_parse_module() {
    let src = r#"module hello {
        import stdlib
        fn greet() -> string { "Hello from Ne" }
    }"#;
    let result = parse(src).unwrap();
    assert!(matches!(result, NeExpr::Module { .. }));
}

#[test]
fn test_sexpr_roundtrip() {
    let src = "fn add(a: vector, b: vector) -> vector { bind(a, b) }";
    let ast = parse(src).unwrap();
    let sexpr = ast.to_sexpr();
    // The sexpr output should parse back without error
    let ast2 = parse(&sexpr);
    assert!(ast2.is_ok(), "Roundtrip failed: {}", sexpr);
}

#[test]
fn test_sutra_style_decide() {
    // Adapted from Sutra's permutation_conditional.su
    let src = r#"fn decide(smell: vector, hunger: vector) -> string {
        let query = bind(smell, hunger);
        let winner = similarity(query, [proto_ph, proto_pf, proto_ah, proto_af]);
        match winner {
            0 => "approach",
            1 => "ignore",
            2 => "search",
            _ => "idle"
        }
    }"#;
    let ast = parse(src).unwrap();
    assert!(
        matches!(ast, NeExpr::Function { name: ref n, .. } if n == "decide"),
        "Expected function 'decide', got {:?}",
        ast
    );
    let sexpr = ast.to_sexpr();
    assert!(sexpr.contains("bind"), "sexpr should contain 'bind': {}", sexpr);
    assert!(
        sexpr.contains("similarity"),
        "sexpr should contain 'similarity': {}",
        sexpr
    );
}

#[test]
fn test_parse_curious() {
    let result = parse("curious(reflect())").unwrap();
    assert!(matches!(result, NeExpr::Curious(_)));
}

#[test]
fn test_parse_dream() {
    let result = parse("dream(bind(x, y))").unwrap();
    assert!(matches!(result, NeExpr::Dream(_)));
}

#[test]
fn test_parse_bundle() {
    let result = parse("bundle(a, b, c)").unwrap();
    assert!(matches!(result, NeExpr::Bundle(ref items) if items.len() == 3));
}

#[test]
fn test_parse_permute() {
    let result = parse("permute(a, b)").unwrap();
    assert!(matches!(result, NeExpr::Permute(_, _)));
}

#[test]
fn test_parse_similarity() {
    let result = parse("similarity(x, y)").unwrap();
    assert!(matches!(result, NeExpr::Similarity(_, _)));
}

#[test]
fn test_parse_lit_int() {
    let result = parse("42").unwrap();
    assert_eq!(result, NeExpr::LitInt(42));
}

#[test]
fn test_parse_lit_float() {
    let result = parse("3.14").unwrap();
    assert!(matches!(result, NeExpr::LitFloat(f) if (f - 3.14).abs() < 1e-10));
}

#[test]
fn test_parse_lit_string() {
    let result = parse("\"hello world\"").unwrap();
    assert_eq!(result, NeExpr::LitString("hello world".to_string()));
}

#[test]
fn test_parse_var() {
    let result = parse("my_var").unwrap();
    assert_eq!(result, NeExpr::Var("my_var".to_string()));
}

#[test]
fn test_parse_sexpr_bind() {
    let result = parse("(bind x y)").unwrap();
    assert!(matches!(result, NeExpr::Bind(_, _)));
}

#[test]
fn test_parse_sexpr_curious() {
    let result = parse("(curious (reflect))").unwrap();
    assert!(matches!(result, NeExpr::Curious(ref inner) if matches!(**inner, NeExpr::Reflect(None))));
}

#[test]
fn test_parse_sexpr_bundle() {
    let result = parse("(bundle a b c)").unwrap();
    assert!(matches!(result, NeExpr::Bundle(ref items) if items.len() == 3));
}

#[test]
fn test_parse_seq() {
    let result = parse("(seq a b c)").unwrap();
    assert!(matches!(result, NeExpr::Seq(ref items) if items.len() == 3));
}

#[test]
fn test_parse_empty_sexpr() {
    let result = parse("()").unwrap();
    assert!(matches!(result, NeExpr::Bundle(ref items) if items.is_empty()));
}

#[test]
fn test_parse_nested_sexpr() {
    let result = parse("(bind (curious x) (dream y))").unwrap();
    assert!(matches!(result, NeExpr::Bind(_, _)));
}

#[test]
fn test_edit_directive() {
    let result = parse("edit(handler.inner_critic.threshold, 0.5)").unwrap();
    assert!(matches!(result, NeExpr::Edit(ref e) if e.target == vec![
        "handler".to_string(),
        "inner_critic".to_string(),
        "threshold".to_string()
    ]));
}

#[test]
fn test_sutra_sexpr_roundtrip() {
    // Verify that Sutra-style surface syntax produces a valid sexpr
    let src = "similarity(query, [proto_ph, proto_pf])";
    let ast = parse(src).unwrap();
    let sexpr = ast.to_sexpr();
    assert!(sexpr.contains("similarity"));
    // The sexpr should be parseable
    let ast2 = parse(&sexpr);
    assert!(ast2.is_ok(), "Sexpr roundtrip failed: {}", sexpr);
}

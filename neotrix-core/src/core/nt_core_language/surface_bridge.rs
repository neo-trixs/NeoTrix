use super::expr::NeExpr as CoreNeExpr;
use super::value::NeValue;

/// Convert a ne_surface AST expression into the core evaluator's AST.
pub fn surface_to_core(expr: &ne_surface::ast::NeExpr) -> Result<CoreNeExpr, String> {
    match expr {
        // Direct mappings
        ne_surface::ast::NeExpr::Bind(a, b) => Ok(CoreNeExpr::Bind(
            Box::new(surface_to_core(a)?),
            Box::new(surface_to_core(b)?),
        )),
        ne_surface::ast::NeExpr::Bundle(xs) => {
            let converted: Result<Vec<CoreNeExpr>, String> =
                xs.iter().map(surface_to_core).collect();
            Ok(CoreNeExpr::Bundle(converted?))
        }
        ne_surface::ast::NeExpr::Similarity(a, b) => Ok(CoreNeExpr::Similarity(
            Box::new(surface_to_core(a)?),
            Box::new(surface_to_core(b)?),
        )),
        ne_surface::ast::NeExpr::Var(v) => Ok(CoreNeExpr::Var(v.clone())),
        ne_surface::ast::NeExpr::LitInt(i) => Ok(CoreNeExpr::Literal(NeValue::Int(*i))),
        ne_surface::ast::NeExpr::LitFloat(f) => Ok(CoreNeExpr::Literal(NeValue::Float(*f))),
        ne_surface::ast::NeExpr::LitString(s) => Ok(CoreNeExpr::Literal(NeValue::Str(s.clone()))),
        ne_surface::ast::NeExpr::LitVector(items) => {
            let bytes: Vec<u8> = items.iter().map(|&b| b as u8).collect();
            Ok(CoreNeExpr::Literal(NeValue::Vsa(bytes)))
        }
        ne_surface::ast::NeExpr::Let(name, value, body) => Ok(CoreNeExpr::Let(
            name.clone(),
            Box::new(surface_to_core(value)?),
            Box::new(surface_to_core(body)?),
        )),
        ne_surface::ast::NeExpr::Seq(items) => {
            let converted: Result<Vec<CoreNeExpr>, String> =
                items.iter().map(surface_to_core).collect();
            Ok(CoreNeExpr::Seq(converted?))
        }
        ne_surface::ast::NeExpr::Call(name, args) => {
            let converted: Result<Vec<CoreNeExpr>, String> =
                args.iter().map(surface_to_core).collect();
            let args = converted?;
            match name.as_str() {
                "assert" => {
                    let condition = args.into_iter().next().ok_or("assert needs a condition")?;
                    Ok(CoreNeExpr::Assert {
                        condition: Box::new(condition),
                        message: String::new(),
                        tolerance: 0.0,
                    })
                }
                "test" => {
                    if args.len() < 2 {
                        return Err("test needs name and body".into());
                    }
                    let name_val = match &args[0] {
                        CoreNeExpr::Literal(NeValue::Str(s)) => s.clone(),
                        _ => return Err("test name must be a string".into()),
                    };
                    let body = args[1].clone();
                    let expected = args.get(2).map_or(true, |e| match e {
                        CoreNeExpr::Literal(NeValue::Bool(b)) => *b,
                        _ => true,
                    });
                    Ok(CoreNeExpr::Test {
                        name: name_val,
                        body: Box::new(body),
                        expected,
                    })
                }
                "property" => {
                    if args.len() < 3 {
                        return Err("property needs name, generator, and property".into());
                    }
                    let name_val = match &args[0] {
                        CoreNeExpr::Literal(NeValue::Str(s)) => s.clone(),
                        _ => return Err("property name must be a string".into()),
                    };
                    Ok(CoreNeExpr::Property {
                        name: name_val,
                        generator: Box::new(args[1].clone()),
                        property: Box::new(args[2].clone()),
                        iterations: 100,
                    })
                }
                _ => Ok(CoreNeExpr::Call(name.clone(), args)),
            }
        }

        // Permute: surface has (vec, shift), core has unary(shift=1).
        // Route through call so eval dispatches to permute primitive with shift.
        ne_surface::ast::NeExpr::Permute(vec_expr, shift_expr) => {
            let vec_core = surface_to_core(vec_expr)?;
            let shift_core = surface_to_core(shift_expr)?;
            Ok(CoreNeExpr::Call(
                "permute".into(),
                vec![vec_core, shift_core],
            ))
        }

        // Match: with optional default arm
        ne_surface::ast::NeExpr::Match(scrutinee, arms, default) => {
            let scrut_core = surface_to_core(scrutinee)?;
            let mut core_arms: Vec<(CoreNeExpr, CoreNeExpr)> = Vec::new();
            for (pat, body) in arms {
                core_arms.push((surface_to_core(pat)?, surface_to_core(body)?));
            }
            if let Some(default_expr) = default {
                let wild = CoreNeExpr::Var("_".into());
                core_arms.push((wild, surface_to_core(default_expr)?));
            }
            Ok(CoreNeExpr::Match {
                value: Box::new(scrut_core),
                arms: core_arms,
            })
        }

        // Consciousness primitives → named calls
        ne_surface::ast::NeExpr::Reflect(None) => Ok(CoreNeExpr::Call("reflect".into(), vec![])),
        ne_surface::ast::NeExpr::Reflect(Some(tag)) => Ok(CoreNeExpr::Call(
            "reflect".into(),
            vec![CoreNeExpr::Literal(NeValue::Str(tag.clone()))],
        )),
        ne_surface::ast::NeExpr::Curious(x) => Ok(CoreNeExpr::Call(
            "curious".into(),
            vec![surface_to_core(x)?],
        )),
        ne_surface::ast::NeExpr::Dream(x) => {
            Ok(CoreNeExpr::Call("dream".into(), vec![surface_to_core(x)?]))
        }

        // Function definition → Let with Lambda
        ne_surface::ast::NeExpr::Function {
            name,
            params,
            body,
            return_type: _,
        } => {
            let body_core = surface_to_core(body)?;
            let lambda = CoreNeExpr::Lambda(params.clone(), Box::new(body_core));
            // Wrap in a synthetic Seq so the function definition returns nil
            Ok(CoreNeExpr::Seq(vec![CoreNeExpr::Let(
                name.clone(),
                Box::new(lambda),
                Box::new(CoreNeExpr::Literal(NeValue::Nil)),
            )]))
        }

        // Module → evaluate body as Seq
        ne_surface::ast::NeExpr::Module {
            name: _,
            imports: _,
            body,
        } => {
            let converted: Result<Vec<CoreNeExpr>, String> =
                body.iter().map(surface_to_core).collect();
            Ok(CoreNeExpr::Seq(converted?))
        }

        // Edit directive → call form
        ne_surface::ast::NeExpr::Edit(e) => {
            let target_str = e.target.join(".");
            Ok(CoreNeExpr::Call(
                "edit".into(),
                vec![
                    CoreNeExpr::Literal(NeValue::Str(target_str)),
                    surface_to_core(&e.value)?,
                ],
            ))
        }
    }
}

use super::super::value::NeValue;
use super::{NeEvaluator, VSA_DIM};

pub fn register_primitives(eval: &mut NeEvaluator) {
    eval.register_primitive("bind", |args| {
        if args.len() < 2 {
            return Err("bind needs 2 vsa".into());
        }
        let a = vsa_arg(&args[0])?;
        let b = vsa_arg(&args[1])?;
        let mut out = Vec::with_capacity(VSA_DIM);
        for i in 0..VSA_DIM.min(a.len()).min(b.len()) {
            out.push(a[i] ^ b[i]);
        }
        Ok(NeValue::Vsa(out))
    });
    eval.register_primitive("bundle", |args| {
        if args.is_empty() {
            return Err("bundle needs >=1 vsa".into());
        }
        let vs: Vec<&[u8]> = args.iter().map(|a| vsa_arg(a)).collect::<Result<_, _>>()?;
        let dim = vs[0].len();
        let count = vs.len();
        let mut out = Vec::with_capacity(dim);
        for i in 0..dim {
            let ones = vs.iter().filter(|v| v[i] > 0).count();
            out.push(if ones > count / 2 { 1 } else { 0 });
        }
        Ok(NeValue::Vsa(out))
    });
    eval.register_primitive("negate", |args| {
        let a = vsa_arg(&args[0])?;
        let out: Vec<u8> = a.iter().map(|b| !b).collect();
        Ok(NeValue::Vsa(out))
    });
    eval.register_primitive("permute", |args| {
        let a = vsa_arg(&args[0])?;
        let shift = match args.get(1) {
            Some(NeValue::Int(n)) => (*n).rem_euclid(VSA_DIM as i64) as usize,
            _ => 1,
        };
        let mut out = vec![0u8; a.len()];
        let dim = a.len();
        for i in 0..dim {
            out[(i + shift) % dim] = a[i];
        }
        Ok(NeValue::Vsa(out))
    });
    eval.register_primitive("cosine", |args| {
        if args.len() < 2 {
            return Err("cosine needs 2 vsa".into());
        }
        let a = vsa_arg(&args[0])?;
        let b = vsa_arg(&args[1])?;
        let dim = a.len().min(b.len());
        let dot = a[..dim]
            .iter()
            .zip(&b[..dim])
            .filter(|(x, y)| **x > 0 && **y > 0)
            .count() as f64;
        let na = a[..dim].iter().filter(|x| **x > 0).count() as f64;
        let nb = b[..dim].iter().filter(|y| **y > 0).count() as f64;
        let norm = na.sqrt() * nb.sqrt();
        Ok(NeValue::Float(if norm == 0.0 { 0.0 } else { dot / norm }))
    });
    eval.register_primitive("hamming", |args| {
        if args.len() < 2 {
            return Err("hamming needs 2 vsa".into());
        }
        let a = vsa_arg(&args[0])?;
        let b = vsa_arg(&args[1])?;
        let dim = a.len().min(b.len());
        let dist = a[..dim]
            .iter()
            .zip(&b[..dim])
            .filter(|(x, y)| x != y)
            .count() as f64;
        Ok(NeValue::Float(1.0 - dist / dim as f64))
    });
    eval.register_primitive("add", |args| {
        let mut sum = 0i64;
        for a in args {
            match a {
                NeValue::Int(n) => sum += n,
                _ => return Err("add needs ints".into()),
            }
        }
        Ok(NeValue::Int(sum))
    });
    eval.register_primitive("sub", |args| {
        if args.is_empty() {
            return Ok(NeValue::Int(0));
        }
        let first = match &args[0] {
            NeValue::Int(n) => *n,
            _ => return Err("sub needs ints".into()),
        };
        if args.len() == 1 {
            return Ok(NeValue::Int(-first));
        }
        let mut rest = 0i64;
        for a in &args[1..] {
            match a {
                NeValue::Int(n) => rest += n,
                _ => return Err("sub needs ints".into()),
            }
        }
        Ok(NeValue::Int(first - rest))
    });
    eval.register_primitive("mul", |args| {
        if args.is_empty() {
            return Ok(NeValue::Int(1));
        }
        let mut prod = 1i64;
        for a in args {
            match a {
                NeValue::Int(n) => prod *= n,
                _ => return Err("mul needs ints".into()),
            }
        }
        Ok(NeValue::Int(prod))
    });
    eval.register_primitive("lt", |args| {
        if args.len() < 2 {
            return Err("lt needs 2 args".into());
        }
        match (&args[0], &args[1]) {
            (NeValue::Int(a), NeValue::Int(b)) => Ok(NeValue::Bool(a < b)),
            (NeValue::Float(a), NeValue::Float(b)) => Ok(NeValue::Bool(a < b)),
            (NeValue::Int(a), NeValue::Float(b)) => Ok(NeValue::Bool((*a as f64) < *b)),
            (NeValue::Float(a), NeValue::Int(b)) => Ok(NeValue::Bool(*a < *b as f64)),
            _ => Err("lt needs numbers".into()),
        }
    });
    eval.register_primitive("eq", |args| {
        if args.len() < 2 {
            return Err("eq needs >=2 args".into());
        }
        let first = &args[0];
        Ok(NeValue::Bool(args[1..].iter().all(|a| a == first)))
    });
    eval.register_primitive("type", |args| {
        if args.is_empty() {
            return Err("type needs arg".into());
        }
        Ok(NeValue::Str(args[0].type_name().into()))
    });
    eval.register_primitive("println", |args| {
        let s: String = args
            .iter()
            .map(|a| a.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        log::info!("{s}");
        Ok(NeValue::Nil)
    });
    eval.register_primitive("cons", |args| {
        if args.len() < 2 {
            return Err("cons needs 2 args".into());
        }
        let mut list = match &args[1] {
            NeValue::List(xs) => xs.clone(),
            _ => return Err("cons second arg must be list".into()),
        };
        list.insert(0, args[0].clone());
        Ok(NeValue::List(list))
    });
    eval.register_primitive("car", |args| match &args[0] {
        NeValue::List(xs) if !xs.is_empty() => Ok(xs[0].clone()),
        _ => Err("car needs non-empty list".into()),
    });
    eval.register_primitive("cdr", |args| match &args[0] {
        NeValue::List(xs) if !xs.is_empty() => Ok(NeValue::List(xs[1..].to_vec())),
        _ => Err("cdr needs non-empty list".into()),
    });
    eval.register_primitive("gt", |args| {
        if args.len() < 2 {
            return Err("gt needs 2 args".into());
        }
        match (&args[0], &args[1]) {
            (NeValue::Int(a), NeValue::Int(b)) => Ok(NeValue::Bool(a > b)),
            (NeValue::Float(a), NeValue::Float(b)) => Ok(NeValue::Bool(a > b)),
            (NeValue::Int(a), NeValue::Float(b)) => Ok(NeValue::Bool(*a as f64 > *b)),
            (NeValue::Float(a), NeValue::Int(b)) => Ok(NeValue::Bool(*a > *b as f64)),
            _ => Err("gt needs numbers".into()),
        }
    });
    eval.register_primitive("gte", |args| {
        if args.len() < 2 {
            return Err("gte needs 2 args".into());
        }
        match (&args[0], &args[1]) {
            (NeValue::Int(a), NeValue::Int(b)) => Ok(NeValue::Bool(a >= b)),
            (NeValue::Float(a), NeValue::Float(b)) => Ok(NeValue::Bool(a >= b)),
            (NeValue::Int(a), NeValue::Float(b)) => Ok(NeValue::Bool(*a as f64 >= *b)),
            (NeValue::Float(a), NeValue::Int(b)) => Ok(NeValue::Bool(*a >= *b as f64)),
            _ => Err("gte needs numbers".into()),
        }
    });
    eval.register_primitive("div", |args| {
        if args.len() < 2 {
            return Err("div needs 2 args".into());
        }
        let a = match &args[0] {
            NeValue::Int(n) => *n,
            _ => return Err("div needs ints".into()),
        };
        let b = match &args[1] {
            NeValue::Int(n) => *n,
            _ => return Err("div needs ints".into()),
        };
        if b == 0 {
            return Err("div by zero".into());
        }
        Ok(NeValue::Int(a / b))
    });

    eval.register_primitive("self-modify", |args| {
        if args.len() < 2 {
            return Err("self-modify needs args: (handler_name action)".into());
        }
        let handler_name = match &args[0] {
            NeValue::Str(s) => s.clone(),
            _ => return Err("first arg must be a string (handler_name)".into()),
        };
        let action = match &args[1] {
            NeValue::Str(s) => s.clone(),
            _ => return Err("second arg must be a string (action)".into()),
        };
        Ok(NeValue::Str(format!(
            "self-modify:queue:{}:{}",
            handler_name, action
        )))
    });

    eval.register_primitive("explore", |args| {
        if args.len() < 1 {
            return Err("explore needs (name)".into());
        }
        let name = match &args[0] {
            NeValue::Str(s) => s.clone(),
            _ => return Err("explore name must be string".into()),
        };
        Ok(NeValue::Str(format!("self-modify:queue:{}:explore", name)))
    });
    eval.register_primitive("exploit", |args| {
        if args.len() < 1 {
            return Err("exploit needs (name)".into());
        }
        let name = match &args[0] {
            NeValue::Str(s) => s.clone(),
            _ => return Err("exploit name must be string".into()),
        };
        Ok(NeValue::Str(format!("self-modify:queue:{}:exploit", name)))
    });
    eval.register_primitive("repair", |args| {
        if args.len() < 1 {
            return Err("repair needs (name)".into());
        }
        let name = match &args[0] {
            NeValue::Str(s) => s.clone(),
            _ => return Err("repair name must be string".into()),
        };
        Ok(NeValue::Str(format!("self-modify:queue:{}:repair", name)))
    });
    eval.register_primitive("innovate", |args| {
        if args.len() < 2 {
            return Err("innovate needs (name1 name2)".into());
        }
        let n1 = match &args[0] {
            NeValue::Str(s) => s.clone(),
            _ => return Err("innovate arg1 must be string".into()),
        };
        let n2 = match &args[1] {
            NeValue::Str(s) => s.clone(),
            _ => return Err("innovate arg2 must be string".into()),
        };
        Ok(NeValue::Str(format!(
            "self-modify:queue:{}:innovate:{}",
            n1, n2
        )))
    });
    eval.register_primitive("harden", |args| {
        if args.len() < 1 {
            return Err("harden needs (name)".into());
        }
        let name = match &args[0] {
            NeValue::Str(s) => s.clone(),
            _ => return Err("harden name must be string".into()),
        };
        Ok(NeValue::Str(format!("self-modify:queue:{}:harden", name)))
    });
    eval.register_primitive("prune", |args| {
        if args.len() < 1 {
            return Err("prune needs (name)".into());
        }
        let name = match &args[0] {
            NeValue::Str(s) => s.clone(),
            _ => return Err("prune name must be string".into()),
        };
        Ok(NeValue::Str(format!("self-modify:queue:{}:prune", name)))
    });

    // Research engine primitives
    eval.register_primitive("propose-research", |args| {
        let title = args
            .get(0)
            .and_then(|a| match a {
                NeValue::Str(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "untitled".to_string());
        let budget = args
            .get(1)
            .and_then(|a| match a {
                NeValue::Int(n) => Some(*n as u64),
                _ => None,
            })
            .unwrap_or(20);
        Ok(NeValue::Str(format!(
            "research:proposed|{}|bgt={}",
            title, budget
        )))
    });
    eval.register_primitive("run-experiment", |_args| {
        Ok(NeValue::Str("research:run|1".to_string()))
    });
    eval.register_primitive("research-ledger", |_args| {
        Ok(NeValue::Str("research:ledger".to_string()))
    });
    eval.register_primitive("research-hypothesis", |args| {
        let status = args
            .get(0)
            .and_then(|a| match a {
                NeValue::Str(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "active".to_string());
        Ok(NeValue::Str(format!("research:hypothesis|{}", status)))
    });

    // Knowledge Graph primitives
    eval.register_primitive("submit-doc", |args| {
        let source = args
            .get(0)
            .and_then(|a| match a {
                NeValue::Str(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "auto".to_string());
        Ok(NeValue::Str(format!("kg:submit|{}", source)))
    });
    eval.register_primitive("extract-graph", |args| {
        let job_id = args
            .get(0)
            .and_then(|a| match a {
                NeValue::Str(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "latest".to_string());
        Ok(NeValue::Str(format!("kg:extract|{}", job_id)))
    });
    eval.register_primitive("export-html", |_args| {
        Ok(NeValue::Str("kg:export|html".to_string()))
    });

    // Job Queue primitives
    eval.register_primitive("enqueue-job", |args| {
        let name = args
            .get(0)
            .and_then(|a| match a {
                NeValue::Str(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "task".to_string());
        let priority = args
            .get(1)
            .and_then(|a| match a {
                NeValue::Str(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "medium".to_string());
        Ok(NeValue::Str(format!(
            "queue:enqueue|{}|pri={}",
            name, priority
        )))
    });
    eval.register_primitive("dequeue-job", |_args| {
        Ok(NeValue::Str("queue:dequeue".to_string()))
    });
    eval.register_primitive("queue-stats", |_args| {
        Ok(NeValue::Str("queue:stats".to_string()))
    });
    eval.register_primitive("cancel-job", |args| {
        let job_id = args
            .get(0)
            .and_then(|a| match a {
                NeValue::Str(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "none".to_string());
        Ok(NeValue::Str(format!("queue:cancel|{}", job_id)))
    });

    // Self-Harness primitives
    eval.register_primitive("mine-weaknesses", |_args| {
        Ok(NeValue::Str("harness:mine".to_string()))
    });
    eval.register_primitive("propose-harness", |args| {
        let target = args
            .get(0)
            .and_then(|a| match a {
                NeValue::Str(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "auto".to_string());
        Ok(NeValue::Str(format!("harness:propose|{}", target)))
    });
    eval.register_primitive("validate-proposal", |args| {
        let proposal = args
            .get(0)
            .and_then(|a| match a {
                NeValue::Str(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "latest".to_string());
        Ok(NeValue::Str(format!("harness:validate|{}", proposal)))
    });
    eval.register_primitive("harness-stats", |_args| {
        Ok(NeValue::Str("harness:stats".to_string()))
    });

    // ContextCompressor primitives
    eval.register_primitive("compress-context", |_args| {
        Ok(NeValue::Str("compressor:compress".to_string()))
    });
    eval.register_primitive("compressor-stats", |_args| {
        Ok(NeValue::Str("compressor:stats".to_string()))
    });
    // EGPO primitives
    eval.register_primitive("egpo", |_args| Ok(NeValue::Str("egpo:tick".to_string())));
    eval.register_primitive("egpo-stats", |_args| {
        Ok(NeValue::Str("egpo:stats".to_string()))
    });
    eval.register_primitive("record-trajectory", |args| {
        let action = args
            .get(0)
            .and_then(|a| match a {
                NeValue::Str(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "unknown".to_string());
        Ok(NeValue::Str(format!("egpo:record|{}", action)))
    });
    eval.register_primitive("exploration-reward", |_args| {
        Ok(NeValue::Str("egpo:exploration_reward".to_string()))
    });

    eval.register_primitive("update-guideline", |args| {
        let analysis = args
            .get(0)
            .and_then(|a| match a {
                NeValue::Str(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "default".to_string());
        Ok(NeValue::Str(format!("compressor:update|{}", analysis)))
    });

    // Operator symbol aliases for mathematical notation
    for (sym, name) in &[
        ("+", "add"),
        ("-", "sub"),
        ("*", "mul"),
        ("<", "lt"),
        ("=", "eq"),
        (">", "gt"),
        (">=", "gte"),
        ("/", "div"),
    ] {
        if let Some(&f) = eval.primitives.get(*name) {
            eval.primitives.insert(sym.to_string(), f);
        }
    }
}

fn vsa_arg(v: &NeValue) -> Result<&[u8], String> {
    match v {
        NeValue::Vsa(data) => Ok(data),
        _ => Err(format!("expected VSA vector, got {}", v.type_name())),
    }
}

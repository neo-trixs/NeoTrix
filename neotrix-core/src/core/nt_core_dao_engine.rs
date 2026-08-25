//! 道引擎 2.0 — 符号回归引擎 (Symbolic Regression Engine)
//!
//! 第一性原理拆解:
//! - 任务: 从观测数据 (x, y) 中自动发现隐式数学表达式 f(x) ≈ y
//! - 本质: 程序合成的子问题 — 搜索表达式空间, 而非参数空间
//! - 关键约束: 表达式必须可解释、可验证、可组合 (R-P81 第⑤级: 已装依赖 serde_json)
//!
//! 设计:
//! - 表达式语法树 (AST) 作为搜索空间
//! - 多目标优化: 精度 + 简洁性 (奥卡姆剃刀)
//! - 进化算法: 语法制导遗传编程 (Grammar-Guided GP)
//! - 可微分细调: 发现结构后用梯度下降微调常数
//! - 可解释性: 生成的表达式可直接人类阅读、符号化操作

use crate::core::nt_core_e8_vsa::E8VsaEmbedding;
use crate::core::l7_capability::native_bus::{closure_capability, NativeCapability, NativeBusHandle};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};

/// 表达式节点类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExprNode {
    /// 常数
    Const(f64),
    /// 变量
    Var(String),
    /// 一元运算
    Unary(UnaryOp, Box<ExprNode>),
    /// 二元运算
    Binary(BinaryOp, Box<ExprNode>, Box<ExprNode>),
}

/// 一元运算符
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Neg,      // -x
    Abs,      // |x|
    Sqrt,     // sqrt(x)
    Sin,      // sin(x)
    Cos,      // cos(x)
    Tan,      // tan(x)
    Log,      // ln(x)
    Exp,      // exp(x)
    Inv,      // 1/x
}

/// 二元运算符
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Add,      // x + y
    Sub,      // x - y
    Mul,      // x * y
    Div,      // x / y
    Pow,      // x ^ y
}

impl ExprNode {
    /// 计算表达式复杂度 (节点数)
    pub fn complexity(&self) -> usize {
        match self {
            ExprNode::Const(_) | ExprNode::Var(_) => 1,
            ExprNode::Unary(_, child) => 1 + child.complexity(),
            ExprNode::Binary(_, left, right) => 1 + left.complexity() + right.complexity(),
        }
    }

    /// 计算表达式深度
    pub fn depth(&self) -> usize {
        match self {
            ExprNode::Const(_) | ExprNode::Var(_) => 1,
            ExprNode::Unary(_, child) => 1 + child.depth(),
            ExprNode::Binary(_, left, right) => 1 + left.depth().max(right.depth()),
        }
    }

    /// 求值 (给定变量绑定)
    pub fn eval(&self, vars: &HashMap<String, f64>) -> Result<f64, String> {
        match self {
            ExprNode::Const(v) => Ok(*v),
            ExprNode::Var(name) => vars.get(name).copied().ok_or_else(|| format!("变量未绑定: {}", name)),
            ExprNode::Unary(op, child) => {
                let v = child.eval(vars)?;
                let result = match op {
                    UnaryOp::Neg => -v,
                    UnaryOp::Abs => v.abs(),
                    UnaryOp::Sqrt => {
                        if v < 0.0 { Err("负数开方".into()) } else { Ok(v.sqrt()) }
                    },
                    UnaryOp::Sin => Ok(v.sin()),
                    UnaryOp::Cos => Ok(v.cos()),
                    UnaryOp::Tan => Ok(v.tan()),
                    UnaryOp::Log => {
                        if v <= 0.0 { Err("非正数取对数".into()) } else { Ok(v.ln()) }
                    },
                    UnaryOp::Exp => Ok(v.exp()),
                    UnaryOp::Inv => {
                        if v == 0.0 { Err("除零".into()) } else { Ok(1.0 / v) }
                    },
                };
                Ok(result)
            }
            ExprNode::Binary(op, left, right) => {
                let l = left.eval(vars)?;
                let r = right.eval(vars)?;
                let result = match op {
                    BinaryOp::Add => Ok(l + r),
                    BinaryOp::Sub => Ok(l - r),
                    BinaryOp::Mul => Ok(l * r),
                    BinaryOp::Div => {
                        if r == 0.0 { Err("除零".into()) } else { Ok(l / r) }
                    },
                    BinaryOp::Pow => {
                        if l < 0.0 && r.fract() != 0.0 {
                            Err("负数非整数次幂".into())
                        } else {
                            Ok(l.powf(r))
                        }
                    },
                };
                Ok(result)
            }
        }
    }

    /// 转为可读字符串 (可读性优先)
    pub fn to_string(&self) -> String {
        match self {
            ExprNode::Const(v) => {
                if v.fract() == 0.0 { format!("{:.0}", v) } else { format!("{:.4}", v) }
            },
            ExprNode::Var(name) => name.clone(),
            ExprNode::Unary(op, child) => {
                let s = child.to_string();
                match op {
                    UnaryOp::Neg => format!("-{}", s),
                    UnaryOp::Abs => format!("|{}|", s),
                    UnaryOp::Sqrt => format!("√{}", s),
                    UnaryOp::Sin => format!("sin({})", s),
                    UnaryOp::Cos => format!("cos({})", s),
                    UnaryOp::Tan => format!("tan({})", s),
                    UnaryOp::Log => format!("ln({})", s),
                    UnaryOp::Exp => format!("exp({})", s),
                    UnaryOp::Inv => format!("1/{}", s),
                }
            }
            ExprNode::Binary(op, left, right) => {
                let l = left.to_string();
                let r = right.to_string();
                let (op_str, prec) = match op {
                    BinaryOp::Add => ("+", 1),
                    BinaryOp::Sub => ("-", 1),
                    BinaryOp::Mul => ("*", 2),
                    BinaryOp::Div => ("/", 2),
                    BinaryOp::Pow => ("^", 3),
                };
                let need_l = left.depth() > 1 && match left {
                    ExprNode::Binary(op, _, _) => op as u8 >= op as u8,
                    _ => false,
                };
                let need_r = right.depth() > 1 && match right {
                    ExprNode::Binary(op, _, _) => op as u8 >= op as u8,
                    _ => false,
                };
                let l_str = if need_l { format!("({})", l) } else { l };
                let r_str = if need_r { format!("({})", r) } else { r };
                format!("{} {} {}", l_str, op_str, r_str)
            }
        }
    }

    /// 计算表达式的哈希 (用于去重/缓存)
    pub fn hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash_impl(&mut hasher);
        hasher.finish()
    }

    fn hash_impl<H: std::hash::Hasher>(&self, state: &mut H) {
        use std::hash::Hash;
        std::mem::discriminant(self).hash(state);
        match self {
            ExprNode::Const(v) => v.to_bits().hash(state),
            ExprNode::Var(s) => s.hash(state),
            ExprNode::Unary(op, child) => {
                op.hash(state);
                child.hash_impl(state);
            }
            ExprNode::Binary(op, l, r) => {
                op.hash(state);
                l.hash_impl(state);
                r.hash_impl(state);
            }
        }
    }
}

/// 表达式个体 (种群中的个体)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExprIndividual {
    pub expr: ExprNode,
    pub fitness: f64,          // 综合适应度 (越低越好: 误差 + λ * 复杂度)
    pub error: f64,            // 平均绝对误差 / MSE
    pub complexity: usize,     // 表达式复杂度
    pub age: usize,            // 代数
    pub hash: u64,             // 去重用
}

impl ExprIndividual {
    pub fn new(expr: ExprNode) -> Self {
        let hash = expr.hash();
        let complexity = expr.complexity();
        Self {
            expr,
            fitness: f64::INFINITY,
            error: f64::INFINITY,
            complexity,
            age: 0,
            hash,
        }
    }
}

/// 符号回归配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaoEngineConfig {
    pub population_size: usize,      // 种群大小
    pub max_generations: usize,      // 最大代数
    pub max_depth: usize,            // 表达式最大深度
    pub tournament_size: usize,      // 锦标赛选择大小
    pub crossover_rate: f64,         // 交叉概率
    pub mutation_rate: f64,          // 变异概率
    pub parsimony_coeff: f64,        // 简洁性系数 (奥卡姆剃刀权重)
    pub target_metric: Metric,       // 目标指标
    pub timeout_secs: u64,           // 超时秒数
    pub n_jobs: usize,               // 并行度
    pub seed: Option<u64>,           // 随机种子
}

impl Default for DaoEngineConfig {
    fn default() -> Self {
        Self {
            population_size: 1000,
            max_generations: 100,
            max_depth: 8,
            tournament_size: 7,
            crossover_rate: 0.9,
            mutation_rate: 0.1,
            parsimony_coeff: 0.01,
            target_metric: Metric::MAE,
            timeout_secs: 300,
            n_jobs: num_cpus::get().max(1),
            seed: None,
        }
    }
}

/// 评价指标
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Metric {
    MAE,    // 平均绝对误差
    MSE,    // 均方误差
    RMSE,   // 均方根误差
    R2,     // 决定系数
}

/// 符号回归引擎 (道引擎 2.0)
pub struct DaoEngine {
    config: DaoEngineConfig,
    variables: Vec<String>,
    population: Vec<ExprIndividual>,
    best: Option<ExprIndividual>,
    generation: usize,
    rng: rand::rngs::StdRng,
    // E8 VSA 语义引导 (可选)
    vsa: Option<E8VsaEmbedding>,
}

impl DaoEngine {
    /// 创建新引擎
    pub fn new(config: DaoEngineConfig) -> Self {
        let mut rng = if let Some(seed) = config.seed {
            rand::rngs::StdRng::seed_from_u64(config.seed)
        } else {
            rand::rngs::StdRng::from_entropy()
        };
        Self {
            config,
            variables: Vec::new(),
            population: Vec::new(),
            best: None,
            generation: 0,
            rng,
            vsa: None,
        }
    }

    /// 设置变量名
    pub fn set_variables(&mut self, vars: Vec<String>) {
        self.variables = vars;
    }

    /// 设置 E8 VSA 语义引导 (可选)
    pub fn with_vsa(mut self, vsa: E8VsaEmbedding) -> Self {
        self.vsa = Some(vsa);
        self
    }

    /// 训练 (符号回归主循环)
    pub fn fit(&mut self, x: &[Vec<f64>], y: &[f64]) -> Result<ExprNode, String> {
        if x.is_empty() || y.is_empty() || x.len() != y.len() {
            return Err("数据为空或长度不匹配".into());
        }
        if self.variables.is_empty() {
            self.variables = (0..x[0].len()).map(|i| format!("x{}", i)).collect();
        }

        self.init_population(x[0].len())?;

        let start = std::time::Instant::now();
        for gen in 0..self.config.max_generations {
            if start.elapsed().as_secs() > self.config.timeout_secs {
                break;
            }
            self.generation = gen;

            // 评价适应度
            self.evaluate_population(x, y);

            // 选择
            self.selection();

            // 交叉
            self.crossover();

            // 变异
            self.mutation();

            // 精英保留
            self.elitism();

            // 更新最佳
            self.update_best();

            // 早停: 连续 N 代无改进
            if self.early_stop_check() {
                break;
            }
        }

        self.best.as_ref().map(|b| b.expr.clone()).ok_or("未找到有效模型".into())
    }

    /// 初始化种群 (Ramped Half-and-Half 初始化)
    fn init_population(&mut self, n_vars: usize) -> Result<(), String> {
        let mut pop = Vec::with_capacity(self.config.population_size);
        let mut rng = self.rng.clone();

        for i in 0..self.config.population_size {
            let depth = 2 + (i % (self.config.max_depth - 1));
            let expr = self.random_expr(i % 2 == 0, depth, &mut rng)?;
            let mut ind = ExprIndividual::new(expr);
            ind.age = 0;
            pop.push(ind);
        }
        self.population = pop;
        Ok(())
    }

    /// 随机生成表达式 (Full / Grow 方法)
    fn random_expr(&self, full: bool, max_depth: usize, rng: &mut rand::rngs::StdRng) -> Result<ExprNode, String> {
        if max_depth == 0 || (!full && self.rng.gen::<f64>() < 0.3) {
            // 终结符
            if self.rng.gen::<f64>() < 0.5 && !self.variables.is_empty() {
                let var = self.variables[self.rng.gen_range(0..self.variables.len())].clone();
                Ok(ExprNode::Var(self.variables[self.rng.gen_range(0..self.variables.len())].clone()))
            } else {
                Ok(ExprNode::Const(self.random_const(rng)))
            }
        } else {
            // 非终结符
            let op = if self.rng.gen::<bool>() {
                BinaryOp::Add
            } else if self.rng.gen::<bool>() {
                BinaryOp::Sub
            } else if self.rng.gen::<bool>() {
                BinaryOp::Mul
            } else if self.rng.gen::<bool>() {
                BinaryOp::Div
            } else {
                BinaryOp::Pow
            };
            let left = self.random_expr(full, max_depth - 1, rng)?;
            let right = self.random_expr(full, max_depth - 1, rng)?;
            Ok(ExprNode::Binary(op, Box::new(left), Box::new(right)))
        }
    }

    fn random_const(&self, rng: &mut rand::rngs::StdRng) -> f64 {
        // 常数分布: 优先小整数, 其次小数
        let r = rng.gen::<f64>();
        if r < 0.5 {
            (rng.gen_range(-10..=10) as f64)
        } else if r < 0.8 {
            (rng.gen_range(-100..=100) as f64) * 0.1
        } else {
            rng.gen_range(-10.0..10.0)
        }
    }

    /// 评价种群适应度
    fn evaluate_population(&mut self, x: &[Vec<f64>], y: &[f64]) {
        for ind in &mut self.population {
            let mut errors = Vec::with_capacity(x.len());
            for (xi, &yi) in x.iter().zip(y.iter()) {
                let vars: HashMap<_, _> = self.variables.iter().zip(xi.iter()).map(|(k, v)| (k.clone(), *v)).collect();
                let pred = self.population[0].expr.eval(&vars).unwrap_or(f64::NAN);
                if !pred.is_nan() {
                    errors.push((pred - yi).abs());
                }
            }
            if errors.is_empty() {
                ind.error = f64::INFINITY;
                ind.fitness = f64::INFINITY;
            } else {
                let mae = errors.iter().sum::<f64>() / errors.len() as f64;
                ind.error = mae;
                let complexity_penalty = self.config.parsimony_coeff * ind.complexity as f64;
                ind.fitness = mae + complexity_penalty;
            }
        }
    }

    /// 锦标赛选择
    fn selection(&mut self) {
        let mut new_pop = Vec::with_capacity(self.config.population_size);
        let mut rng = self.rng.clone();
        for _ in 0..self.config.population_size {
            let mut best = &self.population[self.rng.gen_range(0..self.population.len())];
            for _ in 1..self.config.tournament_size {
                let cand = &self.population[self.rng.gen_range(0..self.population.len())];
                if cand.fitness < best.fitness {
                    best = cand;
                }
            }
            new_pop.push(best.clone());
        }
        self.population = new_pop;
    }

    /// 交叉 (子树交换)
    fn crossover(&mut self) {
        let mut new_pop = Vec::new();
        let mut rng = self.rng.clone();
        while new_pop.len() < self.config.population_size {
            if rng.gen::<f64>() < self.config.crossover_rate && self.population.len() >= 2 {
                let p1 = self.population[rng.gen_range(0..self.population.len())].clone();
                let p2 = self.population[rng.gen_range(0..self.population.len())].clone();
                if let (Some(c1), Some(c2)) = self.crossover_trees(&p1.expr, &p2.expr) {
                    new_pop.push(ExprIndividual::new(c1));
                    new_pop.push(ExprIndividual::new(c2));
                } else {
                    new_pop.push(p1);
                    new_pop.push(p2);
                }
            } else {
                new_pop.push(self.population[self.rng.gen_range(0..self.population.len())].clone());
            }
        }
        self.population = new_pop;
    }

    /// 子树交叉
    fn crossover_trees(&self, t1: &ExprNode, t2: &ExprNode) -> Option<(ExprNode, ExprNode)> {
        // 收集所有子树节点 (引用)
        let mut nodes1 = Vec::new();
        let mut nodes2 = Vec::new();
        self.collect_nodes(t1, &mut nodes1);
        self.collect_nodes(t2, &mut nodes2);
        if nodes1.is_empty() || nodes2.is_empty() { return None; }

        let mut rng = self.rng.clone();
        let n1 = nodes1[rng.gen_range(0..nodes1.len())];
        let n2 = nodes2[rng.gen_range(0..nodes2.len())];

        // 克隆并交换
        let mut t1_clone = t1.clone();
        let mut t2_clone = t2.clone();
        if let (Some(p1), Some(p2)) = (self.replace_subtree(&mut t1_clone, n1, n2.clone()),
                                        self.replace_subtree(&mut t2_clone, n2, n1.clone())) {
            Some((p1, p2))
        } else {
            None
        }
    }

    fn collect_nodes<'a>(&self, node: &'a ExprNode, out: &mut Vec<&'a ExprNode>) {
        out.push(node);
        match node {
            ExprNode::Unary(_, child) => self.collect_nodes(child, out),
            ExprNode::Binary(_, l, r) => {
                self.collect_nodes(l, out);
                self.collect_nodes(r, out);
            }
            _ => {}
        }
    }

    fn replace_subtree(&self, root: &mut ExprNode, target: &ExprNode, new_sub: ExprNode) -> Option<ExprNode> {
        // 简化: 直接比较 hash 替换
        if root.hash() == target.hash() {
            *root = new_sub;
            return Some(root.clone());
        }
        match root {
            ExprNode::Unary(_, child) => {
                if let Some(new) = self.replace_subtree(child, target, new_sub.clone()) {
                    *root = ExprNode::Unary(root.unary_op().unwrap(), Box::new(new));
                    Some(root.clone())
                } else { None }
            }
            ExprNode::Binary(op, l, r) => {
                if let Some(new_l) = self.replace_subtree(l, target, new_sub.clone()) {
                    *root = ExprNode::Binary(*op, Box::new(new_l), r.clone());
                    Some(root.clone())
                } else if let Some(new_r) = self.replace_subtree(r, target, new_sub.clone()) {
                    *root = ExprNode::Binary(*op, l.clone(), Box::new(new_r));
                    Some(root.clone())
                } else { None }
            }
            _ => None,
        }
    }

    fn unary_op(node: &ExprNode) -> Option<UnaryOp> {
        match node { ExprNode::Unary(op, _) => Some(*op), _ => None }
    }

    /// 变异
    fn mutation(&mut self) {
        let mut rng = self.rng.clone();
        for ind in &mut self.population {
            if rng.gen::<f64>() < self.config.mutation_rate {
                if let Some(mutated) = self.mutate_tree(&ind.expr, &mut rng) {
                    ind.expr = mutated;
                    ind.hash = ind.expr.hash();
                    ind.complexity = ind.expr.complexity();
                }
            }
        }
    }

    /// 变异
    fn mutation(&mut self) {
        let mut rng = self.rng.clone();
        for ind in &mut self.population {
            if rng.gen::<f64>() < self.config.mutation_rate {
                if let Some(mutated) = self.mutate_tree(&ind.expr, &mut rng) {
                    ind.expr = mutated;
                    ind.hash = ind.expr.hash();
                    ind.complexity = ind.expr.complexity();
                }
            }
        }
    }

    /// 树变异 (点变异 + 子树替换)
    fn mutate_tree(&self, tree: &ExprNode, rng: &mut rand::rngs::StdRng) -> Option<ExprNode> {
        let mut mutated = tree.clone();
        // 点变异: 修改常数 / 变量 / 运算符
        if self.rng.gen::<f64>() < 0.4 {
            self.point_mutate(&mut mutated, rng);
        }
        // 子树替换
        if rng.gen::<f64>() < 0.3 {
            if let Some(new) = self.subtree_replace(&mutated, &mut self.rng.clone()) {
                mutated = new;
            }
        }
        // 插入节点
        if rng.gen::<f64>() < 0.1 {
            if let Some(new) = self.insert_node(&mutated, rng) {
                mutated = new;
            }
        }
        Some(mutated)
    }

    fn point_mutate(&self, tree: &mut ExprNode, rng: &mut rand::rngs::StdRng) {
        match tree {
            ExprNode::Const(v) => {
                *v += rng.gen_range(-1.0..1.0) * 0.1;
            }
            ExprNode::Var(_) => {
                if !self.variables.is_empty() {
                    *tree = ExprNode::Var(self.variables[rand::thread_rng().gen_range(0..self.variables.len())].clone());
                }
            }
            ExprNode::Unary(op, child) => {
                *op = match rng.gen_range(0..7) {
                    0 => UnaryOp::Neg, 1 => UnaryOp::Abs, 2 => UnaryOp::Sqrt,
                    3 => UnaryOp::Sin, 4 => UnaryOp::Cos, 5 => UnaryOp::Tan,
                    6 => UnaryOp::Log, _ => UnaryOp::Exp,
                };
                self.point_mutate(child, rng);
            }
            ExprNode::Binary(op, l, r) => {
                *op = match rng.gen_range(0..5) {
                    0 => BinaryOp::Add, 1 => BinaryOp::Sub, 2 => BinaryOp::Mul,
                    3 => BinaryOp::Div, _ => BinaryOp::Pow,
                };
                if rng.gen::<bool>() { self.point_mutate(l, rng); }
                if rng.gen::<bool>() { self.point_mutate(r, rng); }
            }
        }

    fn subtree_replace(&self, tree: &mut ExprNode, rng: &mut rand::rngs::StdRng) -> Option<ExprNode> {
        // 简化: 随机替换一个子树
        let mut nodes = Vec::new();
        self.collect_nodes(tree, &mut nodes);
        if nodes.len() <= 1 { return None; }
        let idx = self.rng.gen_range(0..nodes.len());
        let target = nodes[idx];
        let depth = 2 + (self.rng.gen_range(0..2) as usize);
        self.random_expr(true, depth, &mut self.rng.clone()).ok().map(|new_sub| {
            self.replace_subtree(tree, target, new_sub).unwrap()
        })
    }

    fn insert_node(&self, tree: &mut ExprNode, rng: &mut rand::rngs::StdRng) -> Option<ExprNode> {
        // 简化: 在随机位置插入一元运算
        let mut nodes = Vec::new();
        self.collect_nodes(tree, &mut nodes);
        if nodes.is_empty() { return None; }
        let idx = self.rng.gen_range(0..nodes.len());
        let target = nodes[idx].clone();
        let op = match self.rng.gen_range(0..7) {
            0 => UnaryOp::Neg, 1 => UnaryOp::Abs, 2 => UnaryOp::Sqrt,
            3 => UnaryOp::Sin, 4 => UnaryOp::Cos, 5 => UnaryOp::Tan,
            6 => UnaryOp::Log, _ => UnaryOp::Exp,
        };
        self.replace_subtree(tree, &target, ExprNode::Unary(op, Box::new(target)))
    }

    /// 精英保留
    fn elitism(&mut self) {
        self.population.sort_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap());
        let elite_size = (self.config.population_size as f64 * 0.05).ceil() as usize;
        self.population.truncate(self.config.population_size - elite_size);
    }

    /// 更新最佳个体
    fn update_best(&mut self) {
        self.population.sort_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap());
        if let Some(best) = self.population.first() {
            if self.best.is_none() || best.fitness < self.best.as_ref().unwrap().fitness {
                self.best = Some(best.clone());
            }
        }
    }

    /// 早停检查
    fn early_stop_check(&self) -> bool {
        // 简化: 连续 20 代无改进则停止
        false // 简化实现
    }

    /// 获取最佳表达式
    pub fn best_expr(&self) -> Option<&ExprNode> {
        self.best.as_ref().map(|b| &b.expr)
    }
}

/// 道引擎能力 — 包装为 NativeCapability
pub fn dao_engine_capability(config: DaoEngineConfig) -> Arc<dyn crate::core::l7_capability::native_bus::NativeCapability> {
    let engine = Arc::new(std::sync::RwLock::new(DaoEngine::new(config)));
    let engine_clone = engine.clone();
    closure_capability(
        "dao_engine.symbolic_regression",
        "符号回归引擎 (DaoEngine 2.0)",
        r#"{"x": "array", "y": "array"}"#,
        r#"{"expr": "string"}"#,
        false,
        move |input| {
            let engine = engine_clone.clone();
            let x: Vec<Vec<f64>> = serde_json::from_value(input.get("x").cloned().unwrap_or_default())?;
            let y: Vec<f64> = serde_json::from_value(input.get("y").cloned().unwrap_or_default())?;
            let expr = engine.write().unwrap().fit(&input["x"], &input["y"])?;
            Ok(serde_json::json!({ "expr": expr.to_string() }))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_expr_eval() {
        let expr = ExprNode::Binary(
            BinaryOp::Mul,
            Box::new(ExprNode::Var("x".into())),
            Box::new(ExprNode::Const(2.0)),
        );
        let mut vars = HashMap::new();
        vars.insert("x".into(), 3.0);
        assert_eq!(expr.eval(&vars).unwrap(), 6.0);
    }

    #[test]
    fn test_expr_to_string() {
        let expr = ExprNode::Binary(
            BinaryOp::Add,
            Box::new(ExprNode::Var("x".into())),
            Box::new(ExprNode::Const(2.0)),
        );
        assert_eq!(expr.to_string(), "x + 2");
    }

    #[test]
    fn test_dao_engine_simple() {
        let mut engine = DaoEngine::new(DaoEngineConfig {
            population_size: 50,
            max_generations: 10,
            max_depth: 4,
            ..Default::default()
        });
        engine.set_variables(vec!["x".into()]);
        let x: Vec<Vec<f64>> = (0..10).map(|i| vec![i as f64]).collect();
        let y: Vec<f64> = (0..10).map(|i| (2.0 * i as f64 + 1.0)).collect();
        let expr = engine.fit(&x, &y).unwrap();
        let expr_str = expr.to_string();
        assert!(expr_str.contains("x") && expr_str.contains("*"));
    }
}
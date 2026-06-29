use super::NeEvaluator;

pub struct NeStdLib;

impl NeStdLib {
    /// Returns all standard library mappings (name → Ne S-expression)
    pub fn all() -> Vec<(&'static str, &'static str)> {
        vec![
            ("vsa-zero",   "(lambda () [0 0 0 0 0 0 0 0])"),
            ("vsa-unit",   "(lambda () [1 1 1 1 1 1 1 1])"),
            ("vsa-random", "(lambda (seed) (permute [1 2 3 4 5 6 7 8] seed))"),

            ("vsa-bind2",  "(lambda (a b) (bind a b))"),
            ("vsa-bundle2", "(lambda (a b) (bundle a b))"),
            ("vsa-negate",  "(lambda (a) (negate a))"),
            ("vsa-permute", "(lambda (a k) (permute a k))"),

            ("vsa-cosine",  "(lambda (a b) (cosine a b))"),
            ("vsa-hamming", "(lambda (a b) (hamming a b))"),
            ("vsa-similarity", "(lambda (a b) (cosine a b))"),

            ("vsa-norm",    "(lambda (v) (cosine v v))"),
            ("vsa-zero?",   "(lambda (v) (= (vsa-norm v) 0))"),
            ("vsa-equal?",  "(lambda (a b) (>= (vsa-cosine a b) 0.99))"),

            ("vsa-mean",   "(lambda (vectors) (foldl vsa-bundle2 (vsa-zero) vectors))"),
            ("vsa-sum",    "(lambda (vectors) (foldl vsa-bundle2 (vsa-zero) vectors))"),

            ("vsa-rotate",  "(lambda (v n) (permute v n))"),
            ("vsa-reverse", "(lambda (v) (permute v 2048))"),

            ("vsa-unbind",  "(lambda (a b) (bind a b))"),
            ("vsa-identity", "(lambda (v) v)"),

            ("zero", "(vsa-zero)"),
            ("unit", "(vsa-unit)"),

            ("identity", "(lambda (x) x)"),
            ("const",    "(lambda (x y) x)"),
            ("compose",  "(lambda (f g x) (f (g x)))"),

            ("head",   "(lambda (xs) (bind xs [1 0 0 0 0 0 0 0]))"),
            ("tail",   "(lambda (xs) (bind xs [0 1 1 1 1 1 1 1]))"),
            ("empty?", "(lambda (xs) (vsa-zero? xs))"),

            ("inc",   "(lambda (n) (+ n 1))"),
            ("dec",   "(lambda (n) (- n 1))"),
            ("double", "(lambda (n) (* n 2))"),
            ("half",   "(lambda (n) (/ n 2))"),
            ("square", "(lambda (n) (* n n))"),
            ("abs",    "(lambda (n) (if (< n 0) (- n) n))"),
            ("neg",    "(lambda (n) (- n))"),

            ("not",  "(lambda (x) (if x 0 1))"),
            ("and",  "(lambda (a b) (if a (if b 1 0) 0))"),
            ("or",   "(lambda (a b) (if a 1 (if b 1 0)))"),

            ("gt?",  "(lambda (a b) (> a b))"),
            ("lt?",  "(lambda (a b) (< a b))"),
            ("eq?",  "(lambda (a b) (= a b))"),
            ("gte?", "(lambda (a b) (or (> a b) (= a b)))"),
            ("lte?", "(lambda (a b) (or (< a b) (= a b)))"),

            ("sum",     "(lambda (list) (foldl + 0 list))"),
            ("product", "(lambda (list) (foldl * 1 list))"),
            ("max",     "(lambda (list) (foldl (lambda (a b) (if (> a b) a b)) 0 list))"),
            ("min",     "(lambda (list) (foldl (lambda (a b) (if (< a b) a b)) 0 list))"),
            ("length",  "(lambda (list) (foldl (lambda (acc x) (+ acc 1)) 0 list))"),

            ("map",     "(lambda (f list) (foldl (lambda (acc x) (bundle acc (f x))) (vsa-zero) list))"),
            ("filter",  "(lambda (pred list) (foldl (lambda (acc x) (if (pred x) (bundle acc x) acc)) (vsa-zero) list))"),
            ("foreach", "(lambda (f list) (foldl (lambda (acc x) (do acc (f x))) (vsa-zero) list))"),
        ]
    }

    /// Get a specific stdlib function by name
    pub fn get(name: &str) -> Option<&'static str> {
        Self::all()
            .into_iter()
            .find(|(n, _)| *n == name)
            .map(|(_, expr)| expr)
    }

    /// Register stdlib into the evaluator's environment
    pub fn register(evaluator: &mut NeEvaluator) -> usize {
        let mut count = 0;
        for (name, expr) in Self::all() {
            if evaluator.register_fun(name, expr).is_ok() {
                count += 1;
            }
        }
        count
    }
}

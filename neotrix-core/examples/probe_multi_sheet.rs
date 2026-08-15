use neotrix::neotrix::{read_xlsx_sheets_all, normalize_column_name};

fn main() {
    let dir = "/Users/neo/Downloads/5月份价格表";
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("").to_lowercase();
        if ext != "xlsx" { continue; }
        let name = p.file_name().unwrap().to_string_lossy().to_lowercase();
        if name.starts_with("consolidated") || name.starts_with("native_consolidated") { continue; }
        let Ok(tables) = read_xlsx_sheets_all(&p) else { continue };
        if tables.len() <= 1 { continue; }
        // 抽取每个 sheet 的产品型号集合 (用"产品型号"列或第3列), 计算 sheet 间重叠
        let mut sets: Vec<std::collections::HashSet<String>> = Vec::new();
        for t in &tables {
            let idx = t.headers.iter().position(|h| normalize_column_name(h) == "产品型号");
            let set: std::collections::HashSet<String> = t.rows.iter()
                .filter_map(|r| idx.and_then(|i| r.get(i)).map(|s| s.trim().to_string()))
                .filter(|s| !s.is_empty())
                .collect();
            sets.push(set);
        }
        // 全量并集 vs 各 sheet 计数
        let union: std::collections::HashSet<String> = sets.iter().flatten().cloned().collect();
        let sum: usize = sets.iter().map(|s| s.len()).sum();
        println!("{}: sheets={} 型号并集={} 型号Σ={} 重复率={:.0}%",
            p.file_name().unwrap().to_string_lossy(),
            tables.len(), union.len(), sum,
            if sum>0 {(1.0 - union.len() as f64 / sum as f64)*100.0} else {0.0});
    }
}

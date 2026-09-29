#[test]
fn probe_act_selftests() {
    let results = neotrix::l6_meta::healing::nt_core_self_test_integration::run_lightweight_self_tests();
    for r in &results {
        let n = r.name.as_str();
        if n.starts_with("nt_act") || n.starts_with("nt_agent") {
            println!("ACT[{}] passed={} fails={:?}", n, r.passed, r.failures);
        }
    }
    let act = results
        .iter()
        .filter(|r| r.name.starts_with("nt_act"))
        .count();
    let act_pass = results
        .iter()
        .filter(|r| r.name.starts_with("nt_act") && r.passed)
        .count();
    println!("ACT health would be: {}/{}", act_pass, act);
}

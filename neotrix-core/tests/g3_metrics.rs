//! W3 度量实测 — T4: FieldLedger 跨进程 OS 级哈希一致 (灵境协议6 多进程共识)
//!
//! 父测试起 3 个真实 OS 子进程 (同一测试二进制经 env 开关进入 worker 模式),
//! 各自 stage+tick 若干条; join 后父进程断言三条独立连接视角的
//! field_version / field_head_hash 完全一致, 哈希链整链回放通过,
//! 终态等于可交换合并的多重集最大值期望。
//!
//! 防递归: worker 是独立 #[test], 第一行检查 env 未设则立即 return ——
//! 常规全量跑测时 worker 为空操作; 父进程经 `--exact` 让子进程只运行 worker 自身。

use neotrix::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;
use neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_field_ledger;

/// worker 模式开关 env (存在即 worker 进程)
const ENV_WORKER: &str = "G3_T4_WORKER_ID";
/// worker 目标库路径 env
const ENV_DB: &str = "G3_T4_DB_PATH";

const WRITER_COUNT: usize = 3;
const STAGES_PER_WRITER: usize = 12;
const KEY_SPACE: usize = 4;

fn unique_db_path(tag: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "g3_t4_{}_{}_{}.db",
        tag,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ))
}

fn head_of(kb: &KnowledgeBase) -> (u64, String) {
    let conn = kb.raw_conn().expect("raw_conn");
    (
        nt_field_ledger::field_version(&conn).expect("field_version"),
        nt_field_ledger::field_head_hash(&conn).expect("field_head_hash"),
    )
}

/// worker 进程入口。第一行必须是 env 守卫 (未设则空操作返回)。
#[test]
fn t4_worker_os_writer() {
    let Ok(writer) = std::env::var(ENV_WORKER) else {
        return; // 非 worker 模式 (常规跑测 / 递归防护): 空操作
    };
    let db = std::env::var(ENV_DB).expect("worker 需要 DB 路径");
    let kb = KnowledgeBase::open(Some(db.into())).expect("worker 打开共享库");

    for i in 0..STAGES_PER_WRITER {
        let key = format!("k{}", i % KEY_SPACE);
        let value = format!("{:0>4}_{}", i, writer);
        kb.field_stage("g3t4", &key, &value, &writer)
            .expect("worker stage");
        if i % 5 == 4 {
            kb.field_tick().expect("worker 中途机会性求解");
        }
    }
    let receipt = kb.field_tick().expect("worker 行尾求解");
    let (version, hash) = head_of(&kb);
    println!(
        "[worker {}] drained={:?} view_version={} view_head={}",
        writer,
        receipt.map(|r| r.drained),
        version,
        hash
    );
}

#[test]
fn t4_cross_process_os_level_hash_identity() {
    let exe = std::env::current_exe().expect("current_exe");
    let db = unique_db_path("parent");

    // 起 3 个真实 OS 子进程, 各带自己的 writer id 与同一 db 路径
    let mut children = Vec::new();
    for wid in 0..WRITER_COUNT {
        let child = std::process::Command::new(&exe)
            .args(["--exact", "t4_worker_os_writer", "--nocapture"])
            .env(ENV_WORKER, format!("os_w{}", wid))
            .env(ENV_DB, &db)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("spawn worker {}: {}", wid, e));
        children.push((wid, child));
    }

    for (wid, mut child) in children {
        let out = child.wait_with_output().expect("wait worker");
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            out.status.success(),
            "worker {} 失败\nstdout:\n{}\nstderr:\n{}",
            wid,
            stdout,
            stderr
        );
        println!("[parent] worker {} exit=ok\n{}", wid, stdout.trim());
    }

    // 三条独立连接各自打开同一库 (多实例同库 = G1 的真实部署形态)
    let view_a = KnowledgeBase::open(Some(db.clone())).expect("view a");
    let view_b = KnowledgeBase::open(Some(db.clone())).expect("view b");
    let view_c = KnowledgeBase::open(Some(db)).expect("view c");

    view_a.field_tick().expect("兜底清暂存"); // 幂等: 各 worker 已行尾求解

    let (va, ha) = head_of(&view_a);
    let (vb, hb) = head_of(&view_b);
    let (vc, hc) = head_of(&view_c);

    println!("=== T4 白皮书数据 ===");
    println!("view_a(version={}, head={})", va, ha);
    println!("view_b(version={}, head={})", vb, hb);
    println!("view_c(version={}, head={})", vc, hc);

    assert_eq!(va, vb, "三进程视角版本号一致");
    assert_eq!(va, vc);
    assert_eq!(ha, hb, "三进程视角头哈希一致 (OS 进程级确定性)");
    assert_eq!(ha, hc);
    assert!(
        va >= WRITER_COUNT as u64,
        "至少每个写者各贡献一个版本, 实际 {}",
        va
    );

    for view in [&view_a, &view_b, &view_c] {
        assert!(view.field_verify_chain().expect("verify"), "整链回放通过");
    }

    // 终态 = 可交换合并的多重集最大值 (与 tick 划分无关)
    for k in 0..KEY_SPACE {
        let key = format!("k{}", k);
        let mut candidates: Vec<String> = Vec::new();
        for wid in 0..WRITER_COUNT {
            for i in 0..STAGES_PER_WRITER {
                if i % KEY_SPACE == k {
                    candidates.push(format!("{:0>4}_os_w{}", i, wid));
                }
            }
        }
        candidates.sort();
        let expected = candidates.last().expect("非空候选").clone();
        assert_eq!(
            view_a.kv_get("g3t4", &key).expect("kv_get").as_deref(),
            Some(expected.as_str()),
            "key {} 终态等于多重集最大值",
            key
        );
        print!("merged[{}]={} ", key, expected);
    }
    println!();
}

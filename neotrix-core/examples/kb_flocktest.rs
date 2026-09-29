//! flock 自锁微复现：bundled sqlite 是否占 flock，导致 fs2 锁自死锁。
//!
//! 运行: cargo run --example kb_flocktest（只编 example，lib 不动）。
//! 全程 temp 文件，无副作用。

use fs2::FileExt;
use std::fs::OpenOptions;
use std::io::Write as _;

fn probe(label: &str, f: &std::fs::File) {
    match f.try_lock_exclusive() {
        Ok(()) => {
            eprintln!("[flocktest] {label}: try_lock OK (no holder)");
            let _ = f.unlock();
        }
        Err(e) => eprintln!("[flocktest] {label}: try_lock ERR kind={e:?}"),
    }
    let _ = std::io::stderr().flush();
}

fn main() {
    let dir = std::env::temp_dir().join(format!("neotrix_flocktest_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let db = dir.join("t.db");

    // S1: 单连接 idle（对标 python T1）
    let conn = rusqlite::Connection::open(&db).expect("open");
    conn.execute_batch("CREATE TABLE t(x)").expect("ddl");
    let f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&db)
        .expect("reopen");
    probe("S1 single-conn-idle", &f);

    // S2: 加第二连接（对标 ledger）
    let conn2 = rusqlite::Connection::open(&db).expect("open2");
    conn2.execute_batch("CREATE TABLE t2(x)").expect("ddl2");
    probe("S2 two-conn-idle", &f);

    // S3: 主连接开写事务未提交（对标 python T2）
    conn.execute_batch("BEGIN IMMEDIATE; INSERT INTO t VALUES (1)")
        .expect("begin+ins");
    probe("S3 main-write-txn-open", &f);
    conn.execute_batch("ROLLBACK").expect("rb");
    probe("S4 after-rollback", &f);

    // S5: WAL 模式是否改变行为
    conn.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE t3(x)")
        .expect("wal");
    probe("S5 wal-mode-idle", &f);

    eprintln!("[flocktest] done");
}

//! `NEOBOT_DATA_DIR` 的进程隔离测试。
//!
//! ⛔ **为什么必须在 `tests/` 而不是 `#[cfg(test)]` 里**：改进程级环境变量
//!    会影响**所有并行跑的测试**。上一版放在单元测试里，用锁只串行了本模块
//!    那几条，另外 19 条（`open_store` 间接调 `data_dir`）照跑照看见临时
//!    目录 ⇒ 3 条失败。**锁只能保护自己那几条，保护不了别人。**
//!    `tests/` 是独立测试二进制 ⇒ 天然只有一个进程、只有本文件的测试。

use neobot_desktop::commands::data_dir;

/// 把本文件**全部**测试串起来。
///
/// ⛔ 第一版只想着「换进程就安全了」，结果这 3 条测试**自己**并行跑，
///    都在改 `NEOBOT_DATA_DIR` ⇒ `还原后回到默认` 会在别人设着值的时候跑，
///    于是失败。⇒ 「进程隔离」解决的是**别的测试看不见**，
///    解决不了**本进程内**的互相踩。后者靠锁，而这里可以靠，
///    因为这个二进制里的测试**全都是我的**。
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn serial<T>(f: impl FnOnce() -> T) -> T {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    f()
}

/// ⛔ 旧硬编码版的病：只认 `~/.neobot`，于是设了环境变量后
///    **CLI 与桌面端打开两个不同的库** —— 桌面建了会话、CLI 看不见。
#[test]
fn 设了环境变量时桌面端必须跟着走() {
    serial(|| {
    let custom = "/tmp/nb-custom-data-dir-xyz";
    std::env::set_var("NEOBOT_DATA_DIR", custom);
    let got = data_dir().expect("解析数据目录");
    std::env::remove_var("NEOBOT_DATA_DIR");
    assert_eq!(
        got,
        std::path::PathBuf::from(custom),
        "设了 NEOBOT_DATA_DIR 却仍指向别处 ⇒ 与 CLI 打开了两个库"
    );
    })
}

#[test]
fn 与库的解析结果一致() {
    serial(|| {
    std::env::set_var("NEOBOT_DATA_DIR", "/tmp/nb-lib-parity");
    let app = data_dir().expect("桌面端");
    let lib = neotrix_neobot::NeobotConfig::from_env().expect("库侧").data_dir;
    std::env::remove_var("NEOBOT_DATA_DIR");
    assert_eq!(app, lib, "两端必须给出同一个数据目录");
    })
}

#[test]
fn 还原后回到默认() {
    serial(|| {
    let _ = std::env::remove_var("NEOBOT_DATA_DIR");
    let d = data_dir().expect("解析");
    assert!(d.ends_with(".neobot"), "实际：{d:?}");
    })
}

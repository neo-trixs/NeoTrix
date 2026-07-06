//! 进度指示器 — 基于 indicatif 的轻量包装
//!
//! 用于长时间操作（知识注入、爬取、推理）的终端进度反馈。
//! CLI 的视觉反馈层, 弥合"无输出"到"有进度"的差距。

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct Spinner {
    bar: ProgressBar,
}

impl Spinner {
    pub fn new(msg: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("spinner template should be valid"),
        );
        bar.set_message(msg.to_string());
        bar.enable_steady_tick(Duration::from_millis(100));
        Self { bar }
    }

    pub fn set_message(&self, msg: &str) {
        self.bar.set_message(msg.to_string());
    }

    pub fn done(&self) {
        self.bar.finish_with_message("done");
    }

    pub fn fail(&self, msg: &str) {
        self.bar.finish_with_message(format!("failed: {msg}"));
    }
}

pub struct ProgressCounter {
    bar: ProgressBar,
}

impl ProgressCounter {
    pub fn new(total: u64, msg: &str) -> Self {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .expect("progress template should be valid")
                .progress_chars("#>-"),
        );
        bar.set_message(msg.to_string());
        bar.enable_steady_tick(Duration::from_millis(200));
        Self { bar }
    }

    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    pub fn set_message(&self, msg: &str) {
        self.bar.set_message(msg.to_string());
    }

    pub fn done(&self) {
        self.bar.finish_with_message("done");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_create() {
        let s = Spinner::new("testing");
        s.done();
    }

    #[test]
    fn test_spinner_set_message() {
        let s = Spinner::new("start");
        s.set_message("working");
        s.done();
    }

    #[test]
    fn test_spinner_fail() {
        let s = Spinner::new("task");
        s.fail("error occurred");
    }

    #[test]
    fn test_progress_counter_create() {
        let p = ProgressCounter::new(100, "loading");
        p.inc(50);
        p.done();
    }

    #[test]
    fn test_progress_counter_partial() {
        let p = ProgressCounter::new(10, "steps");
        for _ in 0..5 {
            p.inc(1);
        }
        p.done();
    }
}

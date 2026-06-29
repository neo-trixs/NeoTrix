use super::{NeEvaluator, NeValue};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EvalTraceEntry {
    pub step: u64,
    pub operation: String,
    pub args_summary: String,
    pub result_summary: String,
    pub timestamp: u64,
}

impl NeEvaluator {
    pub fn record_trace(&mut self, op: &str, args: &[NeValue], result: &Result<NeValue, String>) {
        if self.trace.len() >= self.max_trace {
            self.trace.remove(0);
        }
        let result_summary = match result {
            Ok(v) => format!("{}", v),
            Err(e) => format!("error:{}", e),
        };
        let args_summary = if args.is_empty() {
            "()".to_string()
        } else {
            args.iter()
                .map(|a| format!("{}", a))
                .collect::<Vec<_>>()
                .join(", ")
        };
        self.trace.push(EvalTraceEntry {
            step: self.step_count,
            operation: op.to_string(),
            args_summary,
            result_summary,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });
    }

    pub fn get_trace(&self) -> &[EvalTraceEntry] {
        &self.trace
    }

    pub fn clear_trace(&mut self) {
        self.trace.clear();
    }

    pub fn trace_report(&self) -> String {
        let recent: Vec<String> = self
            .trace
            .iter()
            .rev()
            .take(5)
            .map(|e| {
                format!(
                    "#{}: {}({}) -> {}",
                    e.step, e.operation, e.args_summary, e.result_summary
                )
            })
            .collect();
        format!(
            "trace:{}_entries|recent:{}",
            self.trace.len(),
            recent.join("; ")
        )
    }

    pub fn eval_trace_query(&self) -> String {
        self.trace_report()
    }
}

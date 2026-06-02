use crate::neotrix::nt_core_parallel::executor::{ParallelExecutor, ExecMode};
use crate::neotrix::nt_core_parallel::types::Task;

pub struct WorkerNode {
    executor: ParallelExecutor,
}

impl Default for WorkerNode {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkerNode {
    pub fn new() -> Self {
        Self { executor: ParallelExecutor::new(4) }
    }

    /// 将任务转译为 shell 命令并真实执行
    /// task.agent_id 作为命令字符串，task.input 作为参数（Vec<f64> 解码为 ASCII）
    /// 返回 (stdout, stderr) 对
    pub fn execute_tasks(&mut self, tasks: &[Task]) -> Vec<Result<(String, String), String>> {
        self.executor.set_mode(ExecMode::Parallel);
        tasks.iter().map(|task| {
            let cmd = if !task.input.is_empty() {
                ParallelExecutor::decode_command(&task.input)
            } else {
                task.agent_id.clone()
            };
            if cmd.is_empty() {
                return Err("空命令".to_string());
            }
            match ParallelExecutor::execute_shell(&cmd) {
                Ok(stdout) => Ok((stdout, String::new())),
                Err(e) => {
                    // 分离 stdout 和 stderr（在错误信息中）
                    if let Some(stderr) = e.split("stderr=").nth(1) {
                        Ok((String::new(), stderr.to_string()))
                    } else {
                        Err(e)
                    }
                }
            }
        }).collect()
    }
}

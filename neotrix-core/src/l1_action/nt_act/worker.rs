use super::types::Task;

#[derive(Debug, Clone, Copy)]
pub enum ExecMode { Sequential, Parallel }

pub struct ParallelExecutor {
    _max_agents: usize,
    mode: ExecMode,
}

impl ParallelExecutor {
    pub fn new(max_agents: usize) -> Self {
        Self { _max_agents: max_agents, mode: ExecMode::Sequential }
    }

    pub fn set_mode(&mut self, mode: ExecMode) { self.mode = mode; }

    pub fn decode_command(input: &[f64]) -> String {
        input.iter()
            .take(64)
            .map(|&b| (b as u8).clamp(32, 126) as char)
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    fn has_shell_metachars(s: &str) -> bool {
        s.contains(';') || s.contains('|') || s.contains('`') ||
        s.contains('$') || s.contains("&&") || s.contains("||")
    }

    pub fn execute_shell(command: &str) -> Result<String, String> {
        let allowed = ["echo", "ls", "cat", "pwd", "date", "whoami", "uname", "head", "tail", "wc", "sort"];
        let cmd_name = command.split_whitespace().next().unwrap_or("");
        if !allowed.contains(&cmd_name) {
            return Err(format!("command '{}' not in whitelist", cmd_name));
        }
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err("empty command".to_string());
        }
        for arg in &parts[1..] {
            if Self::has_shell_metachars(arg) {
                return Err(format!("arg contains shell metacharacters: '{}'", arg));
            }
        }
        let mut cmd = std::process::Command::new(parts[0]);
        for arg in &parts[1..] {
            cmd.arg(arg);
        }
        let output = cmd.output()
            .map_err(|e| format!("exec failed: {}", e))?;
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if output.status.success() {
            Ok(stdout)
        } else {
            Err(format!("exit={} stderr={}", output.status.code().unwrap_or(-1), stderr))
        }
    }
}

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

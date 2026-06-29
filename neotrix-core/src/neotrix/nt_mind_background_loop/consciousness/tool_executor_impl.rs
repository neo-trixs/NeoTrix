//! Implements core::nt_core_traits::ToolExecutor for neotrix layer tools.
//! Breaks the circular core↔neotrix import for tool_orchestrator.

use crate::core::nt_core_traits::ToolExecutor;

pub struct NeotrixToolExecutor;

impl ToolExecutor for NeotrixToolExecutor {
    fn web_search(&self, query: &str) -> (String, bool) {
        let r = crate::neotrix::nt_tools::tool_websearch(query);
        (r.output, r.success)
    }

    fn web_fetch(&self, url: &str) -> (String, bool) {
        let r = crate::neotrix::nt_tools::tool_webfetch(url);
        (r.output, r.success)
    }

    fn file_read(&self, path: &str) -> (String, bool) {
        let r = crate::neotrix::nt_tools::tool_read(path);
        (r.output, r.success)
    }

    fn file_write(&self, path: &str, content: &str) -> (String, bool) {
        let r = crate::neotrix::nt_tools::tool_write(path, content);
        (r.output, r.success)
    }

    fn file_edit(&self, path: &str, old: &str, new: &str) -> (String, bool) {
        let r = crate::neotrix::nt_tools::tool_edit(path, old, new);
        (r.output, r.success)
    }

    fn bash(&self, cmd: &str) -> (String, bool) {
        let r = crate::neotrix::nt_tools::tool_bash(cmd);
        (r.output, r.success)
    }

    fn glob(&self, pattern: &str) -> (String, bool) {
        let r = crate::neotrix::nt_tools::tool_glob(pattern);
        (r.output, r.success)
    }

    fn grep(&self, pattern: &str, path: &str) -> (String, bool) {
        let r = crate::neotrix::nt_tools::tool_grep(pattern, path);
        (r.output, r.success)
    }
}

# 选择性暂存 patch 纪律（2026-09-24 实测结论）

本仓 `git apply --cached` 手写 patch 多次翻车，实测结论：

1. **hunk 必须带尾随上下文**（≥1 行，最好 3 行）。以 `+` 行结尾的 hunk 在此稳定复现失败
   （对照：同样内容加尾随上下文即过；`git diff` 自产 patch 恒过）。
2. **文件必须以换行结尾**，否则 `corrupt patch`。
3. 空行上下文用**真空行**（git 自产格式），不要补空格。
4. hunk 头行号数到行再写；多人同文件时先 `git show :<file>` 对 index 取上下文，
   不要对 worktree 取（worktree 含他人未提交行，行号漂移）。
5. 共享热文件（根 Cargo.toml 等）首选 **blob 手术**：
   `git show :f > base` → python 精确替换（assert count==1）→
   `git hash-object -w` + `git update-index --cacheinfo` → `git diff --cached` 核对。
   此路径在本仓 100% 可靠。

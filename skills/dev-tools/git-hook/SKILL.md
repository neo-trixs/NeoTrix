---
name: git-hook
description: 安装/卸载 NeoTrix 的 git hooks（pre-commit 等）
when_to_use: 新克隆仓库后需要装上钩子，或排查钩子为何未生效时
tools:
  - dev-tools/git-hook/git-hook.sh
  - dev-tools/git-hook/no-main-direct.sh
# 注意：Rust 侧 `nt_act_dev_tools/git_hook.rs` 会把 no-main-direct.sh
# symlink 进 .git/hooks —— 改这个文件会同时影响两条路径。
---

# Git Hook

## Purpose
Git 钩子管理：TODO 同步检查 + 分支保护

## Trigger Words
- git hook
- pre-commit
- pre-push
- git hooks
- 安装钩子

## Workflow

### Install Hooks
```bash
chmod +x scripts/git-hook.sh
ln -sf ../../scripts/git-hook.sh .git/hooks/post-commit
```

### Hook Logic
1. 检查 TODO.md 和 TODO.yml 同步状态
2. 运行 `neotrix todo sync`
3. 分支保护（no-main-direct.sh）

## Branch Protection
`no-main-direct.sh` 实现:
- 禁止直接提交到 main/master
- 强制创建 feature 分支
- 检查 commit message 格式

## Makefile Integration
```makefile
install-hook:
	@chmod +x scripts/git-hook.sh
	@ln -sf ../../scripts/git-hook.sh .git/hooks/post-commit
```

## Rust Implementation
`nt_act_dev_tools/git_hook.rs` 提供等效功能

# Per-Session Git Isolation Scripts

These scripts enforce git isolation so concurrent NeoTrix sessions stop clobbering each other's uncommitted work.

## Problem

One session keeps resetting the main working tree (`/Users/neo/Downloads/neotrix`) to HEAD, wiping uncommitted changes from other sessions. The fix is **worktree isolation + branch isolation + merge-to-main only when green**. An isolation home already exists at `/Users/neo/Downloads/neotrix-concurrent` on branch `concurrent-wip`.

## Scripts

### `no-main-direct.sh` (commit/push guard hook)

Refuses any direct commit or push to `main` / `master`, telling the user to work on a feature branch and merge only when CI is green.

**Install (option A — repo-wide hook path):**

```sh
git config core.hooksPath scripts
```

**Install (option B — copy into .git):**

```sh
cp scripts/no-main-direct.sh .git/hooks/pre-commit
cp scripts/no-main-direct.sh .git/hooks/pre-push
chmod +x .git/hooks/pre-commit .git/hooks/pre-push
```

### `session-start.sh` (bootstrap an isolated session)

Prints (does not run) the commands to create or reuse a worktree at `/Users/neo/Downloads/neotrix-<name>` on branch `<name>-wip`, plus the `cd` to use it.

```sh
./scripts/session-start.sh mytask
# -> git worktree add -b mytask-wip /Users/neo/Downloads/neotrix-mytask
# -> cd /Users/neo/Downloads/neotrix-mytask
```

Run the printed commands, then do all your work inside that directory. A `git reset` there only affects your own tree.

## Workflow

1. Start each session: `./scripts/session-start.sh <name>` and run the printed `cd`.
2. Commit freely on `<name>-wip` inside the worktree.
3. The `no-main-direct` hook blocks accidental commits/pushes to `main`.
4. When CI is green, merge `<name>-wip` into `main` via PR / `git merge` from a trusted session.

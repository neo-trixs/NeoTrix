# GNHF Absorbed — Pattern #78 (4k★)

## Source: https://github.com/kunchenguid/gnhf (4k★, 302 forks, 152 commits)

### What GNHF Is
Autonomous overnight coding agent orchestrator. One command starts a loop: each iteration makes one small committed change, you wake up to clean work.

---

## Universal Patterns Extracted

### Pattern 1: Iterative Commit-Rollback Loop
```
success → git commit + append notes.md
failure → git reset --hard (except commit failures: preserve uncommitted)
3 consecutive failures → abort
```
**NeoTrix Integration**: SEAL pipeline as state machine with commit/rollback semantics. Each SEAL phase = one iteration.

### Pattern 2: Shared Memory (notes.md)
- Each iteration appends findings to `notes.md`
- Next iteration reads `notes.md` for cross-iteration context
- No append-only history bloat — structured append only on success
**NeoTrix Integration**: ConsciousnessTree notes persistence, KB experience namespace.

### Pattern 3: Failure Classification
- **Retryable**: agent errors with exponential backoff
- **Permanent**: abort immediately (e.g., low credit balance)
- **Rate limit**: wait until reset time, retry same iteration
- **Commit failure**: preserve work, next iteration repairs
**NeoTrix Integration**: nt_shield failure classification, retry policies.

### Pattern 4: Rate Limit Handling
```
if rate limited:
    rollback iteration
    wait until provider reset time
    retry same iteration (no failure count)
    max-rate-limit-wait budget → abort if exceeded
```
**NeoTrix Integration**: Universal model interface rate limit management.

### Pattern 5: Worktree Isolation for Parallel Agents
```
repo/                          ← unchanged
repo-gnhf-worktrees/
  ├── slug-1/                  ← agent 1
  └── slug-2/                  ← agent 2
```
**NeoTrix Integration**: consciousness task parallel execution via worktrees.

### Pattern 6: Structured Agent Output Schema
```json
{
  "success": true,
  "summary": "What was done",
  "key_changes_made": ["file1.rs", "file2.rs"],
  "key_learnings": ["pattern X works"],
  "commit_message": "fix: ...",
  "should_fully_stop": false
}
```
**NeoTrix Integration**: SEAL phase output schema, consciousness task output.

### Pattern 7: Companion Mode (Steer/Review)
- Host agent orchestrates, GNHF executes
- Poll active process, intervene when wrong thing optimized
- Review findings = next acceptance criteria
**NeoTrix Integration**: NT-CORE supervises NT-ACT execution.

### Pattern 8: Graceful Interrupts
```
1st Ctrl+C → graceful stop (let current iteration finish)
2nd Ctrl+C → force stop immediately
SIGTERM → force stop immediately
```
**NeoTrix Integration**: consciousness task interrupt handling.

### Pattern 9: Exit Summary
- Permanent stdout summary with branch, time, iterations, tokens, diff stats
- Review commands included
- Uncommitted work warning if pending
**NeoTrix Integration**: SEAL pipeline exit report, session summary.

### Pattern 10: Sleep Prevention
- macOS: `caffeinate`
- Linux: `systemd-inhibit` (re-exec)
- Windows: `SetThreadExecutionState` PowerShell helper
**NeoTrix Integration**: Long-running task keepalive for overnight runs.

---

## NeoTrix Architecture Integration Points

| GNHF Pattern | NeoTrix Module | Integration |
|-------------|----------------|-------------|
| Iterative loop | nt_core::seal_pipeline | Phase commit/rollback |
| notes.md | nt_memory::kv_store | Cross-iteration persistence |
| Failure classification | nt_shield::error_classifier | Retry/abort policies |
| Rate limit wait | nt_io::universal_model | Provider rate management |
| Worktree isolation | nt_act::parallel_task | Multi-agent execution |
| Structured output | nt_core::consciousness_task | Task output schema |
| Companion mode | nt_core::supervisor | Host/worker delegation |
| Interrupt handling | nt_meta::interrupt | Graceful shutdown |
| Exit summary | nt_mind::session_report | Run statistics |
| Sleep prevention | nt_physical::keepalive | Long-running task support |

---

## Total Sources Absorbed: 78

# Targeted Research #495 — Internal Pain Points (Iteration 49)

**Date**: 2026-09-12
**Scope**: neotrix-core/src/ — hardcoded returns, dead stubs, silent failures

---

## Pain Point 1: Cache L2 Disk Read is a No-Op

**File**: `neotrix-core/src/l1_action/nt_act/actions/nt_act_cache.rs:194-201`
**Severity**: P1

**What's wrong**: The `get()` method has an L2 (disk) cache branch that does literally nothing. When L1 memory misses, L2 is checked but never reads from disk. The bloom filter penetration protection is also unimplemented. Every L2 lookup silently falls through, making the two-tier cache architecture decorative.

```rust
// Current (broken):
if let Some(ref mut _disk_cache) = self.l2_cache {
    // TODO: 实际从磁盘读取
}
if self.config.penetration_protection {
    // TODO: 实现布隆过滤器
}
```

**Fix sketch**:
```rust
if let Some(ref mut disk_cache) = self.l2_cache {
    if let Some(entry) = disk_cache.get(key) {
        if !entry.is_expired() {
            self.stats.hits += 1;
            self.update_hit_rate();
            // Promote to L1
            self.l1_cache.insert(key.to_string(), entry.clone());
            return CacheResult::Hit(entry);
        }
    }
}
if self.config.penetration_protection {
    if self.bloom.check(key) {
        // Bloom says maybe → proceed to L2 (already checked above)
    } else {
        // Bloom says no → skip L2, guaranteed miss
        self.stats.misses += 1;
        self.update_hit_rate();
        return CacheResult::Miss;
    }
}
```

---

## Pain Point 2: VideoObjectStorage::download Returns Empty Vec

**File**: `neotrix-core/src/l1_action/nt_act/actions/video_object_storage.rs:174-178`
**Severity**: P1

**What's wrong**: `download()` returns `Some(vec![])` — callers receive a success signal but zero bytes. This is a silent data loss vector: any downstream consumer trusting the `Some` result processes empty data without error.

```rust
// Current (dangerous):
pub fn download(&mut self, object_id: &str) -> Option<Vec<u8>> {
    if let Some(_object) = self.objects.get(object_id) {
        self.stats.total_downloads += 1;
        // TODO: 实际的下载逻辑
        Some(vec![])
    } else {
        None
    }
}
```

**Fix sketch**:
```rust
pub fn download(&mut self, object_id: &str) -> Option<Vec<u8>> {
    if let Some(object) = self.objects.get(object_id) {
        self.stats.total_downloads += 1;
        if let Some(ref data) = object.data {
            Some(data.clone())
        } else {
            // Object metadata exists but data not in memory — try disk fallback
            let path = std::path::Path::new(&self.config.storage_path)
                .join(&object.key);
            std::fs::read(&path).ok()
        }
    } else {
        None
    }
}
```

---

## Pain Point 3: ProductionOrchestrator Checkpoint Save/Restore is a No-Op

**File**: `neotrix-core/src/l1_action/nt_act/actions/production_orchestrator.rs:224-242`
**Severity**: P1

**What's wrong**: `save_checkpoint()` silently returns `Ok(())` without writing anything. `restore_from_checkpoint()` just flips status to `Paused` without restoring state. The entire checkpoint-recovery mechanism — critical for long-running production workflows — is non-functional. A crash mid-workflow loses all progress.

```rust
// Current (both broken):
pub fn save_checkpoint(&self, workflow_id: &str) -> Result<(), String> {
    if let Some(workflow) = self.workflows.get(workflow_id) {
        // TODO: 实际保存检查点到持久化存储
        let _ = workflow;
        Ok(())  // ← silently does nothing
    } else { Err("工作流不存在".to_string()) }
}

pub fn restore_from_checkpoint(&mut self, workflow_id: &str) -> Result<(), String> {
    // TODO: 实际从持久化存储恢复
    if let Some(workflow) = self.workflows.get_mut(workflow_id) {
        workflow.status = WorkflowStatus::Paused;  // ← no state restored
        Ok(())
    } else { Err("工作流不存在".to_string()) }
}
```

**Fix sketch**:
```rust
pub fn save_checkpoint(&self, workflow_id: &str) -> Result<(), String> {
    if let Some(workflow) = self.workflows.get(workflow_id) {
        let checkpoint = serde_json::to_vec_pretty(workflow)
            .map_err(|e| format!("序列化失败: {}", e))?;
        let path = std::path::Path::new(&self.checkpoint_dir)
            .join(format!("{}.json", workflow_id));
        std::fs::create_dir_all(&self.checkpoint_dir)
            .map_err(|e| e.to_string())?;
        std::fs::write(&path, checkpoint)
            .map_err(|e| format!("写入失败: {}", e))?;
        Ok(())
    } else {
        Err("工作流不存在".to_string())
    }
}
```

---

## Summary

| # | Location | Issue | Severity |
|---|----------|-------|----------|
| 1 | `nt_act_cache.rs:194-201` | L2 disk read + bloom filter are empty stubs | P1 |
| 2 | `video_object_storage.rs:174-178` | `download()` returns `Some(vec![])` — silent data loss | P1 |
| 3 | `production_orchestrator.rs:224-242` | Checkpoint save/restore does nothing | P1 |

**Total issues in codebase**: ~85 TODO/FIXME stubs found, 3 critical ones reported above.

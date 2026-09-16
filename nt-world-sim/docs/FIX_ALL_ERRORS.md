# Compilation Fix Report

**Date**: 2026-09-14
**Target**: `nt-world-sim` crate

## Summary

- **Total errors found**: 5 (test-only, `--all-targets`)
- **Errors fixed**: 5
- **Pre-existing test failure**: 1 (logic bug, not compilation)
- **Status**: All compilation errors resolved

## Errors Fixed by Category

### 1. API Mismatch — ResourceManager (2 errors)

**File**: `src/engine/test_new_modules.rs:35-36`

| Error | Before | After |
|-------|--------|-------|
| `E0599` — no method `add` on `ResourceManager` | `manager.add("test", 42i32)` | `manager.insert("test", 42i32, size_of::<i32>())` |
| `E0308` — expected `ResourceHandle`, got `&{integer}` | `assert_eq!(manager.get::<i32>("test"), Some(&42))` | `get` returns handle → `get_ref(&handle)` returns `Option<&T>` |

**Root cause**: Tests written against an imagined API. `ResourceManager` has `insert(key, data, size_bytes)` and `get()` returns a `ResourceHandle<T>`, not a direct reference. Data access requires `get_ref(&handle)`.

### 2. Type Mismatch — Card::new signature (2 errors)

**File**: `src/engine/test_new_modules.rs:41,47`

| Error | Before | After |
|-------|--------|-------|
| `E0308` — expected `&str`, found integer | `Card::new(1, "Strike", ...)` | `Card::new("1", "Strike", ...)` |
| `E0308` — expected `&str`, found integer | `Card::new(1, "Card1", ...)` | `Card::new("1", "Card1", ...)` |

**Root cause**: `Card::new` signature is `new(id: &str, name: &str, card_type: CardType, cost: i32)` — first arg is `&str`, not numeric.

### 3. Type Mismatch — Deck::new signature (1 error)

**File**: `src/engine/test_new_modules.rs:48`

| Error | Before | After |
|-------|--------|-------|
| `E0308` — expected `usize`, found `Vec<Card>` | `Deck::new(cards)` | `Deck::new(5)` + `deck.add_cards(cards)` |

**Root cause**: `Deck::new(hand_size: usize)` takes a hand size, not cards. Cards must be added via `add_cards()` after construction.

## Files Modified

| File | Changes |
|------|---------|
| `src/engine/test_new_modules.rs` | Fixed 5 test compilation errors (ResourceManager API, Card::new types, Deck::new types) |

## Verification

```
cargo check -p nt-world-sim            → 0 errors, 2 warnings (lib)
cargo check -p nt-world-sim --tests    → 0 errors, 19 warnings (test)
cargo test -p nt-world-sim --lib       → 598 passed, 1 failed (pre-existing logic bug)
```

### Pre-existing Failure (NOT compilation)

`engine::systems::tests::test_system_runner_priority_order` — assertion `cam.position.x > 0.0` fails at `systems.rs:587`. This is a logic/ordering bug, not a compilation issue.

## Warnings (non-blocking, informational)

| Category | Count | Files |
|----------|-------|-------|
| Unused imports | 3 | `architecture.rs`, `game_flow.rs` |
| Unused variables | 2 | `game_flow.rs`, `architecture.rs` |
| Dead code (field) | 2 | `scene_tree.rs`, `architecture.rs` |
| Unused mut | 10+ | `dialogue.rs`, `combat.rs`, `perf.rs` |

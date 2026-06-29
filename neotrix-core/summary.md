# 3-Defect Fix Session

## Task 1: Serialize/Deserialize added to 10 error types

| # | File | Change |
|---|------|--------|
| 1 | `core/nt_core_error.rs` | Added `use serde::{Deserialize, Serialize};` + `Serialize, Deserialize` to `CoreError`; `#[serde(skip)]` on `Io` variant |
| 2 | `core/nt_core_knowledge/error.rs` | Added `use serde::{Deserialize, Serialize};` + `Serialize, Deserialize` to `KnowledgeError` |
| 3 | `core/nt_core_consciousness/error.rs` | Added `use serde::{Deserialize, Serialize};` + `Serialize, Deserialize` to `ConsciousnessError` |
| 4 | `core/nt_core_agent/error.rs` | Added `use serde::{Deserialize, Serialize};` + `Serialize, Deserialize` to `AgentError` |
| 5 | `core/nt_core_hcube/error.rs` | Added `use serde::{Deserialize, Serialize};` + `Serialize, Deserialize` to `HyperCubeError` |
| 6 | `neotrix/nt_memory_kb/error.rs` | Added `use serde::{Deserialize, Serialize};` + `Serialize, Deserialize` to `MemoryKbError` |
| 7 | `core/nt_core_input/pdf_extractor.rs` | Added `use serde::{Deserialize, Serialize};` + `Clone, Serialize, Deserialize` to `PdfError` |
| 8 | `neotrix/nt_mind/element/mod.rs` | Added `use serde::{Deserialize, Serialize};` + `Serialize, Deserialize` to `ElementError` |
| 9 | `neotrix/nt_act_voice/mod.rs` | Added `use serde::{Deserialize, Serialize};` + `Serialize, Deserialize` to `VoiceError` |
| 10 | `neotrix/nt_act_social/connector.rs` | Added `use serde::{Deserialize, Serialize};` + `Serialize, Deserialize` to `PlatformError` |

## Task 2: Timeout added to 5 bare recv().await

| # | File | Line | Timeout |
|---|------|------|---------|
| 1 | `neotrix/nt_mind_background_loop/run.rs` | 388→390 | 5s on `event_rx.recv()` |
| 2 | `neotrix/nt_agent_protocol/a2a` | — | Skip (file restructured, no recv() calls) |
| 3 | `cli/tui/app/app.rs` | 581→loop | 1s on `rx.recv()` stream (first occurrence) |
| 4 | `cli/tui/app/app.rs` | 893→loop | 1s on `rx.recv()` stream (second occurrence) |
| 5 | `nt_mind/reasoning_engine/engine_core/executor.rs` | 247→loop | 30s on `receiver.recv()` |

## Task 3: Silent send errors → logged warnings

| # | File | Count | Change |
|---|------|-------|--------|
| 1 | `core/nt_core_agent/bus.rs` | 6 | `let _ = bus.send(...)` → `if let Err(e) = bus.send(...) { log::warn!(...) }` |
| 2 | `core/nt_core_vision/mod.rs` | 1 | `let _ = tx.send(...)` → `if let Err(e) = tx.send(...) { log::warn!(...) }` |

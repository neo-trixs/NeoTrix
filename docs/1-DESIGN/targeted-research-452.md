# Targeted Research #452 — Internal Pain Points (WISER Iteration)

**Date:** 2026-09-12  
**Method:** Internal-first scan (TODO/FIXME stubs, hardcoded returns, dead code)  
**Status:** 3 pain points identified, ranked by severity

---

## Pain Point 1 — Messaging Providers Silently Fake Success

**File:** `neotrix-core/src/l1_action/nt_io/nt_io_messaging.rs:191-204`  
**Severity:** P0

### What's Wrong

`WhatsAppProvider::send()` generates a random UUID and returns `Ok(msg_id)` without making any HTTP call. `EmailProvider::send()` does the same. Both `receive()` methods return empty vectors, and `get_status()` unconditionally returns `MessageStatus::Sent`. Users would believe their messages were delivered when nothing actually happened.

```rust
// nt_io_messaging.rs:191-203
impl MessagingProvider for WhatsAppProvider {
    fn send(&self, _msg: &Message) -> Result<String, CapabilityError> {
        let msg_id = format!("wa_{}", uuid::Uuid::new_v4());
        // 实际实现: POST {api_url}/{phone_number_id}/messages
        Ok(msg_id)  // ← silent no-op, user thinks message sent
    }
    fn receive(&self, _since: Option<u64>) -> Result<Vec<Message>, CapabilityError> {
        Ok(Vec::new())  // ← never receives anything
    }
    fn get_status(&self, _id: &str) -> Result<MessageStatus, CapabilityError> {
        Ok(MessageStatus::Sent)  // ← always lies: status is "sent"
    }
}
```

### Fix (code sketch)

```rust
impl MessagingProvider for WhatsAppProvider {
    fn send(&self, msg: &Message) -> Result<String, CapabilityError> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/{}/messages", self.api_url, self.phone_number_id);
        let body = serde_json::json!({
            "messaging_product": "whatsapp",
            "to": msg.to,
            "type": "text",
            "text": { "body": &msg.body }
        });
        let resp = client.post(&url)
            .bearer_auth(&self.access_token)
            .json(&body)
            .send()
            .map_err(|e| CapabilityError::Transport(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(CapabilityError::Rejected(
                format!("WhatsApp API {}: {}", resp.status(), resp.text().unwrap_or_default())
            ));
        }
        let v: serde_json::Value = resp.json().unwrap_or_default();
        let id = v["messages"][0]["id"].as_str()
            .unwrap_or("unknown").to_string();
        Ok(id)
    }
    fn receive(&self, since: Option<u64>) -> Result<Vec<Message>, CapabilityError> {
        let client = reqwest::blocking::Client::new();
        let mut url = format!("{}/messages", self.api_url);
        if let Some(t) = since { url = format!("{}?after={}", url, t); }
        let resp = client.get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .map_err(|e| CapabilityError::Transport(e.to_string()))?;
        // parse response into Vec<Message> ...
        Ok(vec![])
    }
    fn get_status(&self, id: &str) -> Result<MessageStatus, CapabilityError> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/messages/{}", self.api_url, id);
        let resp = client.get(&url).bearer_auth(&self.access_token).send()
            .map_err(|e| CapabilityError::Transport(e.to_string()))?;
        let v: serde_json::Value = resp.json().unwrap_or_default();
        // map v["status"] to MessageStatus enum
        Ok(MessageStatus::Sent) // TODO: parse actual status
    }
}
```

Same pattern applies to `EmailProvider` (use `lettre` crate for SMTP).

---

## Pain Point 2 — `HighestQuality` Routing Strategy Falls Back to Priority Sort

**File:** `neotrix-core/src/l1_action/nt_io/model_routing.rs:252-254`  
**Severity:** P1

### What's Wrong

The `RoutingStrategy::HighestQuality` branch is supposed to sort candidates by quality score, but it just reuses the same `Priority` sort (`b.priority.cmp(&a.priority)`). This makes `HighestQuality` indistinguishable from `Priority`, and any user or GWT routing logic that selects "highest quality" gets priority-based selection instead.

```rust
// model_routing.rs:252-255
RoutingStrategy::HighestQuality => {
    // TODO: 实际基于质量分数排序
    candidates.sort_by(|a, b| b.priority.cmp(&a.priority)); // ← identical to Priority
}
```

### Fix (code sketch)

```rust
RoutingStrategy::HighestQuality => {
    // Quality score: combine benchmark score, recency, and availability
    candidates.sort_by(|a, b| {
        let a_quality = self.states.get(&a.id).map_or(0.0, |s| {
            s.benchmark_score * 0.6
            + (1.0 / (1.0 + s.avg_latency_ms as f64 / 1000.0)) * 0.3
            + if s.available { 0.1 } else { 0.0 }
        });
        let b_quality = self.states.get(&b.id).map_or(0.0, |s| {
            s.benchmark_score * 0.6
            + (1.0 / (1.0 + s.avg_latency_ms as f64 / 1000.0)) * 0.3
            + if s.available { 0.1 } else { 0.0 }
        });
        b_quality.partial_cmp(&a_quality).unwrap_or(std::cmp::Ordering::Equal)
    });
}
```

Requires adding `benchmark_score: f64` and `avg_latency_ms: f64` to `ModelState` if not already present.

---

## Pain Point 3 — StoryboardExtractor Splits Shots by Newlines (No LLM Parsing)

**File:** `neotrix-core/src/l5_cognition/nt_core/visual/storyboard_extractor.rs:216-243`  
**Severity:** P1

### What's Wrong

`parse_script_to_shots()` treats each non-empty line as a separate shot with identical metadata: same scene ("默认场景"), same duration, no characters, no camera movement. A 20-line script produces 20 identical shots. This is the core function for narrative structuring — it completely defeats the purpose of a storyboard.

```rust
// storyboard_extractor.rs:216-243
fn parse_script_to_shots(&self, script_text: &str) -> Vec<Storyboard> {
    // TODO: 实际调用 LLM 解析
    let paragraphs: Vec<&str> = script_text.lines().filter(|l| !l.trim().is_empty()).collect();
    paragraphs.iter().enumerate().map(|(i, p)| {
        Storyboard {
            id: format!("shot_{}", i + 1),
            description: p.to_string(),
            characters: vec![],           // ← always empty
            scene: "默认场景".to_string(), // ← always default
            shot_size: ShotSize::Medium,   // ← always medium
            camera_movement: CameraMovement::Static, // ← always static
            duration_secs: self.config.default_shot_duration, // ← always same
            // ... all prompts empty
        }
    }).collect()
}
```

### Fix (code sketch)

```rust
fn parse_script_to_shots(&self, script_text: &str) -> Vec<Storyboard> {
    // Phase 1: Heuristic pre-split (paragraphs or scene markers)
    let raw_chunks = self.pre_split(script_text);
    // Phase 2: For each chunk, call LLM to extract structured shot metadata
    let mut shots = Vec::new();
    for (i, chunk) in raw_chunks.iter().enumerate() {
        let prompt = format!(
            "Extract shot metadata from this scene text. Return JSON: \
             {{\"characters\": [], \"scene\": \"\", \"shot_size\": \"Medium\", \
              \"camera_movement\": \"Static\", \"duration_secs\": 5.0, \
              \"emotion\": \"Neutral\"}}\n\nText: {}",
            chunk
        );
        // Use self.llm_provider or injected provider here
        let meta: ShotMeta = self.llm_extract(&prompt).unwrap_or_default();
        shots.push(Storyboard {
            id: format!("shot_{}", i + 1),
            shot_number: (i + 1) as u32,
            description: chunk.to_string(),
            characters: meta.characters,
            scene: meta.scene,
            shot_size: meta.shot_size,
            camera_movement: meta.camera_movement,
            duration_secs: meta.duration_secs,
            emotion: meta.emotion,
            // ... fill from LLM response
        });
    }
    shots
}

fn pre_split(&self, text: &str) -> Vec<String> {
    // Split on scene markers (INT./EXT., 空行×2, # headings)
    // or by paragraph if no markers found
    text.split("\n\n")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
```

Requires adding an `llm_provider` field to `StoryboardExtractor` and a `ShotMeta` deserialization struct.

---

## Summary

| # | Pain Point | File:Line | Severity | Category |
|---|-----------|-----------|----------|----------|
| 1 | Messaging `send()` silently no-ops | `nt_io_messaging.rs:191-204` | **P0** | Silent failure |
| 2 | `HighestQuality` routing = `Priority` | `model_routing.rs:252-254` | **P1** | Dead logic |
| 3 | Storyboard parses lines, not scenes | `storyboard_extractor.rs:216-243` | **P1** | Placeholder |

**Scan coverage:** 100+ TODO/FIXME matches, 58 empty-return matches, 100+ dead-code annotations examined. These 3 were selected for highest impact-to-fix ratio.

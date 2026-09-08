# Iteration Batch 565 — Spatial Computing / AR-VR / 3D Interaction Research

**Date:** 2026-09-06
**Predecessor:** Batch 564 (CT drift retraining, model registry, feature store, 87% production failure, SEAL CI/CD gap)
**Research Scope:** Spatial computing ecosystem, AR/VR/MR convergence, 3D interaction paradigms

---

## Executive Summary

Batch 564 identified five critical defects in NeoTrix's internal architecture. This iteration examines the **external spatial computing landscape** to find where NeoTrix is blind — not just internal missing features, but **interaction paradigms and platform convergence** that NeoTrix's six-layer architecture has no awareness of. The spatial computing industry has hit an inflection point in 2026: the market crossed $97B, smart glasses overtook headsets as the dominant narrative, and LLM-powered intent-to-operation interaction models are emerging. None of these are reflected in NeoTrix's L3 Embodiment or L5 Cognition layers.

---

## PART I: SPATIAL COMPUTING — Findings

### Finding 1: Spatial AI as the New Frontier (Apple + Meta + OpenAI + Google DeepMind + NVIDIA)

**Source:** Tom's Guide (2026-06-26), Apple WWDC 2026 documentation

- **Apple:** visionOS 27 debuts "Spatial Reframing" — AI repositions a photo's perspective *after* it's taken. Visual Intelligence lets Siri understand objects in passthrough video feeds.
- **Meta:** Investing in "super sensing" for next-gen glasses — real-time recognition of objects, locations, and people (raising privacy concerns).
- **Google DeepMind:** Gemini Robotics combines vision + language + physical reasoning. Genie generates interactive 3D environments from video.
- **World Labs:** Raised $1B for "spatial intelligence."
- **NVIDIA:** Streaming large industrial 3D models directly into headsets via cloud rendering.

**DEFECT #1 — No Spatial Perception Layer in NeoTrix:**
NeoTrix's L2 Perception layer (`nt_world`, `nt_sense`) has no spatial computing primitives — no depth map processing, no SLAM integration, no 3D bounding box reasoning, no gaze-stream ingestion. The industry is converging on AI that understands physical reality (spatial intelligence), but NeoTrix treats perception as text/crawl/parse. **NeoTrix cannot process or generate spatial data streams.**

### Finding 2: visionOS 27 — Held Object Tracking, IR Accessories, Foveated Streaming

**Source:** UploadVR (2026-06-14), Apple Developer WWDC26 session 287

- **Held Object Tracking:** New high-frame-rate (~30Hz) mode for moving and held objects in visionOS 27.
- **IR LED Tracked Accessories Framework:** Custom tracked hardware extends Vision Pro input model with plug-and-play physical controllers.
- **Foveated Streaming:** Eye-tracked foveated video compression enables Mac/PC content streaming to Vision Pro at practical bandwidths.
- **RealityKit updates:** Physical space lighting, cloth simulation, acoustic ray tracing, Gaussian Splatting.
- **Reality Composer Pro 3:** AI-assisted collaborative tools — Animation Graph, Script Graph, enhanced shader materials.
- **OpenUSD Core 1.0:** Alliance for OpenUSD specification for interoperable 3D content.

**DEFECT #2 — No Foveated Rendering or Gaze-Adaptive Compute:**
NeoTrix's GWT (Global Workspace Theory) attention routing operates on semantic salience, not physical gaze data. visionOS 27's foveated streaming and foveated rendering demonstrate that **attention modulation must be coupled with physical gaze hardware** for efficient compute allocation. NeoTrix has no mechanism to consume eye-tracking data or adapt its compute budget based on where the user is physically looking.

### Finding 3: Samsung Galaxy XR — Android XR Enters the Market

**Source:** idevice.com (2026-06-22), Tech Insider (2026-06-17)

- **Samsung Galaxy XR:** $1,799, Snapdragon XR2+ Gen 2, 109° H FOV, shipping now.
- **Android XR platform:** Google's play for spatial computing — AI-native, multimodal.
- **Meta Quest 4:** Targeting sub-$500 for H2 2027, mainstream price point.
- **Vision Pro 2:** On ice until 2028. M5 model ($3,499) is the only option for years.

**DEFECT #3 — No Multi-Platform Spatial Gateway:**
NeoTrix's NT-IO layer has no spatial platform abstraction. The industry now has **three competing spatial OS platforms** (visionOS, Horizon OS, Android XR) plus PCVR (SteamVR/OpenXR). NeoTrix has no `SpatialPlatformGateway` analogous to the `PlatformGateway` for media generation. Any spatial application would need to be reimplemented per-platform.

---

## PART II: AR/VR/MR CONVERGENCE — Findings

### Finding 4: The Death of Category Boundaries

**Source:** Memvers (2026-03-16), Vistrilo (2026-08-04), Unity Industry Trends (2026)

- **Key insight:** "Pure VR and pure AR are becoming features within MR devices, not separate product lines."
- **Smart glasses dominance:** 42% of XR stories now about AR/smart glasses (up from <10% in 2024).
- **Meta Ray-Ban shipments:** Surpassing 5M units — the wearable AI category creator.
- **Google + Samsung:** Android XR glasses arriving in retail stores fall 2026.
- **Snap:** Preparing September 2026 launch of high-end Specs AR glasses.

**DEFECT #4 — NeoTrix's Architecture Assumes Headset-Only Form Factor:**
The `nt_physical` domain (sensors, motors, safety kernel, body schema) is designed around headset-class compute. The convergence on lightweight smart glasses (sub-100g, phone-offloaded) means **NeoTrix's physical embodiment model needs a thin-client mode** where most cognition runs remotely and only minimal sensing/gesture processing runs on-device. No such split exists.

### Finding 5: Enterprise MR Adoption Inflection — 75% Fortune 500

**Source:** N-iX MR (2026-07-23), AInvest (2026-05-20)

- **MR market:** USD 8.41B in 2026, projected USD 50.79B by 2031 (43.3% CAGR).
- **Manufacturing:** 28.10% of MR end-user spending.
- **Healthcare:** Fastest-growing at 43.95% CAGR through 2031.
- **75% of Fortune 500** now using VR for employee training.
- **PwC study:** VR-trained employees completed training 4× faster, 275% more confident.
- **Critical pain:** Fragmentation — hardware, SDKs, rendering pipelines don't align across devices.

**DEFECT #5 — No MR Enterprise Workflow Integration:**
NeoTrix has no enterprise spatial workflow support. The industry's adoption data shows that **spatial computing is an enterprise infrastructure problem** (training, digital twins, remote collaboration), not just a developer tool problem. NeoTrix's `nt_act` (action domain) has no spatial workflow orchestration — no digital twin synchronization, no spatial annotation, no enterprise MR pipeline.

### Finding 6: Passthrough-First Reduces Cybersickness by 44%

**Source:** N-iX MR (2026-07-23)

- Passthrough-first designs cut cybersickness by 44%, removing a major barrier to training/simulation adoption.
- Battery life remains the constraint: most all-in-one headsets top out at 3-4 hours.
- Enterprise headsets still list in thousands of dollars — price is top adoption deterrent.

**DEFECT #6 — No Cybersickness Mitigation or Battery-Aware Compute:**
NeoTrix's safety kernel (`nt_shield`) has no spatial computing-specific health monitoring. Cybersickness detection (vestibular mismatch, frame pacing) is not modeled. Battery-aware compute throttling for spatial workloads — where GPU-intensive rendering competes with battery life — is absent. The 44% cybersickness reduction from passthrough-first design suggests **NeoTrix should model passthrough quality as a safety parameter**.

---

## PART III: 3D INTERACTION — Findings

### Finding 7: SIAgent — Intent-to-Operation Paradigm (97.2% Accuracy)

**Source:** IEEE TVCG (2026-04-22), Wang et al.

- **SIAgent:** LLM-powered framework translating eye-hand motion intent into natural language, then executing via agent.
- **Key shift:** "Operation-to-Intent" (user learns predefined gestures) → "Intent-to-Operation" (system infers intent from natural motion).
- **Accuracy:** 97.2% intent recognition (vs 93.1% for gaze+pinch).
- **Benefits:** Eliminates gesture memorization, accommodates individual motion preferences, high error tolerance, reduces arm fatigue.

**DEFECT #7 — NeoTrix's GWT Has No Motion-Intent Inference:**
NeoTrix's GWT attention routing processes semantic salience from text/KB. It has **no capacity to infer intent from physical motion streams** (eye gaze trajectories, hand motion vectors, head rotation). The SIAgent paradigm proves that physical motion data, when processed through LLM reasoning, yields higher accuracy than traditional gesture-recognition pipelines. NeoTrix's L5 Cognition layer should consume raw motion intent, not just symbolic gestures.

### Finding 8: GazeTune — Cascaded Gaze+Touch for Precise XR Manipulation

**Source:** arXiv:2609.00716 (2026-09-01)

- **Problem:** Gaze+pinch (Vision Pro standard) has 37.12% error rate during motion-induced conditions.
- **Solution:** GazeTune — cascaded gaze+touch using smartwatch as refinement channel.
- **Results:** 4.24% error rate (vs 37.12% gaze+pinch, 9.70% gaze-only).
- **Key insight:** Touch as a refinement channel within gaze pointing — preserving rapid gaze targeting while enabling fine-grained correction.

**DEFECT #8 — No Multimodal Input Fusion Architecture:**
NeoTrix's interaction model assumes discrete input channels. The research shows that **the future of XR input is cascaded multimodal fusion** — gaze for coarse targeting, touch/pinch for refinement, voice for confirmation, all in a single continuous interaction sequence. NeoTrix's `nt_io` has no input fusion layer that can combine gaze+touch+voice into a unified intent stream with per-modality role assignment (who targets, who confirms, who refines).

### Finding 9: AgentHands — LLM-Powered XR Hand Gestures for Spatial Conversation

**Source:** Google Research Blog (2026-08-25), CHI 2026

- **AgentHands:** LLM generates co-speech hand gestures synchronized with TTS for spatially grounded XR conversations.
- **Taxonomy:** Handedness, gesture type (deictic/iconic/expressive), spatiality (mid-air/object-anchored/user-relative), temporal dynamics, visual effects.
- **Object Registration:** Eye gaze + scene reconstruction + LLM creates 3D bounding box registry of physical objects.
- **Results:** Significant gains in spatial grounding (p<0.05), salient safety cues, reduced cognitive load.

**DEFECT #9 — No Spatial Object Registry or Gesture Generation:**
NeoTrix has no spatial object registry — no mechanism to map physical objects to semantic entities via eye gaze + depth reconstruction. AgentHands demonstrates that **LLM-powered gesture generation synchronized with speech** is the next interaction paradigm. NeoTrix's NT-FEEL (emotion expression) and NT-ACT (action execution) have no pathway to generate spatially-aware, gesture-synchronized output.

### Finding 10: DOBI — Dynamic Opportunistic Body Input (Hands-Free XR)

**Source:** arXiv:2608.30341, UIST 2026

- **DOBI:** Redirects XR control to whichever body region remains free (head, shoulder, knee, foot).
- **Key insight:** Users' preferred spare body regions shift dynamically based on physical constraints, but movements share a low-dimensional kinematic structure.
- **Results:** 2.62 bits/s throughput, 5.0% error rate, SUS=84.2.
- **Paradigm:** Gaze targets UI element, brief trigger gesture identifies recruited body region, region's motion drives continuous 1D control.

**DEFECT #10 — No Body Schema for Spatial Interaction:**
NeoTrix's `nt_physical` body schema models sensors and motors generically. It has **no dynamic body-part recruitment model** — no ability to map interaction intent to whichever body region is available. The DOBI paradigm proves that spatial interaction requires a runtime body availability map that adapts to the user's current physical posture and task context.

### Finding 11: Generated Reality — Interactive Video Generation with Hand/Camera Control

**Source:** CVPR 2026 (Xie et al.)

- **Generated Reality:** Autoregressive video generation conditioned on tracked head pose and joint-level hand poses.
- **Key contribution:** First systematic study of hand pose conditioning strategies in video diffusion models.
- **Results:** Hybrid 2D-3D strategy most effective. 11 FPS at 1.4s latency on H100.
- **Applications:** Immersive learning, training, exploration without detailed 3D models — zero-shot environment generation from user motion.

**DEFECT #11 — No Generative Spatial Content Pipeline:**
NeoTrix has no pathway to generate spatial content from motion data. Generated Reality proves that **video diffusion models conditioned on hand/head pose** can create interactive environments without pre-built 3D assets. NeoTrix's NT-WORLD (perception) and NT-ACT (action) have no generative spatial content pipeline — no ability to create immersive environments from user motion streams.

### Finding 12: Multi-Object Selection — Gaze+Pinch vs Voice vs Dwell

**Source:** CHI 2026 (Bashar et al.)

- **Study:** 4 mode-switching techniques × 3 subselection techniques across 6-10 targets.
- **Results:** DoublePinch (persistent mode) + Gaze+Pinch subselection was most efficient.
- **Quasi-modes degrade sharply** as target count increases from 6 to 10.
- **Voice-based switching** well-received but repeated voice commands for subselection less preferred.

**DEFECT #12 — No Multi-Target Spatial Selection Model:**
NeoTrix has no spatial selection model at all. The research demonstrates that **multi-object selection in XR has fundamentally different failure modes than 2D selection** — mode stability, fatigue accumulation, and error amplification across target sets. NeoTrix's interaction model is keyboard/mouse/CLI-centric with no spatial selection primitives.

---

## PART IV: NEW vs Batch 564 — Delta Analysis

| # | Batch 564 Finding | Batch 565 NEW Finding | Relationship |
|---|-------------------|----------------------|--------------|
| 1 | No CT drift-triggered retraining | **Spatial AI convergence** — industry building world models from physical perception | NeoTrix's self-evolution operates on internal metrics only, blind to external spatial perception capabilities |
| 2 | No model registry/lineage | **SIAgent intent-to-operation paradigm** — LLMs now infer intent from motion, not just text | Model lineage must extend to motion-intent models, not just text models |
| 3 | No feature store | **AgentHands spatial object registry** — physical objects mapped to semantic entities via gaze+depth+LLM | Feature store needs spatial features: 3D bounding boxes, object affordances, gaze heatmaps |
| 4 | 87% AI models never reach production | **75% Fortune 500 using VR for training** — enterprise spatial adoption is real, but model deployment in spatial context has its own failure modes | Spatial models face additional deployment barriers: hardware fragmentation, comfort constraints, battery limits |
| 5 | SEAL lacks CI/CD/CT third loop | **Passthrough-first reduces cybersickness 44%** — SEAL needs a spatial health feedback loop | CT (Continuous Training) needs a "Continuous Comfort" loop — user physiology feeds back into model optimization |

---

## PART V: Defects Found — Prioritized

### Critical (Blocks spatial capability)

| ID | Defect | Layer | Impact |
|----|--------|-------|--------|
| **D-565-01** | No Spatial Perception Layer — no depth map, SLAM, gaze-stream, 3D bounding box processing | L2 Perception | NeoTrix cannot understand physical reality |
| **D-565-02** | No Gaze-Adaptive Compute — GWT operates on semantic salience only, not physical gaze | L5 Cognition | Cannot optimize compute based on where user looks |
| **D-565-03** | No Multi-Platform Spatial Gateway — 3 spatial OS platforms with no abstraction | L1 Action | Spatial apps must be reimplemented per-platform |

### High (Degrades spatial experience)

| ID | Defect | Layer | Impact |
|----|--------|-------|--------|
| **D-565-04** | No Thin-Client Physical Mode — headset-assumed form factor, no phone-offloaded split | L3 Embodiment | Cannot run on smart glasses (<100g) |
| **D-565-05** | No Enterprise MR Workflow — no digital twin sync, spatial annotation, enterprise pipeline | L1 Action | Missing $50B+ enterprise MR market |
| **D-565-06** | No Cybersickness/Battery Monitoring — no vestibular mismatch detection, no battery-aware throttling | L3 Embodiment | User health risk, unusable on battery-powered devices |

### Medium (Limits interaction richness)

| ID | Defect | Layer | Impact |
|----|--------|-------|--------|
| **D-565-07** | No Motion-Intent Inference — GWT cannot consume eye/hand motion streams for intent | L5 Cognition | Stuck on predefined gestures, not natural motion |
| **D-565-08** | No Multimodal Input Fusion — no cascaded gaze+touch+voice interaction pipeline | L1 Action | Cannot build modern XR interaction |
| **D-565-09** | No Spatial Object Registry — no gaze+depth+LLM object mapping | L2 Perception | Cannot ground conversation in physical objects |
| **D-565-10** | No Dynamic Body Schema — no runtime body-part recruitment for interaction | L3 Embodiment | Cannot adapt to user's physical posture |
| **D-565-11** | No Generative Spatial Content — no motion-conditioned video/environment generation | L1 Action + L2 Perception | Cannot create immersive content from motion data |
| **D-565-12** | No Multi-Target Spatial Selection — no spatial selection primitives | L1 Action | Cannot support XR multi-object workflows |

---

## PART VI: Sources Cited

| # | Source | URL | Date |
|---|--------|-----|------|
| 1 | Tom's Guide — "Why Apple, Meta and OpenAI are racing toward Spatial AI" | tomsguide.com/ai/forget-chatbots-why-apple-meta-and-openai-are-racing-toward-spatial-ai | 2026-06-26 |
| 2 | UploadVR — "visionOS 27 Is A Much Bigger Update" | uploadvr.com/visionos-27-announced-apple-vision-pro-wwdc-26/ | 2026-06-14 |
| 3 | Apple Developer — "Build next-generation experiences with visionOS 27" | developer.apple.com/videos/play/wwdc2026/287/ | 2026-06-08 |
| 4 | Apple Insider — "Spatial computing & Apple Intelligence upgrades in visionOS 27" | appleinsider.com/articles/26/06/08/spatial-computing-apple-intelligence-upgrades-collide-in-visionos-27 | 2026-06-08 |
| 5 | Tech Insider — "Apple Vision Pro vs Meta Quest 3 2026" | tech-insider.org/apple-vision-pro-vs-meta-quest-3-2026/ | 2026-06-17 |
| 6 | idevice.com — "Best Spatial Computing Headsets in 2026" | idevice.com/head/compare | 2026-06-22 |
| 7 | Counterpoint Research — "Apple Vision Pro Long-Term Review" | counterpointresearch.com/en/insights/apple-vision-pro-long-term-review-reality-check-spatial-computing | 2026-08-31 |
| 8 | Memvers — "AR vs VR vs Mixed Reality in 2026" | memvers.com/blog/ar-vs-vr-mixed-reality-guide | 2026-03-16 |
| 9 | Unity — "2026 Industry Trends Report: Immersive Tech & AI" | unity.com/resources/industry-trends-report-2026 | 2026-01-12 |
| 10 | N-iX MR — "Mixed reality trends 2026" | mr.n-ix.com/mixed-reality-trends/ | 2026-07-23 |
| 11 | AInvest — "VR/AR/MR 2026-2036: The Spatial Computing Inflection Point" | ainvest.com/news/vr-ar-2026-2036-spatial-computing-inflection-point-infrastructure-layers-capturing-exponential-2605/ | 2026-05-20 |
| 12 | Vistrilo — "The AR & VR Revolution of 2026" | vistrilo.com/the-ar-vr-revolution-of-2026-smart-glasses-spatial-computing-the-death-of-the-bulky-headset | 2026-08-04 |
| 13 | Wang et al. — "SIAgent: Spatial Interaction Agent via LLM-Powered Eye-Hand Motion Intent Understanding in VR" | IEEE TVCG, DOI:10.1109/tvcg.2026.3686395 | 2026-04-22 |
| 14 | arXiv — "GazeTune: Facilitating Precise Gaze-Driven Interactions with Cascaded Touch Input" | arxiv.org/html/2609.00716 | 2026-09-01 |
| 15 | Google Research — "AgentHands: Generating interactive hand gestures for spatially grounded agent conversations in XR" | research.google/blog/agenthands-generating-interactive-hand-gestures-for-spatially-grounded-agent-conversations-in-xr/ | 2026-08-25 |
| 16 | arXiv — "DOBI: Dynamic Opportunistic Body Input via Spare Joint Recruitment for Hands-Free XR" | arxiv.org/abs/2608.30341 | 2026-08-31 |
| 17 | Li et al. — "SwEYEpinch: Exploring Intuitive, Efficient Text Entry for Extended Reality" | ACM CHI 2026, DOI:10.1145/3772318.3791820 | 2026-04-13 |
| 18 | Xie et al. — "Generated Reality: Human-Centric World Simulation Using Interactive Video Generation" | CVPR 2026 | 2026 |
| 19 | Bashar et al. — "Eyes on Many: Evaluating Gaze, Hand, and Voice for Multi-Object Selection in XR" | ACM CHI 2026, DOI:10.1145/3772318.3790513 | 2026-04-13 |
| 20 | Future Markets Inc — "The Global VR, AR and MR Market 2026-2036" | futuremarketsinc.com/the-global-virtual-reality-vr-augmented-reality-ar-and-mixed-reality-mr-market-2026-2036/ | 2026-01-19 |
| 21 | EazyTechSol — "Extended Reality in 2026: Spatial Computing Goes Mainstream" | eazytechsol.com/extended-reality-in-2026-spatial-computing-goes-mainstream | 2026-08-13 |
| 22 | Apple — "Apple Vision Pro" | apple.com/apple-vision-pro/ | 2026 |
| 23 | VR.org — "Quest 3 vs Apple Vision Pro" | vr.org/quest-3-vs-vision-pro | 2026-08-24 |

---

## Summary

**What's NEW vs Batch 564:**
Batch 564 was internal-focused (CT drift, model registry, feature store, production failure, SEAL CI/CD). Batch 565 reveals that NeoTrix is **architecturally blind to the spatial computing revolution** happening in 2026. The industry has moved from "can XR work?" to "how do we deploy XR at enterprise scale?" — and the interaction paradigms have shifted from gesture-memorization to LLM-powered intent inference. NeoTrix's six-layer architecture has no spatial primitives in any layer.

**12 defects found** (3 critical, 3 high, 6 medium). The most impactful gap: NeoTrix cannot perceive, generate, or interact with spatial data — a $97B market growing at 43% CAGR.

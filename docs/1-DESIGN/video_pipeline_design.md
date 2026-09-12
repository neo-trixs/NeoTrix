# 视频管线设计文档

> 源码路径: `neotrix-core/src/l1_action/nt_act/actions/video/`
>
> 所属层: L1 Action (行动层) → NT-ACT 域

## 架构概览

视频管线由五个独立模块组成，通过 `VideoJobPipeline` 统一调度：

```
┌─────────────────────────────────────────────────┐
│              VideoJobPipeline                    │
│  (作业队列 · 优先级调度 · 检查点 · 重试)          │
├─────────┬──────────┬──────────┬─────────────────┤
│ Video   │ Video    │ Video    │ Audio           │
│ Spec    │ Object   │ Stitcher │ Orchestrator    │
│ (规格)  │ Storage  │ (拼接)   │ (音频编排)       │
│         │ (存储)   │          │                 │
└─────────┴──────────┴──────────┴─────────────────┘
```

### 模块职责

| 模块 | 文件 | 职责 |
|------|------|------|
| **VideoSpec** | `video_spec.rs` | 类型化视频规格定义 — 内容类型、场景列表、CTA、分辨率、帧率、验证 |
| **VideoJobPipeline** | `video_job_pipeline.rs` | 异步作业队列 — 提交/调度/检查点/恢复/重试/统计 |
| **VideoObjectStorage** | `video_object_storage.rs` | 持久存储 + 生命周期管理 — 上传/下载/预签名URL/CDN/存储类别降级 |
| **VideoStitcher** | `video_stitcher.rs` | 多片段拼接 — 时间线编辑、转场(xfade)、字幕(ASS)、FFmpeg 生成 |
| **AudioOrchestrator** | `audio_orchestrator.rs` | TTS + BGM + 音效混合 — 多轨道、闪避(ducking)、响度归一化 |

---

## 数据流

```
VideoSpec (规格定义)
    │
    ▼
VideoJobPipeline.submit()
    │  JobStatus::Queued → Processing
    │  按 JobPriority 排队 (Critical > High > Normal > Low > Batch)
    │
    ├─ Stage 1: 渲染 (视频片段生成)
    │     │  save_checkpoint("render")
    │     ▼
    │
    ├─ Stage 2: 音频编排
    │     │  AudioOrchestrator.generate_mix_command()
    │     │  save_checkpoint("audio")
    │     ▼
    │
    ├─ Stage 3: 拼接
    │     │  VideoStitcher.stitch() → FFmpeg 执行
    │     │  save_checkpoint("stitch")
    │     ▼
    │
    ├─ Stage 4: 存储
    │     │  VideoObjectStorage.upload() → 对象存储
    │     │  save_checkpoint("storage")
    │     ▼
    │
    └─ Stage 5: 输出
          │  JobStatus::Completed
          │  output_path = CDN URL / 本地路径
          ▼
        完成
```

---

## 模块详细设计

### VideoSpec — 视频规格

**核心类型**:

- `VideoContentType` — 内容类型枚举: Advertisement / Educational / ShortVideo / Film / MusicVideo / Documentary / GameCutscene / ComicDrama
- `OutputFormat` — 输出格式: MP4H264 / MP4H265 / WebMVP9 / MOVProRes / GIF
- `SceneType` — 场景类型: Opening / Setup / Conflict / Climax / Transition / Resolution / CallToAction
- `SceneSpec` — 场景规格: 时长、景别、镜头运动、对白/旁白、提示词、资产引用
- `VideoSpec` — 完整规格: 分辨率、帧率、场景列表、CTA、标签、元数据

**构建器模式**: `VideoSpecBuilder` 提供链式 API 构建规格，自动累加场景时长。

**验证器**: `SpecValidator` 内置三类规则（非空场景、正时长、有效分辨率），可扩展。

**访问级别**: `VideoContentType`、`AssetReference`、`SceneSpec`、`CTASpec`、`SpecValidator` 为 `pub(crate)`，仅管线内部使用；`OutputFormat`、`SceneType`、`VideoSpec`、`VideoSpecBuilder` 为 `pub`，供外部规格构建。

### VideoJobPipeline — 作业管线

**状态机**:

```
Pending → Queued → Processing → Completed
                    │
                    ├→ Checkpointed → (resume) → Queued
                    ├→ Failed → (retry) → Queued
                    └→ Cancelled
```

**配置** (`PipelineConfig`):
- `max_concurrent_jobs`: 10 (默认)
- `queue_size`: 1000
- `job_timeout`: 3600s
- `checkpoint_interval`: 300s
- `max_retries`: 3

**检查点机制**: 每个阶段完成后调用 `save_checkpoint()`，记录阶段名 + 中间产物路径 + 耗时。失败后可通过 `_resume_job()` 从最近检查点恢复。

**优先级**: Critical > High > Normal > Low > Batch (数值越大优先级越高)

**统计**: `PipelineStats` 跟踪提交/处理/完成/失败/检查点数量，`success_rate()` 计算成功率。

### VideoObjectStorage — 对象存储

**存储类别**:
- Standard — 默认，频繁访问
- InfrequentAccess — 30 天后降级
- Archive — 90 天后降级
- DeepArchive — 180 天后降级
- IntelligentTiering — 自动分层

**生命周期规则**: 默认 365 天过期，支持分阶段降级策略。

**访问控制**: `AccessControl` 支持公开/用户级/角色级权限 + 预签名 URL (默认 1 小时过期)。

**CDN 集成**: 可选配置 `cdn_base_url`，生成 CDN 分发 URL。

**完整性**: 上传时计算 SHA-256 哈希，支持完整性校验。

### VideoStitcher — 拼接引擎

**核心概念**:
- `Timeline` — 时间线: 包含视频片段列表、音频轨道、字幕条目
- `VideoClip` — 片段: 文件路径、时间线位置、入/出点裁剪、速度、音量、转场
- `TransitionType` — 转场: Cut / CrossDissolve / Dissolve / Wipe / Push / Zoom

**FFmpeg 生成**: `generate_ffmpeg_command()` 根据 Timeline 自动构建 FFmpeg 命令:
- 多输入 → concat 滤镜 → 编码输出
- 转场通过 xfade 滤镜实现
- 字幕通过 subtitles 滤镜 (ASS 格式) 叠加

**编码配置** (`StitchConfig`):
- 默认: 1920×1080 / 30fps / libx264 / 8Mbps
- 可选 LUT 色彩分级

**统计**: `StitchStats` 跟踪拼接次数、成功率、总时长。

### AudioOrchestrator — 音频编排

**音频类型**:
- `Voiceover` — TTS 旁白 (需 `TTSConfig`)
- `BackgroundMusic` — 背景音乐
- `SoundEffect` — 音效
- `AmbientSound` — 环境音

**混合配置** (`AudioMixConfig`):
- `master_volume`: 主音量
- `enable_ducking`: 旁白时自动降低 BGM (默认开启)
- `ducking_strength`: 闪避强度 0.7
- `enable_compression`: 动态压缩 (默认开启)
- `compression_threshold`: -20 dB

**FFmpeg 混合**: `generate_mix_command()` 构建多输入 amix 滤镜，支持:
- 独立音量控制
- 淡入/淡出 (afade)
- 响度归一化 (loudnorm)

**TTS**: `_generate_tts()` 为占位实现，实际调用需对接 TTS API。

---

## 与其他模块的关系

### 依赖关系

```
nt_act::video (本管线)
  ├── nt_io::inference (推理引擎 — 视频帧生成)
  ├── nt_act::resource (资源管理 — GPU/内存分配)
  └── 外部: FFmpeg CLI (拼接/混音执行)
```

### 被依赖关系

```
nt_act::video (本管线)
  ├── nt_act::mod (pub use 导出全部模块)
  │     └── CLI 命令 (视频生成/拼接/查询)
  ├── nt_act::video_quality_scorer (质量评分)
  ├── nt_act::video_audit_trail (审计轨迹)
  └── l6_meta::nt_repair_hanzi_video (自愈修复)
```

### 上下游数据流

```
NT-WORLD (资产采集) ──→ VideoSpec (规格构建)
                           │
NT-CORE (分镜拆解) ──→ SceneSpec[] (场景列表)
                           │
                      VideoJobPipeline
                           │
NT-IO (推理接口)  ──→ 视频帧渲染
                           │
AudioOrchestrator  ──→ 音频混合
                           │
VideoStitcher      ──→ 时间线拼接
                           │
VideoObjectStorage ──→ 持久存储 + CDN 分发
```

---

## 约束与注意事项

| 约束 | 说明 |
|------|------|
| `#![forbid(unsafe_code)]` | 管线内无 unsafe (R-P1) |
| FFmpeg 依赖 | 拼接/混音依赖外部 FFmpeg CLI，需确保 PATH 可达 |
| 存储后端 | `VideoObjectStorage` 当前为内存模拟，实际部署需对接 S3/MinIO |
| TTS 占位 | `_generate_tts()` 为占位实现，需对接实际 TTS 服务 |
| 闪避占位 | `enable_ducking` 逻辑标记 TODO，待实现 sidechain compression |
| 构建缓存 | 结构变更后执行两次 `cargo build` 获取真实错误计数 (R-P9) |

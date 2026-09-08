# Easel → NT-* Domain Mapping

> 8-Source Batch Absorption, 2026-09-08.
> Maps Easel's 112 skills to NeoTrix NT-* domain taxonomy.
> Principle: absorb methodology (patterns), not code (Python scripts).

## Mapping Table

| Easel Skill Category | Count | → NT-* Domain | Star Node | Implementation Path |
|---------------------|-------|---------------|-----------|---------------------|
| **Foundation** (asset mgmt, batch, profile, template) | 6 | NT-MEMORY | Exp-藏 | `nt_memory::knowledge_hub` |
| **Discover** (trending, competitor, RSS, UGC, algorithm) | 9 | NT-WORLD | — | `nt_world::discovery` |
| **Plan** (positioning, audience, strategy, calendar) | 16 | NT-CORE | Des-观 | `nt_core_self::planning` |
| **Create** (copywriting, image, video, audio, novel) | 50 | NT-ACT | Dev-匠 | `nt_act::creation` |
| **Publish** (XHS/Douyin/Bilibili/Zhihu/Kuaishou upload) | 20 | NT-IO | Edu-灯 | `nt_io::platform_publish` |
| **Attribute** (ROI, comments, postmortem, analytics) | 11 | NT-META | Meta-镜 | `nt_meta::attribution` |

## Detailed Skill → Module Map

### Foundation (6) → NT-MEMORY

| Easel Skill | NT-* Module | Notes |
|-------------|-------------|-------|
| asset_management | `nt_memory::asset_hub` | Media asset registry |
| batch_processing | `nt_act::parallel_task` | ParallelTaskManager |
| profile_crud | `nt_core_self::self_model` | SelfModel extension |
| template_library | `nt_mind::skill_engine` | Skill crystallization |
| output_path_validation | `nt_shield::path_validator` | PathValidator |
| content_guard | `nt_shield::egress_guard` | BLOCK/WARN dual-tier |

### Discover (9) → NT-WORLD

| Easel Skill | NT-* Module | Notes |
|-------------|-------------|-------|
| trending_topics | `nt_world::discovery::trending` | Hot list aggregation |
| competitor_analysis | `nt_world::discovery::competitor` | Intel-watch |
| content_gap | `nt_world::discovery::gap_analysis` | Opportunity detection |
| rss_aggregation | `nt_world::fetchers::rss` | Feed parser |
| ugc_discovery | `nt_world::discovery::ugc` | User content mining |
| algorithm_tracking | `nt_world::discovery::algorithm` | Platform algo changes |
| event_calendar | `nt_world::discovery::events` | Seasonal hooks |
| news_intelligence | `nt_world::discovery::news` | News aggregation |
| cross_platform_diff | `nt_world::discovery::diff` | Cross-platform comparison |

### Plan (16) → NT-CORE

| Easel Skill | NT-* Module | Notes |
|-------------|-------------|-------|
| positioning | `nt_core_self::planning::positioning` | Account positioning |
| audience_profiling | `nt_core_self::planning::audience` | Audience analysis |
| topic_evaluation | `nt_core_self::planning::topic` | Topic scoring |
| content_strategy | `nt_core_self::planning::strategy` | Content strategy |
| content_calendar | `nt_core_self::planning::calendar` | Scheduling |
| hook_generation | `nt_core_self::planning::hooks` | Attention hooks |
| voice_building | `nt_core_self::planning::voice` | Tone/voice |
| campaign_planning | `nt_core_self::planning::campaign` | Campaign design |
| brand_onboarding | `nt_core_self::planning::brand` | Brand setup |
| account_diagnosis | `nt_core_self::planning::diagnosis` | Health check |
| title_generation | `nt_core_self::planning::titles` | Title variants |
| content_script | `nt_core_self::planning::script` | Script writing |
| content_matrix | `nt_core_self::planning::matrix` | Multi-platform matrix |
| seo_optimization | `nt_world::discovery::seo` | SEO analysis |
| persona_check | `nt_core_self::planning::persona` | Consistency check |
| publish_checklist | `nt_act::production_orchestrator` | Quality gates |

### Create (50) → NT-ACT

| Easel Skill | NT-* Module | Notes |
|-------------|-------------|-------|
| social_media_copy | `nt_act::creation::copywriting` | Short-form text |
| long_form_writing | `nt_act::creation::longform` | Article writing |
| novel_writing | `nt_act::creation::fiction` | Fiction generation |
| style_transfer | `nt_act::creation::style` | Writing style |
| de_ai_rewrite | `nt_act::creation::deai` | Remove AI markers |
| paper_explainer | `nt_act::creation::explainer` | Academic explainers |
| card_design | `nt_act::creation::cards` | Visual cards |
| image_generation | `nt_io::reference_generation` | Image gen |
| image_enhancement | `nt_physical::video_post_processor` | Enhancement |
| background_removal | `nt_act::creation::bgremove` | Background removal |
| infographic | `nt_act::creation::infographic` | Data visualization |
| meme_generation | `nt_act::creation::meme` | Meme creation |
| tts | `nt_io::platform_publish::tts` | Text-to-speech |
| multi_voice_dubbing | `nt_io::platform_publish::dubbing` | Multi-voice |
| voice_cloning | `nt_io::platform_publish::clone` | Voice clone |
| ai_music | `nt_io::platform_publish::music` | Music generation |
| noise_reduction | `nt_physical::video_post_processor::audio` | Audio cleanup |
| audio_mixing | `nt_physical::video_post_processor::mix` | Audio mixing |
| audio_visualization | `nt_act::creation::audio_viz` | Waveform visuals |
| ai_video_generation | `nt_io::reference_generation::video` | Video gen |
| auto_short_video | `nt_act::production_orchestrator::short` | Short video pipeline |
| subtitles | `nt_act::creation::subtitles` | Subtitle generation |
| video_editing | `nt_act::creation::editing` | Video editing |
| clipping | `nt_act::creation::clipping` | Clip extraction |
| beat_sync | `nt_act::creation::beatsync` | Beat synchronization |
| green_screen | `nt_act::creation::greenscreen` | Chroma key |
| slideshow | `nt_act::creation::slideshow` | Image slideshow |
| chapter_generation | `nt_act::creation::chapters` | Chapter markers |
| + 22 more creation skills | `nt_act::creation::*` | Mapped to existing modules |

### Publish (20) → NT-IO

| Easel Skill | NT-* Module | Notes |
|-------------|-------------|-------|
| xiaohongshu_publish | `nt_io::platform_publish::xhs` | XHS adapter |
| douyin_publish | `nt_io::platform_publish::douyin` | Douyin adapter |
| bilibili_publish | `nt_io::platform_publish::bilibili` | Bilibili adapter |
| zhihu_publish | `nt_io::platform_publish::zhihu` | Zhihu adapter |
| kuaishou_publish | `nt_io::platform_publish::kuaishou` | Kuaishou adapter |
| wechat_publish | `nt_io::platform_publish::wechat` | WeChat adapter |
| quality_gate | `nt_meta::quality_control` | QualityControlPipeline |
| seo_optimization | `nt_world::discovery::seo` | SEO analysis |
| publish_scheduling | `nt_act::production_orchestrator` | Batch scheduling |
| cross_platform_publish | `nt_io::platform_publish::multi` | One-click multi-platform |
| + 10 more publish skills | `nt_io::platform_publish::*` | Mapped to adapters |

### Attribute (11) → NT-META

| Easel Skill | NT-* Module | Notes |
|-------------|-------------|-------|
| roi_calculation | `nt_meta::attribution::roi` | ROI metrics |
| comment_insights | `nt_meta::attribution::comments` | Sentiment analysis |
| content_postmortem | `nt_meta::attribution::postmortem` | Failure analysis |
| data_tracking | `nt_meta::attribution::tracking` | Data collection |
| publish_analytics | `nt_meta::attribution::analytics` | Performance analytics |
| strategy_advisor | `nt_meta::attribution::advisor` | Data-driven recommendations |
| monthly_review | `nt_meta::attribution::review` | Monthly reports |
| + 4 more attribute skills | `nt_meta::attribution::*` | Mapped to meta modules |

## Integration Principles

1. **R-P42**: Absorb into existing NT-* nodes, no parallel adapter modules
2. **R-P79**: Connect to production in same session, no dead code
3. **Dark Forest**: Only keep skills that connect to real NT-* domain consumers
4. **Easel Python → NeoTrix Rust**: Absorb methodology (patterns), not code (scripts)
5. **Content Guard → NT-SHIELD**: BLOCK (fail-closed) vs WARN (soft) dual-tier
6. **Manifest → SEAL Pipeline**: Thin index for inter-stage coordination

## Priority Matrix

| Priority | Action | Domain | Effort |
|----------|--------|--------|--------|
| P0 | SKILL-SPEC.md contract template | NT-ACT | Low |
| P0 | Easel→NT-* domain mapping (this doc) | NT-META | Done |
| P1 | Manifest-as-thin-index for SEAL | NT-MIND | Medium |
| P1 | Social media publishing module | NT-ACT | High |
| P1 | Content Guard BLOCK/WARN → NT-SHIELD | NT-SHIELD | Low |
| P2 | 4-layer prompt stack → GWT | NT-CORE | High |
| P2 | One-asset-multi-platform → ProductionOrchestrator | NT-ACT | Medium |
| P2 | Profile system → SelfModel extension | NT-CORE+NT-MEMORY | Medium |
| P3 | OpenClaw Agent → consciousness_task eval | NT-CORE | Low |

pub(super) const MINIMAL_WHITE_CSS: &str = r#"
.slide { background: #ffffff; color: #1a1a1a; }
.slide-title { font-size: 2.8rem; font-weight: 700; color: #1a1a1a; margin-bottom: 0.5em; }
.text-content p { font-size: 1.3rem; line-height: 1.7; color: #333; }
.bullet-list { list-style: disc; padding-left: 1.5em; font-size: 1.3rem; line-height: 2; color: #333; }
.two-column { display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; width: 100%; max-width: 1000px; }
.two-column .col { font-size: 1.15rem; line-height: 1.7; color: #333; }
.code-block { width: 100%; max-width: 900px; background: #f5f5f5; border-radius: 8px; overflow: hidden; }
.code-lang { background: #e0e0e0; padding: 6px 14px; font-size: 0.8rem; text-transform: uppercase; color: #555; }
.code-block pre { padding: 20px; overflow-x: auto; font-size: 0.95rem; line-height: 1.5; }
.stat-block { text-align: center; }
.stat-value { font-size: 3.5rem; font-weight: 800; color: #1a1a1a; }
.stat-label { font-size: 1.1rem; color: #666; margin-top: 4px; }
.stat-trend { font-size: 0.95rem; color: #22c55e; display: block; margin-top: 2px; }
.kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 1.5rem; width: 100%; max-width: 1000px; }
.timeline { width: 100%; max-width: 800px; }
.timeline-item { display: flex; gap: 1.5rem; margin-bottom: 1.5rem; padding-left: 1.5rem; border-left: 3px solid #ddd; }
.tl-date { font-weight: 700; color: #888; min-width: 80px; font-size: 0.95rem; }
.tl-content h3 { font-size: 1.2rem; margin-bottom: 0.2rem; }
.tl-content p { font-size: 1rem; color: #555; }
.comparison { display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; width: 100%; max-width: 1000px; }
.comparison h3 { font-size: 1.2rem; margin-bottom: 0.8rem; text-align: center; }
.comparison ul { list-style: disc; padding-left: 1.2em; font-size: 1.1rem; line-height: 1.8; }
.big-quote { max-width: 800px; text-align: center; }
.big-quote p { font-size: 1.8rem; font-style: italic; color: #333; line-height: 1.4; }
.big-quote cite { font-size: 1rem; color: #888; margin-top: 1rem; display: block; }
.image-container img { max-width: 80%; max-height: 60vh; border-radius: 8px; }
.image-container .img-caption { font-size: 0.9rem; color: #666; margin-top: 0.5rem; text-align: center; }
.process-steps { display: flex; flex-direction: column; gap: 1rem; width: 100%; max-width: 600px; }
.step { display: flex; align-items: center; gap: 1rem; }
.step-num { width: 36px; height: 36px; border-radius: 50%; background: #1a1a1a; color: #fff; display: flex; align-items: center; justify-content: center; font-weight: 700; flex-shrink: 0; }
.step-text { font-size: 1.15rem; }
"#;

pub(super) const CYBERPUNK_NEON_CSS: &str = r#"
.slide { background: #0a0a1a; color: #e0e0ff; }
.slide-title { font-size: 2.8rem; font-weight: 700; color: #00ffe1; text-shadow: 0 0 12px #00ffe1, 0 0 24px #00ffe1; margin-bottom: 0.5em; }
.text-content p { font-size: 1.3rem; line-height: 1.7; color: #c0c0f0; }
.bullet-list { list-style: none; padding-left: 0; font-size: 1.3rem; line-height: 2.2; color: #c0c0f0; }
.bullet-list li::before { content: "▸ "; color: #ff00e4; font-weight: 700; }
.two-column { display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; width: 100%; max-width: 1000px; }
.two-column .col { font-size: 1.15rem; line-height: 1.7; color: #c0c0f0; border: 1px solid #2a2a5a; border-radius: 8px; padding: 1.5rem; }
.code-block { width: 100%; max-width: 900px; background: #0d0d2b; border: 1px solid #00ffe1; border-radius: 8px; overflow: hidden; }
.code-lang { background: #1a1a4a; padding: 6px 14px; font-size: 0.8rem; text-transform: uppercase; color: #00ffe1; }
.code-block pre { padding: 20px; overflow-x: auto; font-size: 0.95rem; line-height: 1.5; color: #c0c0f0; }
.stat-block { text-align: center; }
.stat-value { font-size: 3.5rem; font-weight: 800; color: #ff00e4; text-shadow: 0 0 12px #ff00e4; }
.stat-label { font-size: 1.1rem; color: #8888cc; margin-top: 4px; }
.stat-trend { font-size: 0.95rem; color: #00ff88; display: block; margin-top: 2px; }
.kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 1.5rem; width: 100%; max-width: 1000px; }
.timeline { width: 100%; max-width: 800px; }
.timeline-item { display: flex; gap: 1.5rem; margin-bottom: 1.5rem; padding-left: 1.5rem; border-left: 3px solid #00ffe1; }
.tl-date { font-weight: 700; color: #00ffe1; min-width: 80px; font-size: 0.95rem; }
.tl-content h3 { font-size: 1.2rem; margin-bottom: 0.2rem; color: #e0e0ff; }
.tl-content p { font-size: 1rem; color: #8888cc; }
.comparison { display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; width: 100%; max-width: 1000px; }
.comparison h3 { font-size: 1.2rem; margin-bottom: 0.8rem; text-align: center; color: #00ffe1; }
.comparison ul { list-style: none; padding-left: 0; font-size: 1.1rem; line-height: 2; }
.comparison ul li::before { content: "⚡ "; }
.big-quote { max-width: 800px; text-align: center; }
.big-quote p { font-size: 1.8rem; font-style: italic; color: #ff00e4; text-shadow: 0 0 8px #ff00e4; line-height: 1.4; }
.big-quote cite { font-size: 1rem; color: #8888cc; margin-top: 1rem; display: block; }
.image-container img { max-width: 80%; max-height: 60vh; border-radius: 8px; border: 2px solid #2a2a5a; }
.image-container .img-caption { font-size: 0.9rem; color: #8888cc; margin-top: 0.5rem; text-align: center; }
.process-steps { display: flex; flex-direction: column; gap: 1rem; width: 100%; max-width: 600px; }
.step { display: flex; align-items: center; gap: 1rem; }
.step-num { width: 36px; height: 36px; border-radius: 50%; background: #ff00e4; color: #0a0a1a; display: flex; align-items: center; justify-content: center; font-weight: 700; flex-shrink: 0; box-shadow: 0 0 10px #ff00e4; }
.step-text { font-size: 1.15rem; color: #c0c0f0; }
"#;

pub(super) const SOFT_PASTEL_CSS: &str = r#"
.slide { background: linear-gradient(135deg, #fef9f0 0%, #f8f0fe 100%); color: #4a4a5a; }
.slide-title { font-size: 2.8rem; font-weight: 600; color: #7c6f9e; margin-bottom: 0.5em; }
.text-content p { font-size: 1.3rem; line-height: 1.7; color: #6a6a7a; }
.bullet-list { list-style: none; padding-left: 0; font-size: 1.25rem; line-height: 2; color: #6a6a7a; }
.bullet-list li::before { content: "🌸 "; }
.two-column { display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; width: 100%; max-width: 1000px; }
.two-column .col { font-size: 1.15rem; line-height: 1.7; color: #6a6a7a; background: rgba(255,255,255,0.5); border-radius: 16px; padding: 1.5rem; }
.code-block { width: 100%; max-width: 900px; background: #fff5f5; border-radius: 12px; overflow: hidden; box-shadow: 0 2px 12px rgba(0,0,0,0.04); }
.code-lang { background: #f0e6f6; padding: 6px 14px; font-size: 0.8rem; text-transform: uppercase; color: #9b8abf; }
.code-block pre { padding: 20px; overflow-x: auto; font-size: 0.95rem; line-height: 1.5; color: #5a5a6a; }
.stat-block { text-align: center; }
.stat-value { font-size: 3.5rem; font-weight: 700; color: #e8a0c0; }
.stat-label { font-size: 1.1rem; color: #9a9aaa; margin-top: 4px; }
.stat-trend { font-size: 0.95rem; color: #a0d8a0; display: block; margin-top: 2px; }
.kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 1.5rem; width: 100%; max-width: 1000px; }
.timeline { width: 100%; max-width: 800px; }
.timeline-item { display: flex; gap: 1.5rem; margin-bottom: 1.5rem; padding-left: 1.5rem; border-left: 3px solid #d4c4e8; }
.tl-date { font-weight: 600; color: #b8a8cc; min-width: 80px; font-size: 0.95rem; }
.tl-content h3 { font-size: 1.2rem; margin-bottom: 0.2rem; color: #7c6f9e; }
.tl-content p { font-size: 1rem; color: #8a8a9a; }
.comparison { display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; width: 100%; max-width: 1000px; }
.comparison h3 { font-size: 1.2rem; margin-bottom: 0.8rem; text-align: center; color: #7c6f9e; }
.comparison ul { list-style: none; padding-left: 0; font-size: 1.1rem; line-height: 2; }
.comparison ul li::before { content: "✨ "; }
.big-quote { max-width: 800px; text-align: center; }
.big-quote p { font-size: 1.8rem; font-style: italic; color: #9b8abf; line-height: 1.4; }
.big-quote cite { font-size: 1rem; color: #b8a8cc; margin-top: 1rem; display: block; }
.image-container img { max-width: 80%; max-height: 60vh; border-radius: 16px; box-shadow: 0 4px 20px rgba(0,0,0,0.06); }
.image-container .img-caption { font-size: 0.9rem; color: #9a9aaa; margin-top: 0.5rem; text-align: center; }
.process-steps { display: flex; flex-direction: column; gap: 1rem; width: 100%; max-width: 600px; }
.step { display: flex; align-items: center; gap: 1rem; }
.step-num { width: 36px; height: 36px; border-radius: 50%; background: #e8a0c0; color: #fff; display: flex; align-items: center; justify-content: center; font-weight: 600; flex-shrink: 0; }
.step-text { font-size: 1.15rem; color: #6a6a7a; }
"#;

pub(super) const CORPORATE_CLEAN_CSS: &str = r#"
.slide { background: #f8f9fc; color: #2c3e50; }
.slide-title { font-size: 2.4rem; font-weight: 700; color: #1a365d; margin-bottom: 0.5em; letter-spacing: -0.02em; }
.text-content p { font-size: 1.2rem; line-height: 1.7; color: #4a5568; }
.bullet-list { list-style: none; padding-left: 0; font-size: 1.2rem; line-height: 2; color: #4a5568; }
.bullet-list li::before { content: "• "; color: #3182ce; font-weight: 700; }
.two-column { display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; width: 100%; max-width: 1000px; }
.two-column .col { font-size: 1.1rem; line-height: 1.7; color: #4a5568; background: #ffffff; border-radius: 8px; padding: 1.5rem; box-shadow: 0 1px 4px rgba(0,0,0,0.06); }
.code-block { width: 100%; max-width: 900px; background: #1e293b; border-radius: 8px; overflow: hidden; }
.code-lang { background: #0f172a; padding: 6px 14px; font-size: 0.75rem; text-transform: uppercase; color: #94a3b8; }
.code-block pre { padding: 20px; overflow-x: auto; font-size: 0.9rem; line-height: 1.5; color: #e2e8f0; }
.stat-block { text-align: center; background: #ffffff; border-radius: 8px; padding: 1.5rem; box-shadow: 0 1px 4px rgba(0,0,0,0.06); }
.stat-value { font-size: 3rem; font-weight: 800; color: #3182ce; }
.stat-label { font-size: 1rem; color: #718096; margin-top: 4px; }
.stat-trend { font-size: 0.9rem; color: #38a169; display: block; margin-top: 2px; }
.kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 1rem; width: 100%; max-width: 1000px; }
.timeline { width: 100%; max-width: 800px; }
.timeline-item { display: flex; gap: 1.5rem; margin-bottom: 1.5rem; padding-left: 1.5rem; border-left: 3px solid #3182ce; }
.tl-date { font-weight: 600; color: #3182ce; min-width: 80px; font-size: 0.9rem; }
.tl-content h3 { font-size: 1.15rem; margin-bottom: 0.2rem; color: #1a365d; }
.tl-content p { font-size: 0.95rem; color: #718096; }
.comparison { display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; width: 100%; max-width: 1000px; }
.comparison h3 { font-size: 1.15rem; margin-bottom: 0.8rem; text-align: center; color: #1a365d; text-transform: uppercase; letter-spacing: 0.05em; }
.comparison ul { list-style: none; padding-left: 0; font-size: 1rem; line-height: 2; }
.comparison ul li::before { content: "→ "; color: #3182ce; }
.big-quote { max-width: 800px; text-align: center; }
.big-quote p { font-size: 1.6rem; font-style: italic; color: #2d3748; line-height: 1.4; }
.big-quote cite { font-size: 0.95rem; color: #718096; margin-top: 1rem; display: block; }
.image-container img { max-width: 80%; max-height: 60vh; border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.1); }
.image-container .img-caption { font-size: 0.85rem; color: #718096; margin-top: 0.5rem; text-align: center; }
.process-steps { display: flex; flex-direction: column; gap: 1rem; width: 100%; max-width: 600px; }
.step { display: flex; align-items: center; gap: 1rem; }
.step-num { width: 36px; height: 36px; border-radius: 50%; background: #3182ce; color: #fff; display: flex; align-items: center; justify-content: center; font-weight: 700; flex-shrink: 0; }
.step-text { font-size: 1.1rem; color: #4a5568; }
"#;

pub(super) const ACADEMIC_PAPER_CSS: &str = r#"
.slide { background: #fafaf8; color: #2d2d2d; }
.slide-title { font-size: 2.2rem; font-weight: 700; color: #1a1a1a; margin-bottom: 0.5em; font-family: 'Georgia', 'Times New Roman', serif; }
.text-content p { font-size: 1.15rem; line-height: 1.8; color: #333; font-family: 'Georgia', serif; }
.bullet-list { list-style: none; padding-left: 0; font-size: 1.1rem; line-height: 2; color: #333; }
.bullet-list li::before { content: "▪ "; color: #8b4513; }
.two-column { display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; width: 100%; max-width: 1000px; }
.two-column .col { font-size: 1.05rem; line-height: 1.7; color: #333; }
.code-block { width: 100%; max-width: 900px; background: #f4f1ea; border: 1px solid #d4c9b8; border-radius: 4px; overflow: hidden; }
.code-lang { background: #e8e0d0; padding: 4px 12px; font-size: 0.75rem; text-transform: uppercase; color: #5a4a3a; }
.code-block pre { padding: 16px; overflow-x: auto; font-size: 0.85rem; line-height: 1.5; color: #2d2d2d; }
.stat-block { text-align: center; border: 1px solid #d4c9b8; padding: 1.5rem; border-radius: 4px; }
.stat-value { font-size: 3rem; font-weight: 700; color: #8b4513; }
.stat-label { font-size: 1rem; color: #5a4a3a; margin-top: 4px; }
.stat-trend { font-size: 0.9rem; color: #2d6a2d; display: block; margin-top: 2px; }
.kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 1rem; width: 100%; max-width: 1000px; }
.timeline { width: 100%; max-width: 800px; }
.timeline-item { display: flex; gap: 1.5rem; margin-bottom: 1.5rem; padding-left: 1.5rem; border-left: 2px solid #8b4513; }
.tl-date { font-weight: 600; color: #8b4513; min-width: 80px; font-size: 0.9rem; }
.tl-content h3 { font-size: 1.1rem; margin-bottom: 0.2rem; color: #1a1a1a; }
.tl-content p { font-size: 0.95rem; color: #5a4a3a; }
.comparison { display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; width: 100%; max-width: 1000px; }
.comparison h3 { font-size: 1.1rem; margin-bottom: 0.8rem; text-align: center; color: #8b4513; border-bottom: 1px solid #d4c9b8; padding-bottom: 0.5rem; }
.comparison ul { list-style: none; padding-left: 0; font-size: 1rem; line-height: 2; }
.comparison ul li::before { content: "§ "; color: #8b4513; }
.big-quote { max-width: 800px; text-align: center; }
.big-quote p { font-size: 1.5rem; font-style: italic; color: #2d2d2d; line-height: 1.5; font-family: 'Georgia', serif; }
.big-quote cite { font-size: 0.9rem; color: #8b4513; margin-top: 1rem; display: block; }
.image-container img { max-width: 80%; max-height: 60vh; border: 1px solid #d4c9b8; border-radius: 2px; }
.image-container .img-caption { font-size: 0.85rem; color: #5a4a3a; margin-top: 0.5rem; text-align: center; font-style: italic; }
.process-steps { display: flex; flex-direction: column; gap: 1rem; width: 100%; max-width: 600px; }
.step { display: flex; align-items: center; gap: 1rem; }
.step-num { width: 36px; height: 36px; border-radius: 0; background: #8b4513; color: #fafaf8; display: flex; align-items: center; justify-content: center; font-weight: 700; flex-shrink: 0; font-family: 'Georgia', serif; }
.step-text { font-size: 1.05rem; color: #333; font-family: 'Georgia', serif; }
"#;

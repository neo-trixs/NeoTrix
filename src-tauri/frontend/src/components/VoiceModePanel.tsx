import React, { useEffect, useState, useRef, useCallback } from "react";
import {
  voiceStartSession,
  voiceStopSession,
  voiceSendAudio,
  voiceGetTranscription,
  voiceListSessions,
  voiceSessionHistory,
  voiceSynthesize,
  voiceConfig,
  voiceSetConfig,
  voiceTestMicrophone,
  voiceStats,
  voiceExecuteCommand,
} from "../commands";
import type { VoiceTranscript, VoiceConfig, VoiceSession, VoiceStats } from "../commands";

const LANGUAGES = [
  { code: "zh", label: "中文" },
  { code: "en", label: "English" },
  { code: "ja", label: "日本語" },
  { code: "ko", label: "한국어" },
  { code: "fr", label: "Français" },
  { code: "de", label: "Deutsch" },
  { code: "es", label: "Español" },
  { code: "pt", label: "Português" },
  { code: "ru", label: "Русский" },
  { code: "ar", label: "العربية" },
];

const STT_BACKENDS = ["mock", "whisper", "api"];

const TTS_VOICES = [
  { id: "default", label: "Default" },
  { id: "nova", label: "Nova" },
  { id: "alloy", label: "Alloy" },
  { id: "shimmer", label: "Shimmer" },
  { id: "echo", label: "Echo" },
  { id: "fable", label: "Fable" },
  { id: "onyx", label: "Onyx" },
];

const COMMAND_TEMPLATES: { pattern: RegExp; action: string }[] = [
  { pattern: /^(open|create|make)\s+(file|module|class)\s+/i, action: "create_file" },
  { pattern: /^(fix|repair|correct)\s+/i, action: "fix_code" },
  { pattern: /^(test|check|audit)\s+/i, action: "run_check" },
  { pattern: /^(deploy|publish|release)\s+/i, action: "deploy" },
  { pattern: /^(search|find|lookup)\s+/i, action: "search" },
  { pattern: /^(explain|describe|what is)\s+/i, action: "explain" },
  { pattern: /^(refactor|rewrite|restructure)\s+/i, action: "refactor" },
  { pattern: /^(commit|push|sync)\s+/i, action: "git_operation" },
  { pattern: /^(add|install)\s+(dependency|crate|package)/i, action: "add_dependency" },
];

const LABEL_COLORS: Record<string, string> = {
  zh: "#22c55e",
  en: "#3b82f6",
  ja: "#f59e0b",
  ko: "#a855f7",
  fr: "#ef4444",
  de: "#14b8a6",
  es: "#f97316",
  pt: "#ec4899",
  ru: "#06b6d4",
  ar: "#8b5cf6",
};

function detectCommand(text: string): string | null {
  for (const tpl of COMMAND_TEMPLATES) {
    if (tpl.pattern.test(text)) return tpl.action;
  }
  return null;
}

function timeAgo(ts: string): string {
  const diff = Date.now() - new Date(ts).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  return `${Math.floor(hrs / 24)}d ago`;
}

const VoiceModePanel: React.FC = () => {
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [listening, setListening] = useState(false);
  const [language, setLanguage] = useState("en");
  const [sttBackend, setSttBackend] = useState("mock");
  const [ttsEnabled, setTtsEnabled] = useState(false);
  const [ttsVoice, setTtsVoice] = useState("default");
  const [autoSubmit, setAutoSubmit] = useState(false);
  const [wakeWord, setWakeWord] = useState("");
  const [pushToTalk, setPushToTalk] = useState(true);
  const [transcripts, setTranscripts] = useState<VoiceTranscript[]>([]);
  const [sessions, setSessions] = useState<VoiceSession[]>([]);
  const [stats, setStats] = useState<VoiceStats | null>(null);
  const [config, setConfig] = useState<VoiceConfig | null>(null);
  const [micOk, setMicOk] = useState<boolean | null>(null);
  const [micTesting, setMicTesting] = useState(false);
  const [sending, setSending] = useState(false);
  const [statusMsg, setStatusMsg] = useState("");
  const [pulse, setPulse] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const transcriptEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    voiceConfig().then(setConfig).catch(() => {});
    voiceStats().then(setStats).catch(() => {});
    voiceListSessions().then(setSessions).catch(() => {});
  }, []);

  useEffect(() => {
    if (!config) return;
    setLanguage(config.language);
    setSttBackend(config.stt_backend);
    setTtsEnabled(config.tts_enabled);
    setTtsVoice(config.tts_voice);
    setAutoSubmit(config.auto_submit);
    setWakeWord(config.wake_word);
    setPushToTalk(config.push_to_talk);
  }, [config]);

  useEffect(() => {
    if (!listening) { setPulse(false); return; }
    intervalRef.current = setInterval(() => setPulse((p) => !p), 600);
    return () => { if (intervalRef.current) clearInterval(intervalRef.current); };
  }, [listening]);

  useEffect(() => { transcriptEndRef.current?.scrollIntoView({ behavior: "smooth" }); }, [transcripts]);

  const refreshSessions = useCallback(() => {
    voiceListSessions().then(setSessions).catch(() => {});
    voiceStats().then(setStats).catch(() => {});
  }, []);

  const toggleSession = async () => {
    if (sessionId) {
      try {
        const ended = await voiceStopSession(sessionId);
        setStatusMsg(`Session ended — ${ended.commands_executed} commands, ${ended.duration_secs}s`);
        setSessionId(null);
        setListening(false);
      } catch { setStatusMsg("Failed to stop session"); }
      refreshSessions();
    } else {
      try {
        const id = await voiceStartSession(language);
        setSessionId(id);
        setListening(true);
        setStatusMsg("Session active — speaking enabled");
        setTranscripts([]);
      } catch { setStatusMsg("Failed to start session"); }
    }
  };

  const simulatePushToTalk = async () => {
    if (!sessionId || sending) return;
    setSending(true);
    setStatusMsg("Capturing audio...");
    const fakeAudio = btoa("mock-audio-data");
    try {
      const transcript = await voiceSendAudio(sessionId, fakeAudio);
      setTranscripts((prev) => [...prev, transcript]);
      if (transcript.is_final && autoSubmit) {
        const result = await voiceExecuteCommand(transcript.text);
        setStatusMsg(`Command result: ${result.slice(0, 120)}`);
      } else {
        setStatusMsg("Transcribed");
      }
    } catch {
      setStatusMsg("Audio processing failed");
    }
    setSending(false);
  };

  const testMic = async () => {
    setMicTesting(true);
    try {
      const ok = await voiceTestMicrophone();
      setMicOk(ok);
      setStatusMsg(ok ? "Microphone OK" : "Microphone not detected");
    } catch {
      setMicOk(false);
      setStatusMsg("Microphone test failed");
    }
    setMicTesting(false);
  };

  const updateConfig = async (patch: Partial<VoiceConfig>) => {
    if (!config) return;
    const next = { ...config, ...patch };
    try {
      await voiceSetConfig(next);
      setConfig(next);
    } catch { setStatusMsg("Config update failed"); }
  };

  const loadSessionHistory = async (sid: string) => {
    try {
      const history = await voiceSessionHistory(sid);
      setTranscripts(history);
      setStatusMsg(`Loaded ${history.length} transcript(s)`);
    } catch { setStatusMsg("Failed to load history"); }
  };

  const matchedCommand = transcripts.length > 0
    ? detectCommand(transcripts[transcripts.length - 1].text)
    : null;

  return (
    <div style={{
      display: "flex", flexDirection: "column", height: "100%",
      background: "#0b0d10", color: "#e0e2e6", fontFamily: "inherit",
      overflow: "hidden",
    }}>
      <div style={{ display: "flex", flex: 1, overflow: "hidden" }}>
        {/* ─── Left: control panel ─── */}
        <div style={{
          width: 360, minWidth: 360, display: "flex", flexDirection: "column",
          borderRight: "1px solid #1e2025", padding: 20, gap: 16,
          overflowY: "auto", background: "#0d0f13",
        }}>
          {/* Session control */}
          <div style={{ textAlign: "center", padding: "16px 0" }}>
            <button
              onClick={toggleSession}
              style={{
                width: 80, height: 80, borderRadius: "50%", border: "none",
                cursor: "pointer", position: "relative",
                background: listening
                  ? "linear-gradient(135deg, #ef4444, #dc2626)"
                  : "linear-gradient(135deg, #22c55e, #16a34a)",
                boxShadow: listening && pulse
                  ? "0 0 0 8px rgba(239,68,68,0.3), 0 0 0 16px rgba(239,68,68,0.12)"
                  : "0 4px 12px rgba(0,0,0,0.4)",
                transition: "all 0.3s",
              }}
              title={listening ? "Stop session" : "Start session"}
            >
              {listening ? (
                <svg viewBox="0 0 24 24" fill="white" width="32" height="32">
                  <rect x="6" y="4" width="5" height="16" rx="1" />
                  <rect x="13" y="4" width="5" height="16" rx="1" />
                </svg>
              ) : (
                <svg viewBox="0 0 24 24" fill="white" width="32" height="32">
                  <path d="M12 14a3 3 0 003-3V5a3 3 0 00-6 0v6a3 3 0 003 3z" />
                  <path d="M17 11a5 5 0 01-10 0M12 17v4" stroke="white" strokeWidth="1.5" fill="none" />
                </svg>
              )}
            </button>
            <div style={{ fontSize: 13, color: listening ? "#ef4444" : "#6b7280", marginTop: 12 }}>
              {listening ? "● LISTENING" : "STANDBY"}
            </div>
          </div>

          {/* Push to Talk button */}
          <button
            onClick={simulatePushToTalk}
            disabled={!sessionId || sending}
            style={{
              padding: "10px 0", borderRadius: 8, border: "1px solid #2a2d35",
              background: sending ? "#1e2025" : "#15171c",
              color: !sessionId ? "#4b5563" : "#e0e2e6", cursor: !sessionId ? "not-allowed" : "pointer",
              fontSize: 14, fontWeight: 600, display: "flex", alignItems: "center", justifyContent: "center", gap: 8,
            }}
          >
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" width="16" height="16">
              <path d="M8 10a2.5 2.5 0 002.5-2.5V3a2.5 2.5 0 00-5 0v4.5A2.5 2.5 0 008 10z" />
              <path d="M13 7.5a5 5 0 01-10 0" />
              <path d="M8 13v2" />
            </svg>
            {sending ? "Processing..." : "Push to Talk"}
          </button>

          {/* Language selector */}
          <div>
            <div style={{ fontSize: 11, color: "#6b7280", marginBottom: 4, textTransform: "uppercase", letterSpacing: "0.05em" }}>
              Language
            </div>
            <select
              value={language}
              onChange={(e) => { setLanguage(e.target.value); updateConfig({ language: e.target.value }); }}
              style={selectStyle}
            >
              {LANGUAGES.map((l) => (
                <option key={l.code} value={l.code}>{l.label}</option>
              ))}
            </select>
          </div>

          {/* STT Backend + TTS */}
          <div>
            <div style={{ fontSize: 11, color: "#6b7280", marginBottom: 4, textTransform: "uppercase", letterSpacing: "0.05em" }}>
              STT Backend
            </div>
            <div style={{ display: "flex", gap: 4 }}>
              {STT_BACKENDS.map((b) => (
                <button
                  key={b}
                  onClick={() => { setSttBackend(b); updateConfig({ stt_backend: b }); }}
                  style={{
                    flex: 1, padding: "6px 0", borderRadius: 6, border: "1px solid #2a2d35",
                    background: sttBackend === b ? "#2563eb" : "#15171c",
                    color: sttBackend === b ? "white" : "#9ca3af",
                    fontSize: 12, cursor: "pointer",
                  }}
                >{b}</button>
              ))}
            </div>
          </div>

          {/* TTS toggle */}
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
            <span style={{ fontSize: 13 }}>Text-to-Speech</span>
            <div
              onClick={() => { setTtsEnabled(!ttsEnabled); updateConfig({ tts_enabled: !ttsEnabled }); }}
              style={{
                width: 36, height: 20, borderRadius: 10, cursor: "pointer", position: "relative",
                background: ttsEnabled ? "#2563eb" : "#2a2d35", transition: "background 0.2s",
              }}
            >
              <div style={{
                width: 16, height: 16, borderRadius: "50%", background: "white",
                position: "absolute", top: 2, left: ttsEnabled ? 18 : 2, transition: "left 0.2s",
              }} />
            </div>
          </div>

          {/* TTS Voice */}
          {ttsEnabled && (
            <div>
              <div style={{ fontSize: 11, color: "#6b7280", marginBottom: 4, textTransform: "uppercase", letterSpacing: "0.05em" }}>
                Voice
              </div>
              <select
                value={ttsVoice}
                onChange={(e) => { setTtsVoice(e.target.value); updateConfig({ tts_voice: e.target.value }); }}
                style={selectStyle}
              >
                {TTS_VOICES.map((v) => (
                  <option key={v.id} value={v.id}>{v.label}</option>
                ))}
              </select>
            </div>
          )}

          {/* Options */}
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            <label style={{ display: "flex", alignItems: "center", gap: 8, fontSize: 13, cursor: "pointer" }}>
              <input
                type="checkbox" checked={autoSubmit}
                onChange={(e) => { setAutoSubmit(e.target.checked); updateConfig({ auto_submit: e.target.checked }); }}
                style={{ accentColor: "#2563eb" }}
              />
              Auto-submit commands
            </label>
            <label style={{ display: "flex", alignItems: "center", gap: 8, fontSize: 13, cursor: "pointer" }}>
              <input
                type="checkbox" checked={pushToTalk}
                onChange={(e) => { setPushToTalk(e.target.checked); updateConfig({ push_to_talk: e.target.checked }); }}
                style={{ accentColor: "#2563eb" }}
              />
              Push to Talk mode
            </label>
          </div>

          {/* Wake word */}
          <div>
            <div style={{ fontSize: 11, color: "#6b7280", marginBottom: 4, textTransform: "uppercase", letterSpacing: "0.05em" }}>
              Wake Word
            </div>
            <input
              value={wakeWord}
              onChange={(e) => { setWakeWord(e.target.value); updateConfig({ wake_word: e.target.value }); }}
              placeholder="e.g. hey neotrix"
              style={inputStyle}
            />
          </div>

          {/* Test Microphone */}
          <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <button
              onClick={testMic}
              disabled={micTesting}
              style={{
                flex: 1, padding: "8px 0", borderRadius: 6, border: "1px solid #2a2d35",
                background: micTesting ? "#1e2025" : "#15171c",
                color: "#e0e2e6", fontSize: 13, cursor: "pointer",
              }}
            >
              {micTesting ? "Testing..." : "Test Microphone"}
            </button>
            {micOk === true && <span style={{ color: "#22c55e", fontSize: 16 }}>✓</span>}
            {micOk === false && <span style={{ color: "#ef4444", fontSize: 16 }}>✗</span>}
          </div>

          {/* Status */}
          {statusMsg && (
            <div style={{ fontSize: 12, color: "#9ca3af", padding: "6px 10px", background: "#15171c", borderRadius: 6 }}>
              {statusMsg}
            </div>
          )}

          {/* Stats */}
          {stats && (
            <div style={{ marginTop: 8, padding: "10px 12px", background: "#111317", borderRadius: 8, border: "1px solid #1e2025" }}>
              <div style={{ fontSize: 11, color: "#6b7280", marginBottom: 8, textTransform: "uppercase", letterSpacing: "0.05em" }}>
                Statistics
              </div>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "6px 12px", fontSize: 12 }}>
                <span style={{ color: "#9ca3af" }}>Sessions</span>
                <span style={{ color: "#e0e2e6", fontWeight: 600 }}>{stats.total_sessions}</span>
                <span style={{ color: "#9ca3af" }}>Commands Exec.</span>
                <span style={{ color: "#e0e2e6", fontWeight: 600 }}>{stats.commands_executed}</span>
                <span style={{ color: "#9ca3af" }}>Avg Confidence</span>
                <span style={{ color: "#e0e2e6", fontWeight: 600 }}>{(stats.avg_confidence * 100).toFixed(0)}%</span>
                <span style={{ color: "#9ca3af" }}>Top Lang</span>
                <span style={{ color: "#e0e2e6", fontWeight: 600 }}>
                  {stats.top_languages[0]?.[0]?.toUpperCase() ?? "—"}
                </span>
              </div>
              {stats.top_languages.length > 1 && (
                <div style={{ marginTop: 8 }}>
                  <div style={{ fontSize: 10, color: "#6b7280", marginBottom: 4 }}>Languages</div>
                  {stats.top_languages.slice(0, 4).map(([lang, count]) => (
                    <div key={lang} style={{ display: "flex", alignItems: "center", gap: 6, marginBottom: 2 }}>
                      <span style={{
                        width: 6, height: 6, borderRadius: "50%",
                        background: LABEL_COLORS[lang] ?? "#6b7280",
                      }} />
                      <span style={{ fontSize: 11, color: "#9ca3af" }}>{lang.toUpperCase()}</span>
                      <span style={{ fontSize: 11, color: "#6b7280", marginLeft: "auto" }}>{count}</span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>

        {/* ─── Right: transcripts + command mapping ─── */}
        <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "hidden" }}>
          {/* Command mapping banner */}
          {transcripts.length > 0 && (
            <div style={{
              padding: "8px 16px", background: "#111317", borderBottom: "1px solid #1e2025",
              display: "flex", alignItems: "center", gap: 12, fontSize: 13,
            }}>
              <span style={{ color: "#6b7280" }}>Last command:</span>
              <code style={{
                background: "#1e2025", padding: "2px 8px", borderRadius: 4, color: "#93c5fd",
              }}>
                {matchedCommand ?? "unknown"}
              </code>
              {matchedCommand && (
                <span style={{
                  marginLeft: "auto", fontSize: 11, color: "#22c55e",
                  background: "rgba(34,197,94,0.1)", padding: "2px 8px", borderRadius: 4,
                }}>
                  MAPPED
                </span>
              )}
            </div>
          )}

          {/* Transcript list */}
          <div style={{ flex: 1, overflowY: "auto", padding: 16 }}>
            {transcripts.length === 0 && (
              <div style={{
                display: "flex", alignItems: "center", justifyContent: "center", height: "100%",
                color: "#4b5563", fontSize: 14, textAlign: "center", flexDirection: "column", gap: 8,
              }}>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" width="40" height="40" style={{ opacity: 0.3 }}>
                  <path d="M12 1a3 3 0 00-3 3v8a3 3 0 006 0V4a3 3 0 00-3-3z" />
                  <path d="M19 10v2a7 7 0 01-14 0v-2" />
                  <path d="M12 19v4" />
                  <path d="M8 23h8" />
                </svg>
                <div>No transcripts yet</div>
                <div style={{ fontSize: 12 }}>Start a session and use Push to Talk</div>
              </div>
            )}
            {transcripts.map((t, i) => {
              const cmd = detectCommand(t.text);
              return (
                <div
                  key={t.id ?? i}
                  style={{
                    marginBottom: 10, padding: "10px 12px", borderRadius: 8,
                    background: "#111317", border: "1px solid #1e2025",
                  }}
                >
                  <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 6 }}>
                    <span style={{ fontSize: 11, color: "#6b7280" }}>{timeAgo(t.timestamp)}</span>
                    <span style={{
                      fontSize: 10, fontWeight: 600, padding: "1px 6px", borderRadius: 4,
                      background: `${LABEL_COLORS[t.language] ?? "#6b7280"}22`,
                      color: LABEL_COLORS[t.language] ?? "#6b7280",
                    }}>
                      {t.language.toUpperCase()}
                    </span>
                    {t.is_final ? (
                      <span style={{ fontSize: 10, color: "#22c55e" }}>FINAL</span>
                    ) : (
                      <span style={{ fontSize: 10, color: "#f59e0b" }}>PARTIAL</span>
                    )}
                    {cmd && (
                      <span style={{
                        fontSize: 10, marginLeft: "auto", color: "#93c5fd",
                        background: "rgba(59,130,246,0.1)", padding: "1px 6px", borderRadius: 4,
                      }}>
                        {cmd}
                      </span>
                    )}
                  </div>
                  <div style={{ fontSize: 14, color: "#e0e2e6", marginBottom: 6 }}>{t.text}</div>
                  <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                    <div style={{
                      flex: 1, height: 4, borderRadius: 2, background: "#1e2025",
                    }}>
                      <div style={{
                        width: `${(t.confidence * 100).toFixed(0)}%`, height: "100%",
                        borderRadius: 2,
                        background: t.confidence > 0.8 ? "#22c55e"
                          : t.confidence > 0.5 ? "#f59e0b" : "#ef4444",
                        transition: "width 0.3s",
                      }} />
                    </div>
                    <span style={{
                      fontSize: 10, fontWeight: 600, minWidth: 32, textAlign: "right",
                      color: t.confidence > 0.8 ? "#22c55e"
                        : t.confidence > 0.5 ? "#f59e0b" : "#ef4444",
                    }}>
                      {(t.confidence * 100).toFixed(0)}%
                    </span>
                  </div>
                </div>
              );
            })}
            <div ref={transcriptEndRef} />
          </div>

          {/* Sessions footer */}
          {sessions.length > 0 && (
            <div style={{
              padding: "8px 16px", borderTop: "1px solid #1e2025",
              background: "#0d0f13", display: "flex", gap: 8, overflowX: "auto",
            }}>
              {sessions.slice(-6).reverse().map((s) => (
                <button
                  key={s.id}
                  onClick={() => loadSessionHistory(s.id)}
                  style={{
                    padding: "4px 10px", borderRadius: 6, border: "1px solid #2a2d35",
                    background: s.id === sessionId ? "#1e3a5f" : "#15171c",
                    color: s.id === sessionId ? "#93c5fd" : "#9ca3af",
                    fontSize: 11, cursor: "pointer", whiteSpace: "nowrap",
                  }}
                >
                  {new Date(s.started_at).toLocaleDateString()} · {s.commands_executed}cmd
                </button>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

const selectStyle: React.CSSProperties = {
  width: "100%", padding: "6px 8px", borderRadius: 6, border: "1px solid #2a2d35",
  background: "#15171c", color: "#e0e2e6", fontSize: 13, outline: "none",
};

const inputStyle: React.CSSProperties = {
  width: "100%", padding: "6px 8px", borderRadius: 6, border: "1px solid #2a2d35",
  background: "#15171c", color: "#e0e2e6", fontSize: 13, outline: "none", boxSizing: "border-box",
};

export default VoiceModePanel;

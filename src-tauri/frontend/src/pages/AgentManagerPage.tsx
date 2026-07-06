import React, { useState, useCallback } from "react";
import { useStore } from "../stores";

/* ─── Types ─── */

interface ProviderItem {
  name: string;
  region: string;
  provider: string;
  status: "on" | "off" | "warn";
  barPct: number;
  latency: string;
  rpm: string;
  score: "S" | "A" | "B" | "C" | "D";
}

interface ProxyItem {
  name: string;
  region: string;
  provider: string;
  status: "on" | "off" | "warn";
  barPct: number;
  latency: string;
  rpm: string;
  score: "S" | "A" | "B" | "C" | "D";
}

interface ChainStep {
  label: string;
  state: "idle" | "active" | "done";
}

/* ─── Mock Data ─── */

const LLM_PROVIDERS: ProviderItem[] = [
  { name: "Anthropic", region: "us-west", provider: "Claude Opus 4", status: "on", barPct: 94, latency: "12ms", rpm: "450", score: "S" },
  { name: "OpenAI", region: "us-east", provider: "GPT-4o", status: "on", barPct: 88, latency: "24ms", rpm: "380", score: "A" },
  { name: "Gemini", region: "eu-west", provider: "Gemini 2.5 Pro", status: "on", barPct: 76, latency: "48ms", rpm: "210", score: "B" },
  { name: "Groq", region: "us-west", provider: "Llama-3.3 70B", status: "warn", barPct: 62, latency: "85ms", rpm: "95", score: "C" },
  { name: "OpenRouter", region: "multi", provider: "Mixtral 8x22B", status: "on", barPct: 71, latency: "56ms", rpm: "140", score: "B" },
  { name: "Cerebras", region: "us-east", provider: "Llama-3.1 8B", status: "off", barPct: 0, latency: "—", rpm: "0", score: "D" },
];

const IP_PROXIES: ProxyItem[] = [
  { name: "Residential", region: "us-east", provider: "BrightData", status: "on", barPct: 97, latency: "8ms", rpm: "820", score: "S" },
  { name: "Datacenter", region: "us-west", provider: "Oxylabs", status: "on", barPct: 89, latency: "15ms", rpm: "640", score: "A" },
  { name: "ISP", region: "eu-central", provider: "SmartProxy", status: "on", barPct: 78, latency: "34ms", rpm: "310", score: "B" },
  { name: "Mobile", region: "us-east", provider: "NetNut", status: "warn", barPct: 55, latency: "72ms", rpm: "88", score: "C" },
  { name: "SOCKS5", region: "multi", provider: "ProxySeller", status: "on", barPct: 68, latency: "41ms", rpm: "175", score: "B" },
  { name: "Rotating", region: "asia", provider: "IPRoyal", status: "off", barPct: 0, latency: "—", rpm: "0", score: "D" },
];

const NETWORK_ITEMS: ProxyItem[] = [
  { name: "Mainnet", region: "global", provider: "AWS CloudFront", status: "on", barPct: 99, latency: "3ms", rpm: "2100", score: "S" },
  { name: "Backup", region: "eu-west", provider: "GCP Load Balancer", status: "on", barPct: 91, latency: "18ms", rpm: "780", score: "A" },
  { name: "Staging", region: "us-east", provider: "Azure Front Door", status: "warn", barPct: 67, latency: "52ms", rpm: "120", score: "C" },
  { name: "Devnet", region: "us-west", provider: "Fly.io Anycast", status: "on", barPct: 82, latency: "27ms", rpm: "340", score: "B" },
  { name: "CDN", region: "multi", provider: "Cloudflare", status: "on", barPct: 96, latency: "6ms", rpm: "1500", score: "S" },
  { name: "Tor Relay", region: "multi", provider: "Tor Project", status: "off", barPct: 0, latency: "—", rpm: "0", score: "D" },
];

const CHAIN_STEPS: ChainStep[] = [
  { label: "Client", state: "done" },
  { label: "Gateway", state: "done" },
  { label: "Router", state: "active" },
  { label: "Provider", state: "idle" },
  { label: "API", state: "idle" },
];

/* ─── Inline SVG Icons ─── */

const RingSvg = () => (
  <svg viewBox="0 0 56 56" fill="none" xmlns="http://www.w3.org/2000/svg">
    <circle cx="28" cy="28" r="24" stroke="var(--nt-border)" strokeWidth="4" />
    <circle cx="28" cy="28" r="24" stroke="var(--nt-success)" strokeWidth="4" strokeDasharray="150" strokeDashoffset="30" strokeLinecap="round" />
  </svg>
);

const GlobeIcon = () => (
  <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
    <circle cx="8" cy="8" r="6" />
    <path d="M2 8h12M8 2a9.5 9.5 0 0 1 0 12 9.5 9.5 0 0 1 0-12z" />
  </svg>
);

const BoltIcon = () => (
  <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
    <path d="M5.5 1L2 8h4l-.5 5L10 6H6l.5-5z" />
  </svg>
);

const ShieldIcon = () => (
  <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
    <path d="M7 1l5 2v3.5a6.5 6.5 0 0 1-5 6 6.5 6.5 0 0 1-5-6V3l5-2z" />
  </svg>
);

const ActivityIcon = () => (
  <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
    <path d="M1 8h3l2-6 2 10 2-6 3 4" />
  </svg>
);

const ArrowRightIcon = () => (
  <svg width="18" height="18" viewBox="0 0 18 18" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
    <path d="M5 14l6-5-6-5" />
  </svg>
);

/* Score badge letter */
const ScoreBadge: React.FC<{ score: "S" | "A" | "B" | "C" | "D" }> = ({ score }) => (
  <span className={`px-iscore ${score}`}>{score}</span>
);

/* ─── Main Component ─── */

const AgentManagerPage: React.FC = () => {
  const setStatusText = useStore((s) => s.setStatusText);
  const providerConfig = useStore((s) => s.providerConfig);

  const [tab, setTab] = useState<0 | 1 | 2>(0);
  const [subscriptionUrl, setSubscriptionUrl] = useState("");
  const [subscriptions, setSubscriptions] = useState<string[]>([
    "https://proxy.example.com/sub1",
    "https://proxy.example.com/sub2",
  ]);

  const items = tab === 0 ? LLM_PROVIDERS : tab === 1 ? IP_PROXIES : NETWORK_ITEMS;

  const handleAddSubscription = useCallback(() => {
    const url = subscriptionUrl.trim();
    if (!url) return;
    setSubscriptions((prev) => [...prev, url]);
    setSubscriptionUrl("");
    setStatusText(`Subscription added: ${url}`);
  }, [subscriptionUrl, setStatusText]);

  const handleRemoveSubscription = useCallback(
    (url: string) => {
      setSubscriptions((prev) => prev.filter((s) => s !== url));
      setStatusText(`Subscription removed: ${url}`);
    },
    [setStatusText],
  );

  const handleTestAll = useCallback(() => {
    setStatusText("Testing all providers...");
  }, [setStatusText]);

  const activeItems = items.filter((i) => i.status === "on").length;
  const totalItems = items.length;

  return (
    <div className="vw-agent">
      <div className="px">
        {/* ── Hero Card ── */}
        <div className="px-hero">
          <div className="px-hleft">
            <div className="px-ring">
              <RingSvg />
              <span className="px-ring-txt">A+</span>
            </div>
            <div>
              <div className="px-htitle">Gateway Status</div>
              <div className="px-hstatus">
                <span className="px-pulse on" />
                All Systems Operational
              </div>
              <div className="px-hmeta">
                {activeItems}/{totalItems} online &middot; {providerConfig?.model ?? "auto"}
              </div>
            </div>
          </div>
          <div className="px-hright">
            <select className="px-select" defaultValue="auto">
              <option value="auto">Auto Route</option>
              <option value="fastest">Fastest</option>
              <option value="cheapest">Cheapest</option>
              <option value="random">Random</option>
            </select>
            <button className="px-btn" onClick={handleTestAll}>
              Test All
            </button>
          </div>
        </div>

        {/* ── Grid Stats ── */}
        <div className="px-grid-2">
          <div className="px-card">
            <div className="px-cicon"><GlobeIcon /></div>
            <span className="px-clabel">Active Providers</span>
            <span className="px-cval">{activeItems}</span>
          </div>
          <div className="px-card">
            <div className="px-cicon"><BoltIcon /></div>
            <span className="px-clabel">Avg Latency</span>
            <span className="px-cval">28ms</span>
          </div>
          <div className="px-card">
            <div className="px-cicon"><ShieldIcon /></div>
            <span className="px-clabel">Total RPM</span>
            <span className="px-cval">4.2k</span>
          </div>
          <div className="px-card">
            <div className="px-cicon"><ActivityIcon /></div>
            <span className="px-clabel">Uptime</span>
            <span className="px-cval">99.7%</span>
          </div>
        </div>

        {/* ── Chain Visualization ── */}
        <div className="px-chain">
          {CHAIN_STEPS.map((step, idx) => (
            <React.Fragment key={step.label}>
              <div className="px-clink">
                <div className={`px-cldot ${step.state}`} />
                <span className={`px-clabel ${step.state}`}>{step.label}</span>
                {idx < CHAIN_STEPS.length - 1 && (
                  <div className={`px-cline ${step.state}`} />
                )}
              </div>
              {idx < CHAIN_STEPS.length - 1 && (
                <ArrowRightIcon />
              )}
            </React.Fragment>
          ))}
        </div>

        {/* ── Tab Bar ── */}
        <div className="px-tabbar">
          <button className={`px-tab-btn${tab === 0 ? " on" : ""}`} onClick={() => setTab(0)}>
            LLM Providers
          </button>
          <button className={`px-tab-btn${tab === 1 ? " on" : ""}`} onClick={() => setTab(1)}>
            IP Proxies
          </button>
          <button className={`px-tab-btn${tab === 2 ? " on" : ""}`} onClick={() => setTab(2)}>
            Network
          </button>
        </div>

        {/* ── Tab Content ── */}
        <div className={`px-tab-pane${tab === 0 ? " open" : ""}`}>
          {/* Provider / Proxy / Network List */}
          <div className="px-section">
            <div className="px-shead">
              <h3>{tab === 0 ? "LLM Providers" : tab === 1 ? "IP Proxies" : "Network Routes"}</h3>
              <span className="px-smeta">{items.length} endpoints</span>
            </div>
            <div className="px-clist">
              {items.map((item) => (
                <div className="px-item" key={item.name}>
                  <span className="px-iname">{item.name}</span>
                  <span className="px-ireg">{item.region.split("-")[0]}</span>
                  <span className="px-iprov">{item.provider}</span>
                  <div className={`px-idot ${item.status}`} />
                  <div className="px-ibar">
                    <div className="px-ibar-fill" style={{ width: `${item.barPct}%` }} />
                  </div>
                  <span className="px-ilat">{item.latency}</span>
                  <span className="px-irpm">{item.rpm}</span>
                  <ScoreBadge score={item.score} />
                </div>
              ))}
            </div>
          </div>

          {/* Subscription Manager (IP Proxies only) */}
          {tab === 1 && (
            <div className="px-section">
              <div className="px-shead">
                <h3>Subscription Manager</h3>
              </div>
              <div className="px-sub-row">
                <input
                  className="px-sub-input"
                  placeholder="https://proxy.example.com/sub"
                  value={subscriptionUrl}
                  onChange={(e) => setSubscriptionUrl(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && handleAddSubscription()}
                />
                <button className="px-btn" onClick={handleAddSubscription}>
                  Add
                </button>
              </div>
              {subscriptions.length > 0 && (
                <div className="px-sub-list">
                  {subscriptions.map((url) => (
                    <div className="px-sub-item" key={url}>
                      <GlobeIcon />
                      <span style={{ flex: 1, minWidth: 0, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                        {url}
                      </span>
                      <button className="del" onClick={() => handleRemoveSubscription(url)}>
                        ✕
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Settings (Network only) */}
          {tab === 2 && (
            <div className="px-section">
              <div className="px-shead">
                <h3>Network Settings</h3>
              </div>
              <div className="px-cfg-grid">
                <div className="px-cfg-row">
                  <label>DNS</label>
                  <input defaultValue="1.1.1.1, 8.8.8.8" />
                </div>
                <div className="px-cfg-row">
                  <label>Timeout</label>
                  <input defaultValue="5000" />
                </div>
                <div className="px-cfg-row">
                  <label>Retry</label>
                  <input defaultValue="3" />
                </div>
                <div className="px-cfg-row">
                  <label>IPv6</label>
                  <div className={`px-cfg-toggle on`} />
                </div>
                <div className="px-cfg-row">
                  <label>Keep-Alive</label>
                  <div className={`px-cfg-toggle on`} />
                </div>
              </div>
              <div className="px-submit-row">
                <button className="px-btn">Reset</button>
                <button className="px-btn" style={{ background: "var(--nt-primary)", color: "#fff", borderColor: "var(--nt-primary)" }}>
                  Save
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default AgentManagerPage;

import { chromium } from "@playwright/test";

const URL = process.env.URL || "http://127.0.0.1:1425/";
const SHOT = process.env.SHOT || "/tmp/nt-ui-v2.png";

const mock = `
(function(){
  const __handlers = {};
  window.__TAURI_INTERNALS__ = {
    invoke: async (cmd, args) => {
      const log = window.__MOCK_LOG__ || (window.__MOCK_LOG__ = []);
      log.push(cmd);
      switch (cmd) {
        case "plugin:event|listen":
          __handlers[args.event] = args.handler;
          return () => {};
        case "neocodex_send_message_stream":
          const reply = "模拟回复: 已收到你的消息——这是流式文本。";
          setTimeout(() => {
            for (let i = 0; i < reply.length; i += 4) {
              setTimeout(() => {
                if (__handlers["neocodex_stream_token"]) __handlers["neocodex_stream_token"](reply.slice(0, i + 4));
              }, i * 3);
            }
            setTimeout(() => { if (__handlers["neocodex_stream_end"]) __handlers["neocodex_stream_end"](reply); }, reply.length * 3 + 10);
            setTimeout(() => { if (__handlers["neocodex_stream_done"]) __handlers["neocodex_stream_done"]({ cancelled: false }); }, reply.length * 3 + 20);
          }, 50);
          return reply;
        case "neocodex_list_sessions":
          return [
            { id: "s1", name: "架构讨论: E8 推理链路优化", mode: "Agent", message_count: 12, updated_at: 1700000000 },
            { id: "s2", name: "修复结算 Bug", mode: "Plan", message_count: 5, updated_at: 1700000100 },
            { id: "s3", name: "API 客户端重构", mode: "Agent", message_count: 8, updated_at: 1700000200 },
          ];
        case "neocodex_health_report":
          return { context_usage: 0.42, turn_count: 3, git_branch: "main", diff_stats: {} };
        case "neocodex_app_version":
          return "0.18.0";
        case "neocodex_set_mode":
          return "ok";
        case "neocodex_provider_config":
          return {
            provider_count: 6,
            active_model: "llama-3.3-70b-versatile",
            providers: [
              { name: "Groq", model: "llama-3.3-70b-versatile", resolvable: true },
              { name: "Cerebras", model: "llama-3.1-8b-instant", resolvable: true },
              { name: "OpenRouter", model: "mistralai/mixtral-8x7b", resolvable: true },
              { name: "DeepSeek", model: "deepseek-chat", resolvable: true },
              { name: "Pollinations", model: "openai/gpt-4o-mini", resolvable: true },
              { name: "SambaNova", model: "Llama-3.1-70B-Instruct", resolvable: true },
            ],
          };
        case "neocodex_set_provider":
          return "provider set to " + (args && args.name);
        case "neocodex_get_diff":
          return { files: [{ path: "src/main.rs", hunks: [{ lines: [
            { t: "ctx", o: 1, n: 1, s: "fn main() {" },
            { t: "del", o: 2, n: null, s: '  println!("old");' },
            { t: "add", o: null, n: 2, s: '  println!("new");' },
          ] }] }] };
        case "neocodex_search_files":
          return ["src/main.rs", "src/ipc.ts", "Cargo.toml", "src/styles/ui-v2.css"];
        case "neocodex_get_project":
          return "/Users/neo/Downloads/neotrix";
        case "read_dir_recursive":
          return [
            { name: "Cargo.toml", path: "/proj/Cargo.toml", is_dir: false, depth: 0 },
            { name: "src", path: "/proj/src", is_dir: true, depth: 0 },
            { name: "main.rs", path: "/proj/src/main.rs", is_dir: false, depth: 1 },
            { name: "lib.rs", path: "/proj/src/lib.rs", is_dir: false, depth: 1 },
          ];
        case "proxy_pool_nodes":
          return [
            { url: "104.28.0.1:8080", tag: "res-1.us-west", latency_ms: 42, healthy: true, speed_tier: "S", geo_tag: "US" },
            { url: "85.10.0.22:8080", tag: "res-3.eu-berlin", latency_ms: 89, healthy: true, speed_tier: "A", geo_tag: "DE" },
            { url: "31.6.0.44:80", tag: "res-4.eu-london", latency_ms: null, healthy: false, speed_tier: "D", geo_tag: "GB" },
          ];
        case "cmd_agent_status":
          return { running: false, current_task: null, uptime_secs: 0 };
        case "cmd_agent_start":
          return "ok";
        case "cmd_agent_stop":
          return "ok";
        case "neocodex_create_session":
          return { id: "s4", name: "新会话", mode: "Agent", message_count: 0, updated_at: 1700000300 };
        case "neocodex_switch_session":
          return "switched to " + (args && args.session_id);
        case "neocodex_get_session_messages":
          return [
            { id: 0, role: "user", content: "你好，E8", timestamp: 1700000000000 },
            { id: 1, role: "assistant", content: "分析如下：" + String.fromCharCode(10, 10) + String.fromCharCode(96, 96, 96) + "rust" + String.fromCharCode(10) + "fn main() {}" + String.fromCharCode(10) + String.fromCharCode(96, 96, 96), timestamp: 1700000010000 },
          ];
        case "execute_terminal_command":
          return "mock shell: command executed" + String.fromCharCode(10);
        case "brain_stats":
          return { iteration: 209, absorb_count: 102, capability_sum: 6.2, memory_count: 1247, engine_active: true, capability_vector: [1, 2, 3], dimension_names: ["E8", "GWT", "VSA"] };
        case "kb_search":
          return [
            { id: "k1", node_type: "concept", title: "E8 推理链路", summary: "前沿模型融合到 E8 的注意路由与分配", domain: "NT-CORE", confidence: 0.9, importance: 0.8, created_at: 1700000000 },
            { id: "k2", node_type: "paper", title: "Global Workspace Theory", summary: "全局工作空间注意力路由", domain: "NT-CORE", confidence: 0.85, importance: 0.7, created_at: 1690000000 },
          ];
        default:
          return null;
      }
    }
  };
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
})();
`;

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  const errs = [];
  page.on("pageerror", (e) => errs.push("PAGEERROR: " + e.message));
  page.on("console", (m) => { if (m.type() === "error") errs.push("CONSOLE: " + m.text()); });
  await page.addInitScript(mock);
  await page.goto(URL, { waitUntil: "networkidle" });
  await page.waitForTimeout(500);

  // basic render assertions
  const views = await page.evaluate(() => {
    const q = (s) => !!document.querySelector(s);
    return {
      app: q(".app"),
      viewChat: q("#viewChat"),
      viewCowork: q("#viewCowork"),
      viewCode: q("#viewCode"),
      hero: q("#heroSection"),
      chatInput: q("#chatInput"),
      sendBtn: q("#sendBtn"),
      rightbar: q("#rightbar"),
      fileTree: q("#fileTree"),
      settingsModal: q("#overlaySettings"),
      theme: document.documentElement.getAttribute("data-theme"),
    };
  });
  console.log("RENDER:", JSON.stringify(views, null, 2));

  // test sendMsg via IPC mock
  const preSend = await page.evaluate(() => ({
    hasTAURI: !!window.__TAURI_INTERNALS__,
    invokeType: typeof window.__TAURI_INTERNALS__?.invoke,
    log: (window.__MOCK_LOG__ || []),
  }));
  console.log("PRE-SEND:", JSON.stringify(preSend));
  await page.fill("#chatInput", "你好，NeoTrix");
  await page.type("#chatInput", " x");
  await page.waitForTimeout(100);
  const btnEnabled = await page.evaluate(() => !document.getElementById("sendBtn").disabled);
  console.log("sendBtn enabled after input:", btnEnabled);
  await page.click("#sendBtn");
  await page.waitForTimeout(900);
  const msgState = await page.evaluate(() => {
    const msgs = [...document.querySelectorAll("#chatScroll .msg")];
    const ai = msgs.find((m) => m.classList.contains("l"));
    return {
      userMsg: msgs.some((m) => m.classList.contains("r")),
      aiMsg: !!ai,
      aiText: ai ? ai.querySelector(".mb").textContent : "",
      streaming: !!document.querySelector(".mb.streaming"),
      callLog: (window.__MOCK_LOG__ || []).slice(),
    };
  });
  console.log("SEND:", JSON.stringify(msgState, null, 2));

  // in-chat code rendering: fenced blocks become highlighted .msg-code
  const rich = await page.evaluate(() => {
    const host = document.createElement("div");
    host.innerHTML = window.renderRichText("结果如下：\n\n```rust\nfn main() {\n    println!(\"hi\");\n}\n```");
    const code = host.querySelector(".msg-code");
    return {
      hasBlock: !!code,
      lang: code?.querySelector(".msg-code-lang")?.textContent,
      kw: !!code?.querySelector(".msg-code-b .kw"),
      copyBtn: !!code?.querySelector(".msg-code-cp"),
      runBtn: !!code?.querySelector(".msg-code-cp:last-child") || !![...(code?.querySelectorAll(".msg-code-cp") || [])].find((b) => b.textContent === "运行"),
    };
  });
  console.log("RICH TEXT:", JSON.stringify(rich));

  // code run: clicking 运行 appends a result block
  const codeRun = await page.evaluate(async () => {
    const host = document.createElement("div");
    host.innerHTML = window.renderRichText("```sh\necho hi-from-run\n```");
    document.getElementById("viewChat").appendChild(host);
    const runBtn = [...host.querySelectorAll(".msg-code-cp")].find((b) => b.textContent === "运行");
    runBtn?.click();
    await new Promise((r) => setTimeout(r, 200));
    return {
      outBlock: !!host.querySelector(".msg-code-out"),
      resText: host.querySelector(".msg-code-res")?.textContent || "",
      ranShell: (window.__MOCK_LOG__ || []).includes("execute_terminal_command"),
    };
  });
  console.log("CODE RUN:", JSON.stringify(codeRun));

  // test view switching + settings modal
  await page.click('.segb[data-view="cowork"]');
  await page.waitForTimeout(300);
  const cowork = await page.evaluate(() => ({
    viewCoworkVisible: getComputedStyle(document.getElementById("viewCowork")).display,
    sessionCount: document.querySelectorAll("#cwSessionList .cw-sitem").length,
    firstSessionName: document.querySelector("#cwSessionList .cw-sitem")?.textContent?.trim().slice(0, 30),
    agentRow: document.getElementById("cwAgentList")?.textContent?.slice(0, 40),
  }));
  console.log("COWORK:", JSON.stringify(cowork));

  // clicking a session loads its history into the chat
  await page.click('#cwSessionList .cw-sitem');
  await page.waitForTimeout(400);
  const sessLoad = await page.evaluate(() => ({
    chatShown: getComputedStyle(document.getElementById("chatScroll")).display,
    historyMsgs: document.querySelectorAll("#chatScroll .msg").length,
    switched: (window.__MOCK_LOG__ || []).includes("neocodex_switch_session"),
    gotMsgs: (window.__MOCK_LOG__ || []).includes("neocodex_get_session_messages"),
  }));
  console.log("SESSION LOAD:", JSON.stringify(sessLoad));

  // hypercube overlay reflects real brain_stats
  await page.evaluate(() => { window.openOverlay("overlayHypercube"); });
  await page.waitForTimeout(250);
  const hyper = await page.evaluate(() => ({
    nodes: document.getElementById("hcNodes")?.textContent,
    cap: document.getElementById("hcCap")?.textContent,
    vsa: document.getElementById("hcVsa")?.textContent,
  }));
  console.log("HYPERCUBE:", JSON.stringify(hyper));
  await page.evaluate(() => { window.closeOverlay("overlayHypercube"); });
  await page.waitForTimeout(100);

  // KB search in settings → data
  await page.evaluate(() => {
    document.getElementById("overlaySettings").classList.add("open");
    document.querySelectorAll(".st-item").forEach((i) => i.classList.remove("on"));
    window.selectSetting && window.selectSetting(document.querySelector('.st-item[onclick*="data"]'), "data");
  });
  await page.waitForTimeout(150);
  const kb = await page.evaluate(async () => {
    const inp = document.getElementById("kbSearchInput");
    const n0 = document.getElementById("kbNodeCount")?.textContent;
    if (inp) inp.value = "E8";
    window.kbSearch && (await window.kbSearch());
    await new Promise((r) => setTimeout(r, 200));
    return {
      hits: document.querySelectorAll("#kbResults .kb-hit").length,
      firstTitle: document.querySelector("#kbResults .kb-hit-t")?.textContent?.slice(0, 24),
      nodeCount: n0,
    };
  });
  console.log("KB SEARCH:", JSON.stringify(kb));
  await page.evaluate(() => { document.getElementById("overlaySettings").classList.remove("open"); });
  await page.waitForTimeout(100);

  // agent start/stop round trip
  await page.click('.segb[data-view="cowork"]');
  await page.waitForTimeout(300);
  const agentRt = await page.evaluate(async () => {
    const startBtn = [...document.querySelectorAll(".cw-abtn")].find((b) => b.textContent.includes("启动"));
    startBtn?.click();
    await new Promise((r) => setTimeout(r, 250));
    const afterStart = document.getElementById("cwAgentList")?.textContent?.slice(0, 60);
    const stopBtn = [...document.querySelectorAll(".cw-abtn")].find((b) => b.textContent.includes("停止"));
    stopBtn?.click();
    await new Promise((r) => setTimeout(r, 200));
    return {
      started: (window.__MOCK_LOG__ || []).includes("cmd_agent_start"),
      stopped: (window.__MOCK_LOG__ || []).includes("cmd_agent_stop"),
      afterStart,
    };
  });
  console.log("AGENT RT:", JSON.stringify(agentRt));

  // create session round trip
  const createRt = await page.evaluate(async () => {
    window.createSession && (await window.createSession());
    await new Promise((r) => setTimeout(r, 250));
    return {
      created: (window.__MOCK_LOG__ || []).includes("neocodex_create_session"),
      count: document.querySelectorAll("#cwSessionList .cw-sitem").length,
    };
  });
  console.log("CREATE SESSION:", JSON.stringify(createRt));

  await page.click('.segb[data-view="chat"]');
  await page.waitForTimeout(300);
  const code = await page.evaluate(() => ({
    noCodePanel: !document.getElementById("codePanel"),
    noCodeBtn: !document.getElementById("codePanelToggle"),
    noCodeView: !document.getElementById("viewCode"),
  }));
  console.log("CODE:", JSON.stringify(code));

  // gateway settings section should show real proxy nodes
  await page.evaluate(() => {
    document.getElementById("overlaySettings").classList.add("open");
    document.querySelectorAll(".st-item").forEach((i) => i.classList.remove("on"));
    window.selectSetting && window.selectSetting(document.querySelector('.st-item[onclick*="gateway"]'), "gateway");
  });
  await page.waitForTimeout(300);
  const gateway = await page.evaluate(() => ({
    nodes: document.querySelectorAll("#stGwNodeList .px-item").length,
    meta: document.getElementById("stGwNodeMeta")?.textContent,
  }));
  console.log("GATEWAY:", JSON.stringify(gateway));
  await page.evaluate(() => { document.getElementById("overlaySettings").classList.remove("open"); });
  await page.waitForTimeout(100);

  // click user avatar popover -> settings modal opens
  await page.evaluate(() => { window.dispatchEvent(new Event("click")); });
  await page.click("#userBar");
  await page.waitForTimeout(150);
  await page.evaluate(() => { document.getElementById("overlaySettings").classList.add("open"); });
  await page.waitForTimeout(100);
  const settings = await page.evaluate(() => ({
    open: document.getElementById("overlaySettings").classList.contains("open"),
    sections: document.querySelectorAll(".st-section").length,
  }));
  console.log("SETTINGS:", JSON.stringify(settings));

  // ── Claude fusion: + menu, model selector ──
  const fusion = await page.evaluate(async () => {
    const plusMenu = !!document.getElementById("ntxPlusMenu");
    const modelBtn = !!document.getElementById("ntxModelBtn");
    const modelMenuItems = document.querySelectorAll("#ntxModelMenu .ntx-mm-item").length;
    const noUsageRing = !document.getElementById("ntxUsage");
    const modelLabel = document.getElementById("ntxModelLabel")?.textContent;
    document.getElementById("ntxPlusBtn")?.click();
    await new Promise((r) => setTimeout(r, 60));
    const plusOpen = document.getElementById("ntxPlusMenu")?.classList.contains("open");
    const groqItem = [...document.querySelectorAll("#ntxModelMenu .ntx-mm-item")].find((i) => i.dataset.id === "Groq");
    groqItem?.click();
    await new Promise((r) => setTimeout(r, 120));
    const modelAfter = document.getElementById("ntxModelLabel")?.textContent;
    return { plusMenu, plusOpen, modelBtn, modelMenuItems, noUsageRing, modelLabel, modelAfter };
  });
  console.log("FUSION:", JSON.stringify(fusion, null, 2));

  // ── Diff inline comments ──
  const diff = await page.evaluate(async () => {
    document.getElementById("ntxPlusBtn")?.click();
    await new Promise((r) => setTimeout(r, 40));
    const item = [...document.querySelectorAll(".ntx-pm-item")].find((i) => i.dataset.act === "diff");
    item?.click();
    await new Promise((r) => setTimeout(r, 80));
    window.diffAddComment(0, 0, 1);
    await new Promise((r) => setTimeout(r, 60));
    const editor = document.querySelector(".df-cmt-editor textarea");
    if (editor) editor.value = "这里改成新调用后没问题";
    document.querySelector(".dc-save")?.click();
    await new Promise((r) => setTimeout(r, 60));
    const hasCmt = document.querySelectorAll(".df-comment .dc-body").length;
    const cmtText = document.querySelector(".df-comment .dc-body")?.textContent;
    return { hasCmt, cmtText, open: document.getElementById("overlayDiff")?.classList.contains("open") };
  });
  console.log("DIFF:", JSON.stringify(diff));
  await page.evaluate(() => closeOverlay("overlayDiff"));

  const calls = await page.evaluate(() =>
    window.__MOCK_LOG__?.filter((c) => c.includes("neocodex_set_mode") || c.includes("health") || c.includes("search_files")),
  );
  console.log("FUSION CALLS:", JSON.stringify(calls));

  console.log("ERRORS:", JSON.stringify(errs, null, 2));
  await page.screenshot({ path: SHOT, fullPage: false });
  console.log("SHOT:", SHOT);
  await browser.close();
})();

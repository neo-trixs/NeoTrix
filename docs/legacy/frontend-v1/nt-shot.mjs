import { chromium } from "@playwright/test";
const URL = process.env.URL || "http://127.0.0.1:1425/";

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
              setTimeout(() => { if (__handlers["neocodex_stream_token"]) __handlers["neocodex_stream_token"](reply.slice(0, i + 4)); }, i * 3);
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
  await page.waitForTimeout(600);

  // 1. Chat view (fresh) — hide background loop? keep as-is
  await page.screenshot({ path: "/tmp/nt-shot-chat.png" });

  // 2. Open + menu and model menu
  await page.evaluate(() => {
    document.getElementById("ntxPlusBtn")?.click();
    document.getElementById("ntxModelBtn")?.click();
  });
  await page.waitForTimeout(200);
  await page.screenshot({ path: "/tmp/nt-shot-composer.png" });

  // close menus
  await page.evaluate(() => {
    ["ntxPlusMenu", "ntxModelMenu"].forEach((id) => document.getElementById(id)?.classList.remove("open"));
  });

  // 3. Cowork view
  await page.evaluate(() => {
    const btn = document.querySelector('.segb[data-view="cowork"]');
    const views = { chat: "viewChat", cowork: "viewCowork" };
    for (const [v, id] of Object.entries(views)) document.getElementById(id).style.display = v === "cowork" ? "flex" : "none";
    btn?.classList.add("on");
    document.querySelectorAll(".segb").forEach((b) => { if (b !== btn) b.classList.remove("on"); });
  });
  await page.waitForTimeout(200);
  await page.screenshot({ path: "/tmp/nt-shot-cowork.png" });

  // 4. Chat with message — backend result shown in conversation (code block)
  await page.evaluate(() => {
    const btn = document.querySelector('.segb[data-view="chat"]');
    document.getElementById("viewChat").style.display = "flex";
    document.getElementById("viewCowork").style.display = "none";
    btn?.classList.add("on");
    document.querySelectorAll(".segb").forEach((b) => { if (b !== btn) b.classList.remove("on"); });
  });
  await page.fill("#chatInput", "写一段 Rust 代码并运行");
  await page.type("#chatInput", " ");
  await page.waitForTimeout(120);
  await page.click("#sendBtn");
  await page.waitForTimeout(1200);
  await page.screenshot({ path: "/tmp/nt-shot-code.png" });

  console.log("ERRORS:", JSON.stringify(errs));
  await browser.close();
})();

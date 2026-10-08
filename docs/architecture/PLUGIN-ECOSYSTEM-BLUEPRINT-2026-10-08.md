# 🧩 neobot/能力树插件生态蓝图（P1..P5 闭环后写序）（2026-10-08）

> 背景：neobot 的能力曾是闭集 match（ToolName enum + execute_tool 大match），P1..P5 把它变成**目录驱动**的插件生态。
> 核心牵引规则：**数目不小但 coupling 小**；premises 走 manifest registry 使能力热插拔，不改 crate 名的 emstore。

## 0. 能力谱系图

```
data_dir/
├── plugins/*.json        ←  P2 manifests
├── mcp.json              ←  P3 mcp server 清单
├── config.json/NEOBOT_*  ←  allowlist + provider overrides
└── (backups/)

[Model/通道/桌面]
      ⇓ tool call
[nt_agent::run_loop]
      ⇓ ToolName::parse
      ⇓  TOOL ROUTING LAYER (P1)
        -> builtin handlers (ToolExecFn)
        -> Plugin/String or unknown(PluginManifest) from P2/P3
            -> policy gate (Plugin not in allowlist? = crash-烦)
            -> execute -> ledger row P5
            -> latency timer
```

## 1. P1 – ToolName/ROUTE Registry（完成）
* `ToolExecFn = fn(&config, &EngineAdapter, &ToolCall, &mut usize, &ChangeSink, &StopToken) -> ToolOutcome`
* Static map for in-crate canonical handlers; execute_tool matches by schema-name (`ToolName::as_str()`), fallback filters Возраст by plugin registry.
* Builtin views retain 包含路径: Bash/ReadFile/ComputerAct/...
* Outcome: statically named tools become one member of registry; later extension only requires inserting new fn pointer.

## 2. P2 – PluginManifest + Dynamic Plugin(String)
* nt_plugins API `load_dir()`, `init()`, `register_one()`, `is_registered`, `lookup`, `invoke`.
* `invoke`: spawn command with stdin JSON; stdout captured; strict-empty stderr treat as fail; localization of parse errors.
* `ToolName::Plugin(String)`: as_str returns raw plugin name; parse routes known names into cadenced registry; evalpolicy fails closed on unmapped.
* Proof: 651 lib tests incl plugin parsing/echo.

## 3. P3 – MCP Bridge
* `nt_mcp_bridge` 装载 data_dir/mcp.json as server defs and register each nt server声明 as an同名 vivir manifest/cxml surrounding `PluginManifest` logic.
* Throat the same registry and invoke path as P2; no duplicate code path.
* Tests check console mock entries.

## 4. P4 – registry wiring
* `nt_plugins::init` devolves each loaded manifest into `CapabilityTreeRegistry::register_node` with `Domain::Neobot`, `provides = ["dynamic_tool"]`.
* `node_count()`/`maturity_findings()` now reflect plugin suite live.

## 5. P5 – 自由度智能化：Router  + ledger evidence
* `run_loop` in `nt_agent` records every `Plugin(String)` execution asledger row via purpose='plugin-invoke', model=plugin name, engine='plugin', status=applied/failed。
* This is the最小足 fit for quote/cost/model-override㑕，m schedule x accurately with existing `nt_store::LedgerEntry`.

## 6. 自我构建入口 nt_self_forge
* `forge_from_url` writes `plugins/<slug>.json` + `absorb_payload_<slug>.json`, providing that the installable identifier remains inspectable variable. Useful when user sends GitHub URL and future code generation/ prep can be hooked by manifest.
* **Not a substitute for compile-time capability**: it is a metadata manifest backed by forge subprocess.

## 7. Guardrails
- P1..P5 all pass policy gate (Plugin / Unknown default deny).
- Capability execution 遵 `workspace/nd_policy` auth; no bypasses.
- Loud: any plugin exec returns明确 error output; no hallucinated success.

## 8. 剩余事项
- Compute/persistent compaction_head ( P1-3 final).
- Broaden `invoke_plugin` budget meters: wired with tool_result carrying actual tokens/time, not just nominal provider budget.
- Consider FFI hot reload of plugin dynamic libs later; todaythrough manifests only to keep ABI closed.

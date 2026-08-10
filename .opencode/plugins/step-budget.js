// step-budget.js — NeoTrix 动态步骤能力网核心插件 (ConsciousnessStepNetwork)
//
// 目标：替代固定 steps 硬上限，实现"任务真正完成才收敛 + 每步最优 + 自我进化"的
// 动态步骤管理。opencode 原生：不设 steps = 无限迭代直到模型自然停止（动态收敛）。
//
// 七层机制（L1-L7），全部运行时自适应：
//   L1 收敛检测 (环增益): 跟踪"有进展的工具调用"与总步数之比 E(n)，
//      环增益 Aβ = E(n)/E(n-1)。Aβ 持续 <1 判定 CONVERGING（正常收敛），
//      Aβ≈0 判定 STALLING（停滞），>1 振荡/发散。吸收 LoopGain Barkhausen 准则。
//   L2 循环检测: 相同 (tool+args) 连续重复 / A→B→C→A 循环模式 / 只读工具空转。
//      吸收 agent-loop-detector (identical/cyclic/idle 三类) + sentinel LoopDetectionGuard。
//   L3 预算意识注入: 每个 N 步向推理上下文注入剩余"工具预算"状态块，
//      让 agent 感知消耗（BATS Budget Tracker）。富余→建议深挖；将尽→建议收敛。
//   L4 收敛引导: 检测到停滞/振荡时，通过 assistant 消息注入收敛指令（不硬截断），
//      吸收 opencode PR #4062 "system+assistant 双提示最后一步强制文本回复" 的经验。
//   L5 元认知实时修正 (META-STEP): 每一步评估"当前动作是否最优路径"——
//      检测策略漂移（只读探测过多未产出、信息搜索重复无新内容、子代理空转），
//      一旦判定非最优，注入修正指令让流程实时改道（Goldilocks 规划频率：
//      不过度规划也不盲目执行，只在收益>成本时干预）。
//   L6 自我反思 (REFLECT): 会话收敛/空闲时，生成反思报告——识别浪费步骤、
//      高效模式、重复错误，写入 ~/.neotrix/meta-cognition.jsonl（经验吸收前置）。
//   L7 迭代进化 (EVOLVE): 反思结果回流为后续注入的上下文（历史模式记忆），
//      让每次会话都比上次更高效——能力网链路随使用迭代进化。
//
// 设计边界：
// - 本插件是"护栏 + 引导 + 反思"，不是"上限"。它不阻止任务完成，只在失控时引导。
// - 记录到 ~/.neotrix/step-budget.log（追加）+ ~/.neotrix/meta-cognition.jsonl（反思）。

import { appendFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";

const HOME = process.env.HOME || ".";
const LOG_DIR = join(HOME, ".neotrix");
const LOG_PATH = join(LOG_DIR, "step-budget.log");
const REFLECT_PATH = join(LOG_DIR, "meta-cognition.jsonl"); // L6/L7 反思与进化沉淀

function log(msg) {
  try {
    mkdirSync(LOG_DIR, { recursive: true });
    appendFileSync(LOG_PATH, `[${new Date().toISOString()}] ${msg}\n`);
  } catch (_) {}
}

// L7 进化记忆：读取历史反思中的高效/低效模式，供注入参考
function loadEvolutionMemory() {
  try {
    const fs = require("node:fs");
    if (!fs.existsSync(REFLECT_PATH)) return [];
    const lines = fs.readFileSync(REFLECT_PATH, "utf8").trim().split("\n").slice(-50);
    return lines.filter(Boolean).map((l) => {
      try { return JSON.parse(l); } catch { return null; }
    }).filter(Boolean);
  } catch {
    return [];
  }
}

// L6 反思沉淀：把一次收敛的会话经验写入进化记忆
function writeReflection(entry) {
  try {
    mkdirSync(LOG_DIR, { recursive: true });
    appendFileSync(REFLECT_PATH, JSON.stringify({ ts: new Date().toISOString(), ...entry }) + "\n");
  } catch (_) {}
}

// ── 会话级状态 ──────────────────────────────────────────────
// 每个 sessionID 一个状态桶：记录工具调用历史、进展事件、循环检测窗口。
const sessions = new Map();

function getState(sessionID) {
  if (!sessions.has(sessionID)) {
    sessions.set(sessionID, {
      calls: [],          // {tool, argsKey, time, progressing} 最近调用窗口
      windowSize: 10,     // 循环检测滑动窗口
      totalCalls: 0,      // 总工具调用数（预算消耗）
      progressEvents: 0,  // 有进展的工具调用数（文件写/测试/搜索命中）
      lastProgressAt: 0,  // 最近一次进展的时间戳
      budgetNoteEvery: 6, // 每 N 步注入一次预算状态
      injectedAt: 0,      // 最近一次预算注入的步数
      warnedAt: 0,        // 最近一次循环警告的步数
      // L5 元认知状态
      metaCorrectedAt: 0, // 最近一次元认知修正注入的步数
      searchTopics: [],   // 最近搜索主题（检测重复搜索）
      probesSinceProgress: 0, // 连续只读探测步数（无产出）
      toolMix: {},        // 工具使用分布（策略漂移检测）
      reflected: false,   // 本会话是否已反思
      goalHint: null,     // 首个用户消息的简短目标提示（用于漂移判断）
    });
  }
  return sessions.get(sessionID);
}

// ── 工具分类：什么算"进展" ───────────────────────────────────
// 文件写入/编辑、测试运行、搜索/抓取命中 = 进展；只读状态查询 = 非进展（空转风险）。
function isProgressTool(tool) {
  return [
    "edit", "write", "patch", "apply_patch",       // 文件变更
    "bash",                                         // 可能产生实际效果
    "websearch", "webfetch",                        // 信息获取
    "task",                                         // 子代理推进
  ].includes(tool);
}

function isIdleTool(tool) {
  // 只读/状态查询类——连续 N 轮只有这类 = 空转
  return ["read", "glob", "grep", "list", "lsp"].includes(tool);
}

function argsKey(tool, args) {
  try {
    return tool + ":" + JSON.stringify(args ?? {}).slice(0, 200);
  } catch {
    return tool + ":?";
  }
}

// ── 循环检测：三类模式 ──────────────────────────────────────
function detectLoops(state, tool, key) {
  // 1. 相同 (tool+args) 连续重复 ≥3 次
  let identical = 1;
  for (let i = state.calls.length - 1; i >= 0; i--) {
    if (state.calls[i].key === key && state.calls[i].time > 0) identical++;
    else break;
  }
  if (identical >= 3) {
    return `IDENTICAL_LOOP: 相同工具调用 "${tool}" 连续重复 ${identical} 次（参数相同）。可能陷入无进展死循环，请换策略或收敛。`;
  }

  // 2. A→B→C→A 循环模式（窗口内工具序列重复 ≥2 次）
  //    滑动窗口找三连序列重复：从第 0 位到倒数第 4 位（给后面留出完整三连的空间）
  if (state.calls.length >= 7) {
    const seq = state.calls.map((c) => c.tool);
    const last = seq.slice(-3).join(">");
    const limit = seq.length - 3; // 排除最后三个自身（它们构成 last，不应被当历史）
    for (let i = 0; i < limit; i++) {
      // 跳过与 last 重叠的起始位（最后三连之前的位置才算历史）
      if (i > seq.length - 6 && i < seq.length - 3) continue; // 重叠区
      if (seq.slice(i, i + 3).join(">") === last) {
        return `CYCLIC_LOOP: 检测到工具序列 "${last}" 重复出现（A→B→C→A 循环）。进度停滞，建议切换路径。`;
      }
    }
  }

  // 3. 空闲空转：最近 5 次调用全为只读/状态查询且无进展
  if (state.calls.length >= 5) {
    const recent = state.calls.slice(-5);
    if (recent.every((c) => isIdleTool(c.tool)) && !recent.some((c) => c.progressing)) {
      return `IDLE_LOOP: 连续 5 轮只做只读/状态查询（${recent.map((c) => c.tool).join(",")}），无任何产出。请停止空转，采取实际行动。`;
    }
  }

  return null;
}

// ── L1 收敛检测：环增益 Aβ ──────────────────────────────────
// 把"调用序列"切成两半，E(前半) vs E(后半) 的进展事件数之比。
// Aβ 持续显著 <1（进展在衰减）且步数已较多 → STALLING，引导收敛。
function detectStagnation(state) {
  const n = state.calls.length;
  if (n < 10) return null; // 样本不足不判断

  const half = Math.floor(n / 2);
  const firstHalf = state.calls.slice(0, half);
  const secondHalf = state.calls.slice(half);
  const e1 = firstHalf.filter((c) => c.progressing).length;
  const e2 = secondHalf.filter((c) => c.progressing).length;

  // 前半有进展，后半近乎停滞（环增益≈0）且已经走了不少步 → 停滞
  if (e1 >= 3 && e2 <= Math.max(1, Math.floor(e1 * 0.2))) {
    return {
      converging: false,
      msg: `STALLING: 环增益 Aβ≈${(e2 / Math.max(1, e1)).toFixed(2)}（后半段 ${e2}/${secondHalf.length} 步有进展，前段 ${e1}/${firstHalf.length}）。工具消耗 ${n} 步但近期无实质推进，请收敛到已完成部分或明确换策略。`,
    };
  }

  // 持续无进展（整体进展率过低）
  const progressRate = state.progressEvents / Math.max(1, n);
  if (n >= 15 && progressRate < 0.15) {
    return {
      converging: false,
      msg: `LOW_YIELD: 已用 ${n} 步但进展率仅 ${(progressRate * 100).toFixed(0)}%。大概率在低效徘徊，请压缩步骤、聚焦产出。`,
    };
  }

  return { converging: true, msg: null };
}

// ── L5 元认知实时修正：判断"这一步是否最优路径" ──────────────
// 三类非最优信号（Goldilocks：只在干预收益 > 成本时介入，避免过度打断）：
//   A. 过度探测：连续 ≥5 步只读（read/glob/grep/list/lsp）零产出——策略漂移为"研究瘫痪"。
//   B. 重复搜索：同一主题 websearch ≥2 次无新进展——信息获取失效。
//   C. 无产出高耗：工具混合显示 edit 占 0% 且步数 ≥8——只读空转。
function detectMetaInefficiency(state) {
  const n = state.calls.length;
  if (n < 5) return null;

  const recent = state.calls.slice(-6);

  // A. 过度探测（研究瘫痪）：连续只读无产出
  if (recent.length >= 5 && recent.every((c) => isIdleTool(c.tool))) {
    const probes = recent.map((c) => c.tool).join(",");
    return `META_OVERPROBE: 已连续 ${recent.length} 步只做只读探测（${probes}）零产出。这是"研究瘫痪"信号——信息已足够时应立即转入实现/产出，而不是继续探测。请评估：手头信息是否足够开始产出？`;
  }

  // B. 重复搜索：同一搜索主题多次
  const searches = state.calls.filter((c) => c.tool === "websearch" || c.tool === "webfetch");
  if (searches.length >= 3) {
    const topics = searches.map((c) => c.key);
    const unique = new Set(topics);
    if (unique.size <= 2) {
      return `META_RESEARCH_LOOP: 重复搜索同一主题 ${searches.length} 次（${[...unique].join(" | ")}）。继续搜索边际收益递减，请基于已获取信息收敛判断或转向实施。`;
    }
  }

  // C. 只读空转：步数够多但从未写过文件
  const edits = state.calls.filter((c) => ["edit", "write", "patch"].includes(c.tool));
  if (n >= 8 && edits.length === 0) {
    return `META_NO_OUTPUT: 已 ${n} 步但零文件写入。当前任务如需产出（写代码/文档/配置），应尽快进入产出阶段；若为纯调研请明确收敛时间点。`;
  }

  return null;
}

export const StepBudgetPlugin = async ({ $, client }) => {
  return {
    // 每次工具执行后：记录调用 + 更新预算状态 + 预算意识注入（L3）
    "tool.execute.after": async (input) => {
      const sessionID = input.sessionID || "unknown";
      const tool = input.tool;
      const args = input.args ?? {};
      const state = getState(sessionID);

      const isProgress = isProgressTool(tool);
      state.calls.push({
        tool,
        key: argsKey(tool, args),
        time: Date.now(),
        progressing: isProgress,
      });
      if (state.calls.length > state.windowSize * 3) {
        state.calls = state.calls.slice(-state.windowSize * 3);
      }
      state.totalCalls++;
      if (isProgress) {
        state.progressEvents++;
        state.lastProgressAt = Date.now();
      }

      // 更新 L5 状态字段
      state.toolMix[tool] = (state.toolMix[tool] || 0) + 1;
      if (isIdleTool(tool)) {
        state.probesSinceProgress++;
      } else if (isProgressTool(tool)) {
        state.probesSinceProgress = 0;
      }

      // 检测循环（L2）——不阻止执行，只记录；由事件钩子做注入
      const loopMsg = detectLoops(state, tool, state.calls[state.calls.length - 1].key);
      if (loopMsg) {
        log(`LOOP session=${sessionID} step=${state.totalCalls}: ${loopMsg}`);
      }

      // 检测停滞（L1）——记录到日志
      const stag = detectStagnation(state);
      if (stag && !stag.converging) {
        log(`STAGNATION session=${sessionID} step=${state.totalCalls}: ${stag.msg}`);
      }

      // L5 元认知实时修正——检测到非最优路径立即注入修正（不等待 idle）
      // L7 进化记忆：注入时附上历史会话沉淀的高效/低效模式，让修正指令有经验支撑
      const meta = detectMetaInefficiency(state);
      if (meta && state.totalCalls - (state.metaCorrectedAt || 0) >= 5) {
        state.metaCorrectedAt = state.totalCalls;
        log(`META_CORRECT session=${sessionID} step=${state.totalCalls}: ${meta}`);
        // 读取历史教训（最近 5 条含 lessons 的反思），提炼成一行经验注入
        const lessons = loadEvolutionMemory()
          .flatMap((r) => r.lessons || [])
          .slice(-5)
          .map((l) => `· ${l}`)
          .join("\n");
        const injectText = lessons
          ? `[step-budget 元认知修正] ${meta}\n历史经验（来自既往会话反思）:\n${lessons}\n请结合这些经验调整策略。`
          : `[step-budget 元认知修正] ${meta}`;
        try {
          await client.session.prompt({
            path: { id: sessionID },
            body: {
              parts: [{ type: "text", text: injectText }],
            },
          });
        } catch (e) {
          log(`META_INJECT_FAIL session=${sessionID}: ${e.message}`);
        }
      }

      // 预算意识注入（L3）——每 budgetNoteEvery 步注入一次剩余预算状态
      // 注入为 assistant 消息（吸收 PR #4062：assistant 消息比 system 提示更有效）
      if (state.totalCalls >= state.budgetNoteEvery &&
          state.totalCalls - state.injectedAt >= state.budgetNoteEvery) {
        const remaining = Math.max(0, 30 - state.totalCalls); // 软预算 30 步（引导性，非硬限）
        const note = remaining > 15
          ? `[step-budget] 已用 ${state.totalCalls} 步，软预算 30 步内充裕（剩 ~${remaining}）。可继续深挖，但注意每次工具调用都要产出增量。`
          : remaining > 5
            ? `[step-budget] 已用 ${state.totalCalls} 步，软预算将尽（剩 ~${remaining}）。请优先收敛到已有成果，仅在确有把握时追加工具调用。`
            : `[step-budget] 已用 ${state.totalCalls} 步，超出软预算。请立即收敛：总结已完成工作，列出未完成项，停止无把握的工具调用。`;
        state.injectedAt = state.totalCalls;
        log(`BUDGET session=${sessionID} step=${state.totalCalls}: 软预算注入`);
        try {
          await client.session.prompt({
            path: { id: sessionID },
            body: {
              parts: [{ type: "text", text: note }],
            },
          });
        } catch (e) {
          log(`INJECT_FAIL session=${sessionID}: ${e.message}`);
        }
      }
    },

    // 会话空闲时：检测到停滞/循环则注入收敛引导（L4）
    event: async ({ event, client: eventClient }) => {
      if (!event) return;
      // 事件负载结构：{ type, properties: { sessionID } }（SDK EventSessionIdle 等）
      const sessionID = event.properties?.sessionID || event.sessionID;
      if (!sessionID) return;
      if (event.type !== "session.idle") return;
      const state = getState(sessionID);
      const c = eventClient || client;

      // 收敛引导（L4）——检测到停滞/循环且尚未警告过（防刷屏）
      const stag = detectStagnation(state);
      const loopMsg = state.calls.length >= 5
        ? detectLoops(state, state.calls[state.calls.length - 1].tool, state.calls[state.calls.length - 1].key)
        : null;

      const guidance = loopMsg || (stag && !stag.converging ? stag.msg : null);
      if (guidance && state.totalCalls - (state.warnedAt || 0) >= 5) {
        state.warnedAt = state.totalCalls;
        log(`GUIDE session=${sessionID} step=${state.totalCalls}: ${guidance}`);
        try {
          await c.session.prompt({
            path: { id: sessionID },
            body: {
              parts: [{ type: "text", text: `[step-budget 收敛引导] ${guidance}` }],
            },
          });
        } catch (e) {
          log(`GUIDE_FAIL session=${sessionID}: ${e.message}`);
        }
      }

      // L6 自我反思——会话收敛且有过实质工作时，生成反思报告并沉淀进化记忆（L7）
      if (!state.reflected && state.totalCalls >= 6) {
        state.reflected = true;

        // 量化本轮流程：工具分布、进展率、浪费信号
        const toolDist = Object.entries(state.toolMix)
          .sort((a, b) => b[1] - a[1])
          .map(([t, n]) => `${t}x${n}`)
          .join(", ");
        const progressRate = (state.progressEvents / state.totalCalls).toFixed(2);
        const idleOnly = state.calls.filter((c) => isIdleTool(c.tool)).length;
        const editCalls = state.calls.filter((c) => ["edit", "write", "patch"].includes(c.tool)).length;

        const reflection = {
          kind: "step-network-reflection",
          session: sessionID,
          totalSteps: state.totalCalls,
          progressEvents: state.progressEvents,
          progressRate: Number(progressRate),
          toolMix: toolDist,
          idleProbeSteps: idleOnly,
          writeSteps: editCalls,
          signals: [],
          lessons: [],
        };

        // 信号与经验提炼（供 L7 进化）
        if (progressRate < 0.3) {
          reflection.signals.push("low-yield");
          reflection.lessons.push("进展率低——下轮先规划产出路径，减少无产出探测");
        }
        if (idleOnly > state.totalCalls * 0.5) {
          reflection.signals.push("over-probing");
          reflection.lessons.push("只读探测占比过高——下轮限制探测轮次，信息足够即转产出");
        }
        if (editCalls === 0 && state.totalCalls >= 8) {
          reflection.signals.push("no-write");
          reflection.lessons.push("零写入长会话——若任务需产出，应尽早进入写阶段");
        }
        if (state.progressEvents >= 3 && state.totalCalls <= 20) {
          reflection.signals.push("efficient");
          reflection.lessons.push("高效路径：少量高产出调用——保留此策略模式");
        }

        writeReflection(reflection);
        log(`REFLECT session=${sessionID} steps=${state.totalCalls} rate=${progressRate} signals=[${reflection.signals.join(",") || "clean"}]`);
      }
    },
  };
};

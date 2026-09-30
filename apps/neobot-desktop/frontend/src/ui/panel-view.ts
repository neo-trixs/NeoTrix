/**
 * 决策面板视图 —— 交互部分独立于 blocks.ts。
 *
 * # 为什么单独一个文件
 *
 * `blocks.ts` 的渲染器是**纯展示**的（给定 Block 产出 DOM）。
 * 决策面板不一样：它要收集作答、校验版本、回报结果。把「能点」这件事
 * 塞进纯渲染器，会让它同时承担展示与状态，于是
 * 「一块炸了」的兜底路径也会跟着变得复杂。
 *
 * # 核心不变量（来自 block-model.ts，本文件只负责执行）
 *
 *  ① 作答必须带 `candidateSetVersion`；与当前面板对不上 ⇒ **拒绝并提示重选**。
 *     不拦的后果：骨架把上一批的旧选择当新选择用，而界面看不出任何异常。
 *  ② 选项的出处链接只允许 http(s)（模型层已硬校验，这里再挡一次，
 *     因为「模型给了 javascript: URL」是可能发生的，不该只靠一层）。
 *  ③ 作答成功后**面板不自动消失** —— 它要留在会话里作为「当时问了什么、
 *     选了什么」的记录。审计要能回看。
 */

import type { DecisionPanel } from "./block-model.ts";
import { validateAnswer, validatePanel } from "./block-model.ts";
import { esc } from "./core.ts";

export interface AnswerView {
  /** 面板 id —— ⛔ 必填：骨架要靠它定位校验基准（注册表按 id 查）。 */
  panelId: string;
  optionId: string;
  candidateSetVersion: number;
  label: string;
}

export interface PanelCallbacks {
  /** 骨架回传/本地记录用。 */
  onAnswer: (a: AnswerView) => void;
}

/**
 * 渲染决策面板（可交互）。
 *
 * ⛔ `mode === "sample"` 时顶部挂「示例数据」条 —— 拿样本冒充真实是可捕获的。
 * ⛔ 非法面板不渲染成卡，而是渲染成错误：界面得暴露问题，
 *    而不是给一张**看着能用**的坏卡。
 */
export function renderPanelView(p: DecisionPanel, cb: PanelCallbacks): HTMLElement {
  const bad = validatePanel(p);
  if (bad) {
    const box = document.createElement("div");
    box.className = "nb-error";
    box.setAttribute("role", "alert");
    const t = document.createElement("div");
    t.className = "nb-item-title";
    t.textContent = "决策面板无效，未能显示";
    const d = document.createElement("div");
    d.className = "nb-item-sub";
    d.textContent = bad;
    box.append(t, d);
    return box;
  }

  const box = document.createElement("div");
  box.className = "nb-item panel-card";
  box.dataset["panelId"] = p.id;
  box.dataset["version"] = String(p.candidateSetVersion);

  if (p.mode === "sample") {
    const w = document.createElement("div");
    w.className = "panel-warn";
    w.textContent = "示例数据（非真实候选）";
    box.appendChild(w);
  }

  // ── 头部：标题 + 版本 ──
  const head = document.createElement("div");
  head.className = "nb-item-row";
  const left = document.createElement("div");
  left.className = "nb-item-left";
  const title = document.createElement("span");
  title.className = "nb-item-title";
  title.textContent = p.title;
  left.appendChild(title);
  const ver = document.createElement("span");
  ver.className = "nb-item-sub";
  ver.textContent = `v${p.candidateSetVersion}`;
  head.append(left, ver);
  box.appendChild(head);

  // ── 选项 ──
  const group = document.createElement("div");
  group.className = "nb-opts";
  group.setAttribute("role", "radiogroup");
  group.setAttribute("aria-label", p.title);

  for (const o of p.options) {
    const label = document.createElement("label");
    label.className = "nb-opt";
    const input = document.createElement("input");
    input.type = "radio";
    input.name = `panel-${p.id}`;
    input.value = o.id;
    input.checked = o.id === p.selectedId;

    const body = document.createElement("div");
    body.className = "opt-body";
    const lab = document.createElement("div");
    lab.className = "nb-opt-label";
    lab.textContent = o.label;
    body.appendChild(lab);

    for (const d of o.details) {
      const dd = document.createElement("div");
      dd.className = "nb-item-sub";
      dd.textContent = d;
      body.appendChild(dd);
    }

    // 出处：只渲染 http(s)。别处已硬校验，这里再挡一层 ——
    // 「模型给了一个 javascript: URL」是可能发生的，不该只靠单层防御。
    const legal = o.sources.filter((s) => /^https?:\/\//i.test(s.url));
    if (legal.length !== o.sources.length) {
      const warn = document.createElement("div");
      warn.className = "panel-warn";
      warn.textContent = `${o.sources.length - legal.length} 条出处已隐藏（非 http(s) 链接）`;
      body.appendChild(warn);
    }
    if (legal.length > 0) {
      const srcs = document.createElement("div");
      srcs.className = "opt-srcs";
      for (const s of legal) {
        const a = document.createElement("a");
        a.className = "opt-src";
        a.textContent = s.title;
        a.href = s.url;
        a.rel = "noopener noreferrer";
        a.target = "_blank";
        srcs.appendChild(a);
      }
      body.appendChild(srcs);
    }
    label.append(input, body);
    group.appendChild(label);
  }
  box.appendChild(group);

  // ── 反馈区（校验失败时显示在这里，而不是 alert 弹窗） ──
  const note = document.createElement("p");
  note.className = "panel-note";
  note.setAttribute("role", "status");
  note.hidden = true;
  box.appendChild(note);

  // ── 作答 ──
  const bar = document.createElement("div");
  bar.className = "panel-foot";
  const picked = document.createElement("span");
  picked.className = "panel-picked";
  const submit = document.createElement("button");
  submit.type = "button";
  submit.className = "nb-btn nb-btn--primary nb-btn--sm";
  submit.textContent = "确认选择";
  submit.disabled = true;
  bar.append(picked, submit);
  box.appendChild(bar);

  const refresh = () => {
    const sel = group.querySelector<HTMLInputElement>("input:checked");
    picked.textContent = sel ? `已选：${p.options.find((o) => o.id === sel.value)?.label ?? sel.value}` : "";
    submit.disabled = !sel;
  };
  group.addEventListener("change", refresh);
  refresh();

  submit.addEventListener("click", () => {
    const sel = group.querySelector<HTMLInputElement>("input:checked");
    if (!sel) return;
    const err = validateAnswer(p, {
      optionId: sel.value,
      // ⚠️ 关键：**当前面板的版本**，不是作答时记下来的版本。
      //    两者相同才说明用户是在看这一批候选时做的选择。
      candidateSetVersion: Number(box.dataset["version"]),
    });
    if (err) {
      note.textContent = err;
      note.hidden = false;
      // ⛔ 不清空选择、也不自动提交。让用户自己看到问题并重选。
      return;
    }
    note.hidden = true;
    cb.onAnswer({
      panelId: p.id,
      optionId: sel.value,
      candidateSetVersion: p.candidateSetVersion,
      label: p.options.find((o) => o.id === sel.value)?.label ?? sel.value,
    });
    submit.disabled = true;
    submit.textContent = "已确认";
  });

  return box;
}

/** 把 DecisionPanel 序列化成可放进 `text` 块的纯文本（骨架侧用）。 */
export function answerToText(a: AnswerView): string {
  return `[选择] ${a.label}（候选集 v${a.candidateSetVersion}）`;
}

/** 供调试/自测：面板的最小可用 HTML 片段（不走 DOM）。 */
export function panelSummaryText(p: DecisionPanel): string {
  const opts = p.options.map((o) => o.label).join(" / ");
  return `${p.title}（v${p.candidateSetVersion}，${esc(opts)}）`;
}

/**
 * 弹层表单（Sheet）—— DSH `ui-settings-shell` / `ui-chat PreferenceRow` 的无框架复刻。
 *
 * # 为什么要有这个
 *
 * 后端有 3 个命令界面点不到（`neobot_agent_run` / `neobot_convo_group` /
 * `neobot_evidence_summary`）。既有弹层只有 `askConfirm`，它只接受**一个**文本框，
 * 装不下「标题 + 多选成员」和「目标 + 上下文」。与其给每处各写一次弹层，
 * 不如把 DSH 的 `SettingsForm` + `PreferenceRow` 两件套抽成一个可复用原语。
 *
 * # 从 DSH 抄了什么（逐条，见 docs/CAPABILITY-MAP-2026-09-30.md 与 ABSORPTION.md）
 *
 * 1. **两列行**：`title`(14/22) + `desc`(12/18) 在左，控件在右，底部 0.5px 发丝线。
 * 2. **选择后焦点回到触发器** —— DSH `PreferenceRow.selectMode` 里
 *    `selectorRef.current?.focus({ preventScroll: true })`。这一条看着琐碎，
 *    但键盘用户靠它才不会「焦点掉进虚空」。
 * 3. **字段级错误可见**：`role="alert"`，且不吞掉提交失败的原因。
 * 4. **disabled 来自表单状态**（DSH 的 `const disabled = !state.writable`），
 *    不是各处随手 `disabled = true`。
 *
 * # 没有抄什么
 *
 * DSH 的 `SettingsForm` 是 React + `InjectFace` + 插槽。这里是命令式 DOM，
 * 因为本仓前端是 vanilla TS —— 引入 React 只为一个弹层不划算。
 * 形态不同，契约（label/hint/value/error/disabled）保持一致。
 */

import { esc } from "./core.ts";

export type SheetField =
  | {
      kind: "text";
      id: string;
      label: string;
      hint?: string;
      value?: string;
      placeholder?: string;
      multiline?: boolean;
      required?: boolean;
    }
  | {
      kind: "pick";
      id: string;
      label: string;
      hint?: string;
      options: readonly { id: string; label: string; note?: string }[];
      multiple?: boolean;
    };

export interface SheetOptions {
  title: string;
  /** 副标题。DSH 把它放在 title 下当 description。 */
  subtitle?: string;
  fields: readonly SheetField[];
  submit: string;
  cancel?: string;
  /** 返回字符串 = 失败原因，直接显示在 `role="alert"` 处；返回 void 视为成功。 */
  onSubmit: (values: Record<string, string | string[]>) => Promise<string | void> | string | void;
}

let root: HTMLElement | null = null;
/** 打开弹层前的焦点持有者 —— 关掉后还回去（DSH 的 focus 修正）。 */
let restoreTo: HTMLElement | null = null;
let busy = false;

function ensureRoot(): HTMLElement {
  if (root) return root;
  root = document.createElement("div");
  root.id = "nb-sheet";
  root.className = "nb-sheet-scrim hidden";
  root.setAttribute("role", "dialog");
  root.setAttribute("aria-modal", "true");
  document.body.appendChild(root);
  return root;
}

function close(): void {
  if (!root) return;
  root.classList.add("hidden");
  root.innerHTML = "";
  busy = false;
  // 焦点回到触发器。少了这行，键盘用户的焦点会掉到 <body>，再从头 Tab 一遍。
  restoreTo?.focus?.();
  restoreTo = null;
}

/** 读出表单当前值。pick 的多选取数组，单选取标量字符串。 */
function collect(fields: readonly SheetField[]): Record<string, string | string[]> {
  const out: Record<string, string | string[]> = {};
  for (const f of fields) {
    if (f.kind === "text") {
      const el = document.getElementById(`sf-${f.id}`) as HTMLInputElement | HTMLTextAreaElement | null;
      out[f.id] = (el?.value ?? "").trim();
    } else {
      if (!f.multiple) {
        const checked = root?.querySelector<HTMLInputElement>(`input[name="sf-${f.id}"]:checked`);
        out[f.id] = checked?.value ?? "";
      } else {
        out[f.id] = [...(root?.querySelectorAll<HTMLInputElement>(`input[name="sf-${f.id}"]:checked`) ?? [])]
          .map((i) => i.value);
      }
    }
  }
  return out;
}

/** 必填校验。返回第一条错误，null = 通过。 */
function validate(fields: readonly SheetField[]): string | null {
  for (const f of fields) {
    if (f.kind === "text" && f.required && !String(collect([f])[f.id] ?? "").trim()) {
      return `「${f.label}」不能为空`;
    }
  }
  return null;
}

function renderField(f: SheetField): string {
  if (f.kind === "text") {
    const cls = f.multiline ? "nb-field-input nb-field-textarea" : "nb-field-input";
    const control = f.multiline
      ? `<textarea id="sf-${esc(f.id)}" class="${cls}" rows="3" placeholder="${esc(f.placeholder ?? "")}">${esc(f.value ?? "")}</textarea>`
      : `<input id="sf-${esc(f.id)}" class="${cls}" value="${esc(f.value ?? "")}" placeholder="${esc(f.placeholder ?? "")}" />`;
    return `
      <div class="nb-row">
        <div class="nb-row-text">
          <div class="nb-row-title">${esc(f.label)}</div>
          ${f.hint ? `<div class="nb-row-desc">${esc(f.hint)}</div>` : ""}
        </div>
        <div class="nb-row-control">${control}</div>
      </div>`;
  }
  const opts = f.options
    .map((o) => {
      const id = `sf-${f.id}-${o.id}`;
      const note = o.note ? `<span class="nb-opt-note">${esc(o.note)}</span>` : "";
      return `<label class="nb-opt" for="${esc(id)}">
        <input type="${f.multiple ? "checkbox" : "radio"}" id="${esc(id)}" name="sf-${esc(f.id)}" value="${esc(o.id)}" />
        <span class="nb-opt-label">${esc(o.label)}${note}</span>
      </label>`;
    })
    .join("");
  return `
    <div class="nb-row nb-row-stack">
      <div class="nb-row-text">
        <div class="nb-row-title">${esc(f.label)}</div>
        ${f.hint ? `<div class="nb-row-desc">${esc(f.hint)}</div>` : ""}
      </div>
      <div class="nb-opts" role="${f.multiple ? "group" : "radiogroup"}" aria-label="${esc(f.label)}">${opts}</div>
    </div>`;
}

/** 打开弹层表单。返回关闭函数，便于调用方在需要时主动收起。 */
export function openSheet(opts: SheetOptions): () => void {
  const host = ensureRoot();
  restoreTo = (document.activeElement as HTMLElement | null) ?? null;

  host.innerHTML = `
    <div class="nb-sheet-box" role="document">
      <div class="nb-sheet-head">
        <div>
          <div class="nb-sheet-title">${esc(opts.title)}</div>
          ${opts.subtitle ? `<div class="nb-sheet-sub">${esc(opts.subtitle)}</div>` : ""}
        </div>
        <button type="button" class="nb-icon-btn" data-sheet="cancel" aria-label="关闭">✕</button>
      </div>
      <div class="nb-sheet-body">${opts.fields.map(renderField).join("")}</div>
      <p class="nb-sheet-error hidden" role="alert"></p>
      <div class="nb-sheet-foot">
        <button type="button" class="nb-btn nb-btn--secondary" data-sheet="cancel">${esc(opts.cancel ?? "取消")}</button>
        <button type="button" class="nb-btn nb-btn--primary" data-sheet="submit">${esc(opts.submit)}</button>
      </div>
    </div>`;
  host.classList.remove("hidden");

  const errBox = host.querySelector<HTMLElement>(".nb-sheet-error")!;
  const submitBtn = host.querySelector<HTMLButtonElement>('[data-sheet="submit"]')!;
  const first = host.querySelector<HTMLInputElement | HTMLTextAreaElement>("input,textarea");

  const submit = async (): Promise<void> => {
    if (busy) return;                       // 防重入：DSH 那边也是 busy 态禁用
    const bad = validate(opts.fields);
    if (bad) {
      errBox.textContent = bad;
      errBox.classList.remove("hidden");
      first?.focus();
      return;
    }
    busy = true;
    submitBtn.classList.add("nb-btn--busy");
    submitBtn.disabled = true;
    errBox.classList.add("hidden");
    try {
      const err = await opts.onSubmit(collect(opts.fields));
      if (err) {
        // 失败要**留在弹层里**并显示原因。静默关掉等于让用户以为成功了。
        errBox.textContent = err;
        errBox.classList.remove("hidden");
        return;
      }
      close();
    } catch (e) {
      errBox.textContent = String(e instanceof Error ? e.message : e).slice(0, 200);
      errBox.classList.remove("hidden");
    } finally {
      busy = false;
      submitBtn.classList.remove("nb-btn--busy");
      submitBtn.disabled = false;
    }
  };

  host.onclick = (e) => {
    const t = e.target as HTMLElement;
    if (t === host) { close(); return; }                       // 点遮罩关闭
    const act = t.closest<HTMLElement>("[data-sheet]")?.dataset["sheet"];
    if (act === "cancel") close();
    else if (act === "submit") void submit();
  };
  // Esc 关闭。stopPropagation 避免顺手把全局快捷键也一起触发掉。
  host.onkeydown = (e) => {
    if (e.key === "Escape") { e.stopPropagation(); close(); }
  };

  first?.focus();
  return close;
}

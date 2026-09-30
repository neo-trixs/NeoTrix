/**
 * 会话流「块」模型 —— 纯类型 + 纯函数，零 DOM。
 *
 * # 为什么要把会话流从 text 改成块序列
 *
 * 2026-09-30 读了三个参考仓，它们**独立收敛到同一件事**：
 *   · CopilotKit/openmuse  `jev.ts`      —— 骨架发结构化澄清面板（带选项与出处）
 *   · DSH 两个 TUI         block-stream-writer + tool-card —— 工具调用是可折叠卡
 *   · douchat              CodeArtifact + system-message   —— 产物块与系统块
 * 三条不同路径，同一个结论：**扁平文本管子装不下 agent 实际要表达的东西**。
 *
 * 之前 `pushLine(role, text)` 只能承载纯文本，于是：
 *   · 工具调用要么塞进正文（变成噪音），要么丢失
 *   · 骨架要澄清只能吐一段话，界面无法把它渲染成可交互的东西
 *   · 错误只能变成一行字，看不出是「组件没加载」还是「渲染炸了」
 *
 * # 纯逻辑/渲染分离（本仓既有约定）
 *
 * 逻辑放这里以便 plain node 直接 import 跑自测（见 selftest.ts）；
 * DOM 渲染在 blocks.ts。理由同 registry 与 host 的拆分。
 */

// ════════════════════════════════════════════════════════════════
// 块定义
// ════════════════════════════════════════════════════════════════

/** 会话身份标记。**不与内容混在一行** —— 一行消息可能承载多种块。 */
export interface MarkBlock {
  readonly kind: "mark";
  readonly text: string;
}

/** 普通文本（用户或助手的话）。 */
export interface TextBlock {
  readonly kind: "text";
  readonly role: "user" | "bot";
  readonly text: string;
  /** 署名/模型/时间等**下沉到气泡下方**的元信息。 */
  readonly meta?: string;
}

/** 工具调用：折叠卡，摘要行 + 步骤列表。 */
export interface ToolStep {
  readonly kind: string;
  readonly detail: string;
  /** 该步是否失败。失败步不折叠时也要一眼看见。 */
  readonly failed?: boolean;
  /**
   * 产出物（截图、文件、diff…）。键是展示用的短名。
   *
   * 依据 trycua/cua（MIT）的 `ComputerCallOutputMessage` 注释「always a
   * screenshot」：它的工具输出**形状统一**，于是界面只有一套渲染器。
   * 反例是每种工具各带各的载荷，界面就得为每种工具写一个分支 ——
   * 加第 20 种工具时界面必炸。
   */
  readonly outputs?: Readonly<Record<string, string>>;
}
export interface ToolBlock {
  readonly kind: "tool";
  readonly steps: readonly ToolStep[];
}

/**
 * 推理块 —— 模型的思考过程，**一等块，不混进正文**。
 *
 * 来源 trycua/cua（MIT）的 `ReasoningMessage`：它的 `AgentMessage` 判别联合里
 * `reasoning` 与 `computer_call`/`function_call` **平级**，思考是消息类型之一，
 * 不是某条消息里的一段文字。
 *
 * 为什么要补这个：没有它，模型输出里的思考只能
 *  ① 塞进正文 → 用户要在一段话里分辨哪句是结论、哪句是自言自语
 *  ② 直接丢掉 → 排障时看不到「它当时为什么这么选」
 * 两条都不可接受。折叠起来单独一块，是唯一同时满足「默认不打扰」与
 * 「需要时可查」的做法。
 *
 * ⚠️ 折叠且**默认收起**。思考默认展开会把对话变成日志。
 */
export interface ReasoningBlock {
  readonly kind: "reasoning";
  readonly text: string;
  /** token 数（若骨架提供）。只用于折叠标题上的量级提示。 */
  readonly tokens?: number;
}

/** 产物：代码/文档/大段输出，独立于正文。 */
export interface ArtifactBlock {
  readonly kind: "artifact";
  readonly title: string;
  readonly body: string;
  readonly lang?: string;
}

/**
 * 骨架推给界面的结构化决策面板（吸收 openmuse `jev.ts` 的模型，MIT）。
 *
 * ⛔ **本轮只定义类型与不变量，界面还不渲染它**（渲染属第 2 步）。
 *    先把「这条通道存在」和「它必须满足什么」定死，再谈长相 ——
 *    顺序反了就会出现「先画了个卡，再发现选项没出处」。
 */
export interface DecisionPanel {
  readonly id: string;
  readonly threadId: string;
  readonly turnId: string;
  /**
   * 候选集版本。骨架每换一轮候选就 +1。
   * 界面上报的 `selectedId` 必须带着它见过的版本，否则是**过期作答**。
   * 没有这个字段，「你选的那个」和「我给你的那批」就无法对上。
   */
  readonly candidateSetVersion: number;
  readonly type: "clarification" | "comparison";
  readonly title: string;
  readonly options: readonly DecisionOption[];
  /**
   * 已选项。
   *
   * ⛔ 不是 `readonly`：它记录的是**本机用户已回答**这件事 —— 骨架下发时
   *    是 undefined，用户点完之后要写回去，否则活动岛会一直说「等你选择」。
   *    刻意区别于 `options`/`title` 等骨架下发的只读数据。
   */
  selectedId?: string;
  /** sample = 演示数据；live = 真实候选。界面必须区分，不能拿样本冒充。 */
  readonly mode: "sample" | "live";
}

export interface DecisionSource {
  readonly title: string;
  /** 必须是 http(s)。javascript: 会让「出处」变成可执行注入点。 */
  readonly url: string;
}

export interface DecisionOption {
  readonly id: string;
  readonly label: string;
  readonly details: readonly string[];
  readonly sources: readonly DecisionSource[];
}

export interface PanelBlock {
  readonly kind: "panel";
  readonly panel: DecisionPanel;
}

/** 系统消息：错误/警告/提示。`detail` 折叠，摘要必须说清「数据还在不在」。 */
export interface SystemBlock {
  readonly kind: "system";
  readonly level: "error" | "warn" | "info";
  readonly text: string;
  readonly detail?: string;
  /**
   * 失败是否会丢数据。douchat 的 `ChatErrorBoundary` 明确写
   * 「聊天记录未被删除」—— 这类承诺必须是**数据**，不是文案里的暗示。
   */
  readonly dataIntact?: boolean;
}

export type Block =
  | MarkBlock
  | TextBlock
  | ToolBlock
  | ReasoningBlock
  | ArtifactBlock
  | PanelBlock
  | SystemBlock;

// ════════════════════════════════════════════════════════════════
// 纯函数
// ════════════════════════════════════════════════════════════════

/** 单块稳定 key。用于追加而非整体重绘（保住滚动位置与折叠状态）。 */
export function blockKey(b: Block, index: number): string {
  switch (b.kind) {
    case "mark": return `m:${index}:${b.text}`;
    case "text": return `t:${index}:${b.role}`;
    case "tool": return `o:${index}:${b.steps.length}`;
    case "reasoning": return `r:${index}:${b.tokens ?? 0}`;
    case "artifact": return `a:${index}:${b.title}`;
    case "panel": return `p:${b.panel.id}:${b.panel.candidateSetVersion}`;
    case "system": return `s:${index}:${b.level}`;
  }
}

/** 工具卡摘要。空步骤时给出明确说法，而不是渲染出一个空卡。 */
export function summarizeTool(steps: readonly ToolStep[]): string {
  if (steps.length === 0) return "本轮没有工具调用";
  const failed = steps.filter((s) => s.failed).length;
  const base = `本轮工具 ${steps.length} 步`;
  return failed > 0 ? `${base}（${failed} 步失败）` : base;
}

/** 骨架侧发决策面板前的校验。返回第一条错误，null = 通过。 */
export function validatePanel(p: DecisionPanel): string | null {
  if (p.candidateSetVersion < 1) return "candidateSetVersion 必须 ≥ 1";
  if (p.options.length === 0) return "决策面板至少要一个选项";
  if (p.options.length > 12) return "决策面板选项超过 12 个";

  const ids = new Set<string>();
  for (const o of p.options) {
    if (ids.has(o.id)) return `选项 id 重复：${o.id}`;
    ids.add(o.id);
    if (o.details.length > 4) return `选项「${o.label}」的说明超过 4 条`;
    if (o.sources.length > 5) return `选项「${o.label}」的出处超过 5 条`;
    for (const s of o.sources) {
      if (!/^https?:\/\//i.test(s.url)) {
        // ⛔ 非 http(s) 的「出处」不是出处。javascript:/data: 拿到点击位上
        //    就是注入点，所以这条是硬校验，不降级为警告。
        return `选项「${o.label}」的出处必须是 http(s)：${s.url}`;
      }
    }
    // 比较型要求每个选项都有出处 —— 否则「比较」只是在比没有依据的断言
    if (p.type === "comparison" && o.sources.length === 0) {
      return `比较型面板的选项「${o.label}」必须给出处`;
    }
  }
  if (p.type === "comparison" && p.options.length > 3) {
    return "比较型面板最多 3 个选项（超过就失去「比较」的意义）";
  }
  if (p.selectedId && !ids.has(p.selectedId)) return `selectedId 不在选项里：${p.selectedId}`;
  return null;
}

/**
 * 校验界面上报的作答是否**对应当前候选集**。
 *
 * 版本对不上 = 用户在上一批候选里做的选择。若不拦，骨架会把旧选择当新选择用，
 * 而且**界面上看不出任何异常**。
 */
export function validateAnswer(
  p: DecisionPanel,
  answer: { optionId: string; candidateSetVersion: number },
): string | null {
  const bad = validatePanel(p);
  if (bad) return bad;
  if (answer.candidateSetVersion !== p.candidateSetVersion) {
    return `选项已更新（v${answer.candidateSetVersion} → v${p.candidateSetVersion}），请重新选择`;
  }
  if (!p.options.some((o) => o.id === answer.optionId)) {
    return `选项 ${answer.optionId} 不在当前候选集里`;
  }
  return null;
}

/**
 * 一组块里是否有**尚未回答**的决策面板。
 *
 * ⚠️ 「有面板」≠「在等你答」。已带 `selectedId` 的面板是**已回答**的：
 *    实测发现确认之后活动岛仍显示「等你选择」，用户会以为没提交成功
 *    而再点一次 —— 活动岛说的必须是**现在**的状态，不是这轮曾经的状态。
 *    同理，块流里可以有多个面板，只有**最后一个未答的**才算待办。
 */
export function hasPendingPanel(blocks: readonly Block[]): DecisionPanel | undefined {
  let pending: DecisionPanel | undefined;
  for (const b of blocks) {
    if (b.kind !== "panel") continue;
    const bad = validatePanel(b.panel);
    // 骨架发了非法面板时**不能当作有效待答** —— 界面得暴露问题，而不是
    // 渲染一张坏卡让人以为能用。
    if (bad) return undefined;
    if (b.panel.selectedId === undefined) pending = b.panel;
  }
  return pending;
}

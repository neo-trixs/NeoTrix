/**
 * OpenGhost  vendor 垫片 —— 只补渲染器要的两个全局，不实现任何功能。
 *
 * # 为什么需要
 *
 * 逐字入库的 `markdown.js` 在两处读外部全局：
 *   · `Tex.render`（行内/块级公式）—— 由同目录 `tex.js` 提供；
 *   · `I18n.t('code.copy')`（代码块复制按钮的 aria-label）—— 上游 18KB 全量
 *     词条本仓暂不需要，这里只给这一个键的中英映射。
 *
 * # 不是什么
 *
 * 不是 i18n 系统。`t()` 除 `code.copy` 外一律回键名（Honest fallback：
 * 缺词条时显示键而不是空串，空串会让按钮失名）。
 */

export interface OpenghostI18n {
  t(key: string): string;
}

const COPY_LABEL: Record<string, string> = {
  zh: '复制代码',
  en: 'Copy code',
};

/**
 * 判定当前语言。
 *
 * ⛔⛔ 旧实现有个**会覆盖宿主显式设置**的回退链：
 * ```ts
 * if (document.documentElement.lang.startsWith('zh')) return 'zh'  // 应用说 en-US ⇒ 落空
 * if (navigator.language.startsWith('zh')) return 'zh'            // 系统中文 ⇒ 返回 'zh' ❌
 * ```
 * ⇒ **用户在应用里选了英文、但操作系统是中文时，代码块复制按钮永远显示中文**
 * （实测：`documentElement.lang=en-US`、`select.value=en-US`，
 *   而按钮 aria-label 仍是「复制代码」）。
 *
 * ⇒ 修法：**宿主显式设置是权威的**。`i18n/index.ts` 在模块加载时就同步
 *   `document.documentElement.lang`（`main.tsx` 为此专门 import 它），
 *   所以它**一定**有值 ⇒ 直接以它为准。
 *   `navigator.language` 只在宿主**还没**设值时兜底（极早期首帧），
 *   那种情况下它才是唯一信息源。
 */
function lang(): string {
  if (typeof document !== 'undefined') {
    const app = document.documentElement.lang
    if (app) return app.toLowerCase().startsWith('zh') ? 'zh' : 'en'
  }
  if (typeof navigator !== 'undefined' && navigator.language?.startsWith('zh')) return 'zh'
  return 'en'
}

export function installOpenghostShim(): void {
  const g = window as unknown as Record<string, unknown>;
  if (typeof g['I18n'] === 'undefined') {
    const i18n: OpenghostI18n = {
      t: (key: string) => (key === 'code.copy' ? COPY_LABEL[lang()] ?? 'Copy code' : key),
    };
    g['I18n'] = i18n;
  }
}

declare global {
  interface Window {
    Markdown?: {
      render(text: string): string;
      inline(text: string): string;
      blocks(text: string, opts?: { live?: boolean }): string[];
    };
    Tex?: { render(source: string, opts?: { display?: boolean }): string };
    Highlight?: { code(body: string, lang: string): string };
    I18n?: OpenghostI18n;
  }
}

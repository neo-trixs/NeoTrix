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

function lang(): string {
  if (typeof document !== 'undefined' && document.documentElement.lang.startsWith('zh')) return 'zh';
  if (typeof navigator !== 'undefined' && navigator.language.startsWith('zh')) return 'zh';
  return 'en';
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

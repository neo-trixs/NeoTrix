/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // NeoTrix Faction Color Palette
        'nt-core': {
          50: '#f0fdf4',
          100: '#dcfce7',
          200: '#bbf7d0',
          300: '#86efac',
          400: '#4ade80',
          500: '#22c55e',  // NT-CORE: E8引导者 - Green
          600: '#16a34a',
          700: '#15803d',
          800: '#166534',
          900: '#14532d',
        },
        'nt-mind': {
          50: '#faf5ff',
          100: '#f3e8ff',
          200: '#e9d5ff',
          300: '#d8b4fe',
          400: '#c084fc',
          500: '#a855f7',  // NT-MIND: 进化工匠 - Purple
          600: '#9333ea',
          700: '#7e22ce',
          800: '#6b21a8',
          900: '#581c87',
        },
        'nt-memory': {
          50: '#eff6ff',
          100: '#dbeafe',
          200: '#bfdbfe',
          300: '#93c5fd',
          400: '#60a5fa',
          500: '#3b82f6',  // NT-MEMORY: 知识守护者 - Blue
          600: '#2563eb',
          700: '#1d4ed8',
          800: '#1e40af',
          900: '#1e3a8a',
        },
        'nt-world': {
          50: '#fdf4ff',
          100: '#fae8ff',
          200: '#f5d0fe',
          300: '#f0abfc',
          400: '#e879f9',
          500: '#d946ef',  // NT-WORLD: 虚空探索者 - Fuchsia
          600: '#c026d3',
          700: '#a21caf',
          800: '#86198f',
          900: '#701a75',
        },
        'nt-act': {
          50: '#fff7ed',
          100: '#ffedd5',
          200: '#fed7aa',
          300: '#fdba74',
          400: '#fb923c',
          500: '#f97316',  // NT-ACT: 行动执行者 - Orange
          600: '#ea580c',
          700: '#c2410c',
          800: '#9a3412',
          900: '#7c2d12',
        },
        'nt-io': {
          50: '#fff8f1',
          100: '#fdeedf',
          200: '#fbd9b8',
          300: '#f8c08c',
          400: '#f5a862',
          500: '#f0913a',  // NT-IO: 界面使徒 - 浅橙品牌强调 (--pri)
          600: '#e07f2b',  // --pri2
          700: '#bd6720',
          800: '#8f4e19',
          900: '#663911',
        },
        'nt-shield': {
          50: '#f8fafc',
          100: '#f1f5f9',
          200: '#e2e8f0',
          300: '#cbd5e1',
          400: '#94a3b8',
          500: '#64748b',  // NT-SHIELD: 影卫 - Slate
          600: '#475569',
          700: '#334155',
          800: '#1e293b',
          900: '#0f172a',
        },
        'nt-repair': {
          50: '#f0fdfa',
          100: '#ccfbf1',
          200: '#99f6e4',
          300: '#5eead4',
          400: '#2dd4bf',
          500: '#14b8a6',  // NT-REPAIR: 自愈工程师 - Teal
          600: '#0d9488',
          700: '#0f766e',
          800: '#115e59',
          900: '#134e4a',
        },
        // Semantic colors for UI — 雪域白浅橙 (Snowfield White, 极致单主题)
        // 纯白为体, 一丝浅橙仅出现在焦点/选中/品牌强调处
        'bg-primary': '#fbfaf7',     // --bg-base (snowfield white)
        'bg-secondary': '#f5f3ef',   // --glass-L1
        'bg-tertiary': '#efedea',    // --glass-L0
        'text-primary': '#1a1a20',   // --tx (gray-900)
        'text-secondary': '#5a5a62', // --tx2 (gray-600)
        'text-muted': '#909098',     // --tx3 (gray-400)
        'border-primary': '#e5e4e0', // --gray-200,暖白描边
        'border-focus': '#f0913a',   // --pri (浅橙, 一丝)品牌强调
      },
      fontFamily: {
        sans: ['-apple-system', 'BlinkMacSystemFont', 'SF Pro Display', 'SF Pro Text', 'Noto Sans SC', 'sans-serif'],
        serif: ['Georgia', 'Cambria', 'Noto Serif SC', 'serif'],
        mono: ['SF Mono', 'JetBrains Mono', 'Fira Code', 'Menlo', 'monospace'],
      },
      fontSize: {
        // 微字号统一刻度（9–14.5px 收敛，避免 text-[10px]/[12.5px] 硬编码漂移）
        '9px': ['9px', '12px'],
        '10px': ['10px', '13px'],
        '10.5px': ['10.5px', '13.5px'],
        '11px': ['11px', '14px'],
        '12px': ['12px', '15px'],
        '12.5px': ['12.5px', '16px'],
        '13.5px': ['13.5px', '17px'],
        '14.5px': ['14.5px', '18px'],
      },
      boxShadow: {
        'glass-inset': 'inset 0 1px 0 0 rgba(255,255,255,0.45), inset 0 0 0 0.5px rgba(255,255,255,0.25)',
        'glass-pop': '0 24px 64px rgba(40,30,20,0.18), 0 4px 16px rgba(40,30,20,0.08)',
        'hover-surface': 'inset 0 0 0 1px rgba(0,0,0,0.04)',
      },
      animation: {
        'fade-in': 'fadeIn 0.2s ease-out',
        'slide-in': 'slideIn 0.3s ease-out',
        'pulse-soft': 'pulseSoft 2s infinite',
        'pulse-glow': 'pulseGlow 3s ease-in-out infinite',
        'hero-pulse': 'heroPulse 4s ease-in-out infinite',
        'ccw': 'ccw 0.4s cubic-bezier(0.22,1,0.36,1)',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        slideIn: {
          '0%': { transform: 'translateX(-10px)', opacity: '0' },
          '100%': { transform: 'translateX(0)', opacity: '1' },
        },
        pulseSoft: {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.7' },
        },
        pulseGlow: {
          '0%, 100%': { boxShadow: '0 0 12px rgba(240,145,58,0.12)' },
          '50%': { boxShadow: '0 0 24px rgba(240,145,58,0.25)' },
        },
        heroPulse: {
          '0%, 100%': { filter: 'drop-shadow(0 0 6px rgba(240,145,58,0.2)) drop-shadow(0 0 12px rgba(240,145,58,0.1))', transform: 'scale(1) rotate(0deg)' },
          '50%': { filter: 'drop-shadow(0 0 12px rgba(240,145,58,0.35)) drop-shadow(0 0 24px rgba(240,145,58,0.15))', transform: 'scale(1.04) rotate(-8deg)' },
        },
        ccw: {
          '0%': { transform: 'rotate(0deg)' },
          '100%': { transform: 'rotate(-72deg) scale(1.1)' },
        },
      },
    },
  },
  plugins: [],
}
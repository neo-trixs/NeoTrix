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
          50: '#fdf1f1',
          100: '#fadfdf',
          200: '#f5bdbd',
          300: '#f09a9a',
          400: '#ec7878',
          500: '#e85454',  // NT-IO: 界面使徒 - Consciousness Red (--pri)
          600: '#d04040',  // --pri2
          700: '#b83333',
          800: '#8f2626',
          900: '#661a1a',
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
        // Semantic colors for UI — 中性灰阶 + 暖奶油 (Consciousness Glass v2)
        'bg-primary': '#f5f2ec',     // --bg-base (light cream)
        'bg-secondary': '#ece9e3',   // --glass-L1
        'bg-tertiary': '#e7e3dc',    // --glass-L0
        'text-primary': '#1a1a20',   // --tx (gray-900)
        'text-secondary': '#5a5a62', // --tx2 (gray-600)
        'text-muted': '#909098',     // --tx3 (gray-400)
        'border-primary': '#d4d4d8', // --gray-200
        'border-focus': '#e85454',   // --pri
      },
      fontFamily: {
        sans: ['-apple-system', 'BlinkMacSystemFont', 'SF Pro Display', 'SF Pro Text', 'Noto Sans SC', 'sans-serif'],
        serif: ['Georgia', 'Cambria', 'Noto Serif SC', 'serif'],
        mono: ['SF Mono', 'JetBrains Mono', 'Fira Code', 'Menlo', 'monospace'],
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
          '0%, 100%': { boxShadow: '0 0 12px rgba(232,84,84,0.12)' },
          '50%': { boxShadow: '0 0 24px rgba(232,84,84,0.25)' },
        },
        heroPulse: {
          '0%, 100%': { filter: 'drop-shadow(0 0 6px rgba(232,84,84,0.2)) drop-shadow(0 0 12px rgba(232,84,84,0.1))', transform: 'scale(1) rotate(0deg)' },
          '50%': { filter: 'drop-shadow(0 0 12px rgba(232,84,84,0.35)) drop-shadow(0 0 24px rgba(232,84,84,0.15))', transform: 'scale(1.04) rotate(-8deg)' },
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
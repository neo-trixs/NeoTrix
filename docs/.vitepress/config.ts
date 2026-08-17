import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'NeoTrix',
  description: 'Open-source AI-native developer toolkit with self-evolving reasoning, VSA knowledge representation, and attention routing.',
  base: '/',
  lang: 'en-US',
  lastUpdated: true,

  rewrites: {
    '4-GUIDES/:slug': 'guide/:slug',
    '3-API/:slug': 'api/:slug',
  },

  srcExclude: [
    '1-DESIGN/**',
    '2-PLANS/**',
    '3-AUDITS/**',
    '5-LEARNING/**',
    '6-REFERENCE/**',
    'absorption-knowledge-base/**',
    'NT-IO/**',
    'NT-META/**',
    'blueprints/**',
    'architecture/**',
    'legacy/**',
    '0-ARCHITECTURE/**',
    'cycle-120-mass-absorption-2026-07-22.md',
    'experience-tree.md',
    'first-principles-streaming-analysis.md',
    'neotrix-data-flow.png',
    'neotrix-data-flow.svg',
    'neotrix-evolution.html',
    'nt-pack-format.md',
  ],

  head: [
    ['link', { rel: 'icon', href: '/favicon.ico' }],
    ['meta', { name: 'theme-color', content: '#1a0533' }],
  ],

  themeConfig: {
    logo: '/logo.svg',

    nav: [
      { text: 'Home', link: '/' },
      { text: 'Guide', link: '/guide/what-is-neotrix' },
      { text: 'API', link: '/api/overview' },
      { text: 'GitHub', link: 'https://github.com/neo-trixs/NeoTrix' },
    ],

    sidebar: {
      '/guide/': [
        {
          text: 'Guide',
          items: [
            { text: 'What is NeoTrix?', link: '/guide/what-is-neotrix' },
            { text: 'Getting Started', link: '/guide/getting-started' },
            { text: 'CLI Reference', link: '/guide/cli' },
            { text: 'Desktop App', link: '/guide/desktop' },
            { text: 'Configuration', link: '/guide/configuration' },
            { text: 'Upgrading', link: '/guide/upgrading' },
            { text: 'Development', link: '/guide/development' },
          ],
        },
      ],
      '/api/': [
        {
          text: 'API Reference',
          items: [
            { text: 'Overview', link: '/api/overview' },
            { text: 'Events', link: '/api/events' },
          ],
        },
      ],
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/neo-trixs/NeoTrix' },
    ],

    footer: {
      message: 'Released under the MIT License (with proprietary exceptions for security modules).',
      copyright: 'Copyright 2026 NeoTrix',
    },
  },
})

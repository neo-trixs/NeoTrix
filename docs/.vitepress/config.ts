import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'NeoTrix',
  description: 'AI-native developer toolkit — CLI + Desktop',
  base: '/',
  lang: 'en-US',
  lastUpdated: true,

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
      { text: 'GitHub', link: 'https://github.com/neotrix/neotrix' },
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
      { icon: 'github', link: 'https://github.com/neotrix/neotrix' },
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright 2026 NeoTrix',
    },
  },
})

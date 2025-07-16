import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import {themes as prismThemes} from 'prism-react-renderer';

const config: Config = {
  title: 'oxidize.rb',
  tagline: 'Building Ruby extensions with Rust using rb-sys',
  favicon: 'img/favicon.ico',

  url: 'https://oxidize-rb.org',
  baseUrl: '/',

  organizationName: 'oxidize-rb',
  projectName: 'rb-sys',

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  markdown: {
    mermaid: true,
  },

  themes: ['@docusaurus/theme-mermaid'],

  plugins: [
    'docusaurus-plugin-image-zoom',
  ],

  // No custom scripts needed
  scripts: [],

  headTags: [
    {
      tagName: 'meta',
      attributes: {
        name: 'keywords',
        content: 'ruby, rust, extensions, ffi, rb-sys, oxidize, native extensions, ruby gems',
      },
    },
    {
      tagName: 'meta',
      attributes: {
        name: 'description',
        content: 'Build high-performance Ruby extensions with Rust using rb-sys. The easiest way to create native Ruby gems with memory safety and performance.',
      },
    },
    {
      tagName: 'meta',
      attributes: {
        name: 'robots',
        content: 'index, follow',
      },
    },
    {
      tagName: 'meta',
      attributes: {
        name: 'author',
        content: 'oxidize-rb',
      },
    },
    {
      tagName: 'link',
      attributes: {
        rel: 'canonical',
        href: 'https://oxidize-rb.org',
      },
    },
  ],

  presets: [
    [
      '@docusaurus/preset-classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/oxidize-rb/rb-sys/tree/main/docsite/',
          routeBasePath: '/docs',
          showLastUpdateAuthor: true,
          showLastUpdateTime: true,
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
        sitemap: {
          changefreq: 'weekly',
          priority: 0.5,
          ignorePatterns: ['/tags/**'],
          filename: 'sitemap.xml',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/social-card.png',
    metadata: [
      {name: 'twitter:card', content: 'summary_large_image'},
      {name: 'twitter:site', content: '@oxidize_rb'},
      {property: 'og:type', content: 'website'},
      {property: 'og:locale', content: 'en_US'},
    ],
    colorMode: {
      defaultMode: 'dark',
      disableSwitch: false,
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'oxidize.rb',
      logo: {
        alt: 'oxidize.rb Logo',
        src: 'img/logo-oxidize-rb.svg',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Documentation',
        },
        {
          href: 'https://github.com/oxidize-rb/rb-sys',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {
              label: 'Introduction',
              to: '/docs',
            },
            {
              label: 'Getting Started',
              to: '/docs/getting-started',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'Slack',
              href: 'https://join.slack.com/t/oxidize-rb/shared_invite/zt-16zv5tqte-Vi7WfzxCesdo2TqF_RYBCw',
            },
            {
              label: 'GitHub Issues',
              href: 'https://github.com/oxidize-rb/rb-sys/issues',
            },
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/oxidize-rb/rb-sys',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} rb-sys`,
    },
    prism: {
      theme: prismThemes.github,
          darkTheme: {
            plain: {
              color: 'hsl(35, 15%, 88%)',
              backgroundColor: 'hsl(220, 18%, 10%)'
            },
            styles: [
              {
                types: ['comment', 'prolog', 'doctype', 'cdata'],
                style: {
                  color: 'hsl(220, 10%, 50%)',
                  fontStyle: 'italic'
                }
              },
              {
                types: ['namespace'],
                style: {
                  opacity: 0.7
                }
              },
              {
                types: ['string', 'attr-value'],
                style: {
                  color: 'hsl(28, 60%, 70%)'
                }
              },
              {
                types: ['punctuation', 'operator'],
                style: {
                  color: 'hsl(220, 10%, 65%)'
                }
              },
              {
                types: ['entity', 'url', 'symbol', 'number', 'boolean', 'variable', 'constant', 'property', 'regex', 'inserted'],
                style: {
                  color: 'hsl(5, 65%, 75%)'
                }
              },
              {
                types: ['atrule', 'keyword', 'attr-name', 'selector'],
                style: {
                  color: 'hsl(200, 85%, 70%)'
                }
              },
              {
                types: ['function', 'deleted', 'tag'],
                style: {
                  color: 'hsl(5, 65%, 65%)'
                }
              },
              {
                types: ['function-variable'],
                style: {
                  color: 'hsl(28, 60%, 65%)'
                }
              },
              {
                types: ['tag', 'selector', 'keyword'],
                style: {
                  color: 'hsl(200, 85%, 65%)'
                }
              }
            ]
          },
      additionalLanguages: ['rust', 'ruby', 'bash', 'diff', 'json'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;

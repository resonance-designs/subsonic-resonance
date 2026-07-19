import { themes as prismThemes } from 'prism-react-renderer';
import type { Config } from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

import { getData } from './src/data';
import { GlobalConfig } from './src/entities';
import { globalConfig as configData } from './data/';

const globalConfig = getData<GlobalConfig>(configData);
const config: Config = {
  ...globalConfig.site,
  trailingSlash: false,
  favicon: 'img/favicon.ico',
  onBrokenLinks: 'throw',
  markdown: {
    mermaid: true,
    hooks: {
      onBrokenMarkdownLinks: 'throw'
    }
  },
  themes: ['@docusaurus/theme-mermaid'],
  i18n: {
    defaultLocale: 'en',
    locales: ['en']
  },
  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          path: 'docs',
          id: 'default',
          routeBasePath: 'docs',
          exclude: [
            'advanced/**', 'configuration/**', 'core-systems/**', 'pull-requests/**',
            'getting-started/quick-start.md', 'getting-started/features.md',
            'getting-started/examples.md', 'getting-started/real-world-examples.md',
            'CHANGELOG.md', 'README-Organization.md', 'index.md', 'status.md',
            'prd-*.md', 'guides/**'
          ]
        },
        theme: {
          customCss: './static/themes/default.css'
        }
      } satisfies Preset.Options
    ]
  ],
  themeConfig: {
    ...globalConfig.theme,
    image: 'img/docusaurus-social-card.jpg',
    navbar: {
      ...globalConfig?.theme?.navbar,
      hideOnScroll: false,
      items: [
        {
          type: 'doc',
          docId: 'intro',
          position: 'left',
          label: 'Docs'
        },
        {
          type: 'custom-VersionDisplay',
          position: 'right'
        },
        {
          type: 'custom-ThemeSwitcher',
          position: 'right'
        },
        {
          type: 'custom-TextSizeSwitcher',
          position: 'right'
        },
        {
          type: 'custom-ReaderMode',
          position: 'right'
        },
        {
          type: 'custom-Admin',
          position: 'right'
        }
      ]
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Project',
          items: [
            { label: 'Documentation', to: '/docs/intro' },
            { label: 'Roadmap', to: '/docs/project/roadmap' },
            { label: 'Changelog', to: '/docs/releases/changelog' }
          ]
        },
        {
          title: 'Development',
          items: [
            {
              label: 'GitHub repository',
              href: 'https://github.com/resonance-designs/subsonic-resonance'
            },
            {
              label: 'Releases',
              href: 'https://github.com/resonance-designs/subsonic-resonance/releases'
            }
          ]
        },
        {
          title: 'Legal',
          items: [{ label: 'Licensing guide', to: '/docs/project/licensing' }]
        }
      ],
      copyright: `<span class="resonance-footer__description">A unified streaming client for OpenSubsonic and other music sources.</span><span class="resonance-footer__copyright">Copyright © ${new Date().getFullYear()} Resonance Designs.</span>`
    },
    colorMode: {
      defaultMode: 'dark',
      disableSwitch: true,
      respectPrefersColorScheme: false
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula
    }
  } satisfies Preset.ThemeConfig
};

export default config;

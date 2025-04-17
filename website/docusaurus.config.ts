import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: 'OpenSecret SDK',
  tagline: 'Secure SDK for OpenSecret Cloud',
  favicon: 'img/favicon.ico',

  // Set the production url of your site here
  url: 'https://docs.opensecretcloud.com',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'OpenSecretCloud', // Usually your GitHub org/user name.
  projectName: 'OpenSecret-SDK', // Usually your repo name.

  onBrokenLinks: 'warn',
  onBrokenMarkdownLinks: 'warn',
  onDuplicateRoutes: 'warn',
  markdown: {
    mdx1Compat: {
      comments: true,
      admonitions: false,
      headingIds: false
    }
  },

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  plugins: [
    [
      'docusaurus-plugin-typedoc',
      {
        entryPoints: ['../src/lib/index.ts'],
        tsconfig: '../tsconfig.build.json',
        out: 'api',
        sidebar: {
          sidebarFile: 'typedoc-sidebar.js'
        },
        skipErrorChecking: true,
        cleanOutputDir: true,
        excludeExternals: false,
        excludeInternal: true,
        excludePrivate: true,
        excludeProtected: false,
        plugin: ['typedoc-plugin-markdown'],
        sort: ['alphabetical'],
        readme: 'none',
        disableSources: true
      }
    ]
  ],

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl:
            'https://github.com/OpenSecretCloud/OpenSecret-SDK/tree/master/',
          routeBasePath: 'docs',
          path: 'docs',
          sidebarCollapsible: true,
          sidebarCollapsed: false,
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    // Replace with your project's social card
    image: 'img/docusaurus-social-card.jpg',
    navbar: {
      title: 'OpenSecret SDK Docs',
      logo: {
        alt: 'OpenSecret Logo',
        src: 'img/logo.svg',
      },
      items: [
        {
          to: '/docs/guides/getting-started',
          position: 'left',
          label: 'Guides',
        },
        {
          to: '/docs/api',
          position: 'left',
          label: 'API Reference',
        },
        {
          href: 'https://github.com/OpenSecretCloud/OpenSecret-SDK',
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
              label: 'Getting Started',
              to: '/docs/guides/getting-started',
            },
            {
              label: 'API Reference',
              to: '/docs/api/',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/OpenSecretCloud/OpenSecret-SDK',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} OpenSecret Cloud. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;

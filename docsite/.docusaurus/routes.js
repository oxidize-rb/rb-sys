import React from 'react';
import ComponentCreator from '@docusaurus/ComponentCreator';

export default [
  {
    path: '/__docusaurus/debug',
    component: ComponentCreator('/__docusaurus/debug', '5ff'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/config',
    component: ComponentCreator('/__docusaurus/debug/config', '5ba'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/content',
    component: ComponentCreator('/__docusaurus/debug/content', 'a2b'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/globalData',
    component: ComponentCreator('/__docusaurus/debug/globalData', 'c3c'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/metadata',
    component: ComponentCreator('/__docusaurus/debug/metadata', '156'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/registry',
    component: ComponentCreator('/__docusaurus/debug/registry', '88c'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/routes',
    component: ComponentCreator('/__docusaurus/debug/routes', '000'),
    exact: true
  },
  {
    path: '/docs',
    component: ComponentCreator('/docs', 'eeb'),
    routes: [
      {
        path: '/docs',
        component: ComponentCreator('/docs', '75e'),
        routes: [
          {
            path: '/docs',
            component: ComponentCreator('/docs', 'c09'),
            routes: [
              {
                path: '/docs/',
                component: ComponentCreator('/docs/', 'c76'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/api-reference/rb-sys-features',
                component: ComponentCreator('/docs/api-reference/rb-sys-features', '134'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/api-reference/rb-sys-gem-config',
                component: ComponentCreator('/docs/api-reference/rb-sys-gem-config', 'a49'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/api-reference/test-helpers',
                component: ComponentCreator('/docs/api-reference/test-helpers', 'e7c'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/build-process',
                component: ComponentCreator('/docs/build-process', '954'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/classes-and-modules',
                component: ComponentCreator('/docs/classes-and-modules', 'c46'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/community-support',
                component: ComponentCreator('/docs/community-support', '5b6'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/cross-platform',
                component: ComponentCreator('/docs/cross-platform', '0be'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/debugging',
                component: ComponentCreator('/docs/debugging', 'ccf'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/development-approaches',
                component: ComponentCreator('/docs/development-approaches', 'db8'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/error-handling',
                component: ComponentCreator('/docs/error-handling', 'b31'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/examples',
                component: ComponentCreator('/docs/examples', 'bfa'),
                exact: true
              },
              {
                path: '/docs/getting-started',
                component: ComponentCreator('/docs/getting-started', '41b'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/hello-rusty-documentation',
                component: ComponentCreator('/docs/hello-rusty-documentation', 'af1'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/intro',
                component: ComponentCreator('/docs/intro', '853'),
                exact: true
              },
              {
                path: '/docs/memory-management',
                component: ComponentCreator('/docs/memory-management', 'b8a'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/project-setup',
                component: ComponentCreator('/docs/project-setup', '4b7'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/quick-start',
                component: ComponentCreator('/docs/quick-start', '8d8'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/testing',
                component: ComponentCreator('/docs/testing', 'cf2'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/troubleshooting',
                component: ComponentCreator('/docs/troubleshooting', 'b4e'),
                exact: true,
                sidebar: "docsSidebar"
              },
              {
                path: '/docs/working-with-ruby-objects',
                component: ComponentCreator('/docs/working-with-ruby-objects', 'd4f'),
                exact: true,
                sidebar: "docsSidebar"
              }
            ]
          }
        ]
      }
    ]
  },
  {
    path: '/',
    component: ComponentCreator('/', 'e5f'),
    exact: true
  },
  {
    path: '*',
    component: ComponentCreator('*'),
  },
];

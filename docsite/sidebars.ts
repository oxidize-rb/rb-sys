import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docsSidebar: [
    'introduction',
    {
      type: 'category',
      label: 'Getting Started',
      items: ['getting-started', 'quick-start'],
    },
    {
      type: 'category',
      label: 'Core Concepts',
      items: [
        'project-setup',
        'hello-rusty-documentation',
        'development-approaches',
        'working-with-ruby-objects',
        'classes-and-modules',
        'error-handling',
      ],
    },
    {
      type: 'category',
      label: 'Advanced Topics',
      items: [
        'memory-management',
        'build-process',
        'cross-platform',
      ],
    },
    {
      type: 'category',
      label: 'Practical Development',
      items: [
        'testing',
        'debugging',
        'troubleshooting',
      ],
    },
    {
      type: 'category',
      label: 'API Reference',
      items: [
        'api-reference/rb-sys-features',
        'api-reference/rb-sys-gem-config',
        'api-reference/test-helpers',
      ],
    },
    'community-support',
  ],
};

export default sidebars;
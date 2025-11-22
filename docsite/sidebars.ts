import type { SidebarsConfig } from "@docusaurus/plugin-content-docs";

const sidebars: SidebarsConfig = {
  docsSidebar: [
    "introduction",
    {
      type: "category",
      label: "Getting Started",
      collapsed: false,
      items: ["getting-started", "quick-start", "core-concepts"],
    },
    {
      type: "category",
      label: "Building Extensions",
      items: ["project-setup", "basic-patterns", "working-with-ruby-objects", "classes-and-modules", "error-handling"],
    },
    {
      type: "category",
      label: "Development",
      items: ["testing", "debugging", "troubleshooting"],
    },
    {
      type: "category",
      label: "Performance & Memory",
      items: ["performance", "memory-management", "build-process"],
    },
    {
      type: "category",
      label: "Deployment & Distribution",
      items: ["deployment"],
    },
    {
      type: "category",
      label: "Reference",
      items: ["api-reference/rb-sys-features", "api-reference/rb-sys-gem-config", "api-reference/test-helpers"],
    },
    {
      type: "category",
      label: "Resources",
      items: ["cookbook", "faq", "glossary", "community-support"],
    },
    "contributing",
  ],
};

export default sidebars;

import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */
const sidebars: SidebarsConfig = {
  docsSidebar: [
    {
      type: 'category',
      label: 'Getting Started',
      collapsed: false,
      items: [
        'getting-started/installation',
        'getting-started/quickstart',
      ],
    },
    {
      type: 'category',
      label: 'Core Concepts',
      items: [
        'concepts/manifests', // Future work
        // 'concepts/storage',   // Explaining ~/.distromanifesto
      ],
    },
    {
      type: 'category',
      label: 'TUI',
      items: [
        'tui/cauldron',
      ],
    },
    // {
    //   type: 'category',
    //   label: 'Reference',
    //   items: [
    //     'reference/cli',
    //   ],
    // },
  ],
};

export default sidebars;

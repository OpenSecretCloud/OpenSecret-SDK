import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */

// For the first build, typedoc-sidebar.js might not exist yet
let apiSidebar = [];
try {
  // Dynamic import for the TypeDoc generated sidebar
  apiSidebar = require('./docs/api/typedoc-sidebar.js');
} catch (e) {
  console.log('TypeDoc sidebar not found. This is expected during first build.');
}

const sidebars: SidebarsConfig = {
  // Main documentation sidebar for guides and general documentation
  tutorialSidebar: [
    'index', // Introduction first
    {
      type: 'category',
      label: 'Guides',
      items: [
        'guides/getting-started',
        'guides/authentication'
      ]
    }
  ],
  
  // API Reference sidebar populated by TypeDoc, with key components highlighted
  apiSidebar: [
    {
      type: 'category',
      label: 'TypeScript SDK API',
      link: { type: 'doc', id: 'api/index' },
      items: [
        // Core API - The most important parts developers will use
        {
          type: 'category',
          label: '🔑 Core API',
          collapsed: false,
          items: [
            'api/type-aliases/OpenSecretContextType',
            'api/functions/OpenSecretProvider',
            'api/functions/useOpenSecret',
            'api/variables/OpenSecretContext',
          ]
        },
        // Developer API - Hidden for now
        /* Hiding developer API for now to keep focus on Core API
        {
          type: 'category',
          label: '👩‍💻 Developer API',
          items: [
            'api/type-aliases/OpenSecretDeveloperContextType',
            'api/functions/OpenSecretDeveloper',
            'api/functions/useOpenSecretDeveloper',
            'api/variables/OpenSecretDeveloperContext',
          ]
        },
        */
        // Other types and utilities
        {
          type: 'category',
          label: 'Types & Utilities',
          items: [
            'api/type-aliases/OpenSecretAuthState',
            'api/type-aliases/OpenSecretDeveloperAuthState',
            'api/type-aliases/AttestationDocument',
            'api/type-aliases/ParsedAttestationView',
            'api/functions/generateSecureSecret',
            'api/functions/hashSecret',
            'api/variables/apiConfig',
            // Add any other important utility types/functions
          ]
        },
        // Full API Reference (auto-generated)
        ...(apiSidebar.length > 0 ? [{
          type: 'category',
          label: 'Full API Reference',
          collapsed: true,
          link: { type: 'doc', id: 'api/README' },
          items: apiSidebar,
        }] : []),
      ],
    },
  ],
};

export default sidebars;

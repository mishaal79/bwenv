module.exports = {
  extends: ['@commitlint/config-conventional'],

  rules: {
    'type-enum': [
      2,
      'always',
      [
        'feat',      // New feature
        'fix',       // Bug fix
        'docs',      // Documentation
        'style',     // Formatting
        'refactor',  // Code restructuring
        'perf',      // Performance
        'test',      // Tests
        'build',     // Build system
        'ci',        // CI/CD
        'chore',     // Maintenance
        'revert'     // Revert commit
      ]
    ],

    'scope-enum': [
      1,  // Warning level (not blocking)
      'always',
      [
        'cli',       // Command-line interface
        'sdk',       // Bitwarden SDK integration
        'config',    // Configuration management
        'env',       // .env file parsing
        'sync',      // Sync engine
        'logging',   // Logging system
        'deps'       // Dependencies
      ]
    ],

    'subject-case': [2, 'never', ['sentence-case', 'start-case', 'pascal-case', 'upper-case']],
    'subject-max-length': [2, 'always', 100],
    'body-max-line-length': [2, 'always', 100],
    'footer-leading-blank': [2, 'always']
  },

  helpUrl: 'https://github.com/yourusername/bwenv/blob/main/CONTRIBUTING.md#commit-messages'
};

module.exports = tseslint => ({
  files: ['src/**/*.{ts,tsx}', 'scripts/**/*.{ts,mjs}'],
  languageOptions: {
    parser: tseslint.parser,
    parserOptions: {
      ecmaVersion: 'latest',
      sourceType: 'module',
    },
  },
  rules: {
    'no-unused-vars': 'off',
    '@typescript-eslint/no-unused-vars': 'error',
    'no-console': ['error', { allow: ['warn', 'error'] }],
  },
});

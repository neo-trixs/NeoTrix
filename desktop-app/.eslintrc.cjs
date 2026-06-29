module.exports = {
  root: true,
  env: { browser: true, es2020: true },
  extends: [],
  parser: '@typescript-eslint/parser',
  parserOptions: { ecmaVersion: 'latest', sourceType: 'module' },
  plugins: [],
  rules: {
    'no-unused-vars': 'off',
    'no-console': 'warn',
  },
};

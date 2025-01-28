import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import eslintPluginUnicorn from 'eslint-plugin-unicorn';
import globals from 'globals';

const files = ['js/**/src/**/*.ts', 'js/test/*.ts'];

export default [
  {
    ignores: ['js/cli/build/**', '.yarn/**', '**/lib/**', 'js/test/demo/**'],
  },
  ...[eslint.configs.recommended, ...tseslint.configs.recommended, eslintPluginUnicorn.configs['flat/recommended']].map(
    (config) => ({
      ...config,
      files,
    }),
  ),
  {
    files,
    rules: {
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          args: 'all',
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          caughtErrorsIgnorePattern: '^_',
          destructuredArrayIgnorePattern: '^_',
          ignoreRestSiblings: true,
        },
      ],
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-empty-object-type': ['error', { allowInterfaces: 'with-single-extends' }],
      'prefer-rest-params': 'off',
      'unicorn/prevent-abbreviations': 'off',
      'unicorn/no-null': 'off',
    },
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.es2020,
        ...globals.node,
      },
    },
  },
];

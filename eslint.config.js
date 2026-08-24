import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import globals from 'globals';

export default tseslint.config(
  { ignores: ['dist/', 'src-tauri/', 'landing/'] },
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      ecmaVersion: 2022,
      globals: globals.browser,
    },
    plugins: {
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      'react-refresh/only-export-components': 'warn',
      // Kontrak IPC Tauri memakai `any` di tepi (payload event) — sudah
      // dipagari tsc strict di boundary invoke/listen.
      '@typescript-eslint/no-explicit-any': 'off',
      // Konvensi `_arg` untuk parameter placeholder (mis. stub time-stretch).
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
    },
  },
  {
    // File test vitest boleh pakai API test tanpa import-an berlebih.
    files: ['**/*.test.{ts,tsx}'],
    languageOptions: { globals: globals.node },
  }
);

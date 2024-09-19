import { defineConfig } from 'vitest/config';

const HOOKS_REGEX = '**/*-hooks.test.ts?(x)';

export default defineConfig({
  // resolving manually cuz vitest is using nodejs resolution, remove after hooks update
  // https://github.com/vitest-dev/vitest/discussions/4233
  resolve: { alias: { '@gear-js/react-hooks': '@gear-js/react-hooks/dist/esm/index.mjs' } },

  test: {
    include: [HOOKS_REGEX], // targeting only hooks
    environment: 'happy-dom', // faster than jsdom
    watch: false, // fire one time
    server: { deps: { inline: ['@gear-js/react-hooks'] } }, // patching esm to allow spying
    typecheck: { enabled: true, include: [HOOKS_REGEX], ignoreSourceErrors: true }, // test types
  },
});

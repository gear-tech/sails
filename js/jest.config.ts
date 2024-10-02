import type { Config } from '@jest/types';

const config: Config.InitialOptions = {
  globalSetup: './test/setup.js',
  clearMocks: true,
  coverageProvider: 'v8',
  testEnvironment: 'node',
  verbose: true,
  preset: 'ts-jest/presets/js-with-babel',
  transformIgnorePatterns: ['node_modules/(?!@polkadot)/'],
  extensionsToTreatAsEsm: ['.ts'],
  moduleNameMapper: {
    '^(\\.{1,2}/.*)\\.js$': '$1',
  },
  transform: {
    '^.+\\.tsx?$': ['ts-jest', { useESM: true }],
  },
  testTimeout: 15_000,
  testPathIgnorePatterns: ['demo-hooks.test.tsx'], // ignore hooks test
};

export default config;

import type { Config } from '@jest/types';

const config: Config.InitialOptions = {
  rootDir: '..',
  testMatch: ['<rootDir>/test/decode.bench.ts'],
  clearMocks: true,
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
  testTimeout: 30_000,
};

export default config;

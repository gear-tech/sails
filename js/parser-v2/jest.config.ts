import type { Config } from '@jest/types';

const config: Config.InitialOptions = {
  testMatch: ['<rootDir>/test/*.test.ts'],
  clearMocks: true,
  coverageProvider: 'v8',
  testEnvironment: 'node',
  verbose: true,
  preset: 'ts-jest/presets/js-with-babel',
  extensionsToTreatAsEsm: ['.ts'],
  moduleNameMapper: {
    '^\\./wasm-bytes\\.js$': '<rootDir>/lib/cjs/wasm-bytes.cjs',
    '^(\\.{1,2}/.*)\\.js$': '$1',
    '^sails-js-types-v2$': '<rootDir>/../types-v2/src/index.ts',
  },
  transform: {
    '^.+\\.tsx?$': [
      'ts-jest', {
        useESM: true,
      },
    ],
  },
  testTimeout: 15_000,
};

export default config;

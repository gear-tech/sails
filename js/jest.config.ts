import type { Config } from '@jest/types';
import { createDefaultEsmPreset } from 'ts-jest';

const config: Config.InitialOptions = {
  ...createDefaultEsmPreset(),
  globalSetup: './test/setup.js',
  clearMocks: true,
  coverageProvider: 'v8',
  testEnvironment: 'node',
  verbose: true,
  moduleNameMapper: {
    '^(\\.{1,2}/.*)\\.js$': '$1',
  },
  testTimeout: 15_000,
};

export default config;

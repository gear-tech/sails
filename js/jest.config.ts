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

    // manually resolving hooks to esm, cuz somehow jest resolves it to cjs?
    // if main property in a package.json is pointing to esm, there's no problem whatsoever.
    // maybe it has something to do with the package folder structure
    '^@gear-js/react-hooks$': '<rootDir>/../node_modules/@gear-js/react-hooks/dist/esm/index.mjs',
  },

  testTimeout: 15_000,

  // cuz hooks testEnvironment is jsdom. there was a problem with rxjs for example
  // https://github.com/microsoft/accessibility-insights-web/pull/5421#issuecomment-1109168149
  // https://jest-archive-august-2023.netlify.app/docs/28.x/upgrading-to-jest28/#packagejson-exports
  testEnvironmentOptions: {
    customExportConditions: ['node'],
  },
};

export default config;

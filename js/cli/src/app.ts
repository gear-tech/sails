#!/usr/bin/env node

import { readFileSync, mkdirSync, existsSync, writeFileSync } from 'fs';
import { SailsIdlParser } from 'sails-js-parser';
import { Command } from 'commander';
import { Sails } from 'sails-js';
import * as _path from 'path';
import { confirm } from '@inquirer/prompts';

import { generateHooks, generateLib } from './generate/index.js';
import * as config from './config.json';

const program = new Command();

const handler = async (path: string, out: string, name: string, project: boolean, withHooks: boolean) => {
  const parser = new SailsIdlParser();
  await parser.init();
  const sails = new Sails(parser);

  const idl = readFileSync(path, 'utf-8');

  out = out || '.';
  const dir = out;
  const libFile = project ? _path.join(dir, 'src', 'lib.ts') : _path.join(dir, 'lib.ts');
  let hooksFile = '';

  if (withHooks) {
    hooksFile = project ? _path.join(dir, 'src', 'hooks.ts') : _path.join(dir, 'hooks.ts');
  }

  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }

  if (!project) {
    if (existsSync(libFile)) {
      const answer = await confirm({
        message: `File ${libFile} exists. Do you want to overwrite?`,
        default: false,
      });

      if (!answer) {
        process.exit(0);
      }
    }

    if (hooksFile && existsSync(hooksFile)) {
      const answer = await confirm({
        message: `File ${hooksFile} exists. Do you want to overwrite?`,
        default: false,
      });

      if (!answer) {
        process.exit(0);
      }
    }
  }

  let libCode: string;
  let hooksCode: string;

  try {
    libCode = generateLib(sails.parseIdl(idl), name);
    hooksCode = withHooks ? generateHooks(sails.parseIdl(idl)) : '';
  } catch (e) {
    console.log(e.message, e.stack);
    process.exit(1);
  }

  if (!project) {
    writeFileSync(libFile, libCode);

    if (hooksFile && hooksCode) {
      writeFileSync(hooksFile, hooksCode);
      console.log(`Lib and hooks are generated at ${libFile} and ${hooksFile}`);
    } else {
      console.log(`Lib generated at ${libFile}`);
    }
  } else {
    const srcDir = _path.join(dir, 'src');
    const tsconfigPath = _path.join(dir, 'tsconfig.json');
    const pkgJsonPath = _path.join(dir, 'package.json');

    let writeTsconfig = true;
    let writePkgJson = true;

    if (existsSync(tsconfigPath)) {
      const answer = await confirm({
        message: `File ${tsconfigPath} exists. Do you want to overwrite?`,
        default: false,
      });

      if (!answer) {
        writeTsconfig = false;
      }
    }

    if (existsSync(pkgJsonPath)) {
      const answer = await confirm({
        message: `File ${pkgJsonPath} exists. Do you want to overwrite?`,
        default: false,
      });

      if (!answer) {
        writePkgJson = false;
      }
    }

    if (!existsSync(srcDir)) {
      mkdirSync(srcDir, { recursive: true });
    }

    writeFileSync(_path.join(srcDir, 'lib.ts'), libCode);

    if (hooksCode) {
      writeFileSync(_path.join(srcDir, 'hooks.ts'), hooksCode);
    }

    if (writeTsconfig) {
      writeFileSync(_path.join(dir, 'tsconfig.json'), JSON.stringify(config.tsconfig, null, 2));
    }

    if (writePkgJson) {
      const packageJson = {
        name,
        type: 'module',
        dependencies: {
          '@gear-js/api': config.versions['gear-js'],
          '@polkadot/api': config.versions['polkadot-api'],
          'sails-js': config.versions['sails-js'],
        },
        devDependencies: {
          typescript: config.versions['typescript'],
        },
        scripts: {
          build: 'tsc',
        },
      };

      if (withHooks) {
        packageJson.dependencies['@gear-js/react-hooks'] = config.versions['gear-js-hooks'];
      }

      writeFileSync(_path.join(dir, 'package.json'), JSON.stringify(packageJson, null, 2));
    }

    console.log(`Lib generated at ${dir}`);
  }
};

program
  .command('generate <path-to-file.sails.idl>')
  .option('--no-project', 'Generate single file without project structure')
  .option('--with-hooks', 'Generate React hooks')
  .option('-n --name <name>', 'Name of the library', 'program')
  .option('-o --out <path-to-dir>', 'Output directory')
  .description('Generate typescript library based on .sails.idl file')
  .action(async (path, options: { out: string; name: string; project: boolean; withHooks: boolean }) => {
    try {
      await handler(path, options.out, options.name, options.project, options.withHooks);
    } catch (error) {
      console.error(error.message);
      process.exit(1);
    }
    process.exit(0);
  });

program.parse();

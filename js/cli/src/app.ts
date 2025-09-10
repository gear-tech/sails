#!/usr/bin/env node

import { SailsIdlParser } from 'sails-js-parser';
import { Command } from 'commander';
import { Sails } from 'sails-js';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { exec } from 'node:child_process';
import { promisify } from 'node:util';

import { ProjectBuilder } from './generate/index.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const packageJson = JSON.parse(readFileSync(path.join(__dirname, '../package.json'), 'utf8'));

const execAsync = promisify(exec);

const program = new Command();
program.version(packageJson.version);

const handler = async (path: string, out: string, name: string, project: boolean, yes: boolean) => {
  const parser = new SailsIdlParser();
  await parser.init();
  const sails = new Sails(parser);

  const projectBuilder = new ProjectBuilder(sails, name)
    .setRootPath(out)
    .setIdlPath(path)
    .setIsProject(project)
    .setAutomaticOverride(yes);

  await projectBuilder.build();
};

program
  .command('generate <path-to-file.sails.idl>')
  .option('--no-project', 'Generate single file without project structure')
  .option('-n --name <name>', 'Name of the library', 'SailsProgram')
  .option('-o --out <path-to-dir>', 'Output directory')
  .option('-y --yes', 'Automatic yes to file override prompts')
  .description('Generate typescript library based on .sails.idl file')
  .action(
    async (
      path,
      options: {
        out: string;
        name: string;
        project: boolean;
        yes: boolean;
      },
    ) => {
      try {
        await handler(path, options.out, options.name, options.project, options.yes);
      } catch (error) {
        console.error(error.message);
        process.exit(1);
      }
      process.exit(0);
    },
  );

program
  .command('upgrade')
  .description('Upgrade sails-js-cli to the latest version')
  .action(async () => {
    try {
      console.log('Checking for updates...');

      const { stdout: latestVersion } = await execAsync('npm view sails-js-cli version');
      const latest = latestVersion.trim();
      const current = packageJson.version;

      console.log(`Current version: ${current}`);
      console.log(`Latest version: ${latest}`);

      if (current === latest) {
        console.log('You are already using the latest version!');
        return;
      }

      console.log('Upgrading...');
      await execAsync('npm install -g sails-js-cli@latest');
      console.log(`Successfully upgraded from ${current} to ${latest}!`);
      console.log('Please restart your terminal or run "hash -r" to use the new version.');
    } catch (error) {
      console.error('Failed to upgrade:', error.message);
      if (error.message.includes('EACCES') || error.message.includes('permission denied')) {
        console.error('Try running with sudo: sudo npm install -g sails-js-cli@latest');
      }
      process.exit(1);
    }
  });

program.parse();

export { ProjectBuilder } from './generate/index.js';

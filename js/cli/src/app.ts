#!/usr/bin/env node

import { SailsIdlParser } from 'sails-js-parser';
import { Command } from 'commander';
import { Sails } from 'sails-js';

import { ProjectBuilder } from './generate/index.js';

const program = new Command();

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

program.parse();

export { ProjectBuilder } from './generate/index.js';

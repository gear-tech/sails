#!/usr/bin/env node

import { Command } from 'commander';
import { readFileSync, mkdirSync, existsSync } from 'fs';
import { SailsIdlParser } from 'sails-js-parser';

import { generate } from './generate/index.js';
import { Sails } from './sails.js';

const program = new Command();

const handler = async (path: string, out: string, name: string) => {
  const parser = new SailsIdlParser();
  await parser.init();
  const sails = new Sails(parser);

  const idl = readFileSync(path, 'utf-8');

  out = out || '.';
  const dir = out.endsWith('.ts') ? out.split('/').slice(0, -1).join('/') : out;
  const file = out.endsWith('.ts') ? out.split('/').slice(-1)[0] : 'lib.ts';

  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }

  try {
    generate(sails.parseIdl(idl), dir, file, name);
  } catch (e) {
    console.log(e.message, e.stack);
    process.exit(1);
  }
};

program
  .command('generate <path-to-file.sails.idl>')
  .option('-o --out <path-to-dir-or-file>', 'Output directory or .ts file')
  .option('-n --name <name>', 'Name of the generated class', 'Program')
  .description('Generate typescript code from .sails.idl file')
  .action(async (path, options: { out: string; name: string }) => {
    try {
      await handler(path, options.out, options.name);
    } catch (error) {
      console.error(error.message);
      process.exit(1);
    }
    process.exit(0);
  });

program.parse();

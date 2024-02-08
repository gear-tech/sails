#!/usr/bin/env node

import { Command } from 'commander';
import { readFileSync } from 'fs';

import { generate } from './generate/index.js';
import { Sails } from './sails.js';

const program = new Command();

program
  .command('generate <path-to-file.sails.idl>')
  .option('-o --out <path-to-dir>', 'Output directory')
  .description('Generate typescript code from .sails.idl file')
  .action(async (path, options) => {
    const sails = await Sails.new();

    const idl = readFileSync(path, 'utf-8');

    generate(sails.parseIdl(idl), options.out || '.');

    process.exit(0);
  });

program.addHelpCommand().parse();

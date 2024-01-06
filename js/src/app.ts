#!/usr/bin/env node

import { Command } from 'commander';
import { readFileSync, writeFileSync } from 'fs';

import { parse } from './parser/index.js';
import { generate } from './generate/index.js';

const program = new Command();

program
  .command('generate <path-to-file.sails.idl>')
  .option('-o --out <path-to-dir>', 'Output directory')
  .description('Generate typescript code from .sails.idl file')
  .action((path, options) => {
    const idl = readFileSync(path, 'utf-8');
    const parsed = parse(idl);

    generate(parsed, options.out || '.');
  });

program
  .command('parse-and-print <path-to-file.sails.idl>')
  .description('Parse .sails.idl file and print the result')
  .action((path) => {
    const idl = readFileSync(path, 'utf-8');
    const parsed = parse(idl);

    console.log(JSON.stringify(parsed, null, 2));
  });

program
  .command('parse-into-file <path-to-file.sails.idl> <path-to-output.json>')
  .description('Parse .sails.idl file and write the result to a json file')
  .action((path, out) => {
    const idl = readFileSync(path, 'utf-8');
    const parsed = parse(idl);

    const json = JSON.stringify(parsed, null, 2);

    writeFileSync(out, json);
  });

program.addHelpCommand().parse();

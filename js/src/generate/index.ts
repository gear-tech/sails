import { fileURLToPath } from 'url';
import { cpSync } from 'fs';
import path from 'path';

import { ServiceGenerator } from './service-gen.js';
import { TypesGenerator } from './types-gen.js';
import { Output } from './output.js';
import { Sails } from '../sails.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export function generate(sails: Sails, outDir: string) {
  const out = new Output();

  const typesGen = new TypesGenerator(out, sails.program);
  typesGen.generate();

  const serviceGen = new ServiceGenerator(out, sails.program, sails.scaleCodecTypes);

  serviceGen.generate();

  out.save(path.join(outDir, 'lib.ts'));

  cpSync(path.join(__dirname, '..', '..', 'templates', 'transaction.ts'), path.join(outDir, 'transaction.ts'));
}

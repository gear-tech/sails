import { fileURLToPath } from 'url';
import { Sails } from 'sails-js';
import * as path from 'path';

import { ServiceGenerator } from './service-gen.js';
import { TypesGenerator } from './types-gen.js';
import { Output } from './output.js';

const __filename = fileURLToPath(import.meta.url);

export function generate(sails: Sails, outDir: string, outFile = 'lib.ts', className = 'Program') {
  const out = new Output();

  const typesGen = new TypesGenerator(out, sails.program);
  typesGen.generate();

  const serviceGen = new ServiceGenerator(out, sails.program, sails.scaleCodecTypes);

  serviceGen.generate(className);

  out.save(path.join(outDir, outFile));
}

import { cpSync } from 'fs';
import path from 'path';

import { ServiceGenerator } from './service-gen.js';
import { IType, IService } from '../types/index.js';
import { TypesGenerator } from './types-gen.js';
import { Output } from './output.js';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export function generate(definition: { types: IType[]; services: IService[] }, outDir: string) {
  const out = new Output();
  const typesGen = new TypesGenerator(out);
  const serviceGen = new ServiceGenerator(out);

  typesGen.prepare(definition.types);
  typesGen.generate(definition.types);

  for (const service of definition.services) {
    serviceGen.generate(service, typesGen.scaleTypes);
  }

  out.save(path.join(outDir, 'lib.ts'));

  cpSync(path.join(__dirname, '..', '..', 'templates', 'transaction.ts'), path.join(outDir, 'transaction.ts'));
}

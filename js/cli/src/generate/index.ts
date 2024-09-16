import { Sails } from 'sails-js';

import { ServiceGenerator } from './service-gen.js';
import { TypesGenerator } from './types-gen.js';
import { Output } from './output.js';

export function generateLib(sails: Sails, className = 'Program', typesOnly = false): string {
  const out = new Output();

  const typesGen = new TypesGenerator(out, sails.program);
  typesGen.generate();

  if (typesOnly) {
    const serviceGen = new ServiceGenerator(out, sails.program, sails.scaleCodecTypes);

    serviceGen.generate(className);
  }

  return out.finalize();
}

import { TypeDef } from '../parser/types.js';
import { Output } from './output.js';
import { getJsTypeDef, PayloadMethod } from '../utils/index.js';

export class BaseGenerator {
  constructor(protected _out: Output) {}

  protected getType(def: TypeDef, payloadMethod?: PayloadMethod) {
    const type = getJsTypeDef(def, payloadMethod);

    if (type.imports) {
      for (const imp of type.imports) {
        this._out.import('sails-js', imp);
      }
    }

    return type.type;
  }
}

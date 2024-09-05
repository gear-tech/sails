import { getJsTypeDef, PayloadMethod } from 'sails-js-util';
import { ISailsTypeDef } from 'sails-js-types';
import { Output } from './output.js';

export class BaseGenerator {
  constructor(protected _out: Output) {}

  protected getType(def: ISailsTypeDef, payloadMethod?: PayloadMethod) {
    const type = getJsTypeDef(def, payloadMethod);

    if (type.imports) {
      for (const imp of type.imports) {
        this._out.import('sails-js', imp);
      }
    }

    return type.type;
  }
}

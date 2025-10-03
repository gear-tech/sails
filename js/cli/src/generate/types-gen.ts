import { ISailsEnumDef, ISailsEnumVariant, ISailsProgram, ISailsTypeDef } from 'sails-js-types';

import { Output } from './output.js';
import { BaseGenerator } from './base.js';
import { formatDocs } from './format.js';

export class TypesGenerator extends BaseGenerator {
  constructor(
    out: Output,
    private _program: ISailsProgram,
    private _isStandaloneFile: boolean,
  ) {
    super(out);
  }

  public generate() {
    if (this._program.types.length === 0) {
      return;
    }

    if (this._isStandaloneFile) this._out.line('declare global {', false).increaseIndent();

    for (let i = 0; i < this._program.types.length; i++) {
      const { name, def, docs } = this._program.types[i];
      this._out.lines(formatDocs(docs), false);

      if (def.isStruct) {
        this.generateStruct(name, def);
      } else if (def.isEnum) {
        this.generateEnum(name, def.asEnum);
      } else if (def.isPrimitive || def.isOptional || def.isResult || def.asVec) {
        this._out.line(`export type ${name} = ${this.getType(def)}`);
      } else {
        throw new Error(`Unknown type: ${JSON.stringify(def)}`);
      }
      if (i < this._program.types.length - 1) {
        this._out.line();
      }
    }

    if (this._isStandaloneFile) {
      this._out.reduceIndent().line('}');
    } else {
      this._out.line();
    }

    return this._out;
  }

  private generateStruct(name: string, def: ISailsTypeDef) {
    if (def.asStruct.isTuple) {
      return this._out.line(`export type ${name} = ${this.getType(def)}`);
    }

    return this._out.block(`export interface ${name}`, () => {
      for (const field of def.asStruct.fields) {
        this._out.lines(formatDocs(field.docs), false).line(`${field.name}: ${this.getType(field.def)}`);
      }
    });
  }

  private generateEnum(typeName: string, def: ISailsEnumDef) {
    if (def.isNesting) {
      this._out.line(`export type ${typeName} = `, false).increaseIndent();
      for (const [i, variant] of def.variants.entries()) {
        this._out
          .lines(formatDocs(variant.docs), false)
          .line(`| ${this.getEnumFieldString(variant)}`, i === def.variants.length - 1);
      }
      this._out.reduceIndent();
    } else {
      this._out.line(`export type ${typeName} = ${def.variants.map((v) => `"${v.name}"`).join(' | ')}`);
    }
  }

  private getEnumFieldString(f: ISailsEnumVariant) {
    return f.def ? `{ ${f.name}: ${this.getType(f.def)} }` : `{ ${f.name}: null }`;
  }
}

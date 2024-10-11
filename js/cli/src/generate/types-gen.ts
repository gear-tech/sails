import { ISailsEnumDef, ISailsEnumVariant, ISailsProgram, ISailsTypeDef } from 'sails-js-types';
import { toLowerCaseFirst } from 'sails-js-util';

import { Output } from './output.js';
import { BaseGenerator } from './base.js';
import { formatDocs } from './format.js';

export class TypesGenerator extends BaseGenerator {
  constructor(
    out: Output,
    private _program: ISailsProgram,
  ) {
    super(out);
  }

  public generate() {
    for (const { name, def, docs } of this._program.types) {
      formatDocs(docs).forEach(line => this._out.line(line, false));

      if (def.isStruct) {
        this.generateStruct(name, def);
      } else if (def.isEnum) {
        this.generateEnum(name, def.asEnum);
      } else if (def.isPrimitive || def.isOptional || def.isResult || def.asVec) {
        this._out.line(`export type ${name} = ${this.getType(def)}`).line();
      } else {
        throw new Error(`Unknown type: ${JSON.stringify(def)}`);
      }
    }
  }

  private generateStruct(name: string, def: ISailsTypeDef) {
    if (def.asStruct.isTuple) {
      return this._out.line(`export type ${name} = ${this.getType(def)}`).line();
    }

    return this._out
      .block(`export interface ${name}`, () => {
        for (const field of def.asStruct.fields) {
          formatDocs(field.docs).forEach(line => this._out.line(line, false));
          this._out.line(`${field.name}: ${this.getType(field.def)}`);
        }
      })
      .line();
  }

  private generateEnum(typeName: string, def: ISailsEnumDef) {
    if (def.isNesting) {
      this._out.line(`export type ${typeName} = `, false).increaseIndent();
      for (const [i, variant] of def.variants.entries()) {
        formatDocs(variant.docs).forEach(line => this._out.line(line, false));
        this._out.line(`| ${this.getEnumFieldString(variant)}`, i === def.variants.length - 1);
      }
      this._out.reduceIndent().line();
    } else {
      this._out
        .line(`export type ${typeName} = ${def.variants.map((v) => `"${toLowerCaseFirst(v.name)}"`).join(' | ')}`)
        .line();
    }
  }

  private getEnumFieldString(f: ISailsEnumVariant) {
    if (!f.def) {
      return `{ ${toLowerCaseFirst(f.name)}: null }`;
    } else {
      return `{ ${toLowerCaseFirst(f.name)}: ${this.getType(f.def)} }`;
    }
  }
}

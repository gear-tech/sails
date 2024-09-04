import { EnumDef, EnumVariant, TypeDef } from '../parser/types.js';
import { Program } from '../parser/program.js';
import { toLowerCaseFirst } from '../utils/index.js';
import { Output } from './output.js';
import { BaseGenerator } from './base.js';

export class TypesGenerator extends BaseGenerator {
  constructor(
    out: Output,
    private _program: Program,
  ) {
    super(out);
  }

  public generate() {
    for (const { name, def } of this._program.types) {
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

  private generateStruct(name: string, def: TypeDef) {
    if (def.asStruct.isTuple) {
      return this._out.line(`export type ${name} = ${this.getType(def)}`).line();
    }

    return this._out
      .block(`export interface ${name}`, () => {
        for (const field of def.asStruct.fields) {
          this._out.line(`${field.name}: ${this.getType(field.def)}`);
        }
      })
      .line();
  }

  private generateEnum(typeName: string, def: EnumDef) {
    if (def.isNesting) {
      this._out.line(`export type ${typeName} = `, false).increaseIndent();
      for (let i = 0; i < def.variants.length; i++) {
        this._out.line(`| ${this.getEnumFieldString(def.variants[i])}`, i === def.variants.length - 1);
      }
      this._out.reduceIndent().line();
    } else {
      this._out
        .line(`export type ${typeName} = ${def.variants.map((v) => `"${toLowerCaseFirst(v.name)}"`).join(' | ')}`)
        .line();
    }
  }

  private getEnumFieldString(f: EnumVariant) {
    if (!f.def) {
      return `{ ${toLowerCaseFirst(f.name)}: null }`;
    } else {
      return `{ ${toLowerCaseFirst(f.name)}: ${this.getType(f.def)} }`;
    }
  }
}

import { getTypeDef, getTypeName } from './utils/types.js';
import { Output } from './output.js';
import { IInnerType, IStructType, IType, IEnumField, IEnumType } from '../types/index.js';

const getScaleCodecName = (type: IType | IInnerType) => {
  switch (type.kind) {
    case 'typeName': {
      return getTypeName(type.def, false);
    }
    case 'option': {
      return `Option<${getScaleCodecName(type.def)}>`;
    }
    case 'result': {
      return `Result<${getScaleCodecName(type.def.ok)}, ${getScaleCodecName(type.def.err)}>`;
    }
    case 'vec': {
      return `Vec<${getScaleCodecName(type.def)}>`;
    }
    case 'tuple': {
      return `(${type.def.fields.map((t) => getScaleCodecName(t)).join(', ')})`;
    }
    case 'struct': {
      const result = {};
      for (const field of type.def.fields) {
        result[field.name] = getScaleCodecName(field.type);
      }
      return result;
    }
    case 'enum': {
      const result = {};
      for (const variant of type.def.variants) {
        result[variant.name] = variant.type ? getScaleCodecName(variant.type) : null;
      }
      return { _enum: result };
    }
    default: {
      throw new Error(`Unknown type: ${JSON.stringify(type)}`);
    }
  }
};

export class TypesGenerator {
  private _scaleTypes: Record<string, any>;

  constructor(private _out: Output) {
    this._scaleTypes = {};
  }

  prepare(types: IType[]) {
    for (const type of types) {
      this._scaleTypes[getTypeName(type.type, false)] = getScaleCodecName(type);
    }
  }

  get scaleTypes() {
    return this._scaleTypes;
  }

  public generate(types: IType[]) {
    for (const type of types) {
      switch (type.kind) {
        case 'typeName':
        case 'option':
        case 'result':
        case 'vec':
        case 'tuple':
          this._out.line(`export type ${getTypeName(type.type)} = ${getTypeDef(type)}`).line();
          break;
        case 'struct': {
          this.generateStruct(type);
          break;
        }
        case 'enum': {
          this.generateEnum(type);
          break;
        }

        default: {
          throw new Error(`Unknown type: ${JSON.stringify(type)}`);
        }
      }
    }
  }

  private generateStruct(type: IStructType) {
    let typeName = getTypeName(type.type);

    this._out
      .block(`export interface ${typeName}`, () => {
        for (const field of type.def.fields) {
          this._out.line(`${field.name}: ${getTypeDef(field.type)}`);
        }
      })
      .line();
  }

  private getEnumFieldString(f: IEnumField) {
    if (!f.type) {
      return `{ ${f.name}: null }`;
    } else {
      return `{ ${f.name}: ${getTypeDef(f.type)} }`;
    }
  }

  private generateEnum(type: IEnumType) {
    this._out.line(`export type ${getTypeName(type.type)} = `, false).increaseIndent();
    for (let i = 0; i < type.def.variants.length; i++) {
      this._out.line(`| ${this.getEnumFieldString(type.def.variants[i])}`, i === type.def.variants.length - 1);
    }
    this._out.reduceIndent().line();
  }
}

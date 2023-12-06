import { getTSType } from './types-replace.js';
import { toTitle } from '../utils.js';
import { IInnerType, IStructType, ITypeDecl } from '../../types/index.js';

export const getTypeName = (type: ITypeDecl, intoTs = true) => {
  switch (type.kind) {
    case 'simple': {
      return intoTs ? getTSType(type.name) : type.name;
    }
    case 'generic': {
      return type.name + type.generic.map((g) => getTypeDef(g, true, false)).join('');
    }
    default: {
      throw new Error(`Unknown type: ${JSON.stringify(type)}`);
    }
  }
};

export const getTypeDef = (type: IInnerType | Omit<IStructType, 'type'>, generic = false, intoTS = true) => {
  switch (type.kind) {
    case 'typeName': {
      const name = getTypeName(type.def, intoTS);
      return generic ? toTitle(name) : getTypeName(type.def, intoTS);
    }
    case 'tuple': {
      const tupleItems = type.def.fields.map((f) => getTypeDef(f, generic, intoTS));
      return generic
        ? tupleItems.join('')
        : '[' + type.def.fields.map((f) => getTypeDef(f, generic, intoTS)).join(', ') + ']';
    }
    case 'option': {
      const def = getTypeDef(type.def, generic, intoTS);
      return generic ? 'Option' + toTitle(def) : 'null | ' + getTypeDef(type.def, generic, intoTS);
    }
    case 'result': {
      const ok = getTypeDef(type.def.ok, generic, intoTS);
      const err = getTypeDef(type.def.err, generic, intoTS);
      return generic ? `Result${toTitle(ok)}${toTitle(err)}` : `{ ok: ${ok} } | { err: ${err} }`;
    }
    case 'vec': {
      const def = getTypeDef(type.def, generic, intoTS);
      return generic ? `Vec${toTitle(def)}` : `Array<${def}>`;
    }
    case 'struct': {
      const def = type.def.fields.map((f) => `${f.name}: ${getTypeDef(f.type, generic, intoTS)}`).join('; ');
      return `{ ${def} }`;
    }
    default: {
      throw new Error(`Unknown type: ${JSON.stringify(type)}`);
    }
  }
};

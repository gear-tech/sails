import { IInnerType, IType } from '../../types/index.js';
import { getTypeDef, getTypeName } from './types.js';

export const getType = (t: IInnerType, onlyName = false) => {
  switch (t.kind) {
    case 'typeName': {
      return getTypeName(t.def, false);
    }
    case 'option': {
      return `Option<${getType(t.def, onlyName)}>`;
    }
    case 'result': {
      return `Result<${getType(t.def.ok, onlyName)}, ${getType(t.def.err, onlyName)}>`;
    }
    case 'vec': {
      return `Vec<${getType(t.def, onlyName)}>`;
    }
    case 'tuple': {
      return `(${t.def.fields.map((t) => getType(t, onlyName)).join(', ')})`;
    }
  }
};

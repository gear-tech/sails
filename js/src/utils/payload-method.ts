export enum PaylodMethod {
  toNumber = 'toNumber',
  toBigInt = 'toBigInt',
  toString = 'toString',
  toHex = 'toHex',
  toJSON = 'toJSON',
}

export function getPayloadMethod(type: string) {
  switch (type) {
    case 'u8':
    case 'u16':
    case 'i8':
    case 'i16':
    case 'u32':
    case 'i32':
      return PaylodMethod.toNumber;
    case 'u64':
    case 'u128':
    case 'i64':
    case 'i128':
    case 'U256':
      return PaylodMethod.toBigInt;
    case 'String':
      return PaylodMethod.toString;
    case 'H256':
      return PaylodMethod.toHex;
    default:
      return PaylodMethod.toJSON;
  }
}

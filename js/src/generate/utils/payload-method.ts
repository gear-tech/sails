export function getPayloadMethod(type: string) {
  switch (type) {
    case 'u8':
    case 'u16':
    case 'i8':
    case 'i16':
      return 'toNumber';
    case 'u32':
    case 'u64':
    case 'u128':
    case 'i32':
    case 'i64':
    case 'i128':
      return 'toBigInt';
    case 'String':
      return 'toString';
    default:
      return 'toJSON';
  }
}

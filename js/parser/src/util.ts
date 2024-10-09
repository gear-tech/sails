export const getText = (ptr: number, len: number, memory: WebAssembly.Memory): string => {
  const buf = new Uint8Array(memory.buffer.slice(ptr, ptr + len));
  return new TextDecoder().decode(buf);
};

export const getBool = (ptr: number, offset: number, memory: WebAssembly.Memory): [value: boolean, offset: 4] => {
  const is_query_buf = new Uint8Array(memory.buffer.slice(ptr + offset, ptr + offset + 1));
  const is_query_dv = new DataView(is_query_buf.buffer, 0);
  return [is_query_dv.getUint8(0) === 1, 4];
};

export const getStrPtrAndLen = (
  ptr: number,
  offset: number,
  memory: WebAssembly.Memory,
): [ptr: number, len: number] => {
  const str_ptr_buf = new Uint8Array(memory.buffer.slice(ptr + offset, ptr + offset + 4));
  const str_ptr_dv = new DataView(str_ptr_buf.buffer, 0);
  const str_ptr = str_ptr_dv.getUint32(0, true);
  offset += 4;

  const len_buf = new Uint8Array(memory.buffer.slice(ptr + offset, ptr + offset + 4));
  const len_dv = new DataView(len_buf.buffer, 0);
  const len = len_dv.getUint32(0, true);

  return [str_ptr, len];
};

export const getName = (ptr: number, offset: number, memory: WebAssembly.Memory): [name: string, offset: 8] => {
  const [str_ptr, name_len] = getStrPtrAndLen(ptr, offset, memory);

  const str = getText(str_ptr, name_len, memory);

  return [str, 8];
};

export const getDocs = (
  docsPtr: number,
  offset: number,
  memory: WebAssembly.Memory,
): [docs: string | undefined, offset: 8] => {
  const [ptr, len] = getStrPtrAndLen(docsPtr, offset, memory);

  offset += 8;

  if (ptr === 0) {
    return [undefined, 8];
  }

  const docs = getText(ptr, len, memory);

  return [docs, 8];
};

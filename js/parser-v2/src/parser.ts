import wasmParserBytes from './wasm-bytes.js';
import { IIdlDoc } from 'sails-js-types-v2';
import { fromJson } from './idl-v2-impls.js';

const WASM_PAGE_SIZE = 0x1_00_00;

interface ParserInstance extends WebAssembly.Instance {
  exports: {
    parse_idl_to_json: (idl_utf8: number, idl_len: number) => number;
    free_parse_result: (ptr: number) => void;
  }
}

export class SailsIdlParser {
  private _memory: WebAssembly.Memory;
  private _instance: ParserInstance;
  private _exports: ParserInstance['exports'];
  private _encoder: TextEncoder;
  private _decoder: TextDecoder;
  private _idlPtr: number;
  private _idlLen: number;
  private _numberOfGrownPages: number;

  constructor() {
    this._encoder = new TextEncoder();
    this._decoder = new TextDecoder('utf-8');
    this._idlPtr = 0;
    this._idlLen = 0;
    this._numberOfGrownPages = 0;
  }

  private async _decompressWasm() {
    // if (!wasmParserBytes) {
    //   throw new Error(
    //     'Missing embedded WASM bytes. Run the build to generate wasm-bytes.js or provide parser.wasm next to the bundle.',
    //   );
    // }
    const binaryStr = atob(wasmParserBytes);

    const binaryBase64 = new Uint8Array(binaryStr.length);

    for (let i = 0; i < binaryStr.length; i++) {
      binaryBase64[i] = binaryStr.codePointAt(i);
    }

    const ds = new DecompressionStream('gzip');
    const decompressed = new Response(binaryBase64).body.pipeThrough<Uint8Array>(ds);

    const reader = decompressed.getReader();
    let bytes = [];

    while (true) {
      const { value, done } = await reader.read();

      if (done) break;

      bytes = [...bytes, ...value];
    }

    return new Uint8Array(bytes).buffer;
  }

  private fillMemory(idl: string) {
    const buf = this._encoder.encode(idl);
    this._idlLen = buf.length;

    const numberOfPages = Math.round(buf.length / WASM_PAGE_SIZE) + 1;

    if (!this._idlPtr || numberOfPages > this._numberOfGrownPages) {
      this._idlPtr = this._memory.grow(numberOfPages - this._numberOfGrownPages) * WASM_PAGE_SIZE;
      this._numberOfGrownPages = numberOfPages;
    }

    for (const [i, element] of buf.entries()) {
      new Uint8Array(this._memory.buffer)[i + this._idlPtr] = element;
    }
  }

  private clearMemory() {
    for (let i = 0; i < this._numberOfGrownPages * WASM_PAGE_SIZE; i++) {
      new Uint8Array(this._memory.buffer)[i + this._idlPtr] = 0;
    }
    this._idlLen = null;
  }

  private readCString(ptr: number): string {
    const memory = new Uint8Array(this._memory.buffer);

    if (ptr <= 0 || ptr >= memory.length) {
      throw new Error('Invalid pointer returned from WASM parse_idl_to_json');
    }

    let end = ptr;

    while (end < memory.length && memory[end] !== 0) {
      end++;
    }

    if (end >= memory.length) {
      throw new Error('Unterminated C string returned from WASM parse_idl_to_json');
    }

    return this._decoder.decode(memory.subarray(ptr, end));
  }

  async init(): Promise<void> {
    const wasmBuf = await this._decompressWasm();

    this._memory = new WebAssembly.Memory({ initial: 17 });

    const source = await WebAssembly.instantiate(wasmBuf, {
      env: {
        memory: this._memory,
      },
    });

    this._instance = source.instance as ParserInstance;
    this._exports = this._instance.exports;
  }

  public parse(idl: string): IIdlDoc {
    if (!this._instance || !this._memory) {
      throw new Error('SailsIdlParser is not initialized. Call init() first.');
    }

    this.fillMemory(idl);

    const resultPtr = this._instance.exports.parse_idl_to_json(this._idlPtr, this._idlLen);

    if (!resultPtr) {
      this.clearMemory();
      throw new Error('WASM parse_idl_to_json returned a null pointer');
    }

    try {
      if (resultPtr < 0 || resultPtr + 8 > this._memory.buffer.byteLength) {
        throw new Error('Invalid pointer returned from WASM parse_idl_to_json');
      }

      const view = new DataView(this._memory.buffer);
      const errorCode = view.getUint32(resultPtr, true);
      const strPtr = view.getUint32(resultPtr + 4, true);
      const str = this.readCString(strPtr);
      if (errorCode > 0) {
        throw new Error(`Error code: ${errorCode}, Error details: ${str}`);
      }

      return fromJson(JSON.parse(str) as IIdlDoc);
    } finally {
      this._exports.free_parse_result(resultPtr);
      this.clearMemory();
    }
  }
}

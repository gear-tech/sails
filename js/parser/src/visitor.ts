export class Base {
  protected offset: number;
  public readonly rawPtr: number;

  constructor(public ptr: number, memory: WebAssembly.Memory) {
    const rawPtrBuf = new Uint8Array(memory.buffer.slice(ptr, ptr + 4));
    const rawPtrDv = new DataView(rawPtrBuf.buffer, 0);
    this.rawPtr = rawPtrDv.getUint32(0, true);
    this.offset = 4;
  }
}

import * as fs from 'fs';

const main = async () => {
  const buf = fs.readFileSync('./parser.wasm');

  const cs = new CompressionStream('gzip');

  const compressedReadableStream = new Response(buf).body.pipeThrough(cs);

  const reader = compressedReadableStream.getReader();

  let resultArr = [];

  while (true) {
    const read = await reader.read();

    if (read.done) break;

    resultArr = resultArr.concat(Array.from(read.value));
  }

  const base64Bytes = Buffer.from(Uint8Array.from(resultArr).buffer).toString('base64');

  fs.writeFileSync('./lib/src/wasm-bytes.js', `export default '${base64Bytes}'`);
  fs.writeFileSync(
    './lib/cjs/src/wasm-bytes.js',
    `Object.defineProperty(exports, '__esModule', { value: true });\n\nvar wasmParserBytes = '${base64Bytes}';\n\nexports.default = wasmParserBytes;`,
  );
};

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.log(error);
    process.exit(1);
  });

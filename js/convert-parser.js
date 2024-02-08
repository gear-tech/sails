import * as fs from 'fs';

const file = fs.readFileSync('./parser.wasm');

const bytes = file.toString('base64');

fs.writeFileSync('./lib/parser/wasm-bytes.js', `export default '${bytes}'`);

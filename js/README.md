# Installation
The package can be installed as either a global dependency or a package dependency.

```bash
npm install -g sails-js
```
or
```bash
npm install sails-js
```

# Usage

## CLI
- Generate typescript code from the IDL file
```bash
sails-js generate path/to/sails.idl -o path/to/out/dir
```
This command will generate 2 files to the specified directory.


## Library

### Parse IDL

```javascript
import { Sails } from 'sails-js';

const idl = '<idl content>';
const sails = await Sails.new();

sails.parseIdl(idl);
```

### Get service functions

```javascript
const functions = sails.functions

for (const [name, def] of Object.entries(functions)) {
  console.log(`${name}:
    isQuery: ${def.isQuery}
    arguments: ${def.args.map((arg) => `    name: ${arg.name}, type: ${arg.type}`).join('\n')}
    type of program respons: ${def.returnType}
`);
}
```

#### Encode payload
```javascript
const payload = functions.SomeFunction.encodePayload(arg1, arg2);
```

#### Decode program response
```javascript
const result = 'some bytes';

console.log(functions.SomeFunction.decodeResult(result));
```

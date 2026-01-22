# Overview
The `sails-js-parser-v2` is designed to parse Sails IDL v2 files, producing an output that can be used to generate client libraries or facilitate interaction with Gear blockchain programs.

# Installation
To install the `sails-js-parser-v2` library:

```bash
npm install sails-js-parser-v2
```

# Usage

## Initialization

```javascript
import { SailsIdlParser } from 'sails-js-parser-v2';

const parser = await SailsIdlParser.new();
```

Often used with [`sails-js`](../README.md):

```javascript
import { Sails } from 'sails-js';
import { SailsIdlParser } from 'sails-js-parser-v2';

const parser = await SailsIdlParser.new();
const sails = new Sails(parser);
```

## Parsing

```javascript
const idl = `
  service SailsService {
    functions {
      DoSomething() -> str;
    }    
  }
`
const svc = parser.parse(idl);
```

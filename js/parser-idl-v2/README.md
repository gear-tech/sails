# Overview
The `sails-js-parser-idl-v2` is designed to parse Sails IDL v2 files, producing an output that can be used to generate client libraries or facilitate interaction with Gear blockchain programs.

# Installation
To install the `sails-js-parser-idl-v2` library:

```bash
npm install sails-js-parser-idl-v2
```

# Usage

## Initialization & Parsing

```javascript
import { SailsIdlParser } from 'sails-js-parser-idl-v2';

const parser = new SailsIdlParser();
await parser.init();
```

Often used with [`sails-js`](../README.md):

```javascript
import { SailsProgram } from 'sails-js';
import { SailsIdlParser } from 'sails-js-parser-idl-v2';

const parser = new SailsIdlParser();
await parser.init();

const idl = `
  service SailsService {
    functions {
      DoSomething() -> str;
    }    
  }

  program SailsProgram {
    services {
      SailsService
    }
  }
`;
const doc = parser.parse(idl);

const program = new SailsProgram(doc);
```

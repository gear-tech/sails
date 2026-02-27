# Overview
The `sails-js-parser-idl-v2` package is bundled into [`sails-js`](../README.md) and its exports are available via the `sails-js/parser` subpath. This package is no longer published separately.

# Usage

## Initialization & Parsing

```javascript
import { SailsIdlParser } from 'sails-js/parser';

const parser = new SailsIdlParser();
await parser.init();
```

Often used with [`sails-js`](../README.md):

```javascript
import { SailsProgram } from 'sails-js';
import { SailsIdlParser } from 'sails-js/parser';

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

# Overview
The `sails-js-parser` is designed to parse Sails IDL files, producing an output that can be used to generate client libraries or facilitate interaction with Gear blockchain programs.

# Installation
To install the `sails-js-parser` library:

```bash
npm install sails-js-parser
```

# Usage

## Initialization

```javascript
import { SailsIdlParser } from 'sails-js-parser';

const parser = await SailsIdlParser.new();
```

Often used with [`sails-js`](../README.md):

```javascript
import { Sails } from 'sails-js';
import { SailsIdlParser } from 'sails-js-parser';

const parser = await SailsIdlParser.new();
const sails = new Sails(parser);
```

## Parsing

```javascript
const idl = `
  constructor {
    New : ();
  };

  service SailsApp {
    DoSomething : () -> str;
  };
`
const program = parser.parse(idl);
```

The `parse` method returns a [Program](./src/program.ts#L7) instance representing the parsed IDL.

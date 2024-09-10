# Overview

This is an example of using the generated library.

## How to use

1. Generate the library using the following command:

```bash
npx sails-js-cli generate ../../examples/demo/client/demo.idl .
```

This will generate the library in the current directory.

2. Install the dependencies and build project:

```bash
yarn install
yarn build
```

4. Build demo app using the following command:

```bash
cargo build -p demo --release
```

3. `src/main.ts` file contains the example of how to use the generated library.

```bash
node lib/main.js
```

# Overview
The `sails-js-cli` is a command-line tool designed to generate TypeScript client libraries from Sails IDL files. It automates the process of creating fully functional client libraries based on the interfaces defined in the Sails framework, streamlining development and ensuring consistency between client and on-chain programs.

# Installation
To install the `sails-js-cli` package globally, run the following command:

```bash
npm install -g sails-js-cli
```

Or you can use `npx` to run the command without installing the package:

```bash
npx sails-js-cli command ...args
```

# Generate typescript client library using the IDL file
To generate a TypeScript client library run the following command:

```bash
sails-js generate path/to/sails.idl -o path/to/out/dir
```

In case you want to use the package w/o installing it globally, you can use `npx`:

```bash
npx sails-js-cli generate path/to/sails.idl -o path/to/out/dir
```

If you want to generate only `lib.ts` file without the whole project structure, you can use the `--no-project` flag.

```bash
sails-js generate path/to/sails.idl -o path/to/out/dir --no-project
```

To place type definitions directly in the generated `lib.ts` file instead of relying on `global.d.ts`, use the `--embed-types` flag.

```bash
sails-js generate path/to/sails.idl -o path/to/out/dir --embed-types
```

# Use generated library

## Create an instance

First, connect to the chain using `@gear-js/api`.
```javascript
import { GearApi } from '@gear-js/api';

const api = await GearApi.create();
```

Import `Program` class from the generated file. And create an instance

```javascript
import { Program } from './lib';

const program = new Program(api);

// provide the id of the program if the program is already uploaded to the chain

const programId = '0x...';
const program = new Program(api, programId);
```

The `Program` class has all the functions available in the IDL file.


## Methods

There are a few types of methods available in the `Program` class.

- Query methods
- Message methods
- Constructor methods
- Event subscription methods

### Query methods
Query methods are used to query the program state.
These methods accept the arguments needed to call the function in the program and return the result. Apart from the arguments, these functions also accept optional parameters: `originAddress` is the address of the account that is calling the function (if this parameter isn't provided zero address is used as a default value), `value` is a parameter parameter can be used depending on the function to send some amount of tokens to the correct function execution and `atBlock` to query program state at a specific block.

```javascript
const alice = '0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d';
const result = await program.serviceName.queryFnName(arg1, arg2, alice);
console.log(result);
```

### Message methods
Message methods are used to send messages to the program.
These methods accept the arguments needed to send the message and return [transaction builder](../README.md#transaction-builder) that has a few methods to build and send the transaction.

```javascript
const transaction = program.serviceName.functionName(arg1, arg2);

// ## Set the account that is sending the message

// The method accepts either the KeyringPair instance
import { Keyring } from '@polkadot/api';
const keyring = new Keyring({ type: 'sr25519' });
const pair = keyring.addFromUri('//Alice');
transaction.withAccount(pair)

// Or the address and signerOptions
// This case is mostly used on the frontend with connected wallet.
import { web3FromSource, web3Accounts } from '@polkadot/extension-dapp';
const allAccounts = await web3Accounts();
const account = allAccounts[0];
const injector = await web3FromSource(account.meta.source);
transaction.withAccount(account.address, { signer: injector.signer });

// ## Set the value of the message
transaction.withValue(BigInt(10 * 1e12)); // 10 VARA

// ## Calculate gas
// Optionally you can provide 2 arguments.
// The first argument `allowOtherPanics` either allows or forbids panics in other programs to be triggered. It's set to `false` by default.
// The second argument `increaseGas` is percentage to increase the gas limit. It's set to `0` by default.
await transtaion.calculateGas();

// The `withGas` method can be used instead of `calculateGas` if you want to set the gas limit manually.
transaction.withGas(100000n);

// ## Send the transaction
// `signAndSend` method returns the if of the sent message, the block hash in which the message is included and `response` function that can be used to get the response from the program.
const { msgId, blockHash, response } = await transaction.signAndSend();

const result = await response();

console.log(result)
```

### Constructor methods
Constructor methods are postfixed with `CtorFromCode` and `CtorFromCodeId` in the `Program` class and are used to deploy the program on the chain.
These methods accept either bytes of the wasm or the id of the uploaded code.
They returns the same [transaction builder](../README.md#transaction-builder) as the message methods.

```javascript
const code = fs.readFileSync('path/to/program.wasm');
// Or fetch function can be used to fetch the code on the frontend
const transaction = program.newCtorFromCode(code);

// The same methods as in the message methods can be used to build and send the transaction
```

### Event subscription methods
Event subscription methods are used to subscribe to the specific events emitted by the program.

```javascript
program.subscribeToSomeEvent((data) => {
  console.log(data);
});
```

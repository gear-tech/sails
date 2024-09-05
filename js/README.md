# Overview
`sails-js` is a library that can be used to interact with the programs written with the [Sails](https://github.com/gear-tech/sails) framework and to generate typescript code from the Sails IDL file.

# Installation

The package can be installed as either a global dependency or a local dependency.

As a global dependency the package can be used to generate typescript code from the IDL file.
```bash
npm install -g sails-js
```

As a local dependency the package can be used to parse the IDL file and work with the program on chain. In this case you need to have `@gear-js/api` and `@polkadot/api` packages installed.
```bash
npm install sails-js
```

# Usage

## Library

### Parse IDL

```javascript
import { Sails } from 'sails-js';

const idl = '<idl content>';
const sails = await Sails.new();

sails.parseIdl(idl);
```

`sails` object contains all the constructors, services, functions and events available in the IDL file.
To send messages, create programs and subscribe to events using `Sails` it's necessary [to connect to the chain using `@gear-js/api`](https://github.com/gear-tech/gear-js/blob/main/api/README.md) and set `GearApi` instance using `setApi` method.

```javascript
import { GearApi } from '@gear-js/api';

const api = await GearApi.create();

sails.setApi(api);
```

### Constructors
`sails.ctors` property contains an object with all the constructors available in the IDL file.
The key of the object is the name of the constructor and the value is an object with the following properties:
```javascript
{
  args: Array<{name: string, type: string}>, // array of arguments with their names and scale codec types
  encodePayload: (...args: any): HexString, // function to encode the payload
  decodePayload: (bytes: HexString): any, // function to decode the payload
  fromCode: (code: Uint8Array | Buffer, ...args: unkonwn[]): TransactionBuilder, // function to create a transaction builder to deploy the program using code bytes
  fromCodeId: (codeId: string, ...args: unknown[]): TransactionBuilder // function to create a transaction builder to deploy the program using code id
}
```

To get the constructor object use `sails.ctors.ConstructorName`

`fromCode` and `fromCodeId` methods return an instance of `TransactionBuilder` class that can be used to build and send the transaction to the chain.
Check the [Transaction builder](#transaction-builder) section for more information.


### Services
`sails.services` property contains an object with all the services available in the IDL file.
The key of the object is the name of the service and the value is an object with the following properties:
```javascript
{
  functions: Record<string, SailsServiceFunc>, // object with all the functions available in the service
  queries: Record<string, SailsServiceQuery>, // object with all the queries available in the service
  events: Record<string, SailsServiceEvent>, // object with all the events available in the service
}
```
To get the service object use `sails.services.ServiceName`


### Functions
`sails.services.ServiceName.functions` property contains an object with all the functions from the IDL file that can be used to send messages to the program.
The key of the object is the name of the function and the value can be used either as a function that accepts function arguments and returns instance of `TransactionBuilder` class or as an object with the following properties:
```javascript
{
  args: Array<{name: string, type: string}>, // array of arguments with their names and scale codec types
  returnType: any, // scale codec definition of the return type
  encodePayload: (...args: any): Uint8Array, // function to encode the payload
  decodePayload: (bytes: Uint8Array): any, // function to decode the payload
  decodeResult: (result: Uint8Array): any // function to decode the result
}
```

It's necessary to provide program id so that the function can be called. It can be done using `.setProgramId` method of the `Sails` class
```javascript
sails.setProgramId('0x...');
```

Check the [Transaction builder](#transaction-builder) section for more information about the `TransactionBuilder` class.
```javascript
const transaction = sails.services.ServiceName.functions.FunctionName(arg1, arg2);
```

### Queries
`sails.services.ServiceName.queries` property contains an object with all the queries from the IDL file that can be used to read the program state.
The key of the object is the name of the function.
The value includes the same properties as described in the [Functions](#functions) section above. Note that the function returns the result of the query, not the transaction builder.

*The query function accepts 3 more arguments in addition to arguments from the IDL:*
- *`originAddress` - the address of the account that is calling the function*
- *`value` - (optional, default 0) the amount of tokens that are sent to the function*
- *`atBlock` - (optional) the block at which the query is executed*

```javascript
const alice = 'kGkLEU3e3XXkJp2WK4eNpVmSab5xUNL9QtmLPh8QfCL2EgotW';
// functionArg1, functionArg2 are the arguments of the query function from the IDL file
const result = await sails.services.ServiceName.queries.QueryName(alice, null, null, functionArg1, functionArg2);

console.log(result);
```


### Events
`sails.services.ServiceName.events` property contains an object with all the events available in the IDL file.
The key of the object is the name of the event and the value is an object with the following properties:
```javascript
{
  type: any, // scale codec definition of the event
  is: (event: UserMessageSent), // function to check if the event is of the specific type
  decode: (data: Uint8Array): any // function to decode the event data
  subscribe: (callback: (data: any) => void | Promise<void>) => void // function to subscribe to the event
}
```

To subscribe to the event use `subscribe` method of the event object.
```javascript
sails.services.ServiceName.events.EventName.subscribe((data) => {
  console.log(data);
});
```

### Get function name and decode bytes
Use `getServiceNamePrefix` function to get the service name from the payload bytes.
Use `getFnNamePrefix` method to get the function or event name from the payload bytes.
Use `sails.services.ServiceName.functions.FuncitonName.decodePayload` method of the function object to decode the payload bytes of the send message.
Use `sails.services.ServiceName.functions.FuncitonName.decodeResult` method of the function object to decode the result bytes of the received message.

```javascript
import { getServiceNamePrefix, getFnNamePrefix } from 'sails-js';
const payloadOfSentMessage = '0x<some bytes>';
const serviceName = getServiceNamePrefix(payloadOfSentMessage);
const functionName = getFnNamePrefix(payloadOfSentMessage);
console.log(sails.services[serviceName].functions[functionName].decodeResult(payloadOfSentMessage));

const payloadOfReceivedMessage = '0x<some bytes>';
console.log(sails.service[serviceName].functions[functionName].decodePayload(payloadOfReceivedMessage));
```

The same approach can be used to encode/decode bytes of the contructor or event.

```javascript
sails.ctors.ConstructorName.encodePayload(arg1, arg2);
sails.ctors.ConstructorName.decodePayload('<some bytes>');

sails.events.EventName.decode('<some bytes>')
```

### Encode payload
Use `sails.services.ServiceName.functions.FunctionName.encodePayload` method of the function object to encode the payload for the specific function. The bytes returned by this method can be used to send the message to the program.

```javascript
const payload = sails.functions.SomeFunction.encodePayload(arg1, arg2);
```


## Generate library from IDL

### Generate typescript code from the IDL file
```bash
sails-js generate path/to/sails.idl -o path/to/out/dir
```

This command generates a typescript `lib.ts` file with all functions available in the IDL file.

### Use generated library

#### Create an instance

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


#### Methods

There are a few types of methods available in the `Program` class.

- Query methods
- Message methods
- Constructor methods
- Event subscription methods

##### Query methods
Query methods are used to query the program state.
These methods accept the arguments needed to call the function in the program and return the result. Apart from the arguments, these functions also accept optional parameters: `originAddress` is the address of the account that is calling the function (if this parameter isn't provided zero address is used as a default value), `value` is a parameter parameter can be used depending on the function to send some amount of tokens to the correct function execution and `atBlock` to query program state at a specific block.

```javascript
const alice = '0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d';
const result = await program.serviceName.queryFnName(arg1, arg2, alice);
console.log(result);
```

##### Message methods
Message methods are used to send messages to the program.
These methods accept the arguments needed to send the message and return [transaction builder](#transaction-builder) that has a few methods to build and send the transaction.

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
const signer = web3FromSource();
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

##### Constructor methods
Constructor methods are postfixed with `CtorFromCode` and `CtorFromCodeId` in the `Program` class and are used to deploy the program on the chain.
These methods accept either bytes of the wasm or the id of the uploaded code.
They returns the same transaction builder as the message methods.

```javascript
const code = fs.readFileSync('path/to/program.wasm');
// Or fetch function can be used to fetch the code on the frontend
const transaction = program.newCtorFromCode(code);

// The same methods as in the message methods can be used to build and send the transaction
```

##### Event subscription methods
Event subscription methods are used to subscribe to the specific events emitted by the program.

```javascript
program.subscribeToSomeEvent((data) => {
  console.log(data);
});
```

## Transaction builder

`TransactionBuilder` class is used to build and send the transaction to the chain.

Use `.programId` property to get the id of the program that is used in the transaction.

### Methods to build and send the transaction

- `withAccount` - sets the account that is sending the message
<i>The method accepts either the KeyringPair instance or the address and signerOptions</i>

```javascript
import { Keyring } from '@polkadot/api';
const keyring = new Keyring({ type: 'sr25519' });
const pair = keyring.addFromUri('//Alice');
transaction.withAccount(pair)

// This case is mostly used on the frontend with connected wallet.
import { web3FromSource, web3Accounts } from '@polkadot/extension-dapp';
const allAccounts = await web3Accounts();
const account = allAccounts[0];
const injector = await web3FromSource(account.meta.source);
const signer = web3FromSource();
transaction.withAccount(account.address, { signer: injector.signer });
```

- `withValue` - sets the value of the message
```javascript
transaction.withValue(BigInt(10 * 1e12)); // 10 VARA
```

- `calculateGas` - calculates the gas limit of the transaction
<i>Optionally you can provide 2 arguments.
  - The first argument `allowOtherPanics` either allows or forbids panics in other programs to be triggered. It's set to `false` by default.
  - The second argument `increaseGas` is percentage to increase the gas limit. It's set to `0` by default.</i>

```javascript
await transaction.calculateGas();
```

- `withGas` - sets the gas limit of the transaction manually. Can be used instead of `calculateGas` method.
```javascript
transaction.withGas(100_000_000_000n);
```

- `withVoucher` - sets the voucher id to be used for the transaction
```javascript
const voucherId = '0x...'
transaction.withVoucher(voucherId);
```

- `transactionFee` - get the transaction fee
```javascript
const fee = await transaction.transactionFee();
console.log(fee);
```

- `signAndSend` - sends the transaction to the chain
<i>Returns the id of the sent message, transaction hash, the block hash in which the message is included, `isFinalized` to check if the transaction is finalized and `response` function that can be used to get the response from the program.
The `response` function returns a promise with the response from the program. If an error occurs during message execution the promise will be rejected with the error message.
</i>

```javascript
const { msgId, blockHash, txHash, response, isFinalized } = await transaction.signAndSend();

console.log('Message id:', msgId);
console.log('Transaction hash:', txHash);
console.log('Block hash:', blockHash);
console.log('Is finalized:', await isFinalized);

const result = await response();
console.log(result);
```

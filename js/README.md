# Overview
This directory contains libraries for interacting with programs built using the [Sails](https://github.com/gear-tech/sails) framework and generating TypeScript client libraries from Sails IDL.

`sails-js` - A library that can be used both independently and in generated clients to interact with programs via IDL files.

[`sails-js-cli`](./cli/README.md) - CLI tool to generate TypeScript client libraries from Sails IDL files.

[`sails-js-parser`](./parser/README.md) - Parser library for IDL files, used to generate AST (utilized by the other two libraries).

[`sails-js-types`](./types/README.md) - Library with types used across libraries.

[`sails-js-util`](./util/README.md) - Utility functions used across libraries.

# Installation

The sails-js library requires the `@gear-js/api` and `@polkadot/api` packages to be installed.

To install sails-js, run the following command:
```bash
npm install sails-js
```

# Usage

## Library

### Parse IDL

```javascript
import { Sails } from 'sails-js';
import { SailsIdlParser } from 'sails-js-parser';

const parser = await SailsIdlParser.new();
const sails = new Sails(parser);

const idl = '<idl content>';

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
The value includes the same properties as described in the [Functions](#functions) section above. Note that the function returns a `QueryBuilder` instance, not the result directly.

*Query functions now accept only the function's arguments and return a `QueryBuilder` class. The `QueryBuilder` provides methods to configure the query:*
- *`.withAddress(address)` - (optional) set the origin address of the account calling the function*
- *`.withValue(value)` - (optional, default 0) set the amount of tokens sent to the function*
- *`.withGasLimit(gasLimit)` - (optional, default max) set the gas limit for the query*
- *`.atBlock(blockHash)` - (optional) set the block hash at which the query is executed*
- *`.call()` - execute the query and return the result*

```javascript
const alice = 'kGkLEU3e3XXkJp2WK4eNpVmSab5xUNL9QtmLPh8QfCL2EgotW';
// functionArg1, functionArg2 are the arguments of the query function from the IDL file
const result = await sails.services.ServiceName.queries.QueryName(functionArg1, functionArg2)
  .withAddress(alice)
  .call();

console.log(result);
```

### QueryBuilder

`QueryBuilder` class is used to configure and execute queries to read program state. It provides a fluent interface for setting query parameters before execution.

#### Methods

- `withAddress(address: string)` - sets the origin address of the account calling the function (optional, default: Zero Address)
- `withValue(value: bigint)` - sets the amount of tokens sent to the function (default: 0)
- `withGasLimit(gasLimit: bigint)` - sets the gas limit for the query (default: max block gas limit)
- `atBlock(blockHash: HexString)` - sets the block hash at which the query is executed (default: latest block)
- `call()` - executes the query and returns the result

```javascript
// Basic query (uses Zero Address as default origin)
const result = await sails.services.ServiceName.queries.QueryName(arg1, arg2).call();

// Query with origin address
const result = await sails.services.ServiceName.queries.QueryName(arg1, arg2)
  .withAddress('kGkLEU3e3XXkJp2WK4eNpVmSab5xUNL9QtmLPh8QfCL2EgotW')
  .call();

// Query with full custom configuration
const result = await sails.services.ServiceName.queries.QueryName(arg1, arg2)
  .withAddress('kGkLEU3e3XXkJp2WK4eNpVmSab5xUNL9QtmLPh8QfCL2EgotW')
  .withValue(1000000000000n) // 1 VARA
  .withGasLimit(50000000000n)
  .atBlock('0x1234567890abcdef...')
  .call();
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
Use `sails.services.ServiceName.functions.FunctionName.decodePayload` method of the function object to decode the payload bytes of the send message.
Use `sails.services.ServiceName.functions.FunctionName.decodeResult` method of the function object to decode the result bytes of the received message.

```javascript
import { getServiceNamePrefix, getFnNamePrefix } from 'sails-js';
const payloadOfSentMessage = '0x<some bytes>';
const serviceName = getServiceNamePrefix(payloadOfSentMessage);
const functionName = getFnNamePrefix(payloadOfSentMessage);
console.log(sails.services[serviceName].functions[functionName].decodeResult(payloadOfSentMessage));

const payloadOfReceivedMessage = '0x<some bytes>';
console.log(sails.services[serviceName].functions[functionName].decodePayload(payloadOfReceivedMessage));
```

The same approach can be used to encode/decode bytes of the constructor or event.

```javascript
sails.ctors.ConstructorName.encodePayload(arg1, arg2);
sails.ctors.ConstructorName.decodePayload('<some bytes>');

sails.events.EventName.decode('<some bytes>')
```

### Encode payload
Use `sails.services.ServiceName.functions.FunctionName.encodePayload` method of the function object to encode the payload for the specific function. The bytes returned by this method can be used to send the message to the program.

```javascript
const payload = sails.services.ServiceName.functions.FunctionName.encodePayload(arg1, arg2);
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

- `decodePayload` - decode raw bytes from a reply message into the expected response type

<i>It's automatically called under the hood by the `response()` promise when `rawResult` is false (default).</i>

```javascript
const decodedPayload = transaction.decodePayload(rawPayloadBytes);
console.log(decodedPayload);
```

- `throwOnErrorReply` - validates a `UserMessageSent` message and throws an error if the reply indicates a failure.

<i>It's automatically called under the hood by the `response()` promise.</i>

```javascript
const { data } = await this._api.message.getReplyEvent(programId, msgId, blockHash);

transaction.throwOnErrorReply(data.message);
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

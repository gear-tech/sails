# Overview
`sails-js` is a library that can be used to interact with the programs written with the [sails](https://github.com/gear-tech/sails) framework and to generate typescript code from an IDL file (generated when the program is compiled).

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

### Sending message, creating program, querying program state, subscribing to events

`@gear-js/api` package can be used for all these actions.
`Sails` instance can be used to encode and decode payloads for the specific program functions or events.

### Constructors
`sails.ctors` property contains an object with all the constructors available in the IDL file.
The key of the object is the name of the constructor and the value is an object with the following properties:
```javascript
{
  args: Array<{name: string, type: string}>, // array of arguments with their names and scale codec types
  encodePayload: (...args: any): Uint8Array, // function to encode the payload
}
```

### Services
`sails.services` property contains an object with all the services available in the IDL file.
The key of the object is the name of the service and the value is an object with the following properties:
```javascript
{
  functions: Record<string, SailsServiceFunc>, // object with all the functions available in the service
  events: Record<string, SailsServiceEvent>, // object with all the events available in the service
}
```
To get the service object use `sails.services.ServiceName`


### Functions
`sails.services.ServiceName.functions` property contains an object with all the functions available in the IDL file.
The key of the object is the name of the function and the value is an object with the following properties:
```javascript
{
  args: Array<{name: string, type: string}>, // array of arguments with their names and scale codec types
  returnType: any, // scale codec definition of the return type
  isQuery: boolean, // true if the function is a query function
  encodePayload: (...args: any): Uint8Array, // function to encode the payload
  decodePayload: (bytes: Uint8Array): any, // function to decode the payload
  decodeResult: (result: Uint8Array): any // function to decode the result
}
```

### Available events of the program
`sails.services.ServiceName.events` property contains an object with all the events available in the IDL file.
The key of the object is the name of the event and the value is an object with the following properties:
```javascript
{
  type: any, // scale codec definition of the event
  is: (event: UserMessageSent), // function to check if the event is of the specific type
  decode: (data: Uint8Array): any // function to decode the event data
}
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
These methods accept the arguments needed to call the function in the program and return the result. Apart from the arguments, these functions also accept required parameter `originAddress` which is the address of the account that is calling the function and optional parameters `value` and `atBlock`. `Value` parameter can be used depending on the function to send some amount of tokens to the correct function execution. `atBlock` parameter can be used to query the program state at a specific block.

```javascript
const alice = '0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d';
const result = await program.someQueryFunction(arg1, arg2, alice);
console.log(result);
```

##### Message methods
Message methods are used to send messages to the program. 
These methods accept the arguments needed to send the message and return transaction builder that has a few methods to build and send the transaction.

```javascript
const transaction = program.someMessageFunction(arg1, arg2);

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
import { GearApi } from '@gear-js/api';
import { Keyring } from '@polkadot/api';
import { SailsProgram } from './lib.js';
import { readFileSync } from 'node:fs';

const main = async () => {
  const api = await GearApi.create();
  const keyring = new Keyring({ type: 'sr25519', ss58Format: 137 });

  const alice = keyring.addFromUri('//Alice');

  const program = new SailsProgram(api);

  // Deploy the program

  const code = readFileSync('../../target/wasm32-unknown-unknown/release/demo.opt.wasm');

  const ctorBuilder = await program.newCtorFromCode(code, null, null).withAccount(alice).calculateGas();
  const { blockHash, msgId, txHash } = await ctorBuilder.signAndSend();

  console.log(
    `\nProgram deployed. \n\tprogram id ${program.programId}, \n\tblock hash: ${blockHash}, \n\ttx hash: ${txHash}, \n\tinit message id: ${msgId}`,
  );

  // Call the program

  const pingBuilder = await program.pingPong.ping('ping').withAccount(alice).calculateGas();
  const { blockHash: blockHashPing, msgId: msgIdPing, txHash: txHashPing, response } = await pingBuilder.signAndSend();

  console.log(
    `\nPing message sent. \n\tBlock hash: ${blockHashPing}, \n\ttx hash: ${txHashPing}, \n\tmessage id: ${msgIdPing}`,
  );
  const reply = await response();
  console.log(`\nProgram replied: \n\t${JSON.stringify(reply)}`);
};

await main();

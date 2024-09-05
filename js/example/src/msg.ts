import { IKeyringPair } from '@polkadot/types/types';
import { decodeAddress } from '@gear-js/api';

import { RmrkCatalog } from './catalog';
import { RmrkResource } from './resource';

export const catalogAddEquippables = async (catalog: RmrkCatalog, account: IKeyringPair) => {
  const transaction = await catalog.addEquippables(0, []).withAccount(account).calculateGas();

  const { msgId, blockHash, response } = await transaction.signAndSend();

  console.log(`AddEquippables msg included in block ${blockHash}. Message id: ${msgId}`);

  try {
    const result = await response();

    console.log('AddEquippables executed successfully. Response:', result);
  } catch (error) {
    console.error(error);
  }
};

export const catalogAddSlotPart = async (catalog: RmrkCatalog, account: IKeyringPair) => {
  const transaction = await catalog
    .addParts({ 0: { slot: { equippable: [], z: 0, metadata_uri: 'metadata' } } })
    .withAccount(account)
    .calculateGas();

  const { msgId, blockHash, response } = await transaction.signAndSend();

  console.log(`AddSlotPart msg included in block ${blockHash}. Message id: ${msgId}`);

  try {
    const result = await response();

    console.log('AddSlotPart executed successfully. Response:', result);
  } catch (error) {
    console.error(error);
  }
};

export const resourceAddResourceEntry = async (resource: RmrkResource, account: IKeyringPair) => {
  const transaction = await resource
    .addResourceEntry(0, {
      slot: { src: 'src', thumb: 'thumb', metadata_uri: 'metadata', base: decodeAddress(account.address), slot: 0 },
    })
    .withAccount(account)
    .calculateGas();

  const { msgId, blockHash, response } = await transaction.signAndSend();

  console.log(`AddResourceEntry msg included in block ${blockHash}. Message id: ${msgId}`);

  try {
    const result = await response();

    console.log('AddResourceEntry executed successfully. Response:', result);
  } catch (error) {
    console.error(error);
  }
};

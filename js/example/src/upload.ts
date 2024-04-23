import { IKeyringPair } from '@polkadot/types/types';
import { GearApi } from '@gear-js/api';
import * as fs from 'fs';

import { RmrkResource } from './resource.js';
import { RmrkCatalog } from './catalog.js';

const PATH_TO_CATALOG_WASM = '../../target/wasm32-unknown-unknown/debug/rmrk_catalog.opt.wasm';
const PATH_TO_RESOURCE_WASM = '../../target/wasm32-unknown-unknown/debug/rmrk_resource.opt.wasm';

export const uploadCatalog = async (api: GearApi, account: IKeyringPair): Promise<RmrkCatalog> => {
  const catalog = new RmrkCatalog(api);
  const code = fs.readFileSync(PATH_TO_CATALOG_WASM);

  const transaction = await catalog.newCtorFromCode(code).withAccount(account).calculateGas();

  const { msgId, blockHash, response } = await transaction.signAndSend();

  try {
    await response();
  } catch (error) {
    console.error(error);
  }

  console.log(`RMRK catalog uploaded with msgId: ${msgId}, blockHash: ${blockHash}. Program id: ${catalog.programId}`);

  return catalog;
};

export const uploadResource = async (api: GearApi, account: IKeyringPair): Promise<RmrkResource> => {
  const resource = new RmrkResource(api);
  const code = fs.readFileSync(PATH_TO_RESOURCE_WASM);

  const transaction = await resource.newCtorFromCode(code).withAccount(account).calculateGas();

  const { msgId, blockHash, response } = await transaction.signAndSend();

  try {
    await response();
  } catch (error) {
    console.error(error);
  }

  console.log(
    `RMRK resource uploaded with msgId: ${msgId}, blockHash: ${blockHash}. Program id: ${resource.programId}`,
  );

  return resource;
};

import { IKeyringPair } from '@polkadot/types/types';
import { RmrkCatalog } from './catalog.js';

export const catalogReadPart = async (catalog: RmrkCatalog, account: IKeyringPair) => {
  const part = await catalog.part(0, account.address);

  console.log('Part:', part);
};

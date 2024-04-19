import { GearApi } from '@gear-js/api';
import { Keyring } from '@polkadot/api';

import { catalogAddEquippables, catalogAddSlotPart, resourceAddResourceEntry } from './msg.js';
import { uploadCatalog, uploadResource } from './upload.js';
import { subscribeToResourceAddedEvent } from './events.js';
import { catalogReadPart } from './state.js';

const main = async () => {
  const api = await GearApi.create();
  const keyring = new Keyring({ type: 'sr25519', ss58Format: 137 });

  const alice = keyring.addFromUri('//Alice');

  const catalog = await uploadCatalog(api, alice);

  const resource = await uploadResource(api, alice);

  subscribeToResourceAddedEvent(resource);

  await catalogAddSlotPart(catalog, alice);
  await catalogAddEquippables(catalog, alice);
  await resourceAddResourceEntry(resource, alice);

  await catalogReadPart(catalog, alice);
};

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.log(error);
    process.exit(1);
  });

import { RmrkResource } from './resource.js';

export const subscribeToResourceAddedEvent = async (resource: RmrkResource) => {
  resource.subscribeToResourceAddedEvent(({ resource_id }) => {
    console.log(`Resource added with id: ${resource_id}`);
  });
};

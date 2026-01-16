use crate::catalogs::rmrk_catalog::{RmrkCatalog, RmrkCatalogImpl};
use errors::{Error, Result};
use resources::{ComposedResource, PartId, Resource, ResourceId};
use sails_rs::{
    client::*,
    collections::HashMap,
    gstd::{Syscall, service},
    prelude::*,
};

pub mod errors;
pub mod resources;

// Fully hidden service state
static mut RESOURCE_STORAGE_DATA: Option<ResourceStorageData> = None;
static mut RESOURCE_STORAGE_ADMIN: Option<ActorId> = None;

#[derive(Default)]
struct ResourceStorageData {
    resources: HashMap<ResourceId, Resource>,
}

// Service event type definition
#[event]
#[derive(TypeInfo, Encode, ReflectHash)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
#[reflect_hash(crate = sails_rs)]
pub enum ResourceStorageEvent {
    ResourceAdded {
        resource_id: ResourceId,
    },
    PartAdded {
        resource_id: ResourceId,
        part_id: PartId,
    },
}

pub struct ResourceStorage<TCatalogClient = Service<RmrkCatalogImpl, GstdEnv>> {
    catalog_client: TCatalogClient,
}

impl<TCatalogClient> ResourceStorage<TCatalogClient>
where
    TCatalogClient: RmrkCatalog<Env = GstdEnv>,
{
    // This function needs to be called before any other function
    pub fn seed() {
        unsafe {
            RESOURCE_STORAGE_DATA = Some(ResourceStorageData::default());
            RESOURCE_STORAGE_ADMIN = Some(Syscall::message_source());
        }
    }

    pub fn new(catalog_client: TCatalogClient) -> Self {
        Self { catalog_client }
    }
}

// Declare the service can emit events of type ResourceStorageEvent
#[service(events = ResourceStorageEvent)]
impl<TCatalogClient> ResourceStorage<TCatalogClient>
where
    TCatalogClient: RmrkCatalog<Env = GstdEnv>,
{
    #[export]
    pub fn add_resource_entry(
        &mut self,
        resource_id: ResourceId,
        resource: Resource,
    ) -> Result<(ResourceId, Resource)> {
        self.require_admin()?;

        if resource_id == 0 {
            return Err(Error::ZeroResourceId);
        }

        if self
            .data_mut()
            .resources
            .insert(resource_id, resource.clone())
            .is_some()
        {
            return Err(Error::ResourceAlreadyExists);
        }

        // Emit event right before the method returns via
        // the generated `emit_event` method
        self.emit_event(ResourceStorageEvent::ResourceAdded { resource_id })
            .unwrap();

        Ok((resource_id, resource))
    }

    #[export]
    pub async fn add_part_to_resource(
        &mut self,
        resource_id: ResourceId,
        part_id: PartId,
    ) -> Result<PartId> {
        self.require_admin()?;

        let resource = self
            .data_mut()
            .resources
            .get_mut(&resource_id)
            .ok_or(Error::ResourceNotFound)?;

        if let Resource::Composed(ComposedResource { base, parts, .. }) = resource {
            // Caution: The execution of this method pauses right after the call to `recv` method due to
            //          its asynchronous nature , and all changes made to the state are saved, i.e. if we
            //          modify the `resource` variable here, the new value will be available to the other
            //          calls of this or another method (e.g. `add_resource_entry`) working with the same
            //          data before this method returns.

            // Call `Rmrk Catalog` via the generated client
            let part = self
                .catalog_client
                .part(part_id)
                .with_destination(*base)
                .await
                .unwrap();

            // Caution: Reading from the `resource` variable here may yield unexpected value.
            //          This can happen because execution after asynchronous calls can resume
            //          after a number of blocks, and the `resources` map can be modified by that time
            //          by a call of this or another method (e.g. `add_resource_entry`) working
            //          with the same data.

            if part.is_none() {
                return Err(Error::PartNotFound);
            }
            parts.push(part_id);
        } else {
            return Err(Error::WrongResourceType);
        }

        // Emit event right before the method returns via
        // the generated `emit_event` method
        self.emit_event(ResourceStorageEvent::PartAdded {
            resource_id,
            part_id,
        })
        .unwrap();

        Ok(part_id)
    }

    #[export]
    pub fn resource(&self, resource_id: ResourceId) -> Result<Resource> {
        self.data()
            .resources
            .get(&resource_id)
            .cloned()
            .ok_or(Error::ResourceNotFound)
    }

    fn require_admin(&self) -> Result<()> {
        if Syscall::message_source() != resource_storage_admin() {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    #[allow(static_mut_refs)]
    fn data(&self) -> &'static ResourceStorageData {
        unsafe { RESOURCE_STORAGE_DATA.as_ref().unwrap() }
    }

    #[allow(static_mut_refs)]
    fn data_mut(&mut self) -> &'static mut ResourceStorageData {
        unsafe { RESOURCE_STORAGE_DATA.as_mut().unwrap() }
    }
}

fn resource_storage_admin() -> ActorId {
    unsafe { RESOURCE_STORAGE_ADMIN.unwrap() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalogs::rmrk_catalog::{FixedPart, Part, mockall::MockRmrkCatalog};
    use resources::ComposedResource;
    use sails_rs::{client::PendingCall, gstd::services::Service};

    #[tokio::test]
    async fn test_add_resource_entry() {
        Syscall::with_message_source(ActorId::from(1));

        ResourceStorage::<MockRmrkCatalog>::seed();
        let mut resource_storage = ResourceStorage::new(MockRmrkCatalog::new()).expose(1);

        let resource = Resource::Composed(ComposedResource {
            src: "src".to_string(),
            thumb: "thumb".to_string(),
            metadata_uri: "metadata_uri".to_string(),
            base: 1.into(),
            parts: vec![],
        });
        let (actual_resource_id, actual_resource) = resource_storage
            .add_resource_entry(1, resource.clone())
            .unwrap();
        assert_eq!(actual_resource_id, 1);
        assert_eq!(actual_resource, resource);

        // add_part_to_resource
        resource_storage
            .catalog_client
            .expect_part()
            .with(mockall::predicate::eq(1))
            .return_once(|_| {
                PendingCall::from_output(Some(Part::Fixed(FixedPart {
                    z: None,
                    metadata_uri: "metadata_uri".to_string(),
                })))
            });

        let actual_part_id = resource_storage
            .add_part_to_resource(actual_resource_id, 1)
            .await
            .unwrap();
        assert_eq!(actual_part_id, 1);
    }
}

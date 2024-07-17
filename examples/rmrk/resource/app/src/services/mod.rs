use crate::catalogs::traits::RmrkCatalog;
use errors::{Error, Result};
use resources::{ComposedResource, PartId, Resource, ResourceId};
use sails::{
    calls::Query,
    collections::HashMap,
    gstd::{gservice, ExecContext},
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
#[derive(TypeInfo, Encode)]
pub enum ResourceStorageEvent {
    ResourceAdded {
        resource_id: ResourceId,
    },
    PartAdded {
        resource_id: ResourceId,
        part_id: PartId,
    },
}

pub struct ResourceStorage<TExecContext, TCatalogClient> {
    exec_context: TExecContext,
    catalog_client: TCatalogClient,
}

// Declare the service can emit events of type ResourceStorageEvent
#[gservice(events = ResourceStorageEvent)]
impl<TExecContext, TCatalogClient> ResourceStorage<TExecContext, TCatalogClient>
where
    TExecContext: ExecContext,
    TCatalogClient: RmrkCatalog,
{
    // This function needs to be called before any other function
    pub fn seed(exec_context: TExecContext) {
        unsafe {
            RESOURCE_STORAGE_DATA = Some(ResourceStorageData::default());
            RESOURCE_STORAGE_ADMIN = Some(exec_context.actor_id());
        }
    }

    pub fn new(exec_context: TExecContext, catalog_client: TCatalogClient) -> Self {
        Self {
            exec_context,
            catalog_client,
        }
    }

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
        // the generated `notify_on` method
        self.notify_on(ResourceStorageEvent::ResourceAdded { resource_id })
            .unwrap();

        Ok((resource_id, resource))
    }

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

            // Call `Rmrk Catalog` via the generated client and the `recv` method
            let part = self.catalog_client.part(part_id).recv(*base).await.unwrap();

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
        // the generated `notify_on` method
        self.notify_on(ResourceStorageEvent::PartAdded {
            resource_id,
            part_id,
        })
        .unwrap();

        Ok(part_id)
    }

    pub fn resource(&self, resource_id: ResourceId) -> Result<Resource> {
        self.data()
            .resources
            .get(&resource_id)
            .cloned()
            .ok_or(Error::ResourceNotFound)
    }

    fn require_admin(&self) -> Result<()> {
        if self.exec_context.actor_id() != resource_storage_admin() {
            return Err(Error::NotAuthorized);
        }
        Ok(())
    }

    fn data(&self) -> &'static ResourceStorageData {
        unsafe { RESOURCE_STORAGE_DATA.as_ref().unwrap() }
    }

    fn data_mut(&mut self) -> &'static mut ResourceStorageData {
        unsafe { RESOURCE_STORAGE_DATA.as_mut().unwrap() }
    }
}

fn resource_storage_admin() -> ActorId {
    unsafe { RESOURCE_STORAGE_ADMIN.unwrap() }
}

#[cfg(test)]
mod tests {
    use core::marker::PhantomData;

    use super::*;
    use crate::catalogs::{Error, Part};
    use resources::BasicResource;
    use sails::{calls::*, collections::*, gstd::calls::GStdRemoting, mocks::*, ActorId};

    #[test]
    fn test_add_resource_entry() {
        ResourceStorage::<ExecContextMock, MockCatalogClient<GStdRemoting>>::seed(
            ExecContextMock {
                actor_id: 1.into(),
                message_id: 1.into(),
            },
        );
        let mut resource_storage = ResourceStorage::new(
            ExecContextMock {
                actor_id: 1.into(),
                message_id: 1.into(),
            },
            MockCatalogClient::<GStdRemoting> { _r: PhantomData },
        );
        let resource = Resource::Basic(BasicResource {
            src: "src".to_string(),
            thumb: None,
            metadata_uri: "metadata_uri".to_string(),
        });
        let (actual_resource_id, actual_resource) = resource_storage
            .add_resource_entry(1, resource.clone())
            .unwrap();
        assert_eq!(actual_resource_id, 1);
        assert_eq!(actual_resource, resource);
    }

    struct ExecContextMock {
        actor_id: ActorId,
        message_id: MessageId,
    }

    impl ExecContext for ExecContextMock {
        fn actor_id(&self) -> ActorId {
            self.actor_id
        }

        fn message_id(&self) -> MessageId {
            self.message_id
        }
    }

    struct MockCatalogClient<R: Remoting> {
        _r: PhantomData<R>,
    }

    impl<R: Remoting> RmrkCatalog for MockCatalogClient<R> {
        type Args = R::Args;

        fn add_parts(
            &mut self,
            _parts: BTreeMap<u32, Part>,
        ) -> impl Call<Output = Result<BTreeMap<u32, Part>, Error>, Args = R::Args> {
            MockCall::new()
        }

        fn remove_parts(
            &mut self,
            _part_ids: Vec<u32>,
        ) -> impl Call<Output = Result<Vec<u32>, Error>, Args = R::Args> {
            MockCall::new()
        }

        fn add_equippables(
            &mut self,
            _part_id: u32,
            _collection_ids: Vec<ActorId>,
        ) -> impl Call<Output = Result<(u32, Vec<ActorId>), Error>, Args = R::Args> {
            MockCall::new()
        }

        fn remove_equippable(
            &mut self,
            _part_id: u32,
            _collection_id: ActorId,
        ) -> impl Call<Output = Result<(u32, ActorId), Error>, Args = R::Args> {
            MockCall::new()
        }

        fn reset_equippables(
            &mut self,
            _part_id: u32,
        ) -> impl Call<Output = Result<(), Error>, Args = R::Args> {
            MockCall::new()
        }

        fn set_equippables_to_all(
            &mut self,
            _part_id: u32,
        ) -> impl Call<Output = Result<(), Error>, Args = R::Args> {
            MockCall::new()
        }

        fn part(&self, _part_id: u32) -> impl Query<Output = Option<Part>, Args = R::Args> {
            MockQuery::new()
        }

        fn equippable(
            &self,
            _part_id: u32,
            _collection_id: ActorId,
        ) -> impl Query<Output = Result<bool, Error>, Args = R::Args> {
            MockQuery::new()
        }
    }
}

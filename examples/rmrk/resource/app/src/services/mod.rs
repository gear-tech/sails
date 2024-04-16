use crate::catalogs::traits::RmrkCatalog;
use errors::{Error, Result};
use resources::{ComposedResource, PartId, Resource, ResourceId};
use sails_macros::gservice;
use sails_rtl::calls::Call;
use sails_rtl::gstd::calls::{Args, Remoting};
use sails_rtl::{collections::HashMap, gstd::events::EventTrigger, ActorId, *};

pub mod errors;
pub mod resources;

static mut RESOURCE_STORAGE_DATA: Option<ResourceStorageData> = None;

static mut RESOURCE_STORAGE_ADMIN: Option<ActorId> = None;

#[derive(Default)]
struct ResourceStorageData {
    resources: HashMap<ResourceId, Resource>,
}

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

pub struct ResourceStorage<TExecContext, TCatalogClient, TEventTrigger> {
    exec_context: TExecContext,
    catalog_client: TCatalogClient,
    event_trigger: TEventTrigger,
}

#[gservice]
impl<TExecContext, TCatalogClient, TEventTrigger>
    ResourceStorage<TExecContext, TCatalogClient, TEventTrigger>
where
    TExecContext: ExecContext,
    TCatalogClient: RmrkCatalog<Remoting, Args>,
    TEventTrigger: EventTrigger<ResourceStorageEvent>,
{
    pub fn seed(exec_context: TExecContext) {
        unsafe {
            RESOURCE_STORAGE_DATA = Some(ResourceStorageData::default());
            RESOURCE_STORAGE_ADMIN = Some(*exec_context.actor_id());
        }
    }

    pub fn new(
        exec_context: TExecContext,
        catalog_client: TCatalogClient,
        event_trigger: TEventTrigger,
    ) -> Self {
        Self {
            exec_context,
            catalog_client,
            event_trigger,
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

        self.event_trigger
            .trigger(ResourceStorageEvent::ResourceAdded { resource_id })
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
            let part_call = self
                .catalog_client
                .part(part_id)
                .publish(*base)
                .await
                .unwrap();
            let part_reply = part_call.reply().await.unwrap();
            if part_reply.is_none() {
                return Err(Error::PartNotFound);
            }
            parts.push(part_id);
        } else {
            return Err(Error::WrongResourceType);
        }

        self.event_trigger
            .trigger(ResourceStorageEvent::PartAdded {
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

fn resource_storage_admin() -> &'static ActorId {
    unsafe { RESOURCE_STORAGE_ADMIN.as_ref().unwrap() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalogs::{Error, Part};
    use resources::BasicResource;
    use sails_rtl::{
        calls::{Remoting, RemotingAction},
        collections::BTreeMap,
        gstd::events::mocks::MockEventTrigger,
        ActorId,
    };

    type MockResourceStorageEventTrigger = MockEventTrigger<ResourceStorageEvent>;

    #[test]
    fn test_add_resource_entry() {
        ResourceStorage::<_, MockCatalogClient, MockResourceStorageEventTrigger>::seed(
            ExecContextMock { actor_id: 1.into() },
        );
        let mut resource_storage = ResourceStorage::new(
            ExecContextMock { actor_id: 1.into() },
            MockCatalogClient,
            MockResourceStorageEventTrigger::new(),
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
    }

    impl ExecContext for ExecContextMock {
        fn actor_id(&self) -> &ActorId {
            &self.actor_id
        }
    }

    struct MockCatalogClient;

    impl<R, A> RmrkCatalog<R, A> for MockCatalogClient
    where
        R: Remoting<A>,
        A: Default,
    {
        fn add_parts(
            &mut self,
            _parts: BTreeMap<u32, Part>,
        ) -> RemotingAction<R, A, Result<BTreeMap<u32, Part>, Error>> {
            unimplemented!()
        }

        fn remove_parts(
            &mut self,
            _part_ids: Vec<u32>,
        ) -> RemotingAction<R, A, Result<Vec<u32>, Error>> {
            unimplemented!()
        }

        fn add_equippables(
            &mut self,
            _part_id: u32,
            _collection_ids: Vec<ActorId>,
        ) -> RemotingAction<R, A, Result<(u32, Vec<ActorId>), Error>> {
            unimplemented!()
        }

        fn remove_equippable(
            &mut self,
            _part_id: u32,
            _collection_id: ActorId,
        ) -> RemotingAction<R, A, Result<(u32, ActorId), Error>> {
            unimplemented!()
        }

        fn reset_equippables(&mut self, _part_id: u32) -> RemotingAction<R, A, Result<(), Error>> {
            unimplemented!()
        }

        fn set_equippables_to_all(
            &mut self,
            _part_id: u32,
        ) -> RemotingAction<R, A, Result<(), Error>> {
            unimplemented!()
        }

        fn part(&self, _part_id: u32) -> RemotingAction<R, A, Option<Part>> {
            unimplemented!()
        }

        fn equippable(
            &self,
            _part_id: u32,
            _collection_id: ActorId,
        ) -> RemotingAction<R, A, Result<bool, Error>> {
            unimplemented!()
        }
    }
}

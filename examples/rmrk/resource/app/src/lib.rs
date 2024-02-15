#![no_std]

pub use catalogs::Client as CatalogClientImpl;
use catalogs::Service as CatalogClient;
use errors::{Error, Result};
use gstd::ActorId as GStdActorId;
use resources::{ComposedResource, PartId, Resource, ResourceId};
use sails_macros::gservice;
use sails_rtl::{collections::HashMap, *};

mod catalogs;
pub mod errors;
pub mod resources;

static mut RESOURCE_STORAGE_DATA: Option<ResourceStorageData> = None;

static mut RESOURCE_STORAGE_ADMIN: Option<ActorId> = None;

#[derive(Default)]
struct ResourceStorageData {
    resources: HashMap<ResourceId, Resource>,
}

pub struct ResourceStorage<TExecContext, TCatalogClient> {
    exec_context: TExecContext,
    catalog_client: TCatalogClient,
}

#[gservice]
impl<TExecContext, TCatalogClient> ResourceStorage<TExecContext, TCatalogClient>
where
    TExecContext: ExecContext,
    TCatalogClient: CatalogClient,
{
    pub fn new(exec_context: TExecContext, catalog_client: TCatalogClient) -> Self {
        unsafe {
            RESOURCE_STORAGE_DATA.get_or_insert_with(Default::default);
            RESOURCE_STORAGE_ADMIN.get_or_insert_with(|| *exec_context.actor_id());
        }
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

        if resource_storage_data_mut()
            .resources
            .insert(resource_id, resource.clone())
            .is_some()
        {
            return Err(Error::ResourceAlreadyExists);
        }

        Ok((resource_id, resource))
    }

    pub async fn add_part_to_resource(
        &mut self,
        resource_id: ResourceId,
        part_id: PartId,
    ) -> Result<PartId> {
        self.require_admin()?;

        let resource = resource_storage_data_mut()
            .resources
            .get_mut(&resource_id)
            .ok_or(Error::ResourceNotFound)?;

        if let Resource::Composed(ComposedResource { base, parts, .. }) = resource {
            let part_call = self
                .catalog_client
                .part(part_id)
                .send(GStdActorId::from_slice(base.as_ref()).unwrap())
                .await
                .unwrap();
            let part_response = part_call.response().await.unwrap();
            if part_response.is_none() {
                return Err(Error::PartNotFound);
            }
            parts.push(part_id);
        } else {
            return Err(Error::WrongResourceType);
        }

        Ok(part_id)
    }

    pub fn resource(&self, resource_id: ResourceId) -> Result<Resource> {
        resource_storage_data_mut()
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
}

fn resource_storage_data_mut() -> &'static mut ResourceStorageData {
    unsafe { RESOURCE_STORAGE_DATA.as_mut().unwrap() }
}

fn resource_storage_admin() -> &'static ActorId {
    unsafe { RESOURCE_STORAGE_ADMIN.as_ref().unwrap() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catalogs::{ActorId as CatalogActorId, Error, Part};
    use resources::BasicResource;
    use sails_rtl::{collections::BTreeMap, Result as RtlResult};
    use sails_sender::Call;

    #[test]
    fn test_add_resource_entry() {
        let mut resource_storage =
            ResourceStorage::new(ExecContextMock { actor_id: 1.into() }, CatalogClientMock);
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

    struct CatalogClientMock;

    impl CatalogClient for CatalogClientMock {
        fn add_parts(
            &mut self,
            _parts: BTreeMap<u32, Part>,
        ) -> Call<RtlResult<BTreeMap<u32, Part>, Error>> {
            unimplemented!()
        }

        fn remove_parts(&mut self, _part_ids: Vec<u32>) -> Call<RtlResult<Vec<u32>, Error>> {
            unimplemented!()
        }

        fn add_equippables(
            &mut self,
            _part_id: u32,
            _collection_ids: Vec<CatalogActorId>,
        ) -> Call<RtlResult<(u32, Vec<CatalogActorId>), Error>> {
            unimplemented!()
        }

        fn remove_equippable(
            &mut self,
            _part_id: u32,
            _collection_id: CatalogActorId,
        ) -> Call<RtlResult<(u32, CatalogActorId), Error>> {
            unimplemented!()
        }

        fn reset_equippables(&mut self, _part_id: u32) -> Call<RtlResult<(), Error>> {
            unimplemented!()
        }

        fn set_equippables_to_all(&mut self, _part_id: u32) -> Call<RtlResult<(), Error>> {
            unimplemented!()
        }

        fn part(&self, _part_id: u32) -> Call<Option<Part>> {
            unimplemented!()
        }

        fn equippable(
            &self,
            _part_id: u32,
            _collection_id: CatalogActorId,
        ) -> Call<RtlResult<bool, Error>> {
            unimplemented!()
        }
    }
}

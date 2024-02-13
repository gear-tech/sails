#![no_std]

pub use catalogs::Client as CatalogClientImpl;
use catalogs::Service as CatalogClient;
use errors::{Error, Result};
use gstd::{collections::HashMap, prelude::*, ActorId};
use resources::{ComposedResource, PartId, Resource, ResourceId};
use sails_exec_context_abstractions::ExecContext;
use sails_macros::gservice;

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
    TExecContext: ExecContext<ActorId = ActorId>,
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
            let part_call = self.catalog_client.part(part_id).send(*base).await.unwrap();
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

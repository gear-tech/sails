use crate::resource_client::traits::RmrkResource;
use rmrk_catalog::services::parts::{FixedPart, Part};
use rmrk_resource_app::services::{
    ResourceStorageEvent,
    errors::{Error as ResourceStorageError, Result as ResourceStorageResult},
    resources::{ComposedResource, PartId, Resource, ResourceId},
};
use sails_rs::{
    ActorId, Decode, Encode,
    calls::{Call, Query, Remoting},
    collections::BTreeMap,
    errors::Result,
    gtest::{
        BlockRunResult, Program, System,
        calls::{GTestArgs, GTestRemoting, WithArgs as _},
    },
};

mod resource_client;
type RmrkResourceClient = crate::resource_client::RmrkResource<GTestRemoting>;

const CATALOG_PROGRAM_WASM_PATH: &str = "../../../../target/wasm32-gear/debug/rmrk_catalog.wasm";
const RESOURCE_PROGRAM_WASM_PATH: &str = "../../../../target/wasm32-gear/debug/rmrk_resource.wasm";

const ADMIN_ID: u64 = 10;
const NON_ADMIN_ID: u64 = 11;

mod resources {
    pub const CTOR_FUNC_NAME: &str = "New";
    pub const RESOURCE_SERVICE_NAME: &str = "RmrkResource";
    pub const ADD_RESOURCE_ENTRY_FUNC_NAME: &str = "AddResourceEntry";
    pub const ADD_PART_TO_RESOURCE_FUNC_NAME: &str = "AddPartToResource";
    pub const RESOURCE_FUNC_NAME: &str = "Resource";
}

mod catalog {
    pub const CTOR_FUNC_NAME: &str = "New";
    pub const ADD_PARTS_FUNC_NAME: &str = "AddParts";
    pub const CATALOG_SERVICE_NAME: &str = "RmrkCatalog";
}

const RESOURCE_ID: ResourceId = 42;
const PART_ID: PartId = 15;

#[test]
fn adding_resource_to_storage_by_admin_succeeds() {
    // Arrange
    let fixture = SystemFixture::new();

    // Act
    let resource = Resource::Composed(ComposedResource {
        src: "<src_uri>".into(),
        thumb: "<thumb_uri>".into(),
        metadata_uri: "<metadata_uri>".into(),
        base: fixture.catalog_program_id,
        parts: vec![1, 2, 3],
    });
    let run_result = fixture.add_resource(ADMIN_ID, RESOURCE_ID, &resource);

    // Assert
    let expected_response = [
        resources::RESOURCE_SERVICE_NAME.encode(),
        resources::ADD_RESOURCE_ENTRY_FUNC_NAME.encode(),
        (Ok((RESOURCE_ID, &resource)) as ResourceStorageResult<(u8, &Resource)>).encode(),
    ]
    .concat();
    assert!(run_result.contains(&(ADMIN_ID, expected_response)));

    let expected_event = [
        resources::RESOURCE_SERVICE_NAME.encode().as_slice(),
        "ResourceAdded".encode().as_slice(),
        &ResourceStorageEvent::ResourceAdded {
            resource_id: RESOURCE_ID,
        }
        .encode()
        .as_slice()[1..],
    ]
    .concat();
    assert!(run_result.contains(&(0, expected_event)));

    assert_eq!(
        resource.encode(),
        fixture
            .get_resource(NON_ADMIN_ID, RESOURCE_ID)
            .unwrap()
            .unwrap()
            .encode()
    );
}

#[tokio::test]
async fn adding_resource_to_storage_by_admin_succeeds_async() {
    // Arrange
    let fixture = Fixture::new();

    // Act
    let resource = Resource::Composed(ComposedResource {
        src: "<src_uri>".into(),
        thumb: "<thumb_uri>".into(),
        metadata_uri: "<metadata_uri>".into(),
        base: fixture.catalog_program_id,
        parts: vec![1, 2, 3],
    });
    let reply = fixture
        .add_resource_async(ADMIN_ID, RESOURCE_ID, &resource)
        .await;

    // Assert
    assert!(reply.is_ok());
    let reply = reply.unwrap();

    let expected_reply = [
        resources::RESOURCE_SERVICE_NAME.encode(),
        resources::ADD_RESOURCE_ENTRY_FUNC_NAME.encode(),
        (Ok((RESOURCE_ID, &resource)) as ResourceStorageResult<(u8, &Resource)>).encode(),
    ]
    .concat();
    assert_eq!(expected_reply, reply);
}

#[tokio::test]
async fn adding_resource_to_storage_by_admin_via_client_succeeds() {
    // Arrange
    let fixture = Fixture::new();

    // Act
    let resource = resource_client::Resource::Composed(resource_client::ComposedResource {
        src: "<src_uri>".into(),
        thumb: "<thumb_uri>".into(),
        metadata_uri: "<metadata_uri>".into(),
        base: fixture.catalog_program_id,
        parts: vec![1, 2, 3],
    });
    let add_reply = fixture
        .add_resource_via_client(ADMIN_ID, RESOURCE_ID, resource)
        .await
        .unwrap();

    // Assert
    assert_eq!(Ok(RESOURCE_ID), add_reply.map(|r| r.0));
}

#[test]
fn adding_existing_part_to_resource_by_admin_succeeds() {
    // Arrange
    let fixture = SystemFixture::new();

    let mut parts = BTreeMap::new();
    parts.insert(
        PART_ID,
        Part::Fixed(FixedPart {
            z: Some(1),
            metadata_uri: "<metadata_uri>".into(),
        }),
    );
    fixture.add_parts(ADMIN_ID, &parts);

    fixture.add_resource(
        ADMIN_ID,
        RESOURCE_ID,
        &Resource::Composed(ComposedResource {
            src: "<src_uri>".into(),
            thumb: "<thumb_uri>".into(),
            metadata_uri: "<metadata_uri>".into(),
            base: fixture.catalog_program_id,
            parts: vec![1, 2, 3],
        }),
    );

    // Act
    let run_result = fixture.add_part_to_resource(ADMIN_ID, RESOURCE_ID, PART_ID);

    // Assert
    let expected_response = [
        resources::RESOURCE_SERVICE_NAME.encode(),
        resources::ADD_PART_TO_RESOURCE_FUNC_NAME.encode(),
        (Ok(PART_ID) as ResourceStorageResult<PartId>).encode(),
    ]
    .concat();
    assert!(run_result.contains(&(ADMIN_ID, expected_response)));

    let resource = fixture
        .get_resource(NON_ADMIN_ID, RESOURCE_ID)
        .unwrap()
        .unwrap();
    if let Resource::Composed(ComposedResource { parts, .. }) = resource {
        assert_eq!(vec![1, 2, 3, PART_ID], parts);
    } else {
        panic!("Resource is not composed");
    }
}

#[tokio::test]
async fn adding_existing_part_to_resource_by_admin_via_client_succeeds() {
    // Arrange
    let fixture = Fixture::new();
    let resource = resource_client::Resource::Composed(resource_client::ComposedResource {
        src: "<src_uri>".into(),
        thumb: "<thumb_uri>".into(),
        metadata_uri: "<metadata_uri>".into(),
        base: fixture.catalog_program_id,
        parts: vec![1, 2, 3],
    });
    let _ = fixture
        .add_resource_via_client(ADMIN_ID, RESOURCE_ID, resource)
        .await
        .unwrap()
        .unwrap();

    // Act
    let add_part_reply = fixture
        .add_part_to_resource_via_client(ADMIN_ID, RESOURCE_ID, PART_ID)
        .await
        .unwrap();

    // Assert
    assert_eq!(Ok(PART_ID), add_part_reply);

    let resource_reply = fixture
        .get_resource_via_client(ADMIN_ID, RESOURCE_ID)
        .await
        .unwrap()
        .unwrap();

    if let resource_client::Resource::Composed(resource_client::ComposedResource {
        parts, ..
    }) = resource_reply
    {
        assert_eq!(vec![1, 2, 3, PART_ID], parts);
    } else {
        panic!("Resource is not composed");
    }
}

#[test]
fn adding_non_existing_part_to_resource_fails() {
    // Arrange
    let fixture = SystemFixture::new();

    fixture.add_resource(
        ADMIN_ID,
        RESOURCE_ID,
        &Resource::Composed(ComposedResource {
            src: "<src_uri>".into(),
            thumb: "<thumb_uri>".into(),
            metadata_uri: "<metadata_uri>".into(),
            base: fixture.catalog_program_id,
            parts: vec![1, 2, 3],
        }),
    );

    // Act
    let run_result = fixture.add_part_to_resource(ADMIN_ID, RESOURCE_ID, PART_ID);

    // Assert
    let expected_response = [
        resources::RESOURCE_SERVICE_NAME.encode(),
        resources::ADD_PART_TO_RESOURCE_FUNC_NAME.encode(),
        (Err(ResourceStorageError::PartNotFound) as ResourceStorageResult<PartId>).encode(),
    ]
    .concat();
    assert!(run_result.contains(&(ADMIN_ID, expected_response)));
}

struct SystemFixture {
    system: System,
    catalog_program_id: ActorId,
    resource_program_id: ActorId,
}

impl SystemFixture {
    fn new() -> Self {
        let system = System::new();
        system.init_logger_with_default_filter("gwasm=debug,gtest=info,sails_rs=debug");
        system.mint_to(ADMIN_ID, 1_000_000_000_000_000);
        system.mint_to(NON_ADMIN_ID, 1_000_000_000_000_000);

        let catalog_program_id = Self::create_catalog_program(&system);
        let resource_program_id = Self::create_resource_program(&system);

        Self {
            system,
            catalog_program_id,
            resource_program_id,
        }
    }

    fn create_catalog_program(system: &System) -> ActorId {
        let catalog_program = Program::from_file(system, CATALOG_PROGRAM_WASM_PATH);
        catalog_program.send_bytes(ADMIN_ID, catalog::CTOR_FUNC_NAME.encode());
        catalog_program.id()
    }

    fn create_resource_program(system: &System) -> ActorId {
        let resource_program = Program::from_file(system, RESOURCE_PROGRAM_WASM_PATH);
        resource_program.send_bytes(ADMIN_ID, resources::CTOR_FUNC_NAME.encode());
        resource_program.id()
    }

    fn catalog_program(&self) -> Program {
        self.system.get_program(self.catalog_program_id).unwrap()
    }

    fn resource_program(&self) -> Program {
        self.system.get_program(self.resource_program_id).unwrap()
    }

    fn add_resource(
        &self,
        actor_id: u64,
        resource_id: ResourceId,
        resource: &Resource,
    ) -> BlockRunResult {
        let program = self.resource_program();
        let encoded_request = [
            resources::RESOURCE_SERVICE_NAME.encode(),
            resources::ADD_RESOURCE_ENTRY_FUNC_NAME.encode(),
            resource_id.encode(),
            resource.encode(),
        ]
        .concat();
        let _message_id = program.send_bytes(actor_id, encoded_request);
        self.system.run_next_block()
    }

    fn add_part_to_resource(
        &self,
        actor_id: u64,
        resource_id: ResourceId,
        part_id: PartId,
    ) -> BlockRunResult {
        let program = self.resource_program();
        let encoded_request = [
            resources::RESOURCE_SERVICE_NAME.encode(),
            resources::ADD_PART_TO_RESOURCE_FUNC_NAME.encode(),
            resource_id.encode(),
            part_id.encode(),
        ]
        .concat();
        let _message_id = program.send_bytes(actor_id, encoded_request);
        self.system.run_next_block()
    }

    fn get_resource(
        &self,
        actor_id: u64,
        resource_id: ResourceId,
    ) -> Option<ResourceStorageResult<Resource>> {
        let program = self.resource_program();
        let encoded_service_name = resources::RESOURCE_SERVICE_NAME.encode();
        let encoded_func_name = resources::RESOURCE_FUNC_NAME.encode();
        let encoded_request = [
            encoded_service_name.clone(),
            encoded_func_name.clone(),
            resource_id.encode(),
        ]
        .concat();
        let _message_id = program.send_bytes(actor_id, encoded_request);
        let run_result = self.system.run_next_block();
        run_result
            .log()
            .iter()
            .find(|l| {
                l.destination() == actor_id.into()
                    && l.source() == program.id()
                    && l.payload().starts_with(&encoded_service_name)
                    && l.payload()[encoded_service_name.len()..].starts_with(&encoded_func_name)
            })
            .map(|l| {
                let mut p = &l.payload()[encoded_service_name.len() + encoded_func_name.len()..];
                ResourceStorageResult::<Resource>::decode(&mut p).unwrap()
            })
    }

    fn add_parts(&self, actor_id: u64, parts: &BTreeMap<PartId, Part>) -> BlockRunResult {
        let program = self.catalog_program();
        let encoded_request = [
            catalog::CATALOG_SERVICE_NAME.encode(),
            catalog::ADD_PARTS_FUNC_NAME.encode(),
            parts.encode(),
        ]
        .concat();
        let _message_id = program.send_bytes(actor_id, encoded_request);
        self.system.run_next_block()
    }
}

struct Fixture {
    program_space: GTestRemoting,
    catalog_program_id: ActorId,
    resource_program_id: ActorId,
}

impl Fixture {
    fn new() -> Self {
        let system = System::new();
        system.init_logger();
        system.mint_to(ADMIN_ID, 1_000_000_000_000_000);
        system.mint_to(NON_ADMIN_ID, 1_000_000_000_000_000);

        let catalog_program_id = Self::create_catalog_program(&system);
        let resource_program_id = Self::create_resource_program(&system);

        let program_space = GTestRemoting::new(system, ADMIN_ID.into());

        Self {
            program_space,
            catalog_program_id,
            resource_program_id,
        }
    }

    fn create_catalog_program(system: &System) -> ActorId {
        let catalog_program = Program::from_file(system, CATALOG_PROGRAM_WASM_PATH);
        catalog_program.send_bytes(ADMIN_ID, catalog::CTOR_FUNC_NAME.encode());

        let mut parts = BTreeMap::new();
        parts.insert(
            PART_ID,
            Part::Fixed(FixedPart {
                z: Some(1),
                metadata_uri: "<metadata_uri>".into(),
            }),
        );
        let encoded_request = [
            catalog::CATALOG_SERVICE_NAME.encode(),
            catalog::ADD_PARTS_FUNC_NAME.encode(),
            parts.encode(),
        ]
        .concat();
        let _message_id = catalog_program.send_bytes(ADMIN_ID, encoded_request);
        system.run_next_block();
        catalog_program.id()
    }

    fn create_resource_program(system: &System) -> ActorId {
        let resource_program = Program::from_file(system, RESOURCE_PROGRAM_WASM_PATH);
        resource_program.send_bytes(ADMIN_ID, resources::CTOR_FUNC_NAME.encode());
        resource_program.id()
    }

    fn program_space(&self) -> &GTestRemoting {
        &self.program_space
    }

    fn resource_client(&self) -> RmrkResourceClient {
        RmrkResourceClient::new(self.program_space.clone())
    }

    async fn add_resource_async(
        &self,
        actor_id: u64,
        resource_id: ResourceId,
        resource: &Resource,
    ) -> Result<Vec<u8>> {
        let encoded_request = [
            resources::RESOURCE_SERVICE_NAME.encode(),
            resources::ADD_RESOURCE_ENTRY_FUNC_NAME.encode(),
            resource_id.encode(),
            resource.encode(),
        ]
        .concat();
        let program_space = self.program_space().clone();
        let reply = program_space
            .message(
                self.resource_program_id,
                encoded_request,
                None,
                0,
                GTestArgs::new(actor_id.into()),
            )
            .await?;
        reply.await
    }

    async fn add_resource_via_client(
        &self,
        actor_id: u64,
        resource_id: u8,
        resource: resource_client::Resource,
    ) -> Result<Result<(u8, resource_client::Resource), resource_client::Error>> {
        let mut resource_client = self.resource_client();
        resource_client
            .add_resource_entry(resource_id, resource)
            .with_actor_id(actor_id.into())
            .send_recv(self.resource_program_id)
            .await
    }

    async fn add_part_to_resource_via_client(
        &self,
        actor_id: u64,
        resource_id: u8,
        part_id: u32,
    ) -> Result<Result<u32, resource_client::Error>> {
        let mut resource_client = self.resource_client();
        resource_client
            .add_part_to_resource(resource_id, part_id)
            .with_actor_id(actor_id.into())
            .send_recv(self.resource_program_id)
            .await
    }

    async fn get_resource_via_client(
        &self,
        actor_id: u64,
        resource_id: u8,
    ) -> Result<Result<resource_client::Resource, resource_client::Error>> {
        let resource_client = self.resource_client();
        resource_client
            .resource(resource_id)
            .with_actor_id(actor_id.into())
            .recv(self.resource_program_id)
            .await
    }
}

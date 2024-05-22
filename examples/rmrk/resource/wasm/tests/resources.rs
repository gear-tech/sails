use crate::rmrk_resource_client::traits::RmrkResource;
use core::cell::OnceCell;
use gtest::{Program, RunResult};
use rmrk_catalog::services::parts::{FixedPart, Part};
use rmrk_resource_app::services::{
    errors::{Error as ResourceStorageError, Result as ResourceStorageResult},
    resources::{ComposedResource, PartId, Resource, ResourceId},
    ResourceStorageEvent,
};
use sails_rtl::{
    calls::{Call, Remoting},
    collections::BTreeMap,
    errors::Result,
    gtest::calls::{GTestArgs, GTestRemoting},
    ActorId, Decode, Encode,
};

mod rmrk_resource_client;

const CATALOG_PROGRAM_WASM_PATH: &str =
    "../../../../target/wasm32-unknown-unknown/debug/rmrk_catalog.wasm";
const RESOURCE_PROGRAM_WASM_PATH: &str =
    "../../../../target/wasm32-unknown-unknown/debug/rmrk_resource.wasm";

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
    let fixture = Fixture::new(ADMIN_ID);

    // Act
    let resource = Resource::Composed(ComposedResource {
        src: "<src_uri>".into(),
        thumb: "<thumb_uri>".into(),
        metadata_uri: "<metadata_uri>".into(),
        base: ActorId::from(fixture.catalog_program().id().into_bytes()),
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
    let fixture = Fixture::new(ADMIN_ID);

    // Act
    let resource = Resource::Composed(ComposedResource {
        src: "<src_uri>".into(),
        thumb: "<thumb_uri>".into(),
        metadata_uri: "<metadata_uri>".into(),
        base: ActorId::from(fixture.catalog_program().id().into_bytes()),
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
async fn adding_resource_to_storage_by_client_succeeds() {
    // Arrange
    let fixture = Fixture::new(ADMIN_ID);

    let mut parts = BTreeMap::new();
    parts.insert(
        PART_ID,
        Part::Fixed(FixedPart {
            z: Some(1),
            metadata_uri: "<metadata_uri>".into(),
        }),
    );
    fixture.add_parts(ADMIN_ID, &parts);

    // Act
    let base_id = ActorId::from(fixture.catalog_program().id().into_bytes());
    let actor_id = ActorId::from(
        fixture
            .resource_program_for_async()
            .id()
            .clone()
            .into_bytes(),
    );
    let resource =
        rmrk_resource_client::Resource::Composed(rmrk_resource_client::ComposedResource {
            src: "<src_uri>".into(),
            thumb: "<thumb_uri>".into(),
            metadata_uri: "<metadata_uri>".into(),
            base: base_id,
            parts: vec![1, 2, 3],
        });

    let mut resource_client1 = fixture.resource_client.clone();
    let add_call = resource_client1
        .add_resource_entry(RESOURCE_ID, resource)
        .publish(actor_id.clone())
        .await
        .unwrap();
    let add_reply = add_call.reply().await.unwrap().unwrap();

    // Assert
    assert_eq!(RESOURCE_ID, add_reply.0);

    // Act
    let mut resource_client2 = fixture.resource_client.clone();
    let add_part_call = resource_client2
        .add_part_to_resource(RESOURCE_ID, PART_ID)
        .publish(actor_id.clone())
        .await
        .unwrap();
    let add_part_reply = add_part_call.reply().await.unwrap().unwrap();

    // Assert
    assert_eq!(PART_ID, add_part_reply);

    // Act
    let resource_client3 = fixture.resource_client.clone();
    let resource_call = resource_client3
        .resource(RESOURCE_ID)
        .publish(actor_id.clone())
        .await
        .unwrap();
    let resource_reply = resource_call.reply().await.unwrap().unwrap();

    // Assert
    if let rmrk_resource_client::Resource::Composed(rmrk_resource_client::ComposedResource {
        parts,
        ..
    }) = resource_reply
    {
        assert_eq!(vec![1, 2, 3, PART_ID], parts);
    } else {
        panic!("Resource is not composed");
    }
}

#[test]
fn adding_existing_part_to_resource_by_admin_succeeds() {
    // Arrange
    let fixture = Fixture::new(ADMIN_ID);

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
            base: ActorId::from(fixture.catalog_program().id().into_bytes()),
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

#[test]
fn adding_non_existing_part_to_resource_fails() {
    // Arrange
    let fixture = Fixture::new(ADMIN_ID);

    fixture.add_resource(
        ADMIN_ID,
        RESOURCE_ID,
        &Resource::Composed(ComposedResource {
            src: "<src_uri>".into(),
            thumb: "<thumb_uri>".into(),
            metadata_uri: "<metadata_uri>".into(),
            base: ActorId::from(fixture.catalog_program().id().into_bytes()),
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

struct Fixture<'a> {
    admin_id: u64,
    program_space: GTestRemoting,
    catalog_program: OnceCell<Program<'a>>,
    resource_program: OnceCell<Program<'a>>,
    resource_client: crate::rmrk_resource_client::RmrkResource<GTestRemoting, GTestArgs>,
}

impl<'a> Fixture<'a> {
    fn new(admin_id: u64) -> Self {
        let program_space = GTestRemoting::new();
        program_space.system().init_logger();
        let resource_client = crate::rmrk_resource_client::RmrkResource::new(program_space.clone());

        Self {
            admin_id,
            program_space,
            catalog_program: OnceCell::new(),
            resource_program: OnceCell::new(),
            resource_client,
        }
    }

    fn program_space(&self) -> &GTestRemoting {
        &self.program_space
    }

    fn catalog_program(&'a self) -> &Program<'a> {
        self.catalog_program.get_or_init(|| {
            let program =
                Program::from_file(self.program_space.system(), CATALOG_PROGRAM_WASM_PATH);
            let encoded_request = catalog::CTOR_FUNC_NAME.encode();
            program.send_bytes(self.admin_id, encoded_request);
            program
        })
    }

    fn resource_program_for_async(&'a self) -> &Program<'a> {
        self.resource_program.get_or_init(|| {
            let program =
                Program::from_file(self.program_space.system(), RESOURCE_PROGRAM_WASM_PATH);
            let encoded_request = resources::CTOR_FUNC_NAME.encode();
            program.send_bytes(self.admin_id, encoded_request);
            program
        })
    }

    fn resource_program_for_sync(&'a self) -> &Program<'a> {
        self.resource_program.get_or_init(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    self.__spin_up_program(
                        RESOURCE_PROGRAM_WASM_PATH,
                        &resources::CTOR_FUNC_NAME.encode(),
                    )
                    .await
                })
        })
    }

    async fn __spin_up_program(&'a self, program_path: &str, payload: &[u8]) -> Program<'a> {
        let code_id = self.program_space().system().submit_code_file(program_path);
        let program_space = self.program_space().clone();
        let reply = program_space
            .activate(
                code_id.as_ref().into(),
                "123",
                payload,
                0,
                GTestArgs::new(self.admin_id.into()),
            )
            .await
            .unwrap();
        self.program_space()
            .system()
            .get_program(*reply.await.unwrap().0.as_ref())
            .unwrap()
    }

    fn add_resource(
        &'a self,
        actor_id: u64,
        resource_id: ResourceId,
        resource: &Resource,
    ) -> RunResult {
        let program = self.resource_program_for_sync();
        let encoded_request = [
            resources::RESOURCE_SERVICE_NAME.encode(),
            resources::ADD_RESOURCE_ENTRY_FUNC_NAME.encode(),
            resource_id.encode(),
            resource.encode(),
        ]
        .concat();
        program.send_bytes(actor_id, encoded_request)
    }

    async fn add_resource_async(
        &'a self,
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
                self.resource_program_for_async().id().as_ref().into(),
                encoded_request,
                0,
                GTestArgs::new(actor_id.into()),
            )
            .await?;
        reply.await
    }

    fn add_part_to_resource(
        &'a self,
        actor_id: u64,
        resource_id: ResourceId,
        part_id: PartId,
    ) -> RunResult {
        let program = self.resource_program_for_sync();
        let encoded_request = [
            resources::RESOURCE_SERVICE_NAME.encode(),
            resources::ADD_PART_TO_RESOURCE_FUNC_NAME.encode(),
            resource_id.encode(),
            part_id.encode(),
        ]
        .concat();
        program.send_bytes(actor_id, encoded_request)
    }

    fn get_resource(
        &'a self,
        actor_id: u64,
        resource_id: ResourceId,
    ) -> Option<ResourceStorageResult<Resource>> {
        let program = self.resource_program_for_sync();
        let encoded_service_name = resources::RESOURCE_SERVICE_NAME.encode();
        let encoded_func_name = resources::RESOURCE_FUNC_NAME.encode();
        let encoded_request = [
            encoded_service_name.clone(),
            encoded_func_name.clone(),
            resource_id.encode(),
        ]
        .concat();
        let run_result = program.send_bytes(actor_id, encoded_request);
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

    fn add_parts(&'a self, actor_id: u64, parts: &BTreeMap<PartId, Part>) -> RunResult {
        let program = self.catalog_program();
        let encoded_request = [
            catalog::CATALOG_SERVICE_NAME.encode(),
            catalog::ADD_PARTS_FUNC_NAME.encode(),
            parts.encode(),
        ]
        .concat();
        program.send_bytes(actor_id, encoded_request)
    }
}

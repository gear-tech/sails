use core::cell::OnceCell;
use gtest::{Program, RunResult, System};
use rmrk_catalog::services::parts::{FixedPart, Part};
use rmrk_resource_app::services::{
    errors::{Error as ResourceStorageError, Result as ResourceStorageResult},
    resources::{ComposedResource, PartId, Resource, ResourceId},
};
use sails_rtl::{collections::BTreeMap, ActorId, Decode, Encode};

const CATALOG_PROGRAM_WASM_PATH: &str =
    "../../../../target/wasm32-unknown-unknown/debug/rmrk_catalog.wasm";
const RESOURCE_PROGRAM_WASM_PATH: &str =
    "../../../../target/wasm32-unknown-unknown/debug/rmrk_resource.wasm";

const ADMIN_ID: u64 = 10;
const NON_ADMIN_ID: u64 = 11;

mod resources {
    pub const CTOR_FUNC_NAME: &str = "New";
    pub const ADD_RESOURCE_ENTRY_FUNC_NAME: &str = "AddResourceEntry";
    pub const ADD_PART_TO_RESOURCE_FUNC_NAME: &str = "AddPartToResource";
    pub const RESOURCE_FUNC_NAME: &str = "Resource";
}

mod catalog {
    pub const CTOR_FUNC_NAME: &str = "New";
    pub const ADD_PARTS_FUNC_NAME: &str = "AddParts";
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
        resources::ADD_RESOURCE_ENTRY_FUNC_NAME.encode(),
        (Ok((RESOURCE_ID, &resource)) as ResourceStorageResult<(u8, &Resource)>).encode(),
    ]
    .concat();
    assert!(run_result.contains(&(ADMIN_ID, expected_response)));

    assert_eq!(
        resource.encode(),
        fixture
            .get_resource(NON_ADMIN_ID, RESOURCE_ID)
            .unwrap()
            .unwrap()
            .encode()
    );
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
        resources::ADD_PART_TO_RESOURCE_FUNC_NAME.encode(),
        (Err(ResourceStorageError::PartNotFound) as ResourceStorageResult<PartId>).encode(),
    ]
    .concat();
    assert!(run_result.contains(&(ADMIN_ID, expected_response)));
}

struct Fixture<'a> {
    admin_id: u64,
    system: System,
    catalog_program: OnceCell<Program<'a>>,
    resource_program: OnceCell<Program<'a>>,
}

impl<'a> Fixture<'a> {
    fn new(admin_id: u64) -> Self {
        let system = System::new();
        system.init_logger();

        Self {
            admin_id,
            system,
            catalog_program: OnceCell::new(),
            resource_program: OnceCell::new(),
        }
    }

    fn catalog_program(&'a self) -> &Program<'a> {
        self.catalog_program.get_or_init(|| {
            let program = Program::from_file(&self.system, CATALOG_PROGRAM_WASM_PATH);
            let encoded_request = catalog::CTOR_FUNC_NAME.encode();
            program.send_bytes(self.admin_id, encoded_request);
            program
        })
    }

    fn resource_program(&'a self) -> &Program<'a> {
        self.resource_program.get_or_init(|| {
            let program = Program::from_file(&self.system, RESOURCE_PROGRAM_WASM_PATH);
            let encoded_request = resources::CTOR_FUNC_NAME.encode();
            program.send_bytes(self.admin_id, encoded_request);
            program
        })
    }

    fn add_resource(
        &'a self,
        actor_id: u64,
        resource_id: ResourceId,
        resource: &Resource,
    ) -> RunResult {
        let program = self.resource_program();
        let encoded_request = [
            resources::ADD_RESOURCE_ENTRY_FUNC_NAME.encode(),
            resource_id.encode(),
            resource.encode(),
        ]
        .concat();
        program.send_bytes(actor_id, encoded_request)
    }

    fn add_part_to_resource(
        &'a self,
        actor_id: u64,
        resource_id: ResourceId,
        part_id: PartId,
    ) -> RunResult {
        let program = self.resource_program();
        let encoded_request = [
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
        let program = self.resource_program();
        let encoded_func_name = resources::RESOURCE_FUNC_NAME.encode();
        let encoded_request = [encoded_func_name.clone(), resource_id.encode()].concat();
        let run_result = program.send_bytes(actor_id, encoded_request);
        run_result
            .log()
            .iter()
            .find(|l| {
                l.destination() == actor_id.into()
                    && l.source() == program.id()
                    && l.payload().starts_with(&encoded_func_name)
            })
            .map(|l| {
                let mut p = &l.payload()[encoded_func_name.len()..];
                ResourceStorageResult::<Resource>::decode(&mut p).unwrap()
            })
    }

    fn add_parts(&'a self, actor_id: u64, parts: &BTreeMap<PartId, Part>) {
        let program = self.catalog_program();
        let encoded_request = [catalog::ADD_PARTS_FUNC_NAME.encode(), parts.encode()].concat();
        program.send_bytes(actor_id, encoded_request);
    }
}

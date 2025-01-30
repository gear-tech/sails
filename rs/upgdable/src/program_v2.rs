use super::*;
use cell::{Ref, RefCell, RefMut};
use ops::Deref;
use sails_rs::gstd;

#[derive(Encode, Decode)]
#[codec(crate = sails_rs::scale_codec)]
struct ProgramStorageV2 {
    a_lo: u16,
    a_hi: u16,
    //d: RefCell<u32>,  Encode/Decode is not implemented for RefCell
    d: String,
}

static mut __PROGRAM_STORAGE: Option<ProgramStorageV2> = None;

struct ProgramWrapper {
    program: UpgradableProgramV2,
    storage: RefCell<ProgramStorageV2>,
}

impl Deref for ProgramWrapper {
    type Target = UpgradableProgramV2;

    fn deref(&self) -> &Self::Target {
        &self.program
    }
}

impl ProgramWrapper {
    pub fn storage(&self) -> Ref<'_, ProgramStorageV2> {
        self.storage.borrow()
    }

    pub fn storage_mut(&self) -> RefMut<'_, ProgramStorageV2> {
        self.storage.borrow_mut()
    }
}

static mut __PROGRAM_WRAPPER: Option<ProgramWrapper> = None;

struct UpgradableProgramV2(());

trait ProgramSelf {
    type Type;
}

impl ProgramSelf for UpgradableProgramV2 {
    type Type = ProgramWrapper;
}

#[allow(dead_code)]
//#[program(ProgramStorageV2)]
impl UpgradableProgramV2 {
    pub fn new() -> Self {
        Self(())
    }

    pub fn upgradable(&self) -> UpgradableService {
        let _storage = self.__storage();
        UpgradableService::new()
    }

    pub fn svc_ctor(
        //this: &impl IProgram<ProgramType = Self, StorageType = ProgramStorageV2>, // ref to concrete type for simplicity? type alias? macro? impl trait with assoc type and use this type
        this: &<Self as ProgramSelf>::Type,
        _p1: u32,
    ) -> UpgradableService {
        let _storage = this.storage();
        let _s_mut = this.storage_mut();
        this.upgradable()
    }
}

impl IUpgradableProgram for UpgradableProgramV2 {
    type PrevStorageType = ProgramStorageV1;
    type StorageType = ProgramStorageV2;

    fn migrate(&self, prev_storage: Self::PrevStorageType) -> Self::StorageType {
        ProgramStorageV2 {
            a_lo: prev_storage.a as u16,
            a_hi: (prev_storage.a >> 16) as u16,
            d: Default::default(),
        }
    }
}

impl UpgradableProgramV2 {
    // u32 is fine as we don't support 64-bit for wasm
    pub fn __read_storage(&self, offset: u32, size: u32) -> (Vec<u8>, bool) {
        let storage = self.__storage();
        let encoded_storage = storage.encode();
        let encoded_storage_len = encoded_storage
            .len()
            .try_into()
            .expect("64-bit are not supported");
        let end = cmp::min(offset.saturating_add(size), encoded_storage_len);
        let has_more = end < encoded_storage_len;
        (
            encoded_storage[offset as usize..end as usize].to_vec(),
            has_more,
        )
    }

    pub async fn __migrate_storage(&self, prev_version_id: ActorId) -> () {
        let prev_encoded_storage = read_storage_bytes(prev_version_id).await;
        let prev_storage = <ProgramStorageV1>::decode(&mut &prev_encoded_storage[..]).unwrap();
        let storage = self.migrate(prev_storage);
        unsafe { __PROGRAM_STORAGE = Some(storage) };
    }

    fn __handle() {
        let program_ref = unsafe { __PROGRAM_STORAGE.as_ref().unwrap() };
        let _v = program_ref.a_lo;
    }

    fn __storage(&self) -> &<Self as IUpgradableProgram>::StorageType {
        unsafe { __PROGRAM_STORAGE.as_ref().unwrap() }
    }
}

async fn read_storage_bytes(program_id: ActorId) -> Vec<u8> {
    let mut encoded_storage = Vec::<u8>::new();
    let mut encoded_storage_offset = 0u32;
    const CHUNK_SIZE: u32 = 1024; // What is the optimal value?
    loop {
        let read_storage_payload = vec![
            "__read_storage".encode(),
            encoded_storage_offset.encode(),
            CHUNK_SIZE.encode(),
        ]
        .concat();
        let response = gstd::msg::send_bytes_for_reply(program_id, read_storage_payload, 0, 0)
            .unwrap()
            .await
            .unwrap();
        let (chunk, has_more) = <(Vec<u8>, bool)>::decode(&mut &response[..]).unwrap();
        encoded_storage.extend(chunk);
        if !has_more {
            break;
        }
        encoded_storage_offset = encoded_storage_offset
            .checked_add(CHUNK_SIZE)
            .expect("storage can't be larger than 4GB");
    }
    encoded_storage
}

#![no_std]

use errors::Error;
use gstd::{
    collections::{BTreeMap, BTreeSet},
    prelude::*,
    ActorId,
};
use parts::{CollectionId, Part, PartId, SlotPart};
use sails_exec_context_abstractions::ExecContext;
use sails_macros::gservice;

mod errors;
mod parts;

static mut CATALOG_ADMIN: Option<ActorId> = None;

type Result<T> = gstd::Result<T, Error>;

type PartMap<K, V> = BTreeMap<K, V>;

#[derive(Default)]
pub struct CatalogData {
    parts: PartMap<PartId, Part>,
    is_equippable_to_all: BTreeSet<PartId>,
}

pub struct Catalog<'a, TExecContext: ExecContext> {
    data: &'a mut CatalogData,
    exec_context: TExecContext,
}

impl<'a, TExecContext: ExecContext> Catalog<'a, TExecContext>
where
    TExecContext: ExecContext<ActorId = ActorId>,
{
    pub fn new(data: &'a mut CatalogData, exec_context: TExecContext) -> Self {
        unsafe {
            CATALOG_ADMIN.get_or_insert_with(|| *exec_context.actor_id());
        }
        Self { data, exec_context }
    }
}

#[gservice]
impl<'a, TExecContext> Catalog<'a, TExecContext>
where
    TExecContext: ExecContext<ActorId = ActorId>,
{
    pub fn add_parts(&mut self, parts: PartMap<PartId, Part>) -> Result<PartMap<PartId, Part>> {
        self.require_admin()?;

        if parts.is_empty() {
            return Err(Error::ZeroLengthPassed);
        }

        for (part_id, part) in &parts {
            if *part_id == 0 {
                return Err(Error::PartIdCantBeZero);
            }
            if self.data.parts.contains_key(part_id) {
                return Err(Error::PartAlreadyExists);
            }
            self.data.parts.insert(*part_id, part.clone());
        }

        Ok(parts)
    }

    pub fn remove_parts(&mut self, part_ids: Vec<PartId>) -> Result<Vec<PartId>> {
        self.require_admin()?;

        if part_ids.is_empty() {
            return Err(Error::ZeroLengthPassed);
        }

        if !part_ids
            .iter()
            .all(|part_id| self.data.parts.contains_key(part_id))
        {
            return Err(Error::PartDoesNotExist);
        }

        for part_id in &part_ids {
            self.data.parts.remove(part_id);
        }

        Ok(part_ids)
    }

    pub fn part(&self, part_id: PartId) -> Option<Part> {
        self.data.parts.get(&part_id).cloned()
    }

    pub fn add_equippables(
        &mut self,
        part_id: PartId,
        collection_ids: Vec<CollectionId>,
    ) -> Result<(PartId, Vec<CollectionId>)> {
        self.require_admin()?;

        if collection_ids.is_empty() {
            return Err(Error::ZeroLengthPassed);
        }

        let part = self
            .data
            .parts
            .get_mut(&part_id)
            .ok_or(Error::PartDoesNotExist)?;

        let equippable = if let Part::Slot(SlotPart { equippable, .. }) = part {
            equippable
        } else {
            return Err(Error::WrongPartFormat);
        };

        for collection_id in &collection_ids {
            equippable.push(*collection_id);
        }

        Ok((part_id, collection_ids))
    }

    pub fn remove_equippable(
        &mut self,
        part_id: PartId,
        collection_id: CollectionId,
    ) -> Result<(PartId, CollectionId)> {
        self.require_admin()?;

        let part = self
            .data
            .parts
            .get_mut(&part_id)
            .ok_or(Error::PartDoesNotExist)?;

        if let Part::Slot(SlotPart { equippable, .. }) = part {
            equippable.retain(|x| x != &collection_id);
        } else {
            return Err(Error::WrongPartFormat);
        }

        Ok((part_id, collection_id))
    }

    pub fn reset_equippables(&mut self, part_id: PartId) -> Result<()> {
        self.require_admin()?;

        let part = self
            .data
            .parts
            .get_mut(&part_id)
            .ok_or(Error::PartDoesNotExist)?;

        let equippable = if let Part::Slot(SlotPart { equippable, .. }) = part {
            equippable
        } else {
            return Err(Error::WrongPartFormat);
        };

        *equippable = vec![];
        self.data.is_equippable_to_all.retain(|x| x != &part_id);

        Ok(())
    }

    pub fn set_equippables_to_all(&mut self, part_id: PartId) -> Result<()> {
        self.require_admin()?;

        let part = self
            .data
            .parts
            .get(&part_id)
            .ok_or(Error::PartDoesNotExist)?;

        if let Part::Fixed { .. } = part {
            return Err(Error::WrongPartFormat);
        }

        self.data.is_equippable_to_all.insert(part_id);

        Ok(())
    }

    pub fn equippable(&self, part_id: PartId, collection_id: CollectionId) -> Result<bool> {
        for equippable in &self.data.is_equippable_to_all {
            if equippable == &part_id {
                return Ok(true);
            }
        }

        let part = self
            .data
            .parts
            .get(&part_id)
            .ok_or(Error::PartDoesNotExist)?;

        if let Part::Slot(SlotPart { equippable, .. }) = part {
            if equippable.iter().any(|x| x == &collection_id) {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(Error::WrongPartFormat)
        }
    }

    fn require_admin(&self) -> Result<()> {
        if self.exec_context.actor_id() != catalog_admin() {
            return Err(Error::NotAllowedToCall);
        }

        Ok(())
    }
}

fn catalog_admin() -> &'static ActorId {
    unsafe { CATALOG_ADMIN.as_ref().unwrap() }
}

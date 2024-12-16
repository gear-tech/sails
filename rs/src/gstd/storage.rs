use crate::boxed::Box;
use core::{
    cell::RefMut,
    ops::{Deref, DerefMut},
};

pub type BoxedStorage<T> = Box<dyn Storage<Item = T>>;

pub trait Storage {
    type Item;

    fn get(&self) -> &Self::Item;

    fn get_mut(&mut self) -> &mut Self::Item;
}

pub trait StorageAccessor<T> {
    fn get() -> impl Storage<Item = T> + 'static;

    fn boxed() -> BoxedStorage<T> {
        Box::new(Self::get())
    }
}

impl<T> Storage for RefMut<'_, T> {
    type Item = T;

    fn get(&self) -> &Self::Item {
        self.deref()
    }

    fn get_mut(&mut self) -> &mut Self::Item {
        self.deref_mut()
    }
}

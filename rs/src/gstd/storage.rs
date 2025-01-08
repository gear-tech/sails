use crate::boxed::Box;
use core::{
    cell::{RefCell, RefMut, UnsafeCell},
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

pub type BoxedStorage<T> = Box<dyn Storage<Item = T>>;

pub trait Storage {
    type Item;

    fn get(&self) -> &Self::Item;

    fn get_mut(&mut self) -> &mut Self::Item;
}

pub trait StorageAccessor<'a, T> {
    fn get(&'a self) -> impl Storage<Item = T> + 'a;

    fn boxed(&'a self) -> Box<dyn Storage<Item = T> + 'a> {
        Box::new(self.get())
    }
}

impl<T> Storage for &mut T {
    type Item = T;

    fn get(&self) -> &Self::Item {
        self
    }

    fn get_mut(&mut self) -> &mut Self::Item {
        self
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

impl<T> Storage for NonNull<T> {
    type Item = T;

    fn get(&self) -> &Self::Item {
        unsafe { self.as_ref() }
    }

    fn get_mut(&mut self) -> &mut Self::Item {
        unsafe { self.as_mut() }
    }
}

impl<T> Storage for Option<T> {
    type Item = T;

    fn get(&self) -> &Self::Item {
        self.as_ref().expect("storage is not initialized")
    }

    fn get_mut(&mut self) -> &mut Self::Item {
        self.as_mut().expect("storage is not initialized")
    }
}

impl<'a, T> StorageAccessor<'a, T> for RefCell<T> {
    fn get(&'a self) -> impl Storage<Item = T> + 'a {
        self.borrow_mut()
    }
}

#[repr(transparent)]
pub struct SyncUnsafeCell<T: ?Sized> {
    value: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Sync> Sync for SyncUnsafeCell<T> {}

impl<T> SyncUnsafeCell<T> {
    /// Constructs a new instance of `SyncUnsafeCell` which will wrap the specified value.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> SyncUnsafeCell<T> {
    /// Gets a mutable pointer to the wrapped value.
    ///
    /// This can be cast to a pointer of any kind.
    /// Ensure that the access is unique (no active references, mutable or not)
    /// when casting to `&mut T`, and ensure that there are no mutations
    /// or mutable aliases going on when casting to `&T`
    #[inline]
    pub const fn get(&self) -> *mut T {
        self.value.get()
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// This call borrows the `SyncUnsafeCell` mutably (at compile-time) which
    /// guarantees that we possess the only reference.
    #[inline]
    pub const fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }

    /// Gets a mutable pointer to the wrapped value.
    ///
    /// See [`UnsafeCell::get`] for details.
    #[inline]
    pub const fn raw_get(this: *const Self) -> *mut T {
        // We can just cast the pointer from `SyncUnsafeCell<T>` to `T` because
        // of #[repr(transparent)] on both SyncUnsafeCell and UnsafeCell.
        // See UnsafeCell::raw_get.
        this as *const T as *mut T
    }
}

impl<T: Default> Default for SyncUnsafeCell<T> {
    /// Creates an `SyncUnsafeCell`, with the `Default` value for T.
    fn default() -> SyncUnsafeCell<T> {
        SyncUnsafeCell::new(Default::default())
    }
}

impl<T> From<T> for SyncUnsafeCell<T> {
    /// Creates a new `SyncUnsafeCell<T>` containing the given value.
    fn from(t: T) -> SyncUnsafeCell<T> {
        SyncUnsafeCell::new(t)
    }
}

impl<'a, T> StorageAccessor<'a, T> for SyncUnsafeCell<T> {
    fn get(&'a self) -> impl Storage<Item = T> + 'a {
        unsafe { NonNull::new_unchecked(self.get()) }
    }
}

#[macro_export]
macro_rules! static_storage {
    ($type:ty, $init:expr) => {
        impl $type {
            pub(crate) fn storage() -> impl Storage<Item = $type> + 'static {
                static mut STORAGE: $type = $init;
                unsafe { &mut *core::ptr::addr_of_mut!(STORAGE) }
            }
        }
    };
}

#[macro_export]
macro_rules! static_option_storage {
    ($type:ty) => {
        static mut STORAGE: Option<$type> = None;

        impl $type {
            pub(crate) fn init(init_value: $type) {
                unsafe {
                    STORAGE = Some(init_value);
                };
            }

            pub(crate) fn storage() -> impl Storage<Item = $type> + 'static {
                unsafe { STORAGE.as_mut().expect("storage is not initialized") }
            }
        }
    };
}

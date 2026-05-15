//! State abstraction for single-threaded WASM contracts.
//!
//! Services hold a generic state handle and access it through two traits:
//! [`State`] for reads and [`StateMut`] for writes. Both return guards
//! ([`Deref`]/[`DerefMut`]) wrapped in `Result` so backends that can fail
//! (e.g. paused, expired) report the failure inline. Backends that cannot
//! fail set `Error = Infallible` and expose [`State::get`] / [`StateMut::get_mut`]
//! shortcuts that return the guard directly.
//!
//! `read` takes `&self`; `write` takes `&mut self`. The signatures match
//! idiomatic Rust ownership rules even though the underlying storage usually
//! relies on interior mutability — the `&mut self` on `write` constrains the
//! *handle*, not the data.
//!
//! ## Implementations
//!
//! - [`RefCell<T>`] and [`Rc<RefCell<T>>`] implement both traits with
//!   `Error = Infallible`.
//! - `&RefCell<T>` is also a `StateMut` backend; this is the canonical
//!   service-field type when the program owns the cells and services borrow
//!   them.
//! - Blanket impls forward `&S` (read-only) and `&mut S` (read + write) to
//!   the underlying `S`, so any wrapper composes naturally.
//!
//! ## Composition
//!
//! Cross-cutting concerns (pausing, rate-limiting, metering) are expressed
//! as wrappers that themselves implement `State`/`StateMut`, not as parallel
//! trait hierarchies.

extern crate alloc;

use alloc::rc::Rc;
use core::{
    cell::RefCell,
    convert::Infallible,
    ops::{Deref, DerefMut},
};

/// Read access to a stored value with potentially-fallible access.
///
/// Implementations that cannot fail use `Error = Infallible`.
pub trait State {
    /// The type of the stored value.
    type Item: ?Sized;

    /// Error type returned when access fails (e.g. paused, expired).
    type Error: core::error::Error;

    /// Borrow the stored value for reading.
    fn read(&self) -> Result<impl Deref<Target = Self::Item>, Self::Error>;

    /// Infallible shortcut for [`read`](Self::read).
    ///
    /// Available only when `Self::Error = Infallible`, so the unwrap below is
    /// statically guaranteed to never panic — the only inhabitant of
    /// `Result<_, Infallible>` reachable here is `Ok(_)`.
    fn get(&self) -> impl Deref<Target = Self::Item>
    where
        Self: State<Error = Infallible>,
    {
        // `Error = Infallible` makes `Err(_)` uninhabited.
        self.read().unwrap()
    }
}

/// Mutable access to a stored value with potentially-fallible access.
///
/// `write` takes `&mut self`: idiomatic Rust mutation semantics. The
/// implementation is free to use interior mutability (e.g. `RefCell`) so the
/// underlying value need not be uniquely owned, but the caller must hold the
/// state handle exclusively for the duration of the write.
pub trait StateMut: State {
    /// Borrow the stored value for writing.
    fn write(&mut self) -> Result<impl DerefMut<Target = Self::Item>, Self::Error>;

    /// Infallible shortcut for [`write`](Self::write).
    ///
    /// Available only when `Self::Error = Infallible`, so the unwrap below is
    /// statically guaranteed to never panic — the only inhabitant of
    /// `Result<_, Infallible>` reachable here is `Ok(_)`.
    fn get_mut(&mut self) -> impl DerefMut<Target = Self::Item>
    where
        Self: StateMut<Error = Infallible>,
    {
        // `Error = Infallible` makes `Err(_)` uninhabited.
        self.write().unwrap()
    }

    /// Replace the stored value, returning the previous one.
    fn replace(&mut self, value: Self::Item) -> Result<Self::Item, Self::Error>
    where
        Self::Item: Sized,
    {
        let mut guard = self.write()?;
        Ok(core::mem::replace(&mut *guard, value))
    }

    /// Replace the stored value with one produced by `f`, returning the previous one.
    fn replace_with(
        &mut self,
        f: impl FnOnce(&mut Self::Item) -> Self::Item,
    ) -> Result<Self::Item, Self::Error>
    where
        Self::Item: Sized,
    {
        let mut guard = self.write()?;
        let replacement = f(&mut guard);
        Ok(core::mem::replace(&mut *guard, replacement))
    }

    /// Take the stored value, leaving the default in its place.
    fn take(&mut self) -> Result<Self::Item, Self::Error>
    where
        Self::Item: Default + Sized,
    {
        self.replace(Default::default())
    }
}

// ---- Blanket impls for references ----

/// `&S` is `State` whenever `S` is — read only requires shared access.
impl<S: State + ?Sized> State for &S {
    type Item = S::Item;
    type Error = S::Error;

    fn read(&self) -> Result<impl Deref<Target = Self::Item>, Self::Error> {
        S::read(*self)
    }
}

/// `&mut S` is `State` whenever `S` is.
impl<S: State + ?Sized> State for &mut S {
    type Item = S::Item;
    type Error = S::Error;

    fn read(&self) -> Result<impl Deref<Target = Self::Item>, Self::Error> {
        S::read(&**self)
    }
}

/// `&mut S` is `StateMut` whenever `S` is.
///
/// Note there is no equivalent blanket for `&S` — a shared reference cannot
/// produce `&mut S` to forward through. Specific shared-ref impls (e.g.
/// `&RefCell<T>` below) opt in directly via interior mutability.
impl<S: StateMut + ?Sized> StateMut for &mut S {
    fn write(&mut self) -> Result<impl DerefMut<Target = Self::Item>, Self::Error> {
        S::write(&mut **self)
    }
}

// ---- RefCell<T> ----

impl<T: ?Sized> State for RefCell<T> {
    type Item = T;
    type Error = Infallible;

    fn read(&self) -> Result<impl Deref<Target = T>, Infallible> {
        Ok(self.borrow())
    }
}

impl<T: ?Sized> StateMut for RefCell<T> {
    fn write(&mut self) -> Result<impl DerefMut<Target = T>, Infallible> {
        Ok(self.borrow_mut())
    }
}

/// `&RefCell<T>` is `StateMut` directly — interior mutability lets a shared
/// reference produce `&mut T` despite the trait's `&mut self` requirement.
/// This is the canonical service-field type in the WASM static-state pattern.
impl<T: ?Sized> StateMut for &RefCell<T> {
    fn write(&mut self) -> Result<impl DerefMut<Target = T>, Infallible> {
        Ok((*self).borrow_mut())
    }
}

// ---- Rc<RefCell<T>> ----

impl<T: ?Sized> State for Rc<RefCell<T>> {
    type Item = T;
    type Error = Infallible;

    fn read(&self) -> Result<impl Deref<Target = T>, Infallible> {
        Ok((**self).borrow())
    }
}

impl<T: ?Sized> StateMut for Rc<RefCell<T>> {
    fn write(&mut self) -> Result<impl DerefMut<Target = T>, Infallible> {
        Ok((**self).borrow_mut())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    // ---- RefCell<T> ----

    #[test]
    fn refcell_read_returns_value() {
        let cell = RefCell::new(7u32);
        assert_eq!(*cell.read().unwrap(), 7);
    }

    #[test]
    fn refcell_write_mutates_value() {
        let mut cell = RefCell::new(0u32);
        *cell.write().unwrap() = 42;
        assert_eq!(*cell.read().unwrap(), 42);
    }

    #[test]
    fn refcell_replace_returns_old() {
        let mut cell = RefCell::new(1u32);
        // UFCS to disambiguate from inherent RefCell::replace.
        let old = StateMut::replace(&mut cell, 2).unwrap();
        assert_eq!(old, 1);
        assert_eq!(*cell.read().unwrap(), 2);
    }

    #[test]
    fn refcell_replace_with_transforms_value() {
        let mut cell = RefCell::new(3u32);
        let old = StateMut::replace_with(&mut cell, |v| *v * 10).unwrap();
        assert_eq!(old, 3);
        assert_eq!(*cell.read().unwrap(), 30);
    }

    #[test]
    fn refcell_take_leaves_default() {
        let mut cell = RefCell::new(99u32);
        let taken = StateMut::take(&mut cell).unwrap();
        assert_eq!(taken, 99);
        assert_eq!(*cell.read().unwrap(), 0);
    }

    // ---- &RefCell<T> via direct StateMut impl ----

    #[test]
    fn shared_ref_to_refcell_can_write() {
        let cell = RefCell::new(11u32);
        let mut s: &RefCell<u32> = &cell;
        assert_eq!(*s.read().unwrap(), 11);
        *s.write().unwrap() = 12;
        assert_eq!(*cell.read().unwrap(), 12);
    }

    #[test]
    fn two_shared_refs_to_same_refcell_each_writable() {
        // Distinct shared-reference handles can each be used as StateMut
        // because mutation goes through the inner RefCell's interior mutability.
        let cell = RefCell::new(0u32);
        let mut a: &RefCell<u32> = &cell;
        let mut b: &RefCell<u32> = &cell;
        *a.write().unwrap() = 1;
        *b.write().unwrap() = 2;
        assert_eq!(*cell.read().unwrap(), 2);
    }

    // ---- Rc<RefCell<T>> ----

    #[test]
    fn rc_refcell_read_and_write() {
        let mut s: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        *s.write().unwrap() = 5;
        assert_eq!(*s.read().unwrap(), 5);
    }

    #[test]
    fn rc_refcell_replace() {
        let mut s: Rc<RefCell<u32>> = Rc::new(RefCell::new(1));
        let old = StateMut::replace(&mut s, 9).unwrap();
        assert_eq!(old, 1);
        assert_eq!(*s.read().unwrap(), 9);
    }

    // ---- Generic over State / StateMut ----

    fn read_doubled<S: State<Item = u32>>(s: &S) -> Result<u32, S::Error> {
        Ok(*s.read()? * 2)
    }

    fn bump<S: StateMut<Item = u32>>(s: &mut S) -> Result<(), S::Error> {
        *s.write()? += 1;
        Ok(())
    }

    #[test]
    fn generic_functions_work_with_refcell() {
        let mut cell = RefCell::new(10u32);
        assert_eq!(read_doubled(&cell).unwrap(), 20);
        bump(&mut cell).unwrap();
        assert_eq!(*cell.read().unwrap(), 11);
    }

    #[test]
    fn generic_functions_work_with_rc_refcell() {
        let mut s: Rc<RefCell<u32>> = Rc::new(RefCell::new(10));
        assert_eq!(read_doubled(&s).unwrap(), 20);
        bump(&mut s).unwrap();
        assert_eq!(*s.read().unwrap(), 11);
    }

    #[test]
    fn generic_functions_work_with_shared_ref_to_refcell() {
        let cell = RefCell::new(10u32);
        let mut s: &RefCell<u32> = &cell;
        assert_eq!(read_doubled(&s).unwrap(), 20);
        bump(&mut s).unwrap();
        assert_eq!(*cell.read().unwrap(), 11);
    }

    // ---- Service-style usage (the motivating pattern) ----

    struct CounterService<'a, S: StateMut<Item = u32, Error = Infallible> = &'a RefCell<u32>> {
        counter: S,
        _phantom: core::marker::PhantomData<&'a ()>,
    }

    impl<'a, S: StateMut<Item = u32, Error = Infallible>> CounterService<'a, S> {
        fn new(counter: S) -> Self {
            Self {
                counter,
                _phantom: core::marker::PhantomData,
            }
        }

        // Mutation requires &mut self — idiomatic.
        fn increment(&mut self) {
            *self.counter.get_mut() += 1;
        }

        // Reads stay on &self.
        fn value(&self) -> u32 {
            *self.counter.get()
        }
    }

    #[test]
    fn service_with_refcell_default() {
        let cell = RefCell::new(0u32);
        let mut svc = CounterService::new(&cell);
        svc.increment();
        svc.increment();
        svc.increment();
        assert_eq!(svc.value(), 3);
        assert_eq!(*cell.read().unwrap(), 3);
    }

    // ---- Composition example: a fallible wrapper ----

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum GatedError<E> {
        Closed,
        Inner(E),
    }

    impl<E: core::fmt::Display> core::fmt::Display for GatedError<E> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                GatedError::Closed => f.write_str("gated: closed"),
                GatedError::Inner(e) => write!(f, "gated: {}", e),
            }
        }
    }

    impl<E: core::error::Error> core::error::Error for GatedError<E> {}

    struct Gated<'g, S> {
        inner: S,
        open: &'g core::cell::Cell<bool>,
    }

    impl<'g, S: State> State for Gated<'g, S> {
        type Item = S::Item;
        type Error = GatedError<S::Error>;

        fn read(&self) -> Result<impl Deref<Target = Self::Item>, Self::Error> {
            self.inner.read().map_err(GatedError::Inner)
        }
    }

    impl<'g, S: StateMut> StateMut for Gated<'g, S> {
        fn write(&mut self) -> Result<impl DerefMut<Target = Self::Item>, Self::Error> {
            if !self.open.get() {
                return Err(GatedError::Closed);
            }
            self.inner.write().map_err(GatedError::Inner)
        }
    }

    #[test]
    fn gated_blocks_writes_when_closed() {
        let cell = RefCell::new(0u32);
        let open = core::cell::Cell::new(true);
        let mut gated = Gated {
            inner: &cell,
            open: &open,
        };

        *gated.write().unwrap() = 5;
        assert_eq!(*gated.read().unwrap(), 5);

        open.set(false);
        assert!(matches!(gated.write(), Err(GatedError::Closed)));
        // Reads still work.
        assert_eq!(*gated.read().unwrap(), 5);
    }
}

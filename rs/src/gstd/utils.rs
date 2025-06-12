use crate::Output;
use core::{
    cell::Cell,
    mem::{self, MaybeUninit},
};

/// A writer that writes to a buffer of `MaybeUninit<u8>` bytes
/// safely using the `parity-scale-codec::Output` impl.
pub(crate) struct MaybeUninitBufferWriter<'a> {
    buffer: &'a mut [MaybeUninit<u8>],
    offset: usize,
    skip: usize,
}

impl<'a> MaybeUninitBufferWriter<'a> {
    pub(crate) fn new(buffer: &'a mut [MaybeUninit<u8>]) -> Self {
        Self {
            buffer,
            offset: 0,
            skip: 0,
        }
    }

    /// Gives an access to the internal buffer by providing
    /// it as a param for the given closure.
    pub(crate) fn with_buffer<T>(&self, f: impl FnOnce(&'a [u8]) -> T) -> T {
        f(self.buffer_slice())
    }

    /// Safe wrapper for the access to the internal buffer, which itself
    /// is not a safe op.
    fn buffer_slice(&self) -> &'a [u8] {
        unsafe {
            // SAFETY:
            //
            // Same as `MaybeUninit::slice_assume_init_ref(&buffer[..offset])`.
            // 1. `&[T]` and `&[MaybeUninit<T>]` have the same layout.
            // 2. Size and other params of `MaybeUninit<T>` won't be changed safely.
            // 3. The offset is only changed by the `write` method, and set to the
            //    position of the last written/initialized byte. Therefore, accessing
            //    the buffer up to the offset is safe, even if offset is 0.
            &*(&self.buffer[0..self.offset] as *const _ as *const [u8])
        }
    }

    /// Sets the number of bytes to be skipped on next write.
    ///
    /// This value will be reset to 0 after the next write.
    ///
    /// SAFETY: Calling `write` after this method is safe as long as the `skip` value
    /// is less than or equal to the length of the bytes slice to be written.
    pub(crate) fn skip_next(&mut self, skip: usize) {
        self.skip = skip;
    }
}

// use `parity_scale_codec::Output` trait to not add a custom trait
impl Output for MaybeUninitBufferWriter<'_> {
    fn write(&mut self, bytes: &[u8]) {
        // SAFETY:
        //
        // Same as `MaybeUninit::write_slice(&mut self.buffer[self.offset..end_offset], bytes)`.
        // This code transmutes `bytes: &[T]` to `bytes: &[MaybeUninit<T>]`. These types
        // can be safely transmuted since they have the same layout. Then `bytes:
        // &[MaybeUninit<T>]` is written to uninitialized memory via `copy_from_slice`.
        debug_assert!(
            self.skip <= bytes.len(),
            "Skip value must be less than or equal to the length of the bytes slice"
        );
        let end_offset = self.offset + bytes.len() - self.skip;
        let this = unsafe { self.buffer.get_unchecked_mut(self.offset..end_offset) };
        this.copy_from_slice(unsafe {
            mem::transmute::<&[u8], &[MaybeUninit<u8>]>(&bytes[self.skip..])
        });
        self.offset = end_offset;
        self.skip = 0;
    }
}

/// [`Cell`], but [`Sync`].
///
/// See [`Cell`] for details.
///
/// SAFETY: Use `SyncCell` instead of `Cell` to allow it to be shared between threads,
/// if that's intentional, primarily in single-threaded execution environments.
/// Providing proper synchronization is still the task of the user,
/// making this type just as unsafe to use.
/// Can cause data races if called from a separate thread.
pub(crate) struct SyncCell<T: ?Sized>(Cell<T>);

unsafe impl<T: ?Sized + Sync> Sync for SyncCell<T> {}

#[allow(unused)]
impl<T> SyncCell<T> {
    pub const fn new(value: T) -> Self {
        Self(Cell::new(value))
    }

    pub fn get(&self) -> T
    where
        T: Copy,
    {
        self.0.get()
    }

    pub fn set(&self, value: T) {
        self.0.set(value);
    }

    pub fn replace(&self, val: T) -> T {
        self.0.replace(val)
    }

    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }
}

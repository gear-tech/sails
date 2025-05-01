use crate::Output;
use core::mem::{self, MaybeUninit};

#[derive(Debug)]
pub(crate) struct MaybeUninitBufferAccessError;

/// A writer that writes to a buffer of `MaybeUninit<u8>` bytes
/// safely using the `parity-scale-codec::Output` impl.
pub(crate) struct MaybeUninitBufferWriter<'a> {
    buffer: &'a mut [MaybeUninit<u8>],
    offset: usize,
    initialized: bool,
}

impl<'a> MaybeUninitBufferWriter<'a> {
    pub(crate) fn new(buffer: &'a mut [MaybeUninit<u8>]) -> Self {
        Self {
            buffer,
            offset: 0,
            initialized: false,
        }
    }

    /// Gives an access to the internal buffer by providing
    /// it as a param for the given closure.
    ///
    /// Returns an error if the buffer is not initialized.
    pub(crate) fn access_buffer<T>(
        &self,
        f: impl FnOnce(&'a [u8]) -> T,
    ) -> Result<T, MaybeUninitBufferAccessError> {
        if !self.initialized {
            return Err(MaybeUninitBufferAccessError);
        }

        Ok(f(self.access_buffer_inner()))
    }

    /// Safe wrapper for the access to the internal buffer, which itself
    /// is not a safe op.
    fn access_buffer_inner(&self) -> &'a [u8] {
        unsafe {
            // SAFETY:
            //
            // Same as `MaybeUninit::slice_assume_init_ref(&buffer[..offset])`.
            // 1. Method ensures that values of inner buffer were initialized.
            // 2. `&[T]` and `&[MaybeUninit<T>]` have the same layout.
            // 3. Size and other params of `MaybeUninit<T>` won't be changed safely.
            &*(&self.buffer[0..self.offset] as *const _ as *const [u8])
        }
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
        let end_offset = self.offset + bytes.len();
        let this = unsafe { self.buffer.get_unchecked_mut(self.offset..end_offset) };
        this.copy_from_slice(unsafe { mem::transmute::<&[u8], &[MaybeUninit<u8>]>(bytes) });
        self.offset = end_offset;
        self.initialized = true;
    }
}

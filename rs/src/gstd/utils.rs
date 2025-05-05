use crate::Output;
use core::mem::{self, MaybeUninit};

/// A writer that writes to a buffer of `MaybeUninit<u8>` bytes
/// safely using the `parity-scale-codec::Output` impl.
pub(crate) struct MaybeUninitBufferWriter<'a> {
    buffer: &'a mut [MaybeUninit<u8>],
    offset: usize,
}

impl<'a> MaybeUninitBufferWriter<'a> {
    pub(crate) fn new(buffer: &'a mut [MaybeUninit<u8>]) -> Self {
        Self { buffer, offset: 0 }
    }

    /// Gives an access to the internal buffer by providing
    /// it as a param for the given closure.
    ///
    /// Returns an error if the buffer is not initialized.
    pub(crate) fn access_buffer<T>(&self, f: impl FnOnce(&'a [u8]) -> T) -> T {
        f(self.access_buffer_inner())
    }

    /// Safe wrapper for the access to the internal buffer, which itself
    /// is not a safe op.
    fn access_buffer_inner(&self) -> &'a [u8] {
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
    }
}

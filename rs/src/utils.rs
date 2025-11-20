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
pub(crate) struct SyncCell<T: ?Sized>(Cell<T>);

/// SAFETY: Use `SyncCell` instead of `Cell` to allow it to be shared between threads,
/// if that's intentional, primarily in single-threaded execution environments.
/// Providing proper synchronization is still the task of the user,
/// making this type just as unsafe to use.
/// Can cause data races if called from a separate thread.
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

/// Sorts an array of byte arrays in place at compile time using bubble sort.
/// This is used for deterministic ordering of service interface IDs.
///
/// # Examples
///
/// ```
/// const SORTED: [[u8; 4]; 3] = sails_rs::utils::const_bubble_sort_bytes(&[
///     [3, 2, 1, 0],
///     [1, 2, 3, 4],
///     [2, 1, 0, 0],
/// ]);
/// assert_eq!(SORTED, [[1, 2, 3, 4], [2, 1, 0, 0], [3, 2, 1, 0]]);
/// ```
pub const fn const_bubble_sort_bytes<const N: usize, const M: usize>(
    arr: &[[u8; M]; N],
) -> [[u8; M]; N] {
    let mut result = *arr;
    let mut i = 0;
    while i < result.len() {
        let mut j = 0;
        while j < result.len() - i - 1 {
            // Compare arrays lexicographically
            let mut k = 0;
            let mut should_swap = false;
            while k < M {
                if result[j][k] > result[j + 1][k] {
                    should_swap = true;
                    break;
                } else if result[j][k] < result[j + 1][k] {
                    break;
                }
                k += 1;
            }
            if should_swap {
                let temp = result[j];
                result[j] = result[j + 1];
                result[j + 1] = temp;
            }
            j += 1;
        }
        i += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_bubble_sort_bytes() {
        const UNSORTED: [[u8; 4]; 5] = [
            [3, 2, 1, 0],
            [1, 2, 3, 4],
            [2, 1, 0, 0],
            [0, 0, 0, 1],
            [1, 2, 3, 3],
        ];
        const SORTED: [[u8; 4]; 5] = const_bubble_sort_bytes(&UNSORTED);

        assert_eq!(SORTED[0], [0, 0, 0, 1]);
        assert_eq!(SORTED[1], [1, 2, 3, 3]);
        assert_eq!(SORTED[2], [1, 2, 3, 4]);
        assert_eq!(SORTED[3], [2, 1, 0, 0]);
        assert_eq!(SORTED[4], [3, 2, 1, 0]);
    }

    #[test]
    fn test_const_bubble_sort_bytes_32() {
        // Test with 32-byte arrays (like interface IDs)
        const ID1: [u8; 32] = [1; 32];
        const ID2: [u8; 32] = [2; 32];
        const ID3: [u8; 32] = [0; 32];

        const UNSORTED: [[u8; 32]; 3] = [ID2, ID1, ID3];
        const SORTED: [[u8; 32]; 3] = const_bubble_sort_bytes(&UNSORTED);

        assert_eq!(SORTED[0], ID3);
        assert_eq!(SORTED[1], ID1);
        assert_eq!(SORTED[2], ID2);
    }

    #[test]
    fn test_const_bubble_sort_bytes_already_sorted() {
        const ALREADY_SORTED: [[u8; 4]; 3] = [[1, 2, 3, 4], [2, 1, 0, 0], [3, 2, 1, 0]];
        const RESULT: [[u8; 4]; 3] = const_bubble_sort_bytes(&ALREADY_SORTED);

        assert_eq!(RESULT, ALREADY_SORTED);
    }

    #[test]
    fn test_const_bubble_sort_bytes_empty() {
        const EMPTY: [[u8; 4]; 0] = [];
        const RESULT: [[u8; 4]; 0] = const_bubble_sort_bytes(&EMPTY);

        assert_eq!(RESULT, EMPTY);
    }
}

#[macro_use]
extern crate mac;

use std::slice;

pub use impls::arrayvec::ArrayVecBuffer;
pub use impls::buffer_ref::BufferRefBuffer;
pub use impls::vec::VecBuffer;
pub use traits::ReadBuffer;
pub use traits::ReadBufferRef;
pub use traits::read_buffer_ref;

mod impls;
mod traits;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CapacityError;

unsafe fn wildly_unsafe<'a, 'b>(slice: &'a mut [u8]) -> &'b mut [u8] {
    slice::from_raw_parts_mut(slice.as_mut_ptr(), slice.len())
}

pub struct BufferRef<'data, 'size> {
    buffer: &'data mut [u8],
    initialized_: &'size mut usize,
}

impl<'d, 's> BufferRef<'d, 's> {
    pub fn new(buffer: &'d mut [u8], initialized: &'s mut usize) -> BufferRef<'d, 's> {
        debug_assert!(*initialized == 0);
        BufferRef {
            buffer: buffer,
            initialized_: initialized,
        }
    }
    pub unsafe fn uninitialized(&mut self) -> &mut [u8] {
        self.buffer
    }
    pub unsafe fn advance(&mut self, num_bytes: usize) {
        assert!(*self.initialized_ + num_bytes <= self.buffer.len());
        *self.initialized_ += num_bytes;
    }
    pub fn extend<I>(&mut self, bytes: I) -> Result<(), CapacityError>
        where I: Iterator<Item=u8>
    {
        let mut buf_iter = (&mut self.buffer[*self.initialized_..]).into_iter();
        for b in bytes {
            *unwrap_or_return!(buf_iter.next(), Err(CapacityError)) = b;
            *self.initialized_ += 1;
        }
        Ok(())
    }
    pub unsafe fn uninitialized_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[*self.initialized_..]
    }
    pub fn initialized(self) -> &'d [u8] {
        &self.buffer[..*self.initialized_]
    }
    pub fn remaining(&self) -> usize {
        self.buffer.len() - *self.initialized_
    }
}

pub fn with_buffer<'a, T: Buffer<'a>, F, R>(buffer: T, f: F) -> R
    where F: for<'b> FnOnce(BufferRef<'a, 'b>) -> R
{
    let mut intermediate = buffer.to_to_buffer_ref();
    f(intermediate.to_buffer_ref())
}

pub trait Buffer<'data> {
    type Intermediate: ToBufferRef<'data>;
    fn to_to_buffer_ref(self) -> Self::Intermediate;
}

pub trait ToBufferRef<'data> {
    fn to_buffer_ref<'size>(&'size mut self) -> BufferRef<'data, 'size>;
}

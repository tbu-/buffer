extern crate arrayvec;

use arrayvec::ArrayVec;
use std::slice;

unsafe fn wildly_unsafe<'a, 'b>(slice: &'a mut [u8]) -> &'b mut [u8] {
    slice::from_raw_parts_mut(slice.as_mut_ptr(), slice.len())
}

struct VecBuffer<'data> {
    // Will only touch the length of the `Vec` through this reference, except
    // in `VecBuffer::buffer`.
    vec: &'data mut Vec<u8>,
    initialized: usize,
}

impl<'data> VecBuffer<'data> {
    fn new(vec: &'data mut Vec<u8>) -> VecBuffer<'data> {
        VecBuffer {
            vec: vec,
            initialized: 0,
        }
    }
    fn buffer<'size>(&'size mut self) -> BufferRef<'data, 'size> {
        let len = self.vec.len();
        let remaining = self.vec.capacity() - len;
        unsafe {
            let start = self.vec.as_mut_ptr().offset(len as isize);
            BufferRef {
                // This is unsafe, we now have two unique (mutable) references
                // to the same `Vec`. However, we will only access
                // `self.vec.len` through `self` and only the contents through
                // the `BufferRef`.
                buffer: slice::from_raw_parts_mut(start, remaining),
                initialized: &mut self.initialized,
            }
        }
    }
}

impl<'data> Drop for VecBuffer<'data> {
    fn drop(&mut self) {
        let len = self.vec.len();
        unsafe {
            self.vec.set_len(len + self.initialized);
        }
    }
}

struct BufferRefBuffer<'ref_, 'data: 'ref_, 'size: 'ref_> {
    // Will only touch the length of the `BufferRef` through this reference,
    // except in `BufferRefBuffer::buffer`.
    buffer: &'ref_ mut BufferRef<'data, 'size>,
    initialized: usize,
}

impl<'r, 'd, 's> BufferRefBuffer<'r, 'd, 's> {
    fn new(buffer: &'r mut BufferRef<'d, 's>) -> BufferRefBuffer<'r, 'd, 's> {
        BufferRefBuffer {
            buffer: buffer,
            initialized: 0,
        }
    }
    fn buffer<'a>(&'a mut self) -> BufferRef<'d, 'a> {
        let len = *self.buffer.initialized;
        let remaining = self.buffer.buffer.len() - len;
        unsafe {
            BufferRef {
                buffer: wildly_unsafe(&mut self.buffer.buffer[remaining..]),
                initialized: &mut self.initialized,
            }
        }
    }
}

impl<'r, 'd, 's> Drop for BufferRefBuffer<'r, 'd, 's> {
    fn drop(&mut self) {
        *self.buffer.initialized += self.initialized;
    }
}

struct SliceBuffer<'data> {
    // TODO: Double mut.
    slice: &'data mut [u8],
    initialized: usize,
}

struct BufferRef<'data, 'size> {
    buffer: &'data mut [u8],
    initialized: &'size mut usize,
}

fn read<'data, 'size>(mut buf: BufferRef<'data, 'size>) -> Result<&'data [u8], ()> {
    read2(&mut buf);
    buf.buffer[0] = 0;
    *buf.initialized = 1;
    Ok(&buf.buffer[..*buf.initialized])
}

fn read_to_vec(vec: &mut Vec<u8>) -> Result<&[u8], ()> {
    let mut vb = VecBuffer::new(vec);
    read(vb.buffer())
}

fn read2<'a, T: Buffer<'a>>(buffer: T) -> Result<&'a [u8], ()> {
    with_buffer(buffer, read)
}


fn with_buffer<'a, T: Buffer<'a>, F, R>(buffer: T, f: F) -> R
    where F: for<'b> FnOnce(BufferRef<'a, 'b>) -> R
{
    let mut intermediate = buffer.to_to_buffer_ref();
    f(intermediate.to_buffer_ref())
}

trait Buffer<'data> {
    type Intermediate: ToBufferRef<'data>;
    fn to_to_buffer_ref(self) -> Self::Intermediate;
}

trait ToBufferRef<'data> {
    fn to_buffer_ref<'size>(&'size mut self) -> BufferRef<'data, 'size>;
}

impl<'data> Buffer<'data> for &'data mut Vec<u8> {
    type Intermediate = VecBuffer<'data>;
    fn to_to_buffer_ref(self) -> Self::Intermediate {
        VecBuffer::new(self)
    }
}

impl<'data> ToBufferRef<'data> for VecBuffer<'data> {
    fn to_buffer_ref<'size>(&'size mut self) -> BufferRef<'data, 'size> {
        self.buffer()
    }
}

impl<'r, 'd, 's> Buffer<'d> for &'r mut BufferRef<'d, 's> {
    type Intermediate = BufferRefBuffer<'r, 'd, 's>;
    fn to_to_buffer_ref(self) -> Self::Intermediate {
        BufferRefBuffer::new(self)
    }
}

impl<'r, 'd, 's> ToBufferRef<'d> for BufferRefBuffer<'r, 'd, 's> {
    fn to_buffer_ref<'a>(&'a mut self) -> BufferRef<'d, 'a> {
        self.buffer()
    }
}

struct ArrayVecBuffer<'data, A: 'data+arrayvec::Array<Item=u8>> {
    // Will only touch the length of the `ArrayVec` through this reference,
    // except in `ArrayVecBuffer::buffer`.
    vec: &'data mut ArrayVec<A>,
    initialized: usize,
}

impl<'d, A: arrayvec::Array<Item=u8>> ArrayVecBuffer<'d, A> {
    fn new(vec: &'d mut ArrayVec<A>) -> ArrayVecBuffer<'d, A> {
        ArrayVecBuffer {
            vec: vec,
            initialized: 0,
        }
    }
    fn buffer<'s>(&'s mut self) -> BufferRef<'d, 's> {
        let len = self.vec.len();
        let remaining = self.vec.capacity() - len;
        unsafe {
            let start = self.vec.as_mut_ptr().offset(len as isize);
            BufferRef {
                // This is unsafe, we now have two unique (mutable) references
                // to the same `ArrayVec`. However, we will only access
                // `self.vec.len` through `self` and only the contents through
                // the `BufferRef`.
                buffer: slice::from_raw_parts_mut(start, remaining),
                initialized: &mut self.initialized,
            }
        }
    }
}

impl<'d, A: arrayvec::Array<Item=u8>> Drop for ArrayVecBuffer<'d, A> {
    fn drop(&mut self) {
        let len = self.vec.len();
        unsafe {
            self.vec.set_len(len + self.initialized);
        }
    }
}


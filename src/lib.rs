mod impls;

pub use impls::arrayvec::ArrayVecBuffer;
pub use impls::buffer_ref::BufferRefBuffer;
pub use impls::vec::VecBuffer;

pub struct BufferRef<'data, 'size> {
    buffer: &'data mut [u8],
    initialized: &'size mut usize,
}

fn read<'data, 'size>(mut buf: BufferRef<'data, 'size>) -> Result<&'data [u8], ()> {
    buf.buffer[0] = 0;
    *buf.initialized = 1;
    Ok(&buf.buffer[..*buf.initialized])
}

fn read_to_vec(vec: &mut Vec<u8>) -> Result<&[u8], ()> {
    read2(vec)
}

fn read2<'a, T: Buffer<'a>>(buffer: T) -> Result<&'a [u8], ()> {
    with_buffer(buffer, read)
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

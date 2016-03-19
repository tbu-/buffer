use Buffer;
use BufferRef;
use std::fs;
use std::io::Read;
use std::io;
use std::net;
use with_buffer;

/// A utility function for unsafely implementing `ReadBufferRef` for readers
/// that don't read the buffer passed to `Read::read`.
pub unsafe fn read_buffer_ref<'d, 's, T: Read>(reader: &mut T, mut buf: BufferRef<'d, 's>)
    -> io::Result<&'d [u8]>
{
    let read = try!(reader.read(buf.uninitialized_mut()));
    buf.advance(read);
    Ok(buf.initialized())
}

/// An internal trait to be implemented by `T: Read` which do not access the
/// read buffer in `Read::read`.
///
/// Can be implemented using the `read_buffer_ref` helper function.
pub trait ReadBufferRef: Read {
    /// Reads (equivalently to `Read::read`) into the buffer ref and returns
    /// the newly written bytes.
    fn read_buffer_ref<'d, 's>(&mut self, buf: BufferRef<'d, 's>) -> io::Result<&'d [u8]>;
}

/// Trait to read to `T: Buffer`.
///
/// This trait should be imported to read into buffers.
pub trait ReadBuffer: ReadBufferRef {
    /// Reads (equivalently to `Read::read`) into the buffer and returns the
    /// newly read bytes.
    fn read_buffer<'d, B: Buffer<'d>>(&mut self, buf: B) -> io::Result<&'d [u8]> {
        with_buffer(buf, |buf| self.read_buffer_ref(buf))
    }
}

impl<T: ReadBufferRef> ReadBuffer for T { }

macro_rules! unsafe_read_buffer_ref_impl {
    ($($t:ty)*) => {
        $(
            impl ReadBufferRef for $t {
                fn read_buffer_ref<'d, 's>(&mut self, buf: BufferRef<'d, 's>)
                    -> io::Result<&'d [u8]>
                {
                    unsafe {
                        read_buffer_ref(self, buf)
                    }
                }
            }
        )*
    }
}

unsafe_read_buffer_ref_impl! {
    fs::File
    io::Empty
    io::Repeat
    io::Stdin
    net::TcpStream
}

impl<'a> ReadBufferRef for &'a fs::File { fn read_buffer_ref<'d, 's>(&mut self, buf: BufferRef<'d, 's>) -> io::Result<&'d [u8]> { unsafe { read_buffer_ref(self, buf) } } }
impl<'a> ReadBufferRef for &'a net::TcpStream { fn read_buffer_ref<'d, 's>(&mut self, buf: BufferRef<'d, 's>) -> io::Result<&'d [u8]> { unsafe { read_buffer_ref(self, buf) } } }

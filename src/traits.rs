use Buffer;
use BufferRef;
use std::fs;
use std::io::Read;
use std::io;
use std::net;
use with_buffer;

#[macro_export]
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
#[macro_export]
macro_rules! unwrap_or_return {
    ($e:expr, $r:expr) => {
        match $e { Some(e) => e, None => return $r }
    };
    ($e:expr) => {
        unwrap_or_return!($e, None)
    };
}


pub unsafe fn read_buffer_ref<'d, 's, T: Read>(reader: &mut T, mut buf: BufferRef<'d, 's>)
    -> io::Result<&'d [u8]>
{
    let read = try!(reader.read(buf.uninitialized_mut()));
    buf.advance(read);
    Ok(buf.initialized())
}

pub trait ReadBufferRef: Read {
    fn read_buffer_ref<'d, 's>(&mut self, buf: BufferRef<'d, 's>) -> io::Result<&'d [u8]>;
}

pub trait ReadBuffer: ReadBufferRef {
    fn read_buffer<'d, B: Buffer<'d>>(&mut self, buf: B) -> io::Result<&'d [u8]> {
        with_buffer(buf, |buf| self.read_buffer_ref(buf))
    }
}

impl<T: ReadBufferRef> ReadBuffer for T { }

unsafe_read_buffer_ref_impl! {
    fs::File
    io::Empty
    io::Repeat
    io::Stdin
    net::TcpStream
}

impl<'a> ReadBufferRef for &'a fs::File { fn read_buffer_ref<'d, 's>(&mut self, buf: BufferRef<'d, 's>) -> io::Result<&'d [u8]> { unsafe { read_buffer_ref(self, buf) } } }
impl<'a> ReadBufferRef for &'a net::TcpStream { fn read_buffer_ref<'d, 's>(&mut self, buf: BufferRef<'d, 's>) -> io::Result<&'d [u8]> { unsafe { read_buffer_ref(self, buf) } } }

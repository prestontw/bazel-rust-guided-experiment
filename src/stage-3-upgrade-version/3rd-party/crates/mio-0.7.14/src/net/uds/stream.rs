use crate::io_source::IoSource;
use crate::{event, sys, Interest, Registry, Token};

use std::fmt;
use std::io::{self, IoSlice, IoSliceMut, Read, Write};
use std::net::Shutdown;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::os::unix::net;
use std::path::Path;

/// A non-blocking Unix stream socket.
pub struct UnixStream {
    inner: IoSource<net::UnixStream>,
}

impl UnixStream {
    /// Connects to the socket named by `path`.
    pub fn connect<P: AsRef<Path>>(path: P) -> io::Result<UnixStream> {
        sys::uds::stream::connect(path.as_ref()).map(UnixStream::from_std)
    }

    /// Creates a new `UnixStream` from a standard `net::UnixStream`.
    ///
    /// This function is intended to be used to wrap a Unix stream from the
    /// standard library in the Mio equivalent. The conversion assumes nothing
    /// about the underlying stream; it is left up to the user to set it in
    /// non-blocking mode.
    ///
    /// # Note
    ///
    /// The Unix stream here will not have `connect` called on it, so it
    /// should already be connected via some other means (be it manually, or
    /// the standard library).
    pub fn from_std(stream: net::UnixStream) -> UnixStream {
        UnixStream {
            inner: IoSource::new(stream),
        }
    }

    /// Creates an unnamed pair of connected sockets.
    ///
    /// Returns two `UnixStream`s which are connected to each other.
    pub fn pair() -> io::Result<(UnixStream, UnixStream)> {
        sys::uds::stream::pair().map(|(stream1, stream2)| {
            (UnixStream::from_std(stream1), UnixStream::from_std(stream2))
        })
    }

    /// Returns the socket address of the local half of this connection.
    pub fn local_addr(&self) -> io::Result<sys::SocketAddr> {
        sys::uds::stream::local_addr(&self.inner)
    }

    /// Returns the socket address of the remote half of this connection.
    pub fn peer_addr(&self) -> io::Result<sys::SocketAddr> {
        sys::uds::stream::peer_addr(&self.inner)
    }

    /// Returns the value of the `SO_ERROR` option.
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.inner.take_error()
    }

    /// Shuts down the read, write, or both halves of this connection.
    ///
    /// This function will cause all pending and future I/O calls on the
    /// specified portions to immediately return with an appropriate value
    /// (see the documentation of `Shutdown`).
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.inner.shutdown(how)
    }
}

impl Read for UnixStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.do_io(|inner| (&*inner).read(buf))
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.inner.do_io(|inner| (&*inner).read_vectored(bufs))
    }
}

impl<'a> Read for &'a UnixStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.do_io(|inner| (&*inner).read(buf))
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.inner.do_io(|inner| (&*inner).read_vectored(bufs))
    }
}

impl Write for UnixStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.do_io(|inner| (&*inner).write(buf))
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.inner.do_io(|inner| (&*inner).write_vectored(bufs))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.do_io(|inner| (&*inner).flush())
    }
}

impl<'a> Write for &'a UnixStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.do_io(|inner| (&*inner).write(buf))
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.inner.do_io(|inner| (&*inner).write_vectored(bufs))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.do_io(|inner| (&*inner).flush())
    }
}

impl event::Source for UnixStream {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        self.inner.register(registry, token, interests)
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        self.inner.reregister(registry, token, interests)
    }

    fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
        self.inner.deregister(registry)
    }
}

impl fmt::Debug for UnixStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl IntoRawFd for UnixStream {
    fn into_raw_fd(self) -> RawFd {
        self.inner.into_inner().into_raw_fd()
    }
}

impl AsRawFd for UnixStream {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl FromRawFd for UnixStream {
    /// Converts a `RawFd` to a `UnixStream`.
    ///
    /// # Notes
    ///
    /// The caller is responsible for ensuring that the socket is in
    /// non-blocking mode.
    unsafe fn from_raw_fd(fd: RawFd) -> UnixStream {
        UnixStream::from_std(FromRawFd::from_raw_fd(fd))
    }
}
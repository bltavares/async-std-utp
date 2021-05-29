use futures::{future::BoxFuture, ready, AsyncRead, AsyncWrite, FutureExt};

use crate::socket::UtpSocket;
use async_std::{io, net::ToSocketAddrs, sync::RwLock};
use std::{io::Result, net::SocketAddr, sync::Arc, task::Poll};

/// A structure that represents a uTP (Micro Transport Protocol) stream between a local socket and a
/// remote socket.
///
/// The connection will be closed when the value is dropped (either explicitly or when it goes out
/// of scope).
///
/// The default maximum retransmission retries is 5, which translates to about 16 seconds. It can be
/// changed by calling `set_max_retransmission_retries`. Notice that the initial congestion timeout
/// is 500 ms and doubles with each timeout.
///
/// # Examples
///
/// ```no_run
/// # fn main() { async_std::task::block_on(async {
/// use async_std_utp::UtpStream;
/// use async_std::prelude::*;
///
/// let mut stream = UtpStream::bind("127.0.0.1:1234").await.expect("Error binding stream");
/// let _ = stream.write(&[1]).await;
/// let _ = stream.read(&mut [0; 1000]).await;
/// # }); }
/// ```
// #[derive(Clone)] // TODO cloenable upstream
pub struct UtpStream {
    socket: Arc<RwLock<UtpSocket>>,
    futures: UtpStreamFutures,
}

type ReadFuture = Option<BoxFuture<'static, io::Result<(Vec<u8>, usize)>>>;

#[derive(Default)]
struct UtpStreamFutures {
    read: ReadFuture,
    write: Option<BoxFuture<'static, io::Result<usize>>>,
    flush: Option<BoxFuture<'static, io::Result<()>>>,
    close: Option<BoxFuture<'static, io::Result<()>>>,
}

impl UtpStream {
    /// Creates a uTP stream listening on the given address.
    ///
    /// The address type can be any implementer of the `ToSocketAddr` trait. See its documentation
    /// for concrete examples.
    ///
    /// If more than one valid address is specified, only the first will be used.
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> Result<UtpStream> {
        let socket = UtpSocket::bind(addr).await?;
        Ok(UtpStream {
            socket: Arc::new(RwLock::new(socket)),
            futures: UtpStreamFutures::default(),
        })
    }

    /// Opens a uTP connection to a remote host by hostname or IP address.
    ///
    /// The address type can be any implementer of the `ToSocketAddr` trait. See its documentation
    /// for concrete examples.
    ///
    /// If more than one valid address is specified, only the first will be used.
    pub async fn connect<A: ToSocketAddrs>(dst: A) -> Result<UtpStream> {
        // Port 0 means the operating system gets to choose it
        let socket = UtpSocket::connect(dst).await?;
        Ok(UtpStream {
            socket: Arc::new(RwLock::new(socket)),
            futures: UtpStreamFutures::default(),
        })
    }

    /// Gracefully closes connection to peer.
    ///
    /// This method allows both peers to receive all packets still in
    /// flight.
    pub async fn close(&mut self) -> Result<()> {
        self.socket.write().await.close().await
    }

    /// Returns the socket address of the local half of this uTP connection.
    pub async fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.read().await.local_addr()
    }

    /// Changes the maximum number of retransmission retries on the underlying socket.
    pub async fn set_max_retransmission_retries(&mut self, n: u32) {
        self.socket.write().await.max_retransmission_retries = n;
    }
}

impl AsyncRead for UtpStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<Result<usize>> {
        if self.futures.read.is_none() {
            let socket = self.socket.clone();
            let mut vec = Vec::from(&buf[..]);
            self.as_mut().futures.read = Some(Box::pin(async move {
                let (nread, _) = socket.write().await.recv_from(&mut vec).await?;
                Ok((vec, nread))
            }));
        }

        let fut = self.futures.read.as_mut().unwrap();
        let (bytes, nread) = ready!(fut.poll_unpin(cx))?;
        buf.copy_from_slice(&bytes);
        self.futures.read = None;
        Poll::Ready(Ok(nread))
    }
}

impl AsyncWrite for UtpStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize>> {
        if self.futures.write.is_none() {
            let socket = self.socket.clone();
            let vec = Vec::from(buf);
            self.as_mut().futures.write = Some(Box::pin(async move {
                let nread = socket.write().await.send_to(&vec).await?;
                Ok(nread)
            }));
        }

        let fut = self.futures.write.as_mut().unwrap();
        let nread = ready!(fut.poll_unpin(cx))?;
        self.futures.write = None;
        Poll::Ready(Ok(nread))
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<()>> {
        if self.futures.flush.is_none() {
            let socket = self.socket.clone();
            self.as_mut().futures.flush =
                Some(Box::pin(async move { socket.write().await.flush().await }));
        }

        let fut = self.futures.flush.as_mut().unwrap();
        let result = ready!(fut.poll_unpin(cx));
        self.futures.flush = None;
        Poll::Ready(result)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<()>> {
        if self.futures.close.is_none() {
            let socket = self.socket.clone();
            self.as_mut().futures.close =
                Some(Box::pin(async move { socket.write().await.flush().await }));
        }

        let fut = self.futures.close.as_mut().unwrap();
        let result = ready!(fut.poll_unpin(cx));
        self.futures.write = None;
        Poll::Ready(result)
    }
}

impl From<UtpSocket> for UtpStream {
    fn from(socket: UtpSocket) -> Self {
        UtpStream {
            socket: Arc::new(RwLock::new(socket)),
            futures: UtpStreamFutures::default(),
        }
    }
}

//! One stream type for plain and TLS connections, so the session type is the
//! same either way. It also notices when the server closes its end: the IMAP
//! client treats a closed connection as the end of a response, which would
//! make a half-finished answer look complete.

use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;

#[derive(Debug)]
enum Wire {
    Plain(TcpStream),
    Tls(Box<TlsStream<TcpStream>>),
}

/// A running total of the bytes read from the server. It can be read while
/// the stream is busy inside a command, which is how a download that is still
/// coming is told from one that has stopped.
#[derive(Debug, Clone, Default)]
pub struct ReadCounter(Arc<AtomicU64>);

impl ReadCounter {
    /// Every byte read so far.
    pub fn total(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }

    pub(super) fn add(&self, bytes: usize) {
        self.0.fetch_add(bytes as u64, Ordering::Relaxed);
    }
}

/// The wire under an IMAP session.
#[derive(Debug)]
pub struct MailStream {
    wire: Wire,
    peer_closed: bool,
    received: ReadCounter,
}

impl MailStream {
    pub fn plain(tcp: TcpStream) -> Self {
        Self {
            wire: Wire::Plain(tcp),
            peer_closed: false,
            received: ReadCounter::default(),
        }
    }

    pub fn tls(stream: TlsStream<TcpStream>) -> Self {
        Self {
            wire: Wire::Tls(Box::new(stream)),
            peer_closed: false,
            received: ReadCounter::default(),
        }
    }

    /// The bare socket of a connection that is not yet under TLS.
    pub fn into_plain(self) -> Option<TcpStream> {
        match self.wire {
            Wire::Plain(tcp) => Some(tcp),
            Wire::Tls(_) => None,
        }
    }

    /// A handle on the count of bytes read, which stays usable while a
    /// command holds the stream.
    pub fn read_counter(&self) -> ReadCounter {
        self.received.clone()
    }

    /// True once a read has found the server's end closed.
    pub fn peer_closed(&self) -> bool {
        self.peer_closed
    }
}

impl AsyncRead for MailStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let before = buf.filled().len();
        let room = buf.remaining() > 0;
        let polled = match &mut this.wire {
            Wire::Plain(stream) => Pin::new(stream).poll_read(cx, buf),
            Wire::Tls(stream) => Pin::new(stream.as_mut()).poll_read(cx, buf),
        };
        if matches!(polled, Poll::Ready(Ok(()))) {
            let got = buf.filled().len() - before;
            if got > 0 {
                this.received.add(got);
            } else if room {
                this.peer_closed = true;
            }
        }
        polled
    }
}

impl AsyncWrite for MailStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match &mut self.get_mut().wire {
            Wire::Plain(stream) => Pin::new(stream).poll_write(cx, buf),
            Wire::Tls(stream) => Pin::new(stream.as_mut()).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut self.get_mut().wire {
            Wire::Plain(stream) => Pin::new(stream).poll_flush(cx),
            Wire::Tls(stream) => Pin::new(stream.as_mut()).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut self.get_mut().wire {
            Wire::Plain(stream) => Pin::new(stream).poll_shutdown(cx),
            Wire::Tls(stream) => Pin::new(stream.as_mut()).poll_shutdown(cx),
        }
    }
}

#[cfg(test)]
include!("stream.tests.rs");

use bytes::{BufMut, BytesMut};
use futures::future;
use futures::ready;
use pin_project::pin_project;
use strchunk::StrChunk;
use tokio::io::ReadBuf;
use tokio::prelude::*;

use std::mem::MaybeUninit;
use std::pin::Pin;
use std::task::{Context, Poll};

const DEFAULT_BUFFER_CAPACITY: usize = 8 * 1024;

#[pin_project(project = Utf8ReaderProj)]
pub struct Utf8Reader<R> {
    #[pin]
    inner: R,
    buf: BytesMut,
}

impl<R> Utf8Reader<R> {
    pub fn new(inner: R) -> Self {
        Utf8Reader {
            inner,
            buf: BytesMut::with_capacity(DEFAULT_BUFFER_CAPACITY),
        }
    }
}

fn extract_utf8_if_read_not_empty(
    bytes_read: usize,
    buf: &mut BytesMut,
) -> io::Result<StrChunk> {
    if bytes_read == 0 && !buf.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "incomplete UTF-8 sequence in input",
        ));
    }
    StrChunk::extract_utf8(buf)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

impl<R: AsyncRead> Utf8Reader<R> {
    pub fn poll_read_utf8(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<StrChunk>> {
        let Utf8ReaderProj { inner, buf } = self.project();
        debug_assert!(buf.capacity() >= 4);
        let dst = buf.bytes_mut();
        let dst = unsafe { &mut *(dst as *mut _ as *mut [MaybeUninit<u8>]) };
        let mut dst = ReadBuf::uninit(dst);
        ready!(inner.poll_read(cx, &mut dst)?);
        let res = extract_utf8_if_read_not_empty(dst.filled().len(), buf);
        Poll::Ready(res)
    }
}

impl<R: AsyncRead + Unpin> Utf8Reader<R> {
    async fn read_utf8(&mut self) -> io::Result<StrChunk> {
        let mut pinned_self = Pin::new(self);
        future::poll_fn(|cx| pinned_self.as_mut().poll_read_utf8(cx)).await
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let s: &[_] = b"Hello, world!\n";
    let mut out = io::stdout();
    let mut reader = Utf8Reader::new(s);
    loop {
        let chunk = reader.read_utf8().await?;
        if chunk.is_empty() {
            break;
        } else {
            out.write_all(chunk.as_bytes()).await?;
        }
    }
    out.flush().await?;
    Ok(())
}

use bytes::BytesMut;
use futures::future;
use strchunk::StrChunk;
use tokio::prelude::*;
use tokio::runtime::Runtime;

use std::pin::Pin;
use std::task::{Context, Poll};

const DEFAULT_BUFFER_CAPACITY: usize = 8 * 1024;

pub struct Utf8Reader<R> {
    inner: R,
    buf: BytesMut,
}

impl<R: Unpin> Unpin for Utf8Reader<R> {}

impl<R> Utf8Reader<R> {
    fn split_borrows(self: Pin<&mut Self>) -> (Pin<&mut R>, &mut BytesMut) {
        unsafe {
            let this = self.get_unchecked_mut();
            (Pin::new_unchecked(&mut this.inner), &mut this.buf)
        }
    }

    pub fn new(inner: R) -> Self {
        Utf8Reader {
            inner,
            buf: BytesMut::with_capacity(DEFAULT_BUFFER_CAPACITY),
        }
    }
}

impl<R: AsyncRead> Utf8Reader<R> {
    pub fn poll_read_utf8(
        self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<io::Result<StrChunk>> {
        self.poll_read_more(cx).map(|res| {
            res.and_then(|buf| {
                StrChunk::extract_utf8(buf)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            })
        })
    }

    fn poll_read_more(
        self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<io::Result<&mut BytesMut>> {
        let (inner, buf) = self.split_borrows();
        buf.reserve(1);
        inner.poll_read_buf(cx, buf).map(|res| {
            res.and_then(|bytes_read| {
                if bytes_read == 0 && !buf.is_empty() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "incomplete UTF-8 sequence in input",
                    ));
                }
                Ok(buf)
            })
        })
    }
}

async fn forward_all<R, W>(input: R, mut output: W) -> io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut reader = Utf8Reader::new(input);
    loop {
        let chunk =
            future::poll_fn(|cx| Pin::new(&mut reader).poll_read_utf8(cx))
                .await?;
        if chunk.is_empty() {
            break;
        } else {
            output.write_all(chunk.as_str().as_bytes()).await?;
        }
    }
    output.flush().await
}

fn main() {
    let s = b"Hello, world!\n";
    let out = io::stdout();
    let mut runtime = Runtime::new().unwrap();
    runtime.block_on(forward_all(&s[..], out)).unwrap();
}

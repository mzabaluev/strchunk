use bytes::BytesMut;
use strchunk::StrChunk;
use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncWriteExt};

const DEFAULT_BUFFER_CAPACITY: usize = 8 * 1024;

pub struct Utf8Reader<R> {
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

fn extract_utf8_after_read(
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

impl<R: AsyncRead + Unpin> Utf8Reader<R> {
    async fn read_utf8(&mut self) -> io::Result<StrChunk> {
        debug_assert!(self.buf.capacity() >= 4);
        let bytes_read = self.inner.read_buf(&mut self.buf).await?;
        extract_utf8_after_read(bytes_read, &mut self.buf)
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

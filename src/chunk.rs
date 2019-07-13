use crate::chunk_mut::StrChunkMut;

use bytes::{Bytes, BytesMut, IntoBuf};
use range_split::TakeRange;

use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::Deref;
use std::ops::RangeBounds;
use std::str::{self, Utf8Error};

// macro
use range_split::assert_str_range;

/// A reference counted contiguous UTF-8 slice in memory.
///
/// `StrChunk` builds on the memory slice view semantics of `Bytes` from
/// the `bytes` crate, with the added guarantee that the content is a valid
/// UTF-8 string.
#[derive(Clone, Default, Eq, Ord)]
pub struct StrChunk {
    bytes: Bytes,
}

impl StrChunk {
    /// Creates a new empty `StrChunk`.
    ///
    /// This does not allocate and the returned `StrChunk` handle will be empty.
    #[inline]
    pub fn new() -> StrChunk {
        StrChunk {
            bytes: Bytes::new(),
        }
    }

    /// Creates a new `StrChunk` from a static string slice.
    ///
    /// This constructor works similarly to `Bytes::from_static`
    /// and uses the same internal optimizations.
    #[inline]
    pub fn from_static(s: &'static str) -> StrChunk {
        StrChunk {
            bytes: Bytes::from_static(s.as_bytes()),
        }
    }

    /// Returns the length of this `StrChunk` in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns true if the `StrChunk` has a length of 0.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Extracts UTF-8 content from a byte buffer.
    ///
    /// Extracts the longest leading part of the `BytesMut` view that
    /// validates as UTF-8, leaving `src` with the remainder.
    ///
    /// # Errors
    ///
    /// If an invalid UTF-8 sequence is encountered within `src`, an error
    /// value with recovery information is returned in the `Err` variant.
    /// The valid UTF-8 part preceding the invalid sequence is taken
    /// out of `src` and can be obtained from the `ExtractUtf8Error` value.
    ///
    /// # Example
    ///
    /// This function is intended to be used in decoding UTF-8 input from
    /// a byte stream, where the application would read data into a memory
    /// buffer managed under a `BytesMut` instance and then pass it to
    /// `StrChunk::extract_utf8` to consume complete UTF-8 chunks
    /// without copying the data.
    ///
    /// ```rust
    /// use bytes::{BufMut, BytesMut};
    /// use strchunk::StrChunk;
    /// use std::io::{self, Read};
    ///
    /// struct Utf8Reader<R> {
    ///     inner: R,
    ///     buf: BytesMut,
    /// }
    ///
    /// impl<R: Read> Utf8Reader<R> {
    ///     fn read_utf8(&mut self) -> io::Result<StrChunk> {
    ///         self.buf.reserve(1);
    ///         unsafe {
    ///             let bytes_read = self.inner.read(self.buf.bytes_mut())?;
    ///             self.buf.advance_mut(bytes_read);
    ///         }
    ///         StrChunk::extract_utf8(&mut self.buf)
    ///             .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    ///     }
    /// }
    /// #
    /// #   fn main() {
    /// #       let mut reader = Utf8Reader {
    /// #           inner: io::empty(),
    /// #           buf: BytesMut::new(),
    /// #       };
    /// #       let s = reader.read_utf8().unwrap();
    /// #       assert!(s.is_empty());
    /// #   }
    /// ```
    pub fn extract_utf8(
        src: &mut BytesMut,
    ) -> Result<StrChunk, ExtractUtf8Error> {
        match str::from_utf8(src) {
            Ok(_) => {
                // Valid UTF-8 fills the entire source buffer
                let bytes = src.take().freeze();
                Ok(StrChunk { bytes })
            }
            Err(e) => {
                let bytes = src.split_to(e.valid_up_to()).freeze();
                let extracted = StrChunk { bytes };
                match e.error_len() {
                    None => {
                        // Incomplete UTF-8 sequence seen at the end
                        Ok(extracted)
                    }
                    Some(error_len) => {
                        // Invalid UTF-8 encountered
                        Err(ExtractUtf8Error {
                            extracted,
                            error_len,
                        })
                    }
                }
            }
        }
    }

    /// Represents the `StrChunk` contents as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes) }
    }

    pub(crate) fn take_range<R>(&mut self, range: R) -> StrChunk
    where
        R: RangeBounds<usize> + Debug,
        Bytes: TakeRange<R, Output = Bytes>,
    {
        assert_str_range!(self.as_str(), range);
        let bytes = self.bytes.take_range(range);
        StrChunk { bytes }
    }

    pub(crate) fn remove_range<R>(&mut self, range: R)
    where
        R: RangeBounds<usize> + Debug,
        Bytes: TakeRange<R>,
    {
        assert_str_range!(self.as_str(), range);
        self.bytes.remove_range(range);
    }
}

impl Debug for StrChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl Display for StrChunk {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<String> for StrChunk {
    #[inline]
    fn from(src: String) -> StrChunk {
        StrChunk { bytes: src.into() }
    }
}

impl<'a> From<&'a str> for StrChunk {
    #[inline]
    fn from(src: &'a str) -> StrChunk {
        StrChunk { bytes: src.into() }
    }
}

impl TryFrom<Bytes> for StrChunk {
    type Error = Utf8Error;
    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        str::from_utf8(&bytes)?;
        Ok(StrChunk { bytes })
    }
}

impl TryFrom<BytesMut> for StrChunk {
    type Error = Utf8Error;
    fn try_from(bytes: BytesMut) -> Result<Self, Self::Error> {
        bytes.freeze().try_into()
    }
}

impl From<StrChunk> for Bytes {
    #[inline]
    fn from(src: StrChunk) -> Bytes {
        src.bytes
    }
}

impl From<StrChunk> for String {
    #[inline]
    fn from(src: StrChunk) -> String {
        String::from(src.as_str())
    }
}

impl AsRef<[u8]> for StrChunk {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

impl AsRef<str> for StrChunk {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for StrChunk {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Deref for StrChunk {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl Hash for StrChunk {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl IntoBuf for StrChunk {
    type Buf = <Bytes as IntoBuf>::Buf;

    #[inline]
    fn into_buf(self) -> Self::Buf {
        self.bytes.into_buf()
    }
}

impl<'a> IntoBuf for &'a StrChunk {
    type Buf = <&'a Bytes as IntoBuf>::Buf;

    #[inline]
    fn into_buf(self) -> Self::Buf {
        (&self.bytes).into_buf()
    }
}

impl FromIterator<char> for StrChunk {
    fn from_iter<T: IntoIterator<Item = char>>(into_iter: T) -> Self {
        StrChunkMut::from_iter(into_iter).into()
    }
}

/// An error returned by `StrChunk::extract_utf8`.
///
/// `ExtractUtf8Error` indicates an invalid UTF-8 sequence encountered
/// in the input and provides information necessary for lossy recovery
/// of an incremental UTF-8 decoding stream.
///
/// # Example
///
/// ```rust
/// # use bytes::BytesMut;
/// # use strchunk::StrChunk;
/// const TEST_DATA: &[u8] = b"Hello \xF0\x90\x80World";
/// let mut input = BytesMut::from(TEST_DATA);
/// let err = StrChunk::extract_utf8(&mut input).unwrap_err();
/// input.advance(err.error_len());
/// let chunk1 = err.into_extracted();
/// assert_eq!(chunk1, "Hello ");
/// // Can inject a replacement character into the output, e.g. U+FFFD
/// let chunk2 = StrChunk::extract_utf8(&mut input).unwrap();
/// assert_eq!(chunk2, "World");
/// ```
#[derive(Clone, Debug)]
pub struct ExtractUtf8Error {
    extracted: StrChunk,
    error_len: usize,
}

impl ExtractUtf8Error {
    /// Length of the invalid byte sequence.
    /// A lossy decoding procedure should advance the reading position
    /// by the returned amount using the `advance` method of the input buffer
    /// to resume decoding.
    pub fn error_len(&self) -> usize {
        self.error_len
    }

    /// Consumes `self` to obtain the string content extracted up to
    /// the encountered invalid sequence.
    pub fn into_extracted(self) -> StrChunk {
        self.extracted
    }
}

impl Display for ExtractUtf8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid UTF-8 sequence in input")
    }
}

impl Error for ExtractUtf8Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_bytes_via_deref() {
        let s = StrChunk::from_static("Hello");
        assert_eq!(s.as_bytes(), b"Hello");
    }
}

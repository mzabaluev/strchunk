use crate::chunk_mut::StrChunkMut;

use bytes::{Bytes, BytesMut};
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
    pub fn new() -> Self {
        StrChunk {
            bytes: Bytes::new(),
        }
    }

    /// Creates a new `StrChunk` from a static string slice.
    ///
    /// This constructor works similarly to `Bytes::from_static`
    /// and uses the same internal optimizations.
    #[inline]
    pub fn from_static(s: &'static str) -> Self {
        StrChunk {
            bytes: Bytes::from_static(s.as_bytes()),
        }
    }

    /// Creates a `StrChunk` instance from a string slice, by copying it.
    #[inline]
    pub fn copy_from_slice(s: &str) -> Self {
        StrChunk {
            bytes: Bytes::copy_from_slice(s.as_bytes()),
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
    /// Extracts the content of `src` that validates as UTF-8, from the
    /// beginning of the buffer up to a possibly incomplete UTF-8 sequence
    /// at the end, which is left in `src`.
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
    /// Basic usage:
    ///
    /// ```rust
    /// # use bytes::BytesMut;
    /// # use strchunk::StrChunk;
    /// let s1: &[u8] = b"\xd0\x97\xd0\xb4\xd1\x80\xd0\xb0\xd0";
    /// let mut buf = BytesMut::from(s1);
    ///
    /// let chunk = StrChunk::extract_utf8(&mut buf).unwrap();
    /// assert_eq!(chunk, "Здра");
    /// assert_eq!(buf, b"\xd0"[..]);
    ///
    /// let s2: &[u8] = b"\xb2\xd1\x81\xd1\x82\xd0\xb2\xd1\x83\xd0\xb9";
    /// buf.extend_from_slice(s2);
    ///
    /// let chunk = StrChunk::extract_utf8(&mut buf).unwrap();
    /// assert_eq!(chunk, "вствуй");
    /// assert!(buf.is_empty());
    /// ```
    ///
    /// This function is intended to be used in decoding UTF-8 input from
    /// a byte stream, where the application would read data into a memory
    /// buffer managed under a `BytesMut` instance and then pass it to
    /// `StrChunk::extract_utf8` to consume complete UTF-8 chunks
    /// without copying the data. The `async_read` example in this project
    /// provides a fully fledged demonstration using this method.
    pub fn extract_utf8(
        src: &mut BytesMut,
    ) -> Result<StrChunk, ExtractUtf8Error> {
        match str::from_utf8(src) {
            Ok(_) => {
                // Valid UTF-8 fills the entire source buffer
                let bytes = src.split().freeze();
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

    /// Returns a slice of self for the provided range.
    ///
    /// This will increment the reference count for the underlying memory and
    /// return a new `StrChunk` handle set to the slice.
    ///
    /// This operation is `O(1)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use strchunk::StrChunk;
    ///
    /// let a = StrChunk::from(&"hello world"[..]);
    /// let b = a.slice(2..5);
    ///
    /// assert_eq!(&b[..], "llo");
    /// ```
    ///
    /// # Panics
    ///
    /// If one or both of the range bounds are finite, they must be within
    /// the slice view of `self`. Furthermore, it is required that slicing
    /// occurs at UTF-8 code point boundaries. If either of these checks fails,
    /// this function panics.
    pub fn slice(&self, range: impl RangeBounds<usize>) -> StrChunk {
        assert_str_range!(self.as_str(), range);
        let bytes = self.bytes.slice(range);
        StrChunk { bytes }
    }

    /// Returns a slice of self that is equivalent to the given `sub` slice.
    ///
    /// When processing a `StrChunk` buffer with other tools, one often gets a
    /// `&str` which is in fact a slice of the `StrChunk`.
    /// This function turns that `&str` into another `StrChunk`, as if one had
    /// called `self.slice()` with the offsets that correspond to `sub`.
    ///
    /// This operation is `O(1)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use strchunk::StrChunk;
    ///
    /// let s = StrChunk::from(&"012345678"[..]);
    /// let as_slice = s.as_str();
    /// let subset = &as_slice[2..6];
    /// let subslice = s.slice_ref(&subset);
    /// assert_eq!(subslice, "2345");
    /// ```
    ///
    /// # Panics
    ///
    /// Requires that the given `sub` slice is in fact contained within the
    /// `StrChunk` buffer; otherwise this function will panic.
    #[inline]
    pub fn slice_ref(&self, sub: &str) -> StrChunk {
        // The slice must be valid UTF-8, no need for char boundary checks
        let bytes = self.bytes.slice_ref(sub.as_bytes());
        StrChunk { bytes }
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

impl From<&'static str> for StrChunk {
    #[inline]
    fn from(src: &'static str) -> StrChunk {
        StrChunk { bytes: src.into() }
    }
}

impl From<String> for StrChunk {
    #[inline]
    fn from(src: String) -> StrChunk {
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
/// # use bytes::{BytesMut, Buf};
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

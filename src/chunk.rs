use crate::chunk_mut::StrChunkMut;

use bytes::{Bytes, BytesMut, IntoBuf};
use range_split::TakeRange;

use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::iter::FromIterator;
use std::ops::Deref;
use std::ops::RangeBounds;
use std::str::{self, Utf8Error};

// macro
use range_split::assert_str_range;

#[derive(Clone, Default, Eq, Ord)]
pub struct StrChunk {
    bytes: Bytes,
}

impl StrChunk {
    #[inline]
    pub fn new() -> StrChunk {
        StrChunk {
            bytes: Bytes::new(),
        }
    }

    #[inline]
    pub fn from_static(s: &'static str) -> StrChunk {
        StrChunk {
            bytes: Bytes::from_static(s.as_bytes()),
        }
    }

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

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes) }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl Display for StrChunk {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    type Buf = Cursor<Bytes>;

    #[inline]
    fn into_buf(self) -> Self::Buf {
        self.bytes.into_buf()
    }
}

impl<'a> IntoBuf for &'a StrChunk {
    type Buf = Cursor<&'a Bytes>;

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

#[derive(Clone, Debug)]
pub struct ExtractUtf8Error {
    extracted: StrChunk,
    error_len: usize,
}

impl ExtractUtf8Error {
    pub fn into_extracted(self) -> StrChunk {
        self.extracted
    }

    pub fn error_len(&self) -> usize {
        self.error_len
    }
}

impl Display for ExtractUtf8Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid UTF-8 sequence in input")
    }
}

impl Error for ExtractUtf8Error {}

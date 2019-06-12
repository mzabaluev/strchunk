use crate::chunk_mut::StrChunkMut;
use crate::split::TakeRange;

use bytes::{Bytes, BytesMut, IntoBuf};

use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::io::Cursor;
use std::iter::FromIterator;
use std::ops::Deref;
use std::ops::{RangeFrom, RangeFull, RangeTo, RangeToInclusive};
use std::str::{self, Utf8Error};


#[cfg_attr(not(feature = "specialization"), derive(PartialEq))]
#[derive(Clone, Default, Eq, PartialOrd, Ord, Hash)]
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

impl TakeRange<RangeFull> for StrChunk {
    type Output = StrChunk;

    fn take_range(&mut self, _: RangeFull) -> StrChunk {
        let bytes = self.bytes.split_off(0);
        StrChunk { bytes }
    }

    fn remove_range(&mut self, _: RangeFull) {
        self.bytes.clear()
    }
}

impl TakeRange<RangeFrom<usize>> for StrChunk {
    type Output = StrChunk;

    fn take_range(&mut self, range: RangeFrom<usize>) -> Self::Output {
        validate_str_range!(self.as_str(), &range);
        let bytes = self.bytes.split_off(range.start);
        StrChunk { bytes }
    }

    fn remove_range(&mut self, range: RangeFrom<usize>) {
        validate_str_range!(self.as_str(), &range);
        self.bytes.truncate(range.start)
    }
}

impl TakeRange<RangeTo<usize>> for StrChunk {
    type Output = StrChunk;

    fn take_range(&mut self, range: RangeTo<usize>) -> Self::Output {
        validate_str_range!(self.as_str(), &range);
        let bytes = self.bytes.split_to(range.end);
        StrChunk { bytes }
    }

    fn remove_range(&mut self, range: RangeTo<usize>) {
        validate_str_range!(self.as_str(), &range);
        self.bytes.advance(range.end)
    }
}

impl TakeRange<RangeToInclusive<usize>> for StrChunk {
    type Output = StrChunk;

    fn take_range(&mut self, range: RangeToInclusive<usize>) -> Self::Output {
        validate_str_range!(self.as_str(), &range);
        let bytes = self.bytes.split_to(range.end + 1);
        StrChunk { bytes }
    }

    fn remove_range(&mut self, range: RangeToInclusive<usize>) {
        validate_str_range!(self.as_str(), &range);
        self.bytes.advance(range.end + 1)
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

#[cfg(test)]
mod tests {
    use super::StrChunk;
    use crate::split::TakeRange;

    #[should_panic]
    #[test]
    fn take_panic_oob() {
        let mut buf = StrChunk::from("Hello");
        let _ = buf.take_range(..6);
    }

    #[should_panic]
    #[test]
    fn take_panic_split_utf8() {
        let mut buf = StrChunk::from("Привет");
        let _ = buf.take_range(3..);
    }
}

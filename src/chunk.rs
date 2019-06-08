use crate::chunk_mut::StrChunkMut;
use crate::split::{BindSlice, SplitRange, Take};

use bytes::{Bytes, BytesMut, IntoBuf};

use std::{
    borrow::Borrow,
    error::Error,
    fmt::{self, Debug, Display},
    io::Cursor,
    iter::FromIterator,
    ops::Deref,
    str,
};

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
    ) -> Result<Option<StrChunk>, ExtractUtf8Error> {
        match str::from_utf8(src) {
            Ok(_) => {
                // Valid UTF-8 fills the entire source buffer
                let bytes = src.take().freeze();
                Ok(Some(StrChunk { bytes }))
            }
            Err(e) => {
                let valid_len = e.valid_up_to();
                let extracted = if valid_len == 0 {
                    None
                } else {
                    let bytes = src.split_to(valid_len).freeze();
                    Some(StrChunk { bytes })
                };
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

impl Take for StrChunk {
    type Slice = str;
    type Output = StrChunk;

    fn take_range<R>(&mut self, range: R) -> StrChunk
    where
        R: BindSlice<str>,
    {
        let bytes = match range.bind_slice(self.as_str()) {
            SplitRange::Full(_) => self.bytes.split_off(0),
            SplitRange::From(r) => self.bytes.split_off(r.start),
            SplitRange::To(r) => self.bytes.split_to(r.end),
        };
        StrChunk { bytes }
    }

    fn remove_range<R>(&mut self, range: R)
    where
        R: BindSlice<str>,
    {
        match range.bind_slice(self.as_str()) {
            SplitRange::Full(_) => self.bytes.clear(),
            SplitRange::From(r) => self.bytes.truncate(r.start),
            SplitRange::To(r) => self.bytes.advance(r.end),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExtractUtf8Error {
    extracted: Option<StrChunk>,
    error_len: usize,
}

impl ExtractUtf8Error {
    pub fn into_extracted(self) -> Option<StrChunk> {
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
    use crate::split::Take;

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

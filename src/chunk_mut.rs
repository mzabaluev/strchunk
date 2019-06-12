use crate::chunk::StrChunk;
use crate::split::TakeRange;

use bytes::{BufMut, Bytes, BytesMut, IntoBuf};

use std::borrow::{Borrow, BorrowMut};
use std::convert::TryFrom;
use std::fmt::{self, Debug, Display};
use std::io::Cursor;
use std::iter::{FromIterator, Iterator};
use std::ops::{RangeFrom, RangeFull, RangeTo, RangeToInclusive};
use std::str::{self, Utf8Error};

#[cfg_attr(not(feature = "specialization"), derive(PartialEq))]
#[derive(Clone, Default, Eq, PartialOrd, Ord, Hash)]
pub struct StrChunkMut {
    bytes: BytesMut,
}

impl StrChunkMut {
    #[inline]
    pub fn new() -> Self {
        StrChunkMut {
            bytes: BytesMut::new(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        StrChunkMut {
            bytes: BytesMut::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.bytes.capacity()
    }

    #[inline]
    pub fn remaining_mut(&self) -> usize {
        self.bytes.remaining_mut()
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.bytes.reserve(additional)
    }

    #[inline]
    pub fn freeze(self) -> StrChunk {
        StrChunk::from(self)
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&*self.bytes) }
    }

    #[inline]
    pub fn as_mut_str(&mut self) -> &mut str {
        unsafe { str::from_utf8_unchecked_mut(&mut *self.bytes) }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline]
    pub fn put_char(&mut self, c: char) {
        let bytes = &mut self.bytes;
        unsafe {
            let utf8_len = c.encode_utf8(bytes.bytes_mut()).len();
            bytes.advance_mut(utf8_len);
        }
    }

    fn from_iter_internal<T>(iter: T) -> Self
    where
        T: Iterator<Item = char>,
    {
        // Reserve at least as many bytes as there are promised to be
        // characters, plus some overhead so that the reserve call in the loop
        // never reallocates in the ideal case of one byte per character.
        // If the size hint is 0, the first iteration should not reallocate,
        // either.
        let cap = iter.size_hint().0.saturating_add(4);
        let mut buf = StrChunkMut::with_capacity(cap);
        for c in iter {
            buf.reserve(4);
            buf.put_char(c);
        }
        buf
    }
}

impl Debug for StrChunkMut {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl Display for StrChunkMut {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<String> for StrChunkMut {
    #[inline]
    fn from(src: String) -> StrChunkMut {
        StrChunkMut { bytes: src.into() }
    }
}

impl<'a> From<&'a str> for StrChunkMut {
    #[inline]
    fn from(src: &'a str) -> StrChunkMut {
        StrChunkMut { bytes: src.into() }
    }
}

impl TryFrom<Bytes> for StrChunkMut {
    type Error = Utf8Error;
    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        str::from_utf8(&bytes)?;
        Ok(StrChunkMut {
            bytes: bytes.into(),
        })
    }
}

impl TryFrom<BytesMut> for StrChunkMut {
    type Error = Utf8Error;
    fn try_from(bytes: BytesMut) -> Result<Self, Self::Error> {
        str::from_utf8(&bytes)?;
        Ok(StrChunkMut { bytes })
    }
}

impl From<StrChunkMut> for StrChunk {
    #[inline]
    fn from(src: StrChunkMut) -> StrChunk {
        src.freeze()
    }
}

impl From<StrChunkMut> for Bytes {
    #[inline]
    fn from(src: StrChunkMut) -> Bytes {
        src.bytes.freeze()
    }
}

impl From<StrChunkMut> for BytesMut {
    #[inline]
    fn from(src: StrChunkMut) -> BytesMut {
        src.bytes
    }
}

impl AsRef<[u8]> for StrChunkMut {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

impl AsRef<str> for StrChunkMut {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsMut<str> for StrChunkMut {
    #[inline]
    fn as_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl Borrow<str> for StrChunkMut {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl BorrowMut<str> for StrChunkMut {
    #[inline]
    fn borrow_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl IntoBuf for StrChunkMut {
    type Buf = Cursor<BytesMut>;

    #[inline]
    fn into_buf(self) -> Self::Buf {
        self.bytes.into_buf()
    }
}

impl<'a> IntoBuf for &'a StrChunkMut {
    type Buf = Cursor<&'a BytesMut>;

    #[inline]
    fn into_buf(self) -> Self::Buf {
        (&self.bytes).into_buf()
    }
}

impl FromIterator<char> for StrChunkMut {
    fn from_iter<T: IntoIterator<Item = char>>(into_iter: T) -> Self {
        StrChunkMut::from_iter_internal(into_iter.into_iter())
    }
}

impl TakeRange<RangeFull> for StrChunkMut {
    type Output = StrChunkMut;

    fn take_range(&mut self, _: RangeFull) -> Self::Output {
        let bytes = self.bytes.take();
        StrChunkMut { bytes }
    }

    fn remove_range(&mut self, _: RangeFull) {
        self.bytes.clear()
    }
}

impl TakeRange<RangeFrom<usize>> for StrChunkMut {
    type Output = StrChunkMut;

    fn take_range(&mut self, range: RangeFrom<usize>) -> Self::Output {
        validate_str_range!(self.as_str(), &range);
        let bytes = self.bytes.split_off(range.start);
        StrChunkMut { bytes }
    }

    fn remove_range(&mut self, range: RangeFrom<usize>) {
        validate_str_range!(self.as_str(), &range);
        self.bytes.truncate(range.start)
    }
}

impl TakeRange<RangeTo<usize>> for StrChunkMut {
    type Output = StrChunkMut;

    fn take_range(&mut self, range: RangeTo<usize>) -> Self::Output {
        validate_str_range!(self.as_str(), &range);
        let bytes = self.bytes.split_to(range.end);
        StrChunkMut { bytes }
    }

    fn remove_range(&mut self, range: RangeTo<usize>) {
        validate_str_range!(self.as_str(), &range);
        self.bytes.advance(range.end)
    }
}

impl TakeRange<RangeToInclusive<usize>> for StrChunkMut {
    type Output = StrChunkMut;

    fn take_range(&mut self, range: RangeToInclusive<usize>) -> Self::Output {
        validate_str_range!(self.as_str(), &range);
        let bytes = self.bytes.split_to(range.end + 1);
        StrChunkMut { bytes }
    }

    fn remove_range(&mut self, range: RangeToInclusive<usize>) {
        validate_str_range!(self.as_str(), &range);
        self.bytes.advance(range.end + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::StrChunkMut;
    use crate::split::TakeRange;

    #[should_panic]
    #[test]
    fn take_panic_oob() {
        let mut buf = StrChunkMut::from("Hello");
        let _ = buf.take_range(..6);
    }

    #[should_panic]
    #[test]
    fn take_panic_split_utf8() {
        let mut buf = StrChunkMut::from("Привет");
        let _ = buf.take_range(3..);
    }
}

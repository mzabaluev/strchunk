use crate::chunk::StrChunk;

use bytes::{BufMut, Bytes, BytesMut};
use range_split::TakeRange;

use std::borrow::{Borrow, BorrowMut};
use std::convert::TryFrom;
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use std::iter::{FromIterator, Iterator};
use std::ops::RangeBounds;
use std::ops::{Deref, DerefMut};
use std::slice;
use std::str::{self, Utf8Error};

// macro
use range_split::assert_str_range;

/// A unique reference to a contiguous UTF-8 slice in memory.
///
/// `StrChunkMut` builds on the memory slice view semantics of `BytesMut` from
/// the `bytes` crate, with the added guarantee that the content is a valid
/// UTF-8 string.
#[derive(Clone, Default, Eq, Ord)]
pub struct StrChunkMut {
    bytes: BytesMut,
}

impl StrChunkMut {
    /// Creates a new `StrChunkMut` with default capacity.
    ///
    /// The returned buffer has initialized length 0 and unspecified
    /// capacity.
    #[inline]
    pub fn new() -> Self {
        StrChunkMut {
            bytes: BytesMut::new(),
        }
    }

    /// Creates a new `StrChunkMut` with the specified capacity.
    ///
    /// The returned buffer will be able to hold strings with lengths of
    /// at least `capacity` without reallocating. The initialized length
    /// is 0.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        StrChunkMut {
            bytes: BytesMut::with_capacity(capacity),
        }
    }

    /// Returns the length of the initialized string content in this
    /// `StrChunkMut`.
    #[inline]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns true if the initialized string content in `StrChunkMut`
    /// has a length of 0.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Returns the maximum length of a string this `StrChunkMut` can hold
    /// without reallocating.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.bytes.capacity()
    }

    /// Returns the maximum length of string content that can be appended
    /// past the current length without reallocating.
    #[inline]
    pub fn remaining_mut(&self) -> usize {
        self.bytes.remaining_mut()
    }

    /// Reserves capacity for at least `additional` more bytes of string
    /// content to be inserted into this `StrChunkMut`.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.bytes.reserve(additional)
    }

    /// Converts `self` into an immutable `StrChunk`.
    ///
    /// The conversion is zero cost and is used to indicate that the slice
    /// referenced by the handle will no longer be mutated.
    /// Once the conversion is done, the handle can be cloned and shared
    /// across threads.
    #[inline]
    pub fn freeze(self) -> StrChunk {
        StrChunk::from(self)
    }

    /// Represents the `StrChunkMut` contents as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes) }
    }

    /// Represents the `StrChunkMut` contents as a mutable string slice.
    #[inline]
    pub fn as_mut_str(&mut self) -> &mut str {
        unsafe { str::from_utf8_unchecked_mut(&mut self.bytes) }
    }

    /// Appends a Unicode character, encoded into UTF-8, to the initialized
    /// string contents of the `StrChunkMut`.
    ///
    /// # Panics
    ///
    /// Panics if the remaining capacity is not sufficient to encode the
    /// character. Four bytes are enough to encode any `char`.
    #[inline]
    pub fn put_char(&mut self, c: char) {
        let buf = self.bytes.bytes_mut();
        // Safety: OK to transmute from &mut UninitSlice here
        // because encode_utf8 only writes to the buffer, and
        // the cursor is then advanced by the number of bytes written.
        // If encode_utf8 panics, we assume that its unwind path never reads
        // from uninitialized bytes in the buffer, neither does unwind code
        // in this stack frame.
        unsafe {
            let buf = slice::from_raw_parts_mut(buf.as_mut_ptr(), buf.len());
            let s = c.encode_utf8(buf);
            self.bytes.advance_mut(s.len());
        }
    }

    /// Appends a string slice to the initialized string contents of the
    /// `StrChunkMut`.
    ///
    /// # Panics
    ///
    /// Panics if the remaining capacity is not sufficient.
    ///
    pub fn put_str<S: AsRef<str>>(&mut self, string: S) {
        self.bytes.put_slice(string.as_ref().as_bytes())
    }

    fn from_iter_chars<T>(mut iter: T) -> Self
    where
        T: Iterator<Item = char>,
    {
        let ch = match iter.next() {
            None => return StrChunkMut::new(),
            Some(ch) => ch,
        };
        // Reserve at least as many bytes as there are promised to be
        // characters, plus some overhead so that the reserve call in the loop
        // never reallocates in the ideal case of one byte per character.
        // If the iterator returns 0 as an inexact size hint, the first
        // iteration should not reallocate, too.
        let cap = iter.size_hint().0.saturating_add(5);
        let mut buf = StrChunkMut::with_capacity(cap);
        buf.extend_chars_loop(ch, iter);
        buf
    }

    fn extend_chars<I>(&mut self, mut iter: I)
    where
        I: Iterator<Item = char>,
    {
        let ch = match iter.next() {
            None => return,
            Some(ch) => ch,
        };
        // Reserve at least as many bytes as there are promised to be
        // characters, plus some overhead so that the reserve call in the loop
        // never reallocates in the ideal case of one byte per character.
        // If the iterator returns 0 as an inexact size hint, the first
        // iteration should not reallocate, too.
        self.reserve(iter.size_hint().0.saturating_add(5));
        self.extend_chars_loop(ch, iter);
    }

    fn extend_chars_loop<I>(&mut self, mut ch: char, mut iter: I)
    where
        I: Iterator<Item = char>,
    {
        loop {
            self.put_char(ch);
            ch = match iter.next() {
                None => return,
                Some(ch) => ch,
            };
            self.reserve(4);
        }
    }

    fn extend_strs<'a, I>(&mut self, iter: I)
    where
        I: Iterator<Item = &'a str>,
    {
        for s in iter {
            self.reserve(s.len());
            self.put_str(s);
        }
    }

    pub(crate) fn take_range<R>(&mut self, range: R) -> StrChunkMut
    where
        R: RangeBounds<usize> + Debug,
        BytesMut: TakeRange<R, Output = BytesMut>,
    {
        assert_str_range!(self.as_str(), range);
        let bytes = self.bytes.take_range(range);
        StrChunkMut { bytes }
    }

    pub(crate) fn remove_range<R>(&mut self, range: R)
    where
        R: RangeBounds<usize> + Debug,
        BytesMut: TakeRange<R>,
    {
        assert_str_range!(self.as_str(), range);
        self.bytes.remove_range(range);
    }
}

impl Debug for StrChunkMut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl Display for StrChunkMut {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> From<&'a str> for StrChunkMut {
    #[inline]
    fn from(src: &'a str) -> StrChunkMut {
        StrChunkMut { bytes: src.into() }
    }
}

impl TryFrom<BytesMut> for StrChunkMut {
    type Error = Utf8Error;

    #[inline]
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

impl Deref for StrChunkMut {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl DerefMut for StrChunkMut {
    #[inline]
    fn deref_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl Hash for StrChunkMut {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl FromIterator<char> for StrChunkMut {
    fn from_iter<T>(iterable: T) -> Self
    where
        T: IntoIterator<Item = char>,
    {
        StrChunkMut::from_iter_chars(iterable.into_iter())
    }
}

impl Extend<char> for StrChunkMut {
    fn extend<T>(&mut self, iterable: T)
    where
        T: IntoIterator<Item = char>,
    {
        self.extend_chars(iterable.into_iter())
    }
}

impl<'a> FromIterator<&'a str> for StrChunkMut {
    fn from_iter<T>(iterable: T) -> Self
    where
        T: IntoIterator<Item = &'a str>,
    {
        let mut buf = StrChunkMut::new();
        buf.extend_strs(iterable.into_iter());
        buf
    }
}

impl<'a> Extend<&'a str> for StrChunkMut {
    fn extend<T>(&mut self, iterable: T)
    where
        T: IntoIterator<Item = &'a str>,
    {
        self.extend_strs(iterable.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_bytes_via_deref() {
        let s = StrChunkMut::from("Hello");
        assert_eq!(s.as_bytes(), b"Hello");
    }

    #[test]
    fn as_bytes_mut_via_deref_mut() {
        let mut s = StrChunkMut::from("Hello");
        let bytes = unsafe { s.as_bytes_mut() };
        assert_eq!(bytes, b"Hello");
    }
}

use bytes::{Bytes, BytesMut, IntoBuf};
use std::{
    borrow::Borrow,
    fmt::{self, Debug, Display},
    io::Cursor,
    iter::FromIterator,
    ops::Deref,
    str::{self, Utf8Error},
};

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StrChunk {
    bytes: Bytes,
}

impl StrChunk {
    pub fn from_static(s: &'static str) -> StrChunk {
        StrChunk {
            bytes: Bytes::from_static(s.as_bytes()),
        }
    }

    pub fn extract_utf8(
        src: &mut BytesMut,
    ) -> Result<Option<StrChunk>, Utf8Error> {
        match str::from_utf8(src) {
            Ok(_) => {
                // Valid UTF-8 fills the entire source buffer
                let bytes = src.take().into();
                Ok(Some(StrChunk { bytes }))
            }
            Err(e) => {
                match e.error_len() {
                    None => {
                        // Incomplete UTF-8 sequence seen at the end
                        let valid_len = e.valid_up_to();
                        if valid_len == 0 {
                            Ok(None)
                        } else {
                            let bytes = src.split_to(valid_len).into();
                            Ok(Some(StrChunk { bytes }))
                        }
                    }
                    Some(_) => {
                        // Invalid UTF-8 encountered
                        Err(e)
                    }
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes) }
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
    fn from(src: String) -> StrChunk {
        StrChunk { bytes: src.into() }
    }
}

impl<'a> From<&'a str> for StrChunk {
    fn from(src: &'a str) -> StrChunk {
        StrChunk { bytes: src.into() }
    }
}

impl FromIterator<char> for StrChunk {
    fn from_iter<T: IntoIterator<Item = char>>(into_iter: T) -> Self {
        let iter = into_iter.into_iter();
        // Reserve at least as many bytes as there are promised to be characters
        let mut bytes = BytesMut::with_capacity(iter.size_hint().0);
        let mut buf: [u8; 4] = [0; 4];
        for c in iter {
            let utf8 = c.encode_utf8(&mut buf);
            bytes.extend_from_slice(utf8.as_bytes());
        }
        StrChunk {
            bytes: bytes.into(),
        }
    }
}

impl From<StrChunk> for Bytes {
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
        self.as_ref()
    }
}

impl IntoBuf for StrChunk {
    type Buf = Cursor<Bytes>;

    fn into_buf(self) -> Self::Buf {
        self.bytes.into_buf()
    }
}

impl<'a> IntoBuf for &'a StrChunk {
    type Buf = Cursor<&'a Bytes>;

    fn into_buf(self) -> Self::Buf {
        (&self.bytes).into_buf()
    }
}

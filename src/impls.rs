#[cfg(feature = "specialization")]
mod generic {
    use std::borrow::Borrow;
    use {StrChunk, StrChunkMut};

    impl<Rhs> PartialEq<Rhs> for StrChunk
    where
        Rhs: ?Sized + Borrow<str>,
    {
        default fn eq(&self, other: &Rhs) -> bool {
            **self == *other.borrow()
        }
    }

    impl<Rhs> PartialEq<Rhs> for StrChunkMut
    where
        Rhs: ?Sized + Borrow<str>,
    {
        default fn eq(&self, other: &Rhs) -> bool {
            Borrow::<str>::borrow(self) == other.borrow()
        }
    }
}

macro_rules! for_all_str_types {
    ($macro:ident! for $T:ty) => {
        $macro! { $T, str }
        $macro! { $T, &'a str }
        $macro! { $T, String }
        $macro! { $T, Cow<'a, str> }
    };
}

#[cfg(not(feature = "specialization"))]
mod tedious {
    use std::borrow::{Borrow, Cow};
    use {StrChunk, StrChunkMut};

    macro_rules! impl_partial_eq {
        ($T:ty, $Rhs:ty) => {
            impl<'a> PartialEq<$Rhs> for $T {
                #[inline]
                fn eq(&self, other: &$Rhs) -> bool {
                    Borrow::<str>::borrow(self) == &other[..]
                }
            }
        };
    }

    for_all_str_types! { impl_partial_eq! for StrChunk }
    for_all_str_types! { impl_partial_eq! for StrChunkMut }
}

mod foreign {
    use std::borrow::Cow;
    use {StrChunk, StrChunkMut};

    macro_rules! impl_partial_eq_rhs {
        ($T:ty, $Lhs:ty) => {
            impl<'a> PartialEq<$T> for $Lhs {
                #[inline]
                fn eq(&self, other: &$T) -> bool {
                    other == self
                }
            }
        };
    }

    for_all_str_types! { impl_partial_eq_rhs! for StrChunk }
    for_all_str_types! { impl_partial_eq_rhs! for StrChunkMut }
}

#[cfg(test)]
mod tests {

    const TEST_STR: &'static str = "Hello";

    macro_rules! test_all_str_types {
        ($macro:ident!, $v:expr) => {
            $macro! { within_type, $v, $v }
            $macro! { str, $v, *TEST_STR }
            $macro! { str_ref, $v, TEST_STR }
            $macro! { string, $v, String::from(TEST_STR) }
            $macro! { cow_borrowed, $v, Cow::from(TEST_STR) }
            $macro! { cow_owned, $v, Cow::from(String::from(TEST_STR)) }
        };
    }

    mod eq {

        macro_rules! test_eq {
            ($name:ident, $arg1:expr, $arg2:expr) => {
                #[test]
                fn $name() {
                    assert_eq!($arg1, $arg2);
                    assert_eq!($arg2, $arg1);
                }
            };
        }

        mod chunk {
            use super::super::TEST_STR;
            use std::borrow::Cow;
            use StrChunk;

            test_all_str_types! { test_eq!, StrChunk::from_static(TEST_STR) }
        }

        mod chunk_mut {
            use super::super::TEST_STR;
            use std::borrow::Cow;
            use StrChunkMut;

            test_all_str_types! { test_eq!, StrChunkMut::from(TEST_STR) }
        }
    }
}

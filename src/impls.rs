use crate::{StrChunk, StrChunkMut};

use range_split::TakeRange;

use std::ops::{RangeFrom, RangeFull, RangeTo, RangeToInclusive};

// A generic impl implemented through the intrinsic take_range/remove_range
// would be enough for the purposes of this crate, but it would commit to
// Bytes as the internal representation and limit any third-party trait
// implementations to those implemented via Bytes. Also, the concrete impls
// make better documentation.
macro_rules! impl_take_range {
    (<$Range:ty> for $T:path) => {
        impl TakeRange<$Range> for $T {
            type Output = $T;

            fn take_range(&mut self, range: $Range) -> Self::Output {
                Self::take_range(self, range)
            }

            fn remove_range(&mut self, range: $Range) {
                Self::remove_range(self, range)
            }
        }
    };
}

impl_take_range!(<RangeFull> for StrChunk);
impl_take_range!(<RangeFrom<usize>> for StrChunk);
impl_take_range!(<RangeTo<usize>> for StrChunk);
impl_take_range!(<RangeToInclusive<usize>> for StrChunk);
impl_take_range!(<RangeFull> for StrChunkMut);
impl_take_range!(<RangeFrom<usize>> for StrChunkMut);
impl_take_range!(<RangeTo<usize>> for StrChunkMut);
impl_take_range!(<RangeToInclusive<usize>> for StrChunkMut);

#[cfg(feature = "specialization")]
mod generic {

    use crate::{StrChunk, StrChunkMut};
    use std::borrow::Borrow;
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
        $macro! { $T, ::std::borrow::Cow<'a, str> }
    };
}

#[cfg(not(feature = "specialization"))]
mod tedious {
    use crate::{StrChunk, StrChunkMut};
    use std::borrow::Borrow;

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
    use crate::{StrChunk, StrChunkMut};

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

    mod take_range {
        macro_rules! test_take_range_effects_with {
            ($func:expr) => {
                #[test]
                fn full() {
                    let mut buf = "Hello".into();
                    $func(&mut buf, .., "Hello", "");
                }

                #[test]
                fn from_start() {
                    let mut buf = "Hello".into();
                    $func(&mut buf, 0.., "Hello", "");
                }

                #[test]
                fn from_end() {
                    let mut buf = "Hello".into();
                    $func(&mut buf, 5.., "", "Hello");
                }

                #[test]
                fn from_mid() {
                    let mut buf = "Привет".into();
                    $func(&mut buf, 6.., "вет", "При");
                }

                #[test]
                fn to_start() {
                    let mut buf = "Hello".into();
                    $func(&mut buf, ..0, "", "Hello");
                }

                #[test]
                fn to_end() {
                    let mut buf = "Hello".into();
                    $func(&mut buf, ..5, "Hello", "");
                }

                #[test]
                fn to_mid() {
                    let mut buf = "Привет".into();
                    $func(&mut buf, ..6, "При", "вет");
                }

                #[test]
                fn to_inclusive_end() {
                    let mut buf = "Hello".into();
                    $func(&mut buf, ..=4, "Hello", "");
                }

                #[test]
                fn to_inclusive_mid() {
                    let mut buf = "Привет".into();
                    $func(&mut buf, ..=5, "При", "вет");
                }
            };
        }

        macro_rules! test_take_range_panics_with {
            ($func:expr) => {
                #[test]
                #[should_panic]
                fn panics_on_oob_start() {
                    let mut buf = "Hello".into();
                    $func(&mut buf, 6..);
                }

                #[test]
                #[should_panic]
                fn panics_on_oob_end() {
                    let mut buf = "Hello".into();
                    $func(&mut buf, ..6);
                }

                #[test]
                #[should_panic]
                fn panics_on_oob_inclusive_end() {
                    let mut buf = "Hello".into();
                    $func(&mut buf, ..=5);
                }

                #[test]
                #[should_panic]
                fn panics_on_split_utf8_start() {
                    let mut buf = "Привет".into();
                    $func(&mut buf, 3..);
                }

                #[test]
                #[should_panic]
                fn panics_on_split_utf8_end() {
                    let mut buf = "Привет".into();
                    $func(&mut buf, ..3);
                }

                #[test]
                #[should_panic]
                fn panics_on_split_utf8_inclusive_end() {
                    let mut buf = "Привет".into();
                    $func(&mut buf, ..=2);
                }
            };
        }

        macro_rules! test_take_range_for {
            ($T:ty) => {
                mod take_range {
                    use range_split::TakeRange;

                    test_take_range_effects_with!(
                        |buf: &mut $T,
                         range,
                         expected_output,
                         expected_remainder| {
                            let method_dbg =
                                format!("take_range({:?})", &range);
                            let output = TakeRange::take_range(buf, range);
                            assert_eq!(
                                output,
                                expected_output,
                                "expected output of `{}` for `{}`",
                                method_dbg,
                                stringify!($T)
                            );
                            assert_eq!(
                                buf,
                                expected_remainder,
                                "expected buffer content after `{}` for `{}`",
                                method_dbg,
                                stringify!($T)
                            );
                        }
                    );

                    test_take_range_panics_with!(|buf: &mut $T, range| {
                        TakeRange::take_range(buf, range)
                    });
                }

                mod remove_range {
                    use range_split::TakeRange;

                    test_take_range_effects_with!(
                        |buf: &mut $T, range, _, expected_remainder| {
                            let method_dbg =
                                format!("remove_range({:?})", &range);
                            TakeRange::remove_range(buf, range);
                            assert_eq!(
                                buf,
                                expected_remainder,
                                "expected buffer content after `{}` for `{}`",
                                method_dbg,
                                stringify!($T)
                            );
                        }
                    );

                    test_take_range_panics_with!(|buf: &mut $T, range| {
                        TakeRange::remove_range(buf, range);
                    });
                }
            };
        }

        mod chunk {
            test_take_range_for!(crate::StrChunk);
        }
        mod chunk_mut {
            test_take_range_for!(crate::StrChunkMut);
        }
    }

    const TEST_STR: &'static str = "Hello";

    macro_rules! test_all_str_types {
        ($macro:ident!, $v:expr) => {
            $macro! { within_type, $v, $v }
            $macro! { str, $v, *TEST_STR }
            $macro! { str_ref, $v, TEST_STR }
            $macro! { string, $v, String::from(TEST_STR) }
            $macro! { cow_borrowed, $v, ::std::borrow::Cow::from(TEST_STR) }
            $macro! { cow_owned, $v, ::std::borrow::Cow::from(String::from(TEST_STR)) }
        };
    }

    mod eq {
        use super::*;

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
            use super::*;
            use crate::StrChunk;

            test_all_str_types! { test_eq!, StrChunk::from_static(TEST_STR) }
        }

        mod chunk_mut {
            use super::*;
            use crate::StrChunkMut;

            test_all_str_types! { test_eq!, StrChunkMut::from(TEST_STR) }
        }
    }
}

#![cfg_attr(feature = "specialization", allow(unused_macros))]

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
    use std::cmp::Ordering;

    impl<Rhs> PartialEq<Rhs> for StrChunk
    where
        Rhs: ?Sized + Borrow<str>,
    {
        default fn eq(&self, other: &Rhs) -> bool {
            self.as_str() == other.borrow()
        }
    }

    impl<Rhs> PartialEq<Rhs> for StrChunkMut
    where
        Rhs: ?Sized + Borrow<str>,
    {
        default fn eq(&self, other: &Rhs) -> bool {
            self.as_str() == other.borrow()
        }
    }

    impl<Rhs> PartialOrd<Rhs> for StrChunk
    where
        Rhs: ?Sized + Borrow<str>,
    {
        default fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
            PartialOrd::partial_cmp(self.as_str(), other.borrow())
        }
    }

    impl<Rhs> PartialOrd<Rhs> for StrChunkMut
    where
        Rhs: ?Sized + Borrow<str>,
    {
        default fn partial_cmp(&self, other: &Rhs) -> Option<Ordering> {
            PartialOrd::partial_cmp(self.as_str(), other.borrow())
        }
    }
}

macro_rules! for_all_foreign_str_types {
    {
        $impl_macro:ident! for $T:ty
    } => {
        $impl_macro! { impl <str> for $T }
        $impl_macro! { impl<'a> <&'a str> for $T }
        $impl_macro! { impl <String> for $T }
        $impl_macro! { impl<'a> <::std::borrow::Cow<'a, str>> for $T }
    };
}

macro_rules! for_all_str_types {
    {
        $impl_macro:ident! for $T:ty
    } => {
        $impl_macro! { impl <crate::StrChunk> for $T }
        $impl_macro! { impl <crate::StrChunkMut> for $T }
        for_all_foreign_str_types! { $impl_macro! for $T }
    };
}

#[cfg(not(feature = "specialization"))]
mod tedious {
    use crate::{StrChunk, StrChunkMut};
    use std::borrow::Borrow;
    use std::cmp::Ordering;

    macro_rules! impl_partial_eq {
        {
            impl<$a:lifetime> <$Rhs:ty> for $T:ty
        } => {
            impl<$a> PartialEq<$Rhs> for $T {
                #[inline]
                fn eq(&self, other: &$Rhs) -> bool {
                    Borrow::<str>::borrow(self) == Borrow::<str>::borrow(other)
                }
            }
        };
        {
            impl <$Rhs:ty> for $T:ty
        } => {
            impl PartialEq<$Rhs> for $T {
                #[inline]
                fn eq(&self, other: &$Rhs) -> bool {
                    Borrow::<str>::borrow(self) == Borrow::<str>::borrow(other)
                }
            }
        };
    }

    macro_rules! impl_partial_ord {
        {
            impl<$a:lifetime> <$Rhs:ty> for $T:ty
        } => {
            impl<$a> PartialOrd<$Rhs> for $T {
                #[inline]
                fn partial_cmp(&self, other: &$Rhs) -> Option<Ordering> {
                    PartialOrd::partial_cmp(
                        Borrow::<str>::borrow(self),
                        Borrow::<str>::borrow(other),
                    )
                }
            }
        };
        {
            impl <$Rhs:ty> for $T:ty
        } => {
            impl PartialOrd<$Rhs> for $T {
                #[inline]
                fn partial_cmp(&self, other: &$Rhs) -> Option<Ordering> {
                    PartialOrd::partial_cmp(
                        Borrow::<str>::borrow(self),
                        Borrow::<str>::borrow(other),
                    )
                }
            }
        };
    }

    for_all_str_types! { impl_partial_eq! for StrChunk }
    for_all_str_types! { impl_partial_eq! for StrChunkMut }
    for_all_str_types! { impl_partial_ord! for StrChunk }
    for_all_str_types! { impl_partial_ord! for StrChunkMut }
}

mod foreign {
    use crate::{StrChunk, StrChunkMut};
    use std::borrow::Borrow;
    use std::cmp::Ordering;

    macro_rules! impl_partial_eq_rhs {
        {
            impl<$a:lifetime> <$Lhs:ty> for $T:ty
        } => {
            impl<$a> PartialEq<$T> for $Lhs {
                #[inline]
                fn eq(&self, other: &$T) -> bool {
                    other == self
                }
            }
        };
        {
            impl <$Lhs:ty> for $T:ty
        } => {
            impl PartialEq<$T> for $Lhs {
                #[inline]
                fn eq(&self, other: &$T) -> bool {
                    other == self
                }
            }
        };
    }

    macro_rules! impl_partial_ord_rhs {
        {
            impl<$a:lifetime> <$Lhs:ty> for $T:ty
        } => {
            impl<$a> PartialOrd<$T> for $Lhs {
                #[inline]
                fn partial_cmp(&self, other: &$T) -> Option<Ordering> {
                    PartialOrd::partial_cmp(
                        Borrow::<str>::borrow(self),
                        Borrow::<str>::borrow(other),
                    )
                }
            }
        };
        {
            impl <$Lhs:ty> for $T:ty
        } => {
            impl PartialOrd<$T> for $Lhs {
                #[inline]
                fn partial_cmp(&self, other: &$T) -> Option<Ordering> {
                    PartialOrd::partial_cmp(
                        Borrow::<str>::borrow(self),
                        Borrow::<str>::borrow(other),
                    )
                }
            }
        };
    }

    for_all_foreign_str_types! { impl_partial_eq_rhs! for StrChunk }
    for_all_foreign_str_types! { impl_partial_eq_rhs! for StrChunkMut }
    for_all_foreign_str_types! { impl_partial_ord_rhs! for StrChunk }
    for_all_foreign_str_types! { impl_partial_ord_rhs! for StrChunkMut }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::cmp_owned)]

    use crate::{StrChunk, StrChunkMut};

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

    const TEST_STR: &str = "Hello";
    const TEST_STR_LESSER: &str = "Hell";

    macro_rules! test_all_str_types {
        ($macro:ident!, $v:expr) => {
            $macro! { str, $v, *TEST_STR }
            $macro! { str_ref, $v, TEST_STR }
            $macro! { string, $v, String::from(TEST_STR) }
            $macro! { chunk, $v, crate::StrChunk::from(TEST_STR) }
            $macro! { chunk_mut, $v, crate::StrChunkMut::from(TEST_STR) }
            $macro! { cow_borrowed, $v, ::std::borrow::Cow::from(TEST_STR) }
            $macro! { cow_owned, $v, ::std::borrow::Cow::from(String::from(TEST_STR)) }
        };
    }

    mod eq {
        use super::*;

        macro_rules! test_equal {
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
            test_all_str_types! { test_equal!, StrChunk::from_static(TEST_STR) }
        }

        mod chunk_mut {
            use super::*;
            test_all_str_types! { test_equal!, StrChunkMut::from(TEST_STR) }
        }
    }

    mod ord {
        use super::*;

        mod equal {
            use super::*;

            macro_rules! test_equal {
                ($name:ident, $arg1:expr, $arg2:expr) => {
                    #[test]
                    fn $name() {
                        assert!($arg1 <= $arg2);
                        assert!(!($arg1 > $arg2));
                        assert!($arg2 >= $arg1);
                        assert!(!($arg2 < $arg1));
                    }
                };
            }

            mod chunk {
                use super::*;
                test_all_str_types! { test_equal!, StrChunk::from_static(TEST_STR) }
            }

            mod chunk_mut {
                use super::*;
                test_all_str_types! { test_equal!, StrChunkMut::from(TEST_STR) }
            }
        }

        mod unequal {
            use super::*;

            macro_rules! test_lesser {
                ($name:ident, $arg1:expr, $arg2:expr) => {
                    #[test]
                    fn $name() {
                        assert!($arg1 < $arg2);
                        assert!(!($arg1 >= $arg2));
                        assert!($arg2 > $arg1);
                        assert!(!($arg2 <= $arg1));
                    }
                };
            }

            mod chunk {
                use super::*;
                test_all_str_types! { test_lesser!, StrChunk::from_static(TEST_STR_LESSER) }
            }

            mod chunk_mut {
                use super::*;
                test_all_str_types! { test_lesser!, StrChunkMut::from(TEST_STR_LESSER) }
            }
        }
    }

    mod hash {
        use super::*;

        macro_rules! test_hash {
            ($v:expr) => {
                #[test]
                fn same_as_str() {
                    let mut set = ::std::collections::HashSet::new();
                    set.insert($v);
                    assert!(set.contains(TEST_STR));
                }
            };
        }

        mod chunk {
            use super::*;
            test_hash!(StrChunk::from(TEST_STR));
        }

        mod chunk_mut {
            use super::*;
            test_hash!(StrChunkMut::from(TEST_STR));
        }
    }
}

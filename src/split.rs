use std::fmt::Debug;
use std::ops::{Bound, RangeBounds};

pub trait TakeRange<R> {
    type Output;

    fn take_range(&mut self, range: R) -> Self::Output;

    fn remove_range(&mut self, range: R) {
        self.take_range(range);
    }
}

pub fn is_valid_str_range<R>(s: &str, range: &R) -> bool
where
    R: RangeBounds<usize>,
{
    is_valid_str_start_bound(s, range.start_bound())
    && is_valid_str_end_bound(s, range.end_bound())
}

#[inline]
fn is_valid_str_start_bound(s: &str, bound: Bound<&usize>) -> bool {
    use std::ops::Bound::*;

    let index = match bound {
        Unbounded => return true,
        Included(index) => *index,
        Excluded(_) => unreachable!(),
    };

    s.is_char_boundary(index)
}

#[inline]
fn is_valid_str_end_bound(s: &str, bound: Bound<&usize>) -> bool {
    use std::ops::Bound::*;

    let index = match bound {
        Unbounded => return true,
        Excluded(index) => *index,
        Included(index) => convert_inclusive_end_index(*index),
    };

    s.is_char_boundary(index)
}

#[cold]
#[inline(never)]
pub fn str_range_fail<R>(s: &str, range: &R) -> !
where
    R: RangeBounds<usize> + Debug,
{
    use std::ops::Bound::*;

    let index = match (range.start_bound(), range.end_bound()) {
        (Included(index), Unbounded) => *index,
        (Unbounded, Excluded(index)) => *index,
        (Unbounded, Included(index)) => convert_inclusive_end_index(*index),
        _ => unreachable!("unexpected range {:?} for a split failure", range),
    };

    if index > s.len() {
        panic!("range {:?} is out of bounds of the string buffer", range);
    } else {
        panic!("range {:?} does not split on a UTF-8 boundary", range);
    }
}

#[inline]
fn convert_inclusive_end_index(index: usize) -> usize {
    index.checked_add(1).unwrap_or_else(|| {
        panic!(
            "upper bound index {} is too large for a buffer in memory",
            index
        )
    })
}

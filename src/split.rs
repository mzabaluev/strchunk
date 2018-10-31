use std::{
    fmt::{self, Debug},
    ops::{
        Bound::{self, Unbounded},
        RangeBounds, RangeFrom, RangeFull, RangeTo, RangeToInclusive,
    },
};

pub enum SplitRange {
    Full(RangeFull),
    From(RangeFrom<usize>),
    To(RangeTo<usize>),
}

impl Debug for SplitRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SplitRange::Full(r) => r.fmt(f),
            SplitRange::From(r) => r.fmt(f),
            SplitRange::To(r) => r.fmt(f),
        }
    }
}

impl RangeBounds<usize> for SplitRange {
    #[inline]
    fn start_bound(&self) -> Bound<&usize> {
        match *self {
            SplitRange::Full(_) => Unbounded,
            SplitRange::From(ref r) => r.start_bound(),
            SplitRange::To(ref r) => r.start_bound(),
        }
    }

    #[inline]
    fn end_bound(&self) -> Bound<&usize> {
        match *self {
            SplitRange::Full(_) => Unbounded,
            SplitRange::From(ref r) => r.end_bound(),
            SplitRange::To(ref r) => r.end_bound(),
        }
    }
}

pub trait BindSlice<S: ?Sized>: RangeBounds<usize> {
    fn bind_slice(&self, slice: &S) -> SplitRange;
}

impl BindSlice<str> for RangeFull {
    #[inline]
    fn bind_slice(&self, _slice: &str) -> SplitRange {
        SplitRange::Full(*self)
    }
}

impl BindSlice<str> for RangeFrom<usize> {
    #[inline]
    fn bind_slice(&self, slice: &str) -> SplitRange {
        let range = SplitRange::From(self.clone());
        if !slice.is_char_boundary(self.start) {
            str_split_fail(slice, range);
        }
        range
    }
}

impl BindSlice<str> for RangeTo<usize> {
    #[inline]
    fn bind_slice(&self, slice: &str) -> SplitRange {
        let range = SplitRange::To(*self);
        if !slice.is_char_boundary(self.end) {
            str_split_fail(slice, range);
        }
        range
    }
}

impl BindSlice<str> for RangeToInclusive<usize> {
    #[inline]
    fn bind_slice(&self, slice: &str) -> SplitRange {
        let excl_range = convert_inclusive_range(*self);
        excl_range.bind_slice(slice)
    }
}

#[inline(never)]
#[cold]
fn str_split_fail(s: &str, range: SplitRange) -> ! {
    let index = match range {
        SplitRange::From(ref r) => r.start,
        SplitRange::To(ref r) => r.end,
        SplitRange::Full(_) => unreachable!(),
    };

    if index > s.len() {
        panic!("range {:?} is out of bounds of the string buffer", range);
    } else {
        panic!("range {:?} does not split on a UTF-8 boundary", range);
    }
}

fn convert_inclusive_range(range: RangeToInclusive<usize>) -> RangeTo<usize> {
    // This should be always Ok for valid ranges, because usize is
    // capable of representing the length of any slice in memory.
    let excl_end = range.end.checked_add(1).unwrap_or_else(|| {
        panic!(
            "upper bound of range {:?} is too large for a buffer in memory",
            range
        )
    });
    ..excl_end
}

pub trait Take<S: ?Sized> {
    type Output;

    fn take<R>(&mut self, range: R) -> Self::Output
    where
        R: BindSlice<S>;
}

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

impl From<RangeFull> for SplitRange {
    #[inline]
    fn from(src: RangeFull) -> SplitRange {
        SplitRange::Full(src)
    }
}

impl From<RangeFrom<usize>> for SplitRange {
    #[inline]
    fn from(src: RangeFrom<usize>) -> SplitRange {
        SplitRange::From(src)
    }
}

impl From<RangeTo<usize>> for SplitRange {
    #[inline]
    fn from(src: RangeTo<usize>) -> SplitRange {
        SplitRange::To(src)
    }
}

fn convert_inclusive_range(range: RangeToInclusive<usize>) -> RangeTo<usize> {
    // This should be always Ok for valid ranges, because usize is
    // capable of representing the length of any slice in memory.
    let incl_end = range.end;
    ..(incl_end
        .checked_add(1)
        .expect("integer overflow when calculating range"))
}

impl From<RangeToInclusive<usize>> for SplitRange {
    #[inline]
    fn from(src: RangeToInclusive<usize>) -> SplitRange {
        SplitRange::To(convert_inclusive_range(src))
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

pub trait Take {
    type Output;

    fn take<R>(&mut self, range: R) -> Self::Output
    where
        R: Into<SplitRange>;
}

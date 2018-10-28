use split::SplitRange;

#[inline]
pub fn validate_str_split(s: &str, range: &SplitRange) {
    let valid = match range {
        SplitRange::Full(_) => true,
        SplitRange::From(r) => s.is_char_boundary(r.start),
        SplitRange::To(r) => s.is_char_boundary(r.end),
    };
    if !valid {
        str_split_fail(s, range);
    }
}

#[inline(never)]
#[cold]
fn str_split_fail(s: &str, range: &SplitRange) -> ! {
    let index = match range {
        SplitRange::From(r) => r.start,
        SplitRange::To(r) => r.end,
        SplitRange::Full(_) => unreachable!(),
    };

    if index > s.len() {
        panic!("range {:?} is out of bounds of the string buffer", range);
    } else {
        panic!("range {:?} does not split on a UTF-8 boundary", range);
    }
}

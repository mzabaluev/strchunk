#![cfg_attr(feature = "specialization", feature(specialization))]

macro_rules! validate_str_range {
    ($s:expr, $r:expr) => {
        if !$crate::split::is_valid_str_range($s, $r) {
            $crate::split::str_range_fail($s, $r)
        }
    };
}

mod chunk;
mod chunk_mut;
mod impls;
pub mod split;

pub use crate::chunk::{ExtractUtf8Error, StrChunk};
pub use crate::chunk_mut::StrChunkMut;

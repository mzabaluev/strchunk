#![cfg_attr(feature = "specialization", feature(specialization))]
#![warn(clippy::all)]

mod chunk;
mod chunk_mut;
mod impls;

pub use crate::chunk::{ExtractUtf8Error, StrChunk};
pub use crate::chunk_mut::StrChunkMut;

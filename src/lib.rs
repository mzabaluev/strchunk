#![cfg_attr(feature = "specialization", feature(specialization))]

extern crate bytes;

mod chunk;
mod chunk_mut;
mod impls;

pub use chunk::{ExtractUtf8Error, StrChunk};
pub use chunk_mut::StrChunkMut;

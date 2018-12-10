#![cfg_attr(feature = "specialization", feature(specialization))]

extern crate bytes;

mod chunk;
mod chunk_mut;
mod impls;
pub mod split;

pub use crate::chunk::{ExtractUtf8Error, StrChunk};
pub use crate::chunk_mut::StrChunkMut;

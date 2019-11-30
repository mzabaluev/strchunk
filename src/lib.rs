//! UTF-8 string invariants for byte buffers provided by the `bytes` crate.
//!
//! The `strchunk` crate builds on the efficient byte containers provided
//! by the `bytes` crate. Its two container types, `StrChunk` and `StrChunkMut`,
//! wrap around `Bytes` and `BytesMut`, respectively, adding a guarantee
//! for the content to be valid UTF-8 to make it safely usable as
//! Rust string slices.

#![cfg_attr(feature = "specialization", feature(specialization))]
#![warn(rust_2018_idioms)]
#![warn(missing_docs)]
#![warn(clippy::all)]

mod chunk;
mod chunk_mut;
mod impls;

pub use crate::chunk::{ExtractUtf8Error, StrChunk};
pub use crate::chunk_mut::StrChunkMut;

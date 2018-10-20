extern crate bytes;

mod chunk;
pub use chunk::{ExtractUtf8Error, StrChunk};

mod chunk_mut;
pub use chunk_mut::StrChunkMut;

//! Minimal PE/COFF loader for 32-bit Windows XP-era executables.

mod header;
mod loader;

pub use loader::LoadedImage;

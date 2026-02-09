pub mod core;
#[cfg(feature = "file")]
pub mod file;
#[cfg(not(feature = "file"))]
mod file;
mod scanner;

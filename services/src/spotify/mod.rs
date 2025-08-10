#[cfg(feature = "full")]
pub mod spotify;

#[cfg(feature = "full")]
pub use spotify::*;

pub mod dto;
pub use dto::*;

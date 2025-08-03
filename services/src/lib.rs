pub mod auth;
#[cfg(feature = "full")]
pub mod deezer;
pub mod error;
#[cfg(feature = "full")]
pub mod music;
pub mod musicbrainz;
#[cfg(feature = "full")]
pub mod notification;
#[cfg(feature = "full")]
pub mod playlist;
#[cfg(feature = "full")]
pub mod spotify;
#[cfg(feature = "full")]
pub mod test;
#[cfg(feature = "full")]
pub mod user;
#[cfg(feature = "full")]
pub mod user_info;
#[cfg(feature = "full")]
pub use test::*;

#[cfg(feature = "full")]
pub use deezer::*;
#[cfg(feature = "full")]
pub use music::*;

#[cfg(feature = "full")]
pub use musicbrainz::*;

#[cfg(feature = "full")]
pub use playlist::*;

#[cfg(feature = "full")]
pub use spotify::*;

#[cfg(feature = "full")]
pub use user::*;

#[cfg(feature = "full")]
pub use notification::*;

pub mod dto;
pub mod validators;
pub use dto::*;
pub use swaptun_models::*;

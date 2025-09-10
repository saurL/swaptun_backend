#![allow(ambiguous_glob_reexports)]

pub mod auth;
pub mod deezer;

pub mod apple;
pub mod error;
pub mod mail;
pub mod music;
pub mod musicbrainz;
pub mod notification;
pub mod playlist;
pub mod spotify;
#[cfg(feature = "full")]
pub mod test;
pub mod user;
pub mod user_info;
pub mod validators;
pub mod yt_music;
#[cfg(feature = "full")]
pub use test::*;

pub use deezer::*;
pub use music::*;

pub use musicbrainz::*;

pub use playlist::*;

pub use spotify::*;

pub use user::*;

pub use yt_music::*;

pub use apple::*;
pub use notification::*;

pub use swaptun_models::*;

pub use mail::*;

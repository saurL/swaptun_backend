pub mod model;
pub use model::UserInfoRequest;

#[cfg(feature = "full")]
pub mod service;
#[cfg(feature = "full")]
pub use service::UserInfoService;

pub mod model;
pub mod service;
#[cfg(feature = "full")]
pub use model::UserInfoRequest;
#[cfg(feature = "full")]
pub use service::UserInfoService;

#[cfg(feature = "full")]
pub mod jwt;
#[cfg(feature = "full")]
pub mod password;
#[cfg(feature = "full")]
pub use jwt::{Claims, JwtMiddleware, generate_token, validate_token};
#[cfg(feature = "full")]
pub use password::{hash_password, verify_password};

mod roles;
pub use roles::*;

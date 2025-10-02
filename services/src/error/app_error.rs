#[cfg(feature = "full")]
use actix_web::{HttpResponse, ResponseError};
#[cfg(feature = "full")]
use apple_music_api::AppleMusicError;
use log::error;
use sea_orm::DbErr;
use serde::Serialize;
#[cfg(feature = "full")]
use std::env::VarError;
use std::fmt;
#[derive(Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug)]
pub enum AppError {
    Database(DbErr),
    Validation(String),
    NotFound(String),
    Unauthorized(String),
    InternalServerError,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(err) => write!(f, "Database error: {}", err),
            Self::Validation(msg) => write!(f, "Validation error: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            Self::InternalServerError => write!(f, "Internal server error"),
        }
    }
}
#[cfg(feature = "full")]
impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Database(err) => {
                log::error!("Database error: {}", err);
                HttpResponse::InternalServerError().json(ErrorResponse {
                    status: "error".into(),
                    message: "An internal error occurred".into(),
                })
            }
            AppError::Validation(msg) => HttpResponse::BadRequest().json(ErrorResponse {
                status: "error".into(),
                message: msg.clone(),
            }),
            AppError::NotFound(msg) => HttpResponse::NotFound().json(ErrorResponse {
                status: "error".into(),
                message: msg.clone(),
            }),
            AppError::Unauthorized(msg) => HttpResponse::Unauthorized().json(ErrorResponse {
                status: "error".into(),
                message: msg.clone(),
            }),
            AppError::InternalServerError => {
                HttpResponse::InternalServerError().json(ErrorResponse {
                    status: "error".into(),
                    message: "An internal error occurred".into(),
                })
            }
        }
    }
}

impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        AppError::Database(err)
    }
}
#[cfg(feature = "full")]
impl From<oauth2::url::ParseError> for AppError {
    fn from(_: oauth2::url::ParseError) -> Self {
        error!("Failed to parse URL");
        AppError::InternalServerError
    }
}

#[cfg(feature = "full")]
impl From<AppleMusicError> for AppError {
    fn from(err: AppleMusicError) -> Self {
        error!("Error from apple music api {}", err);
        AppError::InternalServerError
    }
}

#[cfg(feature = "full")]
impl From<VarError> for AppError {
    fn from(err: VarError) -> Self {
        error!("Error from environment variable: {}", err);
        AppError::InternalServerError
    }
}

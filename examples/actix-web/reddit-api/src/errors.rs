use std::error::Error;
use std::fmt;

use actix_web::{HttpResponse, ResponseError};
use charybdis::errors::CharybdisError;
use log::{error, warn};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    // 400 | 500
    CharybdisError(CharybdisError),
    SerdeError(serde_json::Error),
    InternalServerError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::CharybdisError(e) => write!(f, "Charybdis Error: {}", e),
            AppError::SerdeError(e) => write!(f, "Serde Error: {}", e),
            AppError::InternalServerError(e) => write!(f, "InternalServerError: {}", e),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::CharybdisError(e) => Some(e),
            _ => None,
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::CharybdisError(e) => match e {
                CharybdisError::NotFoundError(_e) => {
                    warn!("{}", e.to_string());

                    HttpResponse::NotFound().json(json!({
                        "status": 404,
                        "message": "Not Found"
                    }))
                }
                _ => AppError::InternalServerError(format!("CharybdisError: {}", e)).error_response(),
            },
            _ => {
                error!("InternalServerError: {}", self.to_string());

                HttpResponse::InternalServerError().json(json!({
                    "status": 500,
                    "message": "Internal Server Error"
                }))
            }
        }
    }
}

impl From<CharybdisError> for AppError {
    fn from(e: CharybdisError) -> Self {
        AppError::CharybdisError(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::SerdeError(e)
    }
}

impl From<actix_web::Error> for AppError {
    fn from(e: actix_web::Error) -> Self {
        AppError::InternalServerError(format!("{:?}", e))
    }
}

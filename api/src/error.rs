use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

use common::error::CoreError;
use serde_json::json;
use thiserror::Error;
#[derive(Error, Debug)]
#[error(transparent)]
pub struct AsHttpError(#[from] CoreError);

impl ResponseError for AsHttpError {
    fn status_code(&self) -> StatusCode {
        match self.0.clone() {
            CoreError::DataError(_) => StatusCode::BAD_REQUEST,
            CoreError::ResourceNotFound(_) => StatusCode::NOT_FOUND,
            CoreError::OperationNotAuthorized(_) => StatusCode::UNAUTHORIZED,
            CoreError::OperationForbiden(_) => StatusCode::FORBIDDEN,
            CoreError::UnkownError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self.0.clone() {
            CoreError::DataError(s) => HttpResponse::BadRequest().json(json!({
                "description": s.to_owned()
            })),
            CoreError::ResourceNotFound(s) => HttpResponse::NotFound().json(json!({
                "description": s.to_owned()
            })),
            CoreError::OperationNotAuthorized(s) => HttpResponse::Unauthorized().json(json!({
                "description": s.to_owned()
            })),
            CoreError::OperationForbiden(s) => HttpResponse::Forbidden().json(json!({
                "description": s.to_owned()
            })),
            CoreError::UnkownError(s) => HttpResponse::InternalServerError().json(json!({
                "description": s.to_owned()
            })),
        }
    }
}

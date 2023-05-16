use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use sqlx::error::{Error as SqlxError};
use apache_avro::{Error as AvroError};

pub enum AppError {
    DatabaseError(SqlxError),
    AvroError(AvroError),
    JsonError
}

impl From<SqlxError> for AppError {
    fn from(value: SqlxError) -> Self { AppError::DatabaseError(value) }
}

impl From<AvroError> for AppError {
    fn from(value: AvroError) -> Self { AppError::AvroError(value) }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            AppError::AvroError(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            AppError::JsonError => (StatusCode::BAD_REQUEST).into_response()
        }
    }
}
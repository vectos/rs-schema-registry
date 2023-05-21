use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use sqlx::error::{Error as SqlxError};
use apache_avro::{Error as AvroError};
use axum::Json;
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(SqlxError),
    AvroError(AvroError),
    SubjectNotFound(String),
    SchemaNotFound(String, i32),
    IncompatibleSchema,
    JsonError
}

#[derive(Serialize)]
pub struct ApiError {
    error_code: u32,
    message: String
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
            AppError::DatabaseError(error) =>
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error_code: 50001, message: error.to_string() })).into_response(),
            AppError::AvroError(error) =>
                (StatusCode::UNPROCESSABLE_ENTITY, Json(ApiError { error_code: 42201, message: error.to_string() })).into_response(),
            AppError::SubjectNotFound(_) =>
                (StatusCode::NOT_FOUND, Json(ApiError { error_code: 40401, message: String::from("subject was not found") })).into_response(),
            AppError::SchemaNotFound(_, _) =>
                (StatusCode::NOT_FOUND, Json(ApiError { error_code: 40402, message: String::from("schema was not found") })).into_response(),
            AppError::IncompatibleSchema =>
                (StatusCode::CONFLICT, Json(ApiError { error_code: 409, message: String::from("schema incompatible")})).into_response(),
            AppError::JsonError => (StatusCode::BAD_REQUEST).into_response()
        }
    }
}
mod schemas;
mod error;

use axum::{routing::*, Router, Json};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use sqlx::postgres::PgPoolOptions;
use crate::error::AppError;
use crate::schemas::{SchemaRequest, DataStore, RegisterSchemaResponse};

#[tokio::main]
async fn main() {

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5432/postgres")
        .await
        .unwrap();

    sqlx::migrate!().run(&pool).await.unwrap();

    let app = Router::new()
        .route("/subjects", get(list_subjects))
        .route("/subjects/:subject", post(check_schema_existence))
        .route("/subjects/:subject/versions", post(register_schema))
        .route("/subjects/:subject/versions", get(get_subject_versions))
        .with_state(pool);

    axum::Server::bind(&"0.0.0.0:8888".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub async fn list_subjects(State(pool): State<PgPool>) -> Result<Json<Vec<String>>, AppError> {
    let res =
        pool.subjects_all().await?.iter().map(|x| x.name.clone()).collect();

    Ok(Json(res))
}

pub async fn get_subject_versions(State(pool): State<PgPool>, Path(subject): Path<String>) -> Result<Json<Vec<i32>>, AppError> {
    let res =
        pool.subject_versions(&subject).await?;

    Ok(Json(res))
}


pub async fn register_schema(State(pool): State<PgPool>, Path(subject): Path<String>, body: Json<SchemaRequest>) -> Result<Json<RegisterSchemaResponse>, AppError> {
    match pool.schema_find_by_schema(&subject, &body.schema).await? {
        Some(resp) => {
            let res = RegisterSchemaResponse{ id: resp.id};
            Ok(Json(res))
        },
        None => {
            let res = pool.schema_insert(&subject, &body.schema).await?;
            Ok(Json(res))
        }
    }
}


pub async fn check_schema_existence(State(pool) : State<PgPool>, Path(subject): Path<String>, body: Json<SchemaRequest>) -> Result<Response, AppError> {
    match pool.schema_find_by_schema(&subject, &body.schema).await? {
        Some(resp) => Ok((StatusCode::OK, Json(resp)).into_response()),
        None => Ok((StatusCode::NOT_FOUND).into_response())
    }
}
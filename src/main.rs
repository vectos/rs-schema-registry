mod subjects;
mod schemas;

use axum::{routing::*, Router, Json};
use axum::extract::{Path, State};
use axum::handler::Handler;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use sqlx::postgres::PgPoolOptions;
use crate::schemas::{FindBySchemaRequest, FindBySchemaResponse, SchemaRepository};

use crate::subjects::{Subject, SubjectsRepository};

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
        .with_state(pool);

    axum::Server::bind(&"0.0.0.0:8888".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub async fn list_subjects(State(pool): State<PgPool>) -> (StatusCode, Json<Vec<String>>) {
    let res =
        pool.subjects_all().await.unwrap().iter().map(|x| x.name.clone()).collect();

    (StatusCode::OK, Json(res))
}

pub async fn check_schema_existence(State(pool) : State<PgPool>, Path(subject): Path<String>, body: Json<FindBySchemaRequest>) -> Response {
    match pool.schema_find_by_schema(&subject, &body.schema).await.unwrap() {
        Some(resp) => (StatusCode::OK, Json(resp)).into_response(),
        None => (StatusCode::NOT_FOUND).into_response()
    }
}
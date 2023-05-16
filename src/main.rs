mod subjects;

use axum::{routing::get, Router, Json};
use axum::extract::State;
use axum::http::StatusCode;

use sqlx::postgres::PgPoolOptions;

use subjects::PostgresSubjectsRepository;
use crate::subjects::{Subject, SubjectsRepository};

#[tokio::main]
async fn main() {

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5432/postgres")
        .await
        .unwrap();

    sqlx::migrate!().run(&pool).await.unwrap();

    let subjects_repository = PostgresSubjectsRepository::new(pool.clone());

    let app = Router::new()
        .route("/subjects", get(list_subjects))
        .with_state(subjects_repository);

    axum::Server::bind(&"0.0.0.0:8888".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub async fn list_subjects(State(subjects_repo): State<PostgresSubjectsRepository>) -> (StatusCode, Json<Vec<Subject>>) {
    let res =
        subjects_repo.all().await.unwrap();

    (StatusCode::OK, Json(res))
}
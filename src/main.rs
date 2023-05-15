use axum::{routing::get, Router, Extension, Json};
use axum::http::StatusCode;
use sqlx::{PgPool, Row};

use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() {

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5432/postgres")
        .await
        .unwrap();

    // build our application with a single route
    let app = Router::new()
        .route("/", get(do_math))
        .layer(Extension(pool));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:8888".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub async fn do_math(Extension(pool): Extension<PgPool>) -> (StatusCode, Json<i32>) {
    let res =
        sqlx::query(&"SELECT 1+1")
            .fetch_one(&pool)
            .await
            .unwrap()
            .get(0);

    (StatusCode::OK, Json(res))
}

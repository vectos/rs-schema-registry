use sqlx::{Error, FromRow, PgPool};
use async_trait::async_trait;
use serde::Serialize;

#[derive(FromRow, Serialize)]
pub struct Subject {
    pub name: String
}

#[async_trait]
pub trait SubjectsRepository {
    async fn all(&self) -> Result<Vec<Subject>, Error>;
}

#[derive(Clone)]
pub struct PostgresSubjectsRepository {
    pool: PgPool
}

impl PostgresSubjectsRepository {
    pub fn new(pool: PgPool) -> PostgresSubjectsRepository {
        PostgresSubjectsRepository { pool }
    }
}

#[async_trait]
impl SubjectsRepository for PostgresSubjectsRepository {
    async fn all(&self) -> Result<Vec<Subject>, Error> {
        let res = sqlx::query_as::<_, Subject>("SELECT name FROM subjects").fetch_all(&self.pool).await;
        res
    }
}
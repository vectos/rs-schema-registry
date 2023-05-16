use sqlx::{Error, FromRow, PgPool};
use async_trait::async_trait;
use serde::Serialize;

#[derive(FromRow, Serialize)]
pub struct Subject {
    pub name: String
}

#[async_trait]
pub trait SubjectsRepository {
    async fn subjects_all(&self) -> Result<Vec<Subject>, Error>;
}

#[async_trait]
impl SubjectsRepository for PgPool {
    async fn subjects_all(&self) -> Result<Vec<Subject>, Error> {
        sqlx::query_as::<_, Subject>("SELECT name FROM subjects").fetch_all(self).await
    }
}
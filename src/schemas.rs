use sqlx::{FromRow, PgPool};
use async_trait::async_trait;

use serde::{Deserialize, Serialize};

use apache_avro::{Schema as AvroSchema};
use sha2::Sha256;
use crate::error::AppError;

#[derive(FromRow, Serialize)]
pub struct Schema {
    pub fingerprint: String
}

#[derive(Deserialize)]
pub struct SchemaRequest {
    pub schema: String
}

#[derive(FromRow, Serialize)]
pub struct FindBySchemaResponse {
    pub name: String,
    pub version: i32,
    pub id: i64,
    pub schema: String
}

#[derive(FromRow, Serialize)]
pub struct RegisterSchemaResponse {
    pub id: i64
}


#[derive(FromRow, Serialize)]
pub struct Subject {
    pub id: i64,
    pub name: String
}

#[async_trait]
pub trait DataStore {
    async fn schema_find_by_schema(&self, subject: &String, schema: &String) -> Result<Option<FindBySchemaResponse>, AppError>;
    async fn schema_insert(&self, subject: &String, schema: &String) -> Result<RegisterSchemaResponse, AppError>;
    async fn subject_versions(&self, subject: &String) -> Result<Vec<i32>, AppError>;
    async fn subject_find(&self, subject: &String) -> Result<Option<Subject>, AppError>;
    async fn subjects_all(&self) -> Result<Vec<Subject>, AppError>;
}

#[async_trait]
impl DataStore for PgPool {
    async fn subjects_all(&self) -> Result<Vec<Subject>, AppError> {
        let res = sqlx::query_as::<_, Subject>("SELECT name FROM subjects").fetch_all(self).await?;
        Ok(res)
    }

    async fn schema_find_by_schema(&self, subject: &String, schema: &String) -> Result<Option<FindBySchemaResponse>, AppError> {
        let avro_schema = AvroSchema::parse_str(schema.as_str())?;
        let fingerprint = avro_schema.fingerprint::<Sha256>().to_string();

        let res = sqlx::query_as!(FindBySchemaResponse, r#"select sub.name as name, sv.version as version, sch.id as id, sch.json as schema from schemas sch inner join schema_versions sv on sch.id = sv.schema_id inner join subjects sub on sv.subject_id = sub.id where sch.fingerprint = $1 and sub.name = $2;"#, fingerprint, subject)
            .fetch_optional(self)
            .await?;

        Ok(res)
    }

    async fn schema_insert(&self, subject: &String, schema: &String) -> Result<RegisterSchemaResponse, AppError> {
        let avro_schema = AvroSchema::parse_str(schema.as_str())?;
        let fingerprint = avro_schema.fingerprint::<Sha256>().to_string();

        let subject_record = self.subject_find(subject).await?.ok_or(AppError::SubjectNotFound(subject.clone()))?;
        let subject_versions = self.subject_versions(subject).await?;
        let max_version = subject_versions.iter().max().unwrap_or(&1);
        let tx = self.begin().await?;

        let schema_record = sqlx::query!(r#"INSERT INTO schemas (fingerprint, json, created_at, updated_at) VALUES ($1, $2, now(), now()) returning id;"#, fingerprint, schema)
            .fetch_one(self)
            .await?;

        let _ = sqlx::query!(r#"INSERT INTO schema_versions (version, subject_id, schema_id) VALUES ($1, $2, $3)"#, max_version, subject_record.id, schema_record.id)
            .execute(self)
            .await?;

        tx.commit().await?;

        Ok(RegisterSchemaResponse{id: schema_record.id})
    }

    async fn subject_versions(&self, subject: &String) -> Result<Vec<i32>, AppError> {
        let res = sqlx::query!(r#"SELECT version FROM subjects s INNER JOIN schema_versions sv ON s.id = sv.subject_id WHERE s.name = $1;"#, subject)
            .fetch_all(self)
            .await?;

        let transformed = res.iter().map(|x| x.version).collect();

        Ok(transformed)
    }

    async fn subject_find(&self, subject: &String) -> Result<Option<Subject>, AppError> {
        let res = sqlx::query_as!(Subject, r#"SELECT id, name FROM subjects WHERE name = $1"#, subject).fetch_optional(self).await?;

        Ok(res)
    }
}
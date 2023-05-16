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
pub struct FindBySchemaRequest {
    pub schema: String
}

#[derive(FromRow, Serialize)]
pub struct FindBySchemaResponse {
    pub name: String,
    pub version: i32,
    pub id: i32,
    pub schema: String
}

#[async_trait]
pub trait SchemaRepository {
    async fn schema_find_by_schema(&self, subject: &String, schema: &String) -> Result<Option<FindBySchemaResponse>, AppError>;

}

#[async_trait]
impl SchemaRepository for PgPool {
    async fn schema_find_by_schema(&self, subject: &String, schema: &String) -> Result<Option<FindBySchemaResponse>, AppError> {
        let avro_schema = AvroSchema::parse_str(schema.as_str())?;
        let fingerprint = avro_schema.fingerprint::<Sha256>().to_string();

        let res = sqlx::query_as::<_, FindBySchemaResponse>("select sub.name, sv.version, sch.id, sch.json from schemas sch inner join schema_versions sv on sch.id = sv.schema_id inner join subjects sub on sv.subject_id = sub.id where sch.fingerprint = $1 and sub.name = $2;")
            .bind(fingerprint)
            .bind(subject)
            .fetch_optional(self)
            .await?;

        Ok(res)
    }
}
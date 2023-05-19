use std::cmp::Ordering;
use sqlx::{FromRow, PgPool};
use async_trait::async_trait;

use serde::{Deserialize, Serialize};

use apache_avro::{Schema as AvroSchema, schema_compatibility::SchemaCompatibility as AvroSchemaCompatibility};
use sha2::Sha256;
use crate::error::AppError;

#[derive(FromRow, Serialize)]
pub struct Schema {
    pub fingerprint: String
}

#[derive(FromRow, Deserialize, Serialize)]
pub struct SchemaPayload {
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

#[derive(FromRow)]
pub struct VersionedSchema {
    pub version: i32,
    pub id: i64,
    pub schema: String
}

impl Eq for VersionedSchema {}

impl PartialEq for VersionedSchema {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for VersionedSchema {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.version.partial_cmp(&other.version)
    }
}

impl Ord for VersionedSchema {
    fn cmp(&self, other: &Self) -> Ordering {
        self.version.cmp(&other.version)
    }

}

impl Compatibility {
    //TODO: is the right way for sqlx to encode?
    fn as_str(&self) -> &'static str {
        match self {
            Compatibility::Backward => "BACKWARD",
            Compatibility::BackwardTransitive => "BACKWARD_TRANSITIVE",
            Compatibility::Forward => "FORWARD",
            Compatibility::ForwardTransitive => "FORWARD_TRANSITIVE",
            Compatibility::Full => "FULL",
            Compatibility::FullTransitive => "FULL_TRANSITIVE",
            Compatibility::None => "NONE",
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Compatibility {
    Backward,
    BackwardTransitive,
    Forward,
    ForwardTransitive,
    Full,
    FullTransitive,
    None
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct SchemaCompatibility {
    pub compatibility: Compatibility
}

#[async_trait]
pub trait DataStore {
    async fn schema_find_by_id(&self, id: i64) -> Result<Option<SchemaPayload>, AppError>;
    async fn schema_find_by_version(&self, subject: &String, version: i32) -> Result<Option<FindBySchemaResponse>, AppError>;
    async fn schema_find_by_schema(&self, subject: &String, schema: &String) -> Result<Option<FindBySchemaResponse>, AppError>;
    async fn schema_insert(&self, subject: &String, schema: &String) -> Result<RegisterSchemaResponse, AppError>;
    async fn schema_compatibility(&self, schemas: &Vec<VersionedSchema>, incoming: &AvroSchema, compatibility: Compatibility) -> Result<bool, AppError>;

    async fn subject_versions(&self, subject: &String) -> Result<Vec<i32>, AppError>;
    async fn subject_find(&self, subject: &String) -> Result<Option<Subject>, AppError>;
    async fn subject_all(&self) -> Result<Vec<Subject>, AppError>;
    async fn subject_schemas(&self, subject: &String) -> Result<Vec<VersionedSchema>, AppError>;

    async fn config_get_subject(&self, subject: Option<&String>) -> Result<Option<SchemaCompatibility>, AppError>;
    async fn config_set_subject(&self, subject: Option<&String>, compatibility: &Compatibility) -> Result<(), AppError>;

}

#[async_trait]
impl DataStore for PgPool {
    async fn schema_find_by_id(&self, id: i64) -> Result<Option<SchemaPayload>, AppError> {
        let res = sqlx::query_as!(SchemaPayload, r#"select json as schema from schemas where id = $1;"#, id)
            .fetch_optional(self)
            .await?;

        Ok(res)
    }

    async fn schema_find_by_version(&self, subject: &String, version: i32) -> Result<Option<FindBySchemaResponse>, AppError> {
        let res = sqlx::query_as!(FindBySchemaResponse, r#"select sub.name as name, sv.version as version, sch.id as id, sch.json as schema from schemas sch inner join schema_versions sv on sch.id = sv.schema_id inner join subjects sub on sv.subject_id = sub.id where sv.version = $1 and sub.name = $2;"#, version, subject)
            .fetch_optional(self)
            .await?;

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

        let subject_record = self.subject_find(&subject).await?.ok_or(AppError::SubjectNotFound(subject.clone()))?;
        let subject_schemas = self.subject_schemas(&subject).await?;

        let subject_compatibility = self.config_get_subject(Some(subject)).await?.map(|x| x.compatibility);
        let global_compatibility = self.config_get_subject(None).await?.map(|x| x.compatibility);

        //TODO: can this or_else be lazy?
        let compatibility = subject_compatibility.or_else(|| global_compatibility).unwrap_or(Compatibility::Backward);

        let is_compatible = self.schema_compatibility(&subject_schemas, &avro_schema, compatibility).await?;

        if !is_compatible {
            return Err(AppError::IncompatibleSchema)
        }

        let max_version = subject_schemas.first().map(|x| x.version).unwrap_or(0);
        let tx = self.begin().await?;

        let schema_record = sqlx::query!(r#"INSERT INTO schemas (fingerprint, json, created_at, updated_at) VALUES ($1, $2, now(), now()) returning id;"#, fingerprint, schema)
            .fetch_one(self)
            .await?;

        let _ = sqlx::query!(r#"INSERT INTO schema_versions (version, subject_id, schema_id) VALUES ($1, $2, $3)"#, max_version + 1, subject_record.id, schema_record.id)
            .execute(self)
            .await?;

        tx.commit().await?;

        Ok(RegisterSchemaResponse{id: schema_record.id})
    }

    async fn schema_compatibility(&self, schemas: &Vec<VersionedSchema>, incoming: &AvroSchema, compatibility: Compatibility) -> Result<bool, AppError> {
        match compatibility {
            Compatibility::Backward => {
                match schemas.first() {
                    Some(versioned_schema) => {
                        let db_schema = AvroSchema::parse_str(versioned_schema.schema.as_str())?;
                        Ok(AvroSchemaCompatibility::can_read(&db_schema, &incoming))
                    },
                    None => Ok(true)
                }
            },
            Compatibility::BackwardTransitive => {
                for s in schemas {
                    let db_schema = AvroSchema::parse_str(s.schema.as_str())?;
                    if !AvroSchemaCompatibility::can_read(&db_schema, &incoming) {
                        return Ok(false)
                    }
                }

                return Ok(true)
            },
            Compatibility::Forward => {
                match schemas.first() {
                    Some(versioned_schema) => {
                        let db_schema = AvroSchema::parse_str(versioned_schema.schema.as_str())?;
                        Ok(AvroSchemaCompatibility::can_read(&incoming, &db_schema))
                    },
                    None => Ok(true)
                }
            },
            Compatibility::ForwardTransitive => {
                for s in schemas {
                    let db_schema = AvroSchema::parse_str(s.schema.as_str())?;
                    if !AvroSchemaCompatibility::can_read(&incoming, &db_schema) {
                        return Ok(false)
                    }
                }

                return Ok(true)
            },
            Compatibility::Full => {
                let backward = self.schema_compatibility(&schemas, &incoming, Compatibility::Backward).await?;
                let forward = self.schema_compatibility(&schemas, &incoming, Compatibility::Forward).await?;
                Ok(backward && forward)
            },
            Compatibility::FullTransitive => {
                let backward = self.schema_compatibility(&schemas, &incoming, Compatibility::BackwardTransitive).await?;
                let forward = self.schema_compatibility(&schemas, &incoming, Compatibility::ForwardTransitive).await?;
                Ok(backward && forward)
            },
            Compatibility::None => Ok(true)
        }
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

    async fn subject_all(&self) -> Result<Vec<Subject>, AppError> {
        let res = sqlx::query_as::<_, Subject>("SELECT name FROM subjects").fetch_all(self).await?;
        Ok(res)
    }

    async fn subject_schemas(&self, subject: &String) -> Result<Vec<VersionedSchema>, AppError> {
        let res = sqlx::query_as!(VersionedSchema, r#"select sv.version as version, sch.id as id, sch.json as schema from schemas sch inner join schema_versions sv on sch.id = sv.schema_id inner join subjects sub on sv.subject_id = sub.id where sub.name = $1 order by sv.version desc;"#, subject)
            .fetch_all(self)
            .await?;

        Ok(res)
    }

    async fn config_get_subject(&self, subject: Option<&String>) -> Result<Option<SchemaCompatibility>, AppError> {

        let subject_id = match subject {
            Some(sub) => self.subject_find(sub).await?.map(|x| x.id),
            None => None
        };

        let res = sqlx::query_as!(SchemaCompatibility, r#"select compatibility from configs where subject_id = $1"#, subject_id)
            .fetch_optional(self)
            .await?;

        Ok(res)
    }

    async fn config_set_subject(&self, subject: Option<&String>, compatibility: &Compatibility) -> Result<(), AppError> {
        let subject_id = match subject {
            Some(sub) => self.subject_find(sub).await?.map(|x| x.id),
            None => None
        };

        let _ = sqlx::query!(r#"insert into configs (compatibility, created_at, updated_at, subject_id) values ($1, now(), now(), $2) on conflict (subject_id) do update set updated_at = now(), compatibility = excluded.compatibility"#, Some(compatibility.as_str()), subject_id)
            .execute(self)
            .await?;

        Ok(())
    }
}

//TODO: is the right way for sqlx to decode?
impl From<Option<String>> for Compatibility {
    fn from(value: Option<String>) -> Self {
        match value.as_deref() {
            Some("BACKWARD") => Compatibility::Backward,
            Some("BACKWARD_TRANSITIVE") => Compatibility::BackwardTransitive,
            Some("FORWARD") => Compatibility::Forward,
            Some("FORWARD_TRANSITIVE") => Compatibility::ForwardTransitive,
            Some("FULL") => Compatibility::Full,
            Some("FULL_TRANSITIVE") => Compatibility::FullTransitive,
            _ => Compatibility::None
        }
    }
}
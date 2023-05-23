use async_trait::async_trait;
use sqlx::{Error, PgPool};

use crate::data::*;

#[async_trait]
pub trait Repository {
    async fn schema_find_by_id(&self, id: i64) -> Result<Option<SchemaPayload>, Error>;
    async fn schema_soft_delete(&self, schema_id: i64) -> Result<u64, Error>;
    async fn schema_find_by_version(&self, subject: &String, version: i32) -> Result<Option<FindBySchemaResponse>, Error>;
    async fn schema_find_by_schema(&self, subject: &String, fingerprint: &String) -> Result<Option<FindBySchemaResponse>, Error>;
    async fn insert(&self, fingerprint: &String, schema: &String, subject_id: i64, max_version: i32) -> Result<i64, Error>;
    async fn insert_schema(&self, fingerprint: &String, schema: &String) -> Result<i64, Error>;
    async fn insert_schema_version(&self, max_version: i32, subject_id: i64, schema_id: i64) -> Result<(), Error>;
    async fn subject_versions(&self, subject: &String) -> Result<Vec<i32>, Error>;
    async fn subject_find(&self, subject: &String) -> Result<Option<Subject>, Error>;
    async fn subject_all(&self) -> Result<Vec<Subject>, Error>;
    async fn subject_schemas(&self, subject: &String) -> Result<Vec<VersionedSchema>, Error>;
    async fn config_get_subject(&self, subject_id: Option<i64>) -> Result<Option<SchemaCompatibility>, Error>;
    async fn config_set_subject(&self, subject_id: Option<i64>, compatibility: &Compatibility) -> Result<(), Error>;
    async fn max_version(&self, subject: &String) -> Result<Option<MaxVersion>, Error>;
}

#[derive(Clone)]
pub struct PgRepository { pub pool: PgPool }

#[async_trait]
impl Repository for PgRepository {

    async fn schema_find_by_id(&self, id: i64) -> Result<Option<SchemaPayload>, Error> {
        let res = sqlx::query_as!(SchemaPayload, r#"select json as schema from schemas where id = $1;"#, id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(res)
    }

    async fn schema_soft_delete(&self, schema_id: i64) -> Result<u64, Error> {

        let tx = self.pool.begin().await?;

        let affected = sqlx::query!(r#"UPDATE schemas SET deleted_at = now() WHERE id = $1"#, schema_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        let _ = sqlx::query!(r#"DELETE FROM schema_versions WHERE schema_id = $1"#, schema_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        tx.commit().await?;

        Ok(affected)
    }

    async fn schema_find_by_version(&self, subject: &String, version: i32) -> Result<Option<FindBySchemaResponse>, Error> {
        let res = sqlx::query_as!(FindBySchemaResponse, r#"select sub.name as name, sv.version as version, sch.id as id, sch.json as schema from schemas sch inner join schema_versions sv on sch.id = sv.schema_id inner join subjects sub on sv.subject_id = sub.id where sch.deleted_at is null and sv.version = $1 and sub.name = $2;"#, version, subject)
            .fetch_optional(&self.pool)
            .await?;

        Ok(res)
    }

    async fn schema_find_by_schema(&self, subject: &String, fingerprint: &String) -> Result<Option<FindBySchemaResponse>, Error> {
        let res = sqlx::query_as!(FindBySchemaResponse, r#"select sub.name as name, sv.version as version, sch.id as id, sch.json as schema from schemas sch inner join schema_versions sv on sch.id = sv.schema_id inner join subjects sub on sv.subject_id = sub.id where sch.deleted_at is null and sch.fingerprint = $1 and sub.name = $2;"#, fingerprint, subject)
            .fetch_optional(&self.pool)
            .await?;

        Ok(res)
    }

    async fn insert(&self, fingerprint: &String, schema: &String, subject_id: i64, max_version: i32) -> Result<i64, Error> {
        let tx = self.pool.begin().await?;

        let schema_id = self.insert_schema(&fingerprint, &schema).await?;
        let _ = self.insert_schema_version(max_version, subject_id, schema_id).await?;

        tx.commit().await?;

        Ok(schema_id)
    }

    async fn insert_schema(&self, fingerprint: &String, schema: &String) -> Result<i64, Error> {
        let res = sqlx::query!(r#"INSERT INTO schemas (fingerprint, json, created_at) VALUES ($1, $2, now()) returning id;"#, fingerprint, schema)
            .fetch_one(&self.pool)
            .await?;

        Ok(res.id)
    }

    async fn insert_schema_version(&self, max_version: i32, subject_id: i64, schema_id: i64) -> Result<(), Error> {
        let _ = sqlx::query!(r#"INSERT INTO schema_versions (version, subject_id, schema_id) VALUES ($1, $2, $3)"#, max_version + 1, subject_id, schema_id).execute(&self.pool).await?;

        Ok(())
    }

    async fn subject_versions(&self, subject: &String) -> Result<Vec<i32>, Error> {
        let res = sqlx::query!(r#"SELECT version FROM subjects s INNER JOIN schema_versions sv ON s.id = sv.subject_id WHERE s.name = $1;"#, subject)
            .fetch_all(&self.pool)
            .await?;


        Ok(res.iter().map(|x| x.version).collect())
    }

    async fn subject_find(&self, subject: &String) -> Result<Option<Subject>, Error> {
        let res = sqlx::query_as!(Subject, r#"SELECT id, name FROM subjects WHERE name = $1"#, subject).fetch_optional(&self.pool).await?;
        Ok(res)
    }

    async fn subject_all(&self) -> Result<Vec<Subject>, Error> {
        let res = sqlx::query_as!(Subject, r#"SELECT id, name FROM subjects"#).fetch_all(&self.pool).await?;
        Ok(res)
    }

    async fn subject_schemas(&self, subject: &String) -> Result<Vec<VersionedSchema>, Error> {
        let res = sqlx::query_as!(VersionedSchema, r#"select sv.version as version, sch.id as id, sch.json as schema from schemas sch inner join schema_versions sv on sch.id = sv.schema_id inner join subjects sub on sv.subject_id = sub.id where sch.deleted_at is null and sub.name = $1 order by sv.version desc;"#, subject)
            .fetch_all(&self.pool)
            .await?;
        Ok(res)
    }

    async fn config_get_subject(&self, subject_id: Option<i64>) -> Result<Option<SchemaCompatibility>, Error> {
        let res = sqlx::query_as!(SchemaCompatibility, r#"select compatibility from configs where subject_id = $1"#, subject_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(res)
    }

    async fn config_set_subject(&self, subject_id: Option<i64>, compatibility: &Compatibility) -> Result<(), Error> {
        let _ = sqlx::query!(r#"insert into configs (compatibility, created_at, updated_at, subject_id) values ($1, now(), now(), $2) on conflict (subject_id) do update set updated_at = now(), compatibility = excluded.compatibility"#, Some(compatibility.as_str()), subject_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    async fn max_version(&self, subject: &String) -> Result<Option<MaxVersion>, Error> {
        let res = sqlx::query_as!(MaxVersion, r#"select max(version) as max_version from schema_versions sv inner join subjects sub on sv.subject_id = sub.id where sub.name = $1;"#, subject)
            .fetch_optional(&self.pool)
            .await?;
        Ok(res)
    }
}
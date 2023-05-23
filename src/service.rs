use apache_avro::{Schema as AvroSchema, schema_compatibility::SchemaCompatibility as AvroSchemaCompatibility};
use async_recursion::async_recursion;
use sha2::Sha256;
use crate::error::AppError;
use crate::data::*;
use crate::repository::*;

#[derive(Clone)]
pub struct Service<R> {
    pub repository: R
}

impl <R : Repository + Send + Sync> Service<R> {
    pub async fn schema_find_by_id(&self, id: i64) -> Result<Option<SchemaPayload>, AppError> {
        let res = self.repository.schema_find_by_id(id).await?;
        Ok(res)
    }

    pub async fn schema_find_by_version(&self, subject: &String, version_id: &VersionId) -> Result<Option<FindBySchemaResponse>, AppError> {
        let version = self.version_id(&subject, &version_id).await?.ok_or(AppError::SchemaNotFound(subject.clone(), version_id.clone()))?;
        let res = self.repository.schema_find_by_version(&subject, version).await?;

        Ok(res)
    }

    pub async fn schema_delete_by_version(&self, subject: &String, version_id: &VersionId) -> Result<u64, AppError> {
        let version = self.version_id(&subject, &version_id).await?.ok_or(AppError::SchemaNotFound(subject.clone(), version_id.clone()))?;
        let res = self.repository.schema_find_by_version(&subject, version).await?;

        let affected: Result<u64, AppError> = match res {
            Some(resp) => {
                let affected = self.repository.schema_soft_delete(resp.id).await?;
                Ok(affected)
            },
            None => Ok(0)
        };

        affected
    }

    pub async fn schema_find_by_schema(&self, subject: &String, schema: &String) -> Result<Option<FindBySchemaResponse>, AppError> {
        let avro_schema = AvroSchema::parse_str(schema.as_str())?;
        let fingerprint = avro_schema.fingerprint::<Sha256>().to_string();
        let res = self.repository.schema_find_by_schema(&subject, &fingerprint).await?;

        Ok(res)
    }

    pub async fn schema_insert(&self, subject: &String, schema: &String) -> Result<RegisterSchemaResponse, AppError> {
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
        let schema_id = self.repository.insert(&fingerprint, &schema, subject_record.id, max_version).await?;

        Ok(RegisterSchemaResponse{id: schema_id})
    }

    #[async_recursion]
    pub async fn schema_compatibility(&self, schemas: &Vec<VersionedSchema>, incoming: &AvroSchema, compatibility: Compatibility) -> Result<bool, AppError> {
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

    pub async fn subject_versions(&self, subject: &String) -> Result<Vec<i32>, AppError> {
        let res = self.repository.subject_versions(&subject).await?;
        Ok(res)
    }

    pub async fn subject_find(&self, subject: &String) -> Result<Option<Subject>, AppError> {
        let res = self.repository.subject_find(&subject).await?;
        Ok(res)
    }

    pub async fn subject_all(&self) -> Result<Vec<Subject>, AppError> {
        let res = self.repository.subject_all().await?;
        Ok(res)
    }

    pub async fn subject_schemas(&self, subject: &String) -> Result<Vec<VersionedSchema>, AppError> {
        let res = self.repository.subject_schemas(&subject).await?;
        Ok(res)
    }

    pub async fn config_get_subject(&self, subject: Option<&String>) -> Result<Option<SchemaCompatibility>, AppError> {

        let subject_id = match subject {
            Some(sub) => self.subject_find(sub).await?.map(|x| x.id),
            None => None
        };

        let res = self.repository.config_get_subject(subject_id).await?;
        Ok(res)
    }

    pub async fn config_set_subject(&self, subject: Option<&String>, compatibility: &Compatibility) -> Result<(), AppError> {
        let subject_id = match subject {
            Some(sub) => self.subject_find(sub).await?.map(|x| x.id),
            None => None
        };

        let _ = self.repository.config_set_subject(subject_id, &compatibility).await?;

        Ok(())
    }

    pub async fn version_id(&self, subject: &String, version_id: &VersionId) -> Result<Option<i32>, AppError> {
        match version_id {
            VersionId::Latest => {
                let res = self.repository.max_version(&subject).await?;
                Ok(res.and_then(|x| x.max_version))
            },
            VersionId::Version(version) => Ok(Some(*version))
        }
    }

    pub async fn check_compatibility(&self, subject: &String, version_id: &VersionId, incoming: &String) -> Result<Compatibility, AppError> {
        let schema_record = self
            .schema_find_by_version(&subject, &version_id)
            .await?
            .ok_or(AppError::SchemaNotFound(subject.clone(), version_id.clone()))?;

        let db_schema = AvroSchema::parse_str(schema_record.schema.as_str())?;
        let incoming_schema = AvroSchema::parse_str(incoming.as_str())?;
        
        let backward = AvroSchemaCompatibility::can_read(&db_schema, &incoming_schema);
        let forward = AvroSchemaCompatibility::can_read(&incoming_schema, &db_schema);

        if backward && forward {
            return Ok(Compatibility::Full);
        } else if backward {
            return Ok(Compatibility::Backward);
        } else if forward {
            return Ok(Compatibility::Forward);
        }

        return Ok(Compatibility::None)
    }
}
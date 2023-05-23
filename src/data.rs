use std::cmp::Ordering;
use std::str::FromStr;
use sqlx::FromRow;
use serde::{Serialize, Deserialize};

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

#[derive(Deserialize, Clone, Debug)]
pub enum VersionId {
    Latest,
    Version(i32)
}

impl FromStr for VersionId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "latest" {
            return Ok(VersionId::Latest)
        }

        let version = s.parse::<i32>().or(Err(()))?;

        Ok(VersionId::Version(version))
    }
}

#[derive(FromRow)]
pub struct MaxVersion {
    pub max_version: Option<i32>
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

impl Compatibility {
    //TODO: is the right way for sqlx to encode?
    pub fn as_str(&self) -> &'static str {
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

#[derive(FromRow, Serialize, Deserialize)]
pub struct SchemaCompatibility {
    pub compatibility: Compatibility
}


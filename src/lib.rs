use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    pub attribute: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub present: Option<bool>,
}

impl Package {
    pub const fn create_table() -> &'static str {
        r#"
CREATE TABLE packages (
    attribute TEXT NOT NULL,
    name TEXT,
    version TEXT,
    storePath TEXT,
    description TEXT,
    long_description TEXT,
    PRIMARY KEY (attribute)
)
        "#
    }
}

impl<'r, 'd> TryFrom<&'r rusqlite::Row<'d>> for Package {
    type Error = rusqlite::Error;

    fn try_from(row: &'r rusqlite::Row<'d>) -> Result<Self, Self::Error> {
        let attribute: String = row.get("attribute")?;
        let store_path: Option<String> = row.get("storePath")?;
        let name: Option<String> = row.get("name")?;
        let version: Option<String> = row.get("version")?;
        let description: Option<String> = row.get("description")?;
        let long_description: Option<String> = row.get("long_description")?;

        let score = if cfg!(debug_assertions) {
            row.get("score")?
        } else {
            None
        };

        Ok(Package {
            attribute,
            name,
            version,
            description,
            long_description,
            store_path,
            score,
            present: Default::default(),
        })
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    pub attribute: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_paths: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub propagated_build_inputs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub propagated_native_build_inputs: Option<Vec<String>>,
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
    storePaths TEXT,
    propagatedBuildInputs TEXT,
    propagatedNativeBuildInputs TEXT,
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
        let store_paths: Option<String> = row.get("storePaths")?;
        let propagated_build_inputs: Option<String> = row.get("propagatedBuildInputs")?;
        let propagated_native_build_inputs: Option<String> =
            row.get("propagatedNativeBuildInputs")?;
        let name: Option<String> = row.get("name")?;
        let version: Option<String> = row.get("version")?;
        let description: Option<String> = row.get("description")?;
        let long_description: Option<String> = row.get("long_description")?;

        let score = if cfg!(debug_assertions) {
            row.get("score")?
        } else {
            None
        };

        let store_paths = store_paths
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
        let propagated_build_inputs = propagated_build_inputs
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
        let propagated_native_build_inputs = propagated_native_build_inputs
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

        Ok(Package {
            attribute,
            name,
            version,
            description,
            long_description,
            store_paths,
            propagated_build_inputs,
            propagated_native_build_inputs,
            score,
            present: Default::default(),
        })
    }
}

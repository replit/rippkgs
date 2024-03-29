use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    pub attribute: String,
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub long_description: Option<String>,
    pub store_path: Option<String>,
    pub score: Option<f64>,
}

impl<'r, 'd> TryFrom<&'r rusqlite::Row<'d>> for Package {
    type Error = rusqlite::Error;

    fn try_from(row: &'r rusqlite::Row<'d>) -> Result<Self, Self::Error> {
        let attribute: String = row.get("attribute")?;
        let store_path: Option<String> = row.get("outPath")?;
        let name: Option<String> = row.get("name")?;
        let version: Option<String> = row.get("version")?;
        let description: Option<String> = row.get("description")?;
        let homepage: Option<String> = row.get("homepage")?;
        let long_description: Option<String> = row.get("long_description")?;
        let score: Option<f64> = row.get("score").unwrap_or(None);

        Ok(Package {
            attribute,
            name,
            version,
            description,
            homepage,
            long_description,
            store_path,
            score,
        })
    }
}

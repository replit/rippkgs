use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PackageInfo {
    #[serde(default)]
    pub outputs: PackageOutputs,
    pub pname: Option<String>,
    pub version: Option<String>,
    pub meta: Option<PackageMeta>,
}

pub type PackageOutputs = std::collections::HashMap<String, PathBuf>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageMeta {
    pub available: Option<bool>,
    #[serde(default)]
    pub broken: bool,
    pub description: Option<String>,
    pub homepage: Option<OneOrList<String>>,
    #[serde(default)]
    pub insecure: bool,
    pub license: Option<serde_json::Value>,
    pub long_description: Option<String>,
    #[serde(default)]
    pub unfree: bool,
    #[serde(default)]
    pub unsupported: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OneOrList<T> {
    One(T),
    List(Vec<T>),
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    #[test]
    fn one_or_list() {
        assert_matches!(
            serde_json::from_str::<super::OneOrList<String>>("[]"),
            Ok(super::OneOrList::List(_))
        );

        assert_matches!(
            serde_json::from_str::<super::OneOrList<String>>(r#""hi""#),
            Ok(super::OneOrList::One(_))
        );
    }
}


use eyre::Context;
use rippkgs::Package;
use rusqlite::Connection;

pub fn search(query_str: &str, db: &Connection) -> eyre::Result<Option<Package>> {
    let result = db.query_row(
        "SELECT *, NULL AS score FROM packages WHERE attribute = ?1",
        rusqlite::params![query_str],
        |r| Package::try_from(r),
    );

    match result {
        Ok(mut res) => {
            let Some(store_paths) = res.store_paths.as_ref() else {
                // only None when the package is stdenv (not installable) or part of
                // bootstrapping (should use other attrs). We always filter these out because
                // they're almost always irrelevant.
                return Ok(None);
            };

            let Some(_) = store_paths.get("out") else {
                // this is a package that doesn't have an out path, so it's not installable
                return Ok(None);
            };
            res.present = Some(res.is_present());
            Ok(Some(res))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err).context("executing query"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index(with_cache: bool, cached: Option<bool>, out: &str) -> Connection {
        let db = Connection::open_in_memory().unwrap();
        let store_paths = serde_json::json!({ "out": out }).to_string();
        if with_cache {
            db.execute(Package::create_table(), []).unwrap();
            db.execute(
                "INSERT INTO packages (attribute, name, storePaths, cached) VALUES ('figlet', 'figlet', ?1, ?2)",
                rusqlite::params![store_paths, cached],
            )
            .unwrap();
        } else {
            db.execute(
                "CREATE TABLE packages (attribute TEXT PRIMARY KEY, name TEXT, version TEXT, storePaths TEXT, propagatedBuildInputs TEXT, propagatedNativeBuildInputs TEXT, description TEXT, long_description TEXT)",
                [],
            )
            .unwrap();
            db.execute(
                "INSERT INTO packages (attribute, name, storePaths) VALUES ('figlet', 'figlet', ?1)",
                [store_paths],
            )
            .unwrap();
        }
        db
    }

    #[test]
    fn legacy_index_uses_disk_presence_and_reports_unknown_cache_status() {
        let package = search(
            "figlet",
            &index(false, None, "00000000000000000000000000000000-figlet"),
        )
        .unwrap()
        .unwrap();
        assert_eq!(package.cached, None);
        assert_eq!(package.present, Some(false));
        assert_eq!(
            serde_json::to_value(package).unwrap()["cached"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn cached_package_is_present_without_a_disk_path() {
        let package = search(
            "figlet",
            &index(true, Some(true), "00000000000000000000000000000000-figlet"),
        )
        .unwrap()
        .unwrap();
        assert_eq!(package.cached, Some(true));
        assert_eq!(package.present, Some(true));
    }

    #[test]
    fn cache_miss_remains_present_when_on_disk() {
        let package = search("figlet", &index(true, Some(false), "."))
            .unwrap()
            .unwrap();
        assert_eq!(package.cached, Some(false));
        assert_eq!(package.present, Some(true));
    }

    #[test]
    fn cache_miss_without_a_disk_path_is_absent() {
        let package = search(
            "figlet",
            &index(true, Some(false), "00000000000000000000000000000000-figlet"),
        )
        .unwrap()
        .unwrap();
        assert_eq!(package.present, Some(false));
    }
}

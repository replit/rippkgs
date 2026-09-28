use eyre::Context;
use rippkgs::Package;
use rusqlite::Connection;

pub fn search(query_str: &str, db: &Connection, has_cache: bool) -> eyre::Result<Option<Package>> {
    let query = if has_cache {
        "SELECT packages.*, NULL AS score, EXISTS(SELECT 1 FROM cached_packages WHERE attribute = packages.attribute) AS cached FROM packages WHERE packages.attribute = ?1"
    } else {
        "SELECT *, NULL AS score, 0 AS cached FROM packages WHERE attribute = ?1"
    };
    let result = db.query_row(query, rusqlite::params![query_str], |r| {
        Package::try_from(r)
    });

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

    fn index(with_cache: bool, cache_hit: bool, out: &str) -> Connection {
        let db = Connection::open_in_memory().unwrap();
        let store_paths = serde_json::json!({ "out": out }).to_string();
        db.execute(Package::create_table(), []).unwrap();
        if with_cache {
            db.execute(
                "CREATE TABLE cached_packages (attribute TEXT NOT NULL PRIMARY KEY)",
                [],
            )
            .unwrap();
            if cache_hit {
                db.execute("INSERT INTO cached_packages VALUES ('figlet')", [])
                    .unwrap();
            }
        }
        db.execute(
            "INSERT INTO packages (attribute, name, storePaths) VALUES ('figlet', 'figlet', ?1)",
            [store_paths],
        )
        .unwrap();
        db
    }

    #[test]
    fn legacy_index_uses_disk_presence_without_changing_json_fields() {
        let package = search(
            "figlet",
            &index(false, false, "00000000000000000000000000000000-figlet"),
            false,
        )
        .unwrap()
        .unwrap();
        assert_eq!(package.present, Some(false));
        let json = serde_json::to_value(package).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "attribute": "figlet",
                "name": "figlet",
                "store_paths": {"out": "00000000000000000000000000000000-figlet"},
                "present": false
            })
        );
    }

    #[test]
    fn cached_package_is_present_without_a_disk_path() {
        let package = search(
            "figlet",
            &index(true, true, "00000000000000000000000000000000-figlet"),
            true,
        )
        .unwrap()
        .unwrap();
        assert!(package.cached);
        assert_eq!(package.present, Some(true));
        assert!(serde_json::to_value(package)
            .unwrap()
            .get("cached")
            .is_none());
    }

    #[test]
    fn cache_miss_remains_present_when_on_disk() {
        let package = search("figlet", &index(true, false, "."), true)
            .unwrap()
            .unwrap();
        assert!(!package.cached);
        assert_eq!(package.present, Some(true));
    }

    #[test]
    fn cache_miss_without_a_disk_path_is_absent() {
        let package = search(
            "figlet",
            &index(true, false, "00000000000000000000000000000000-figlet"),
            true,
        )
        .unwrap()
        .unwrap();
        assert_eq!(package.present, Some(false));
    }
}

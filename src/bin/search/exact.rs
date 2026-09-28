use eyre::Context;
use rippkgs::{Package, Presence};
use rusqlite::Connection;

pub fn search(
    query_str: &str,
    db: &Connection,
    presence: Presence,
) -> eyre::Result<Option<Package>> {
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

            let Some(_out_path) = store_paths.get("out") else {
                // this is a package that doesn't have an out path, so it's not installable
                return Ok(None);
            };
            res.present = res.presence(presence);
            Ok(Some(res))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err).context("executing query"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index(with_cache: bool) -> Connection {
        let db = Connection::open_in_memory().unwrap();
        if with_cache {
            db.execute(Package::create_table(), []).unwrap();
            db.execute(
                "INSERT INTO packages (attribute, name, storePaths, cached) VALUES (?1, 'figlet', ?2, 1)",
                rusqlite::params!["figlet", r#"{"out":"00000000000000000000000000000000-figlet"}"#],
            )
            .unwrap();
        } else {
            db.execute(
                "CREATE TABLE packages (attribute TEXT PRIMARY KEY, name TEXT, version TEXT, storePaths TEXT, propagatedBuildInputs TEXT, propagatedNativeBuildInputs TEXT, description TEXT, long_description TEXT)",
                [],
            )
            .unwrap();
            db.execute(
                "INSERT INTO packages (attribute, name, storePaths) VALUES (?1, 'figlet', ?2)",
                rusqlite::params![
                    "figlet",
                    r#"{"out":"00000000000000000000000000000000-figlet"}"#
                ],
            )
            .unwrap();
        }
        db
    }

    #[test]
    fn legacy_index_is_readable_with_unknown_cache_status() {
        let package = search("figlet", &index(false), Presence::Cached)
            .unwrap()
            .unwrap();
        assert_eq!(package.cached, None);
        assert_eq!(package.present, None);
        assert_eq!(
            serde_json::to_value(package).unwrap()["cached"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn presence_modes_preserve_disk_default_and_use_cached_status_when_requested() {
        let db = index(true);
        let disk = search("figlet", &db, Presence::Disk).unwrap().unwrap();
        assert_eq!(disk.present, Some(false));
        assert_eq!(disk.cached, Some(true));

        let cached = search("figlet", &db, Presence::Cached).unwrap().unwrap();
        assert_eq!(cached.present, Some(true));
        assert_eq!(
            search("figlet", &db, Presence::Either)
                .unwrap()
                .unwrap()
                .present,
            Some(true)
        );
    }
}

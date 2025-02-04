use eyre::Context;
use rusqlite::Connection;
use std::path::PathBuf;

use rippkgs::Package;

pub fn search(query_str: &str, db: &Connection) -> eyre::Result<Option<Package>> {
    let result = db.query_row(
        "SELECT * FROM packages WHERE attribute = ?1",
        rusqlite::params![query_str],
        |r| Package::try_from(r),
    );

    match result {
        Ok(mut res) => {
            let Some(store_path) = res.store_path.as_ref() else {
                // only None when the package is stdenv (not installable) or part of
                // bootstrapping (should use other attrs). We always filter these out because
                // they're almost always irrelevant.
                return Ok(None);
            };
            res.present = Some(PathBuf::from("/nix/store/").join(store_path).exists());
            Ok(Some(res))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err).context("executing query"),
    }
}

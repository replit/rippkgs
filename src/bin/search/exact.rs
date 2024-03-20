use eyre::Context;
use rusqlite::Connection;

use rippkgs::Package;

pub fn search(query_str: &str, db: &Connection) -> eyre::Result<Option<Package>> {
    let result = db.query_row(
        "SELECT * FROM packages WHERE attribute = ?1",
        rusqlite::params![query_str],
        |r| Package::try_from(r),
    );

    match result {
        Ok(res) => Ok(Some(res)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err).context("executing query"),
    }
}

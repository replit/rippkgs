use std::path::PathBuf;

use eyre::Context;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rusqlite::functions::Context as FunctionContext;
use rusqlite::{functions::FunctionFlags, Connection};

use rippkgs::Package;

pub fn search(
    query_str: &str,
    db: &Connection,
    num_results: u32,
    filter_built: bool,
) -> eyre::Result<Vec<Package>> {
    db.create_scalar_function(
        "fuzzy_score",
        2,
        FunctionFlags::SQLITE_UTF8,
        scalar_fuzzy_score,
    )
    .context("installing `fuzzy_score` function")?;

    let mut query = db
        .prepare(
            r#"
SELECT *, fuzzy_score(name, ?1) as score
FROM packages
ORDER BY score DESC
LIMIT ?2
            "#,
        )
        .context("preparing query")?;

    let res = query
        .query_map(rusqlite::params![query_str, num_results], |r| {
            Package::try_from(r)
        })
        .context("executing query")?
        .filter(|package_res| {
            let Ok(package) = package_res else {
                // carry on the error
                return true;
            };

            let Some(store_path) = package.store_paths.as_ref().and_then(|x| x.get("out")) else {
                // only None when the package is stdenv (not installable) or part of
                // bootstrapping (should use other attrs). We always filter these out because
                // they're almost always irrelevant.
                return false;
            };

            if !filter_built {
                return true;
            }

            PathBuf::from("/nix/store/").join(store_path).exists()
        })
        .map(|package_res| {
            if filter_built {
                return package_res;
            }

            let Ok(mut package) = package_res else {
                return package_res;
            };

            let Some(store_path) = package.store_paths.as_ref().and_then(|x| x.get("out")) else {
                // only None when the package is stdenv (not installable) or part of
                // bootstrapping (should use other attrs). We always filter these out because
                // they're almost always irrelevant.
                return Ok(package);
            };

            package.present = Some(PathBuf::from("/nix/store/").join(store_path).exists());
            Ok(package)
        })
        .take(num_results as _)
        .collect::<Result<Vec<_>, _>>()
        .context("parsing results");

    res
}

fn scalar_fuzzy_score(ctx: &FunctionContext) -> rusqlite::Result<i64> {
    lazy_static::lazy_static! {
      static ref MATCHER: SkimMatcherV2 = SkimMatcherV2::default().ignore_case();
    }

    let choice = ctx.get::<String>(0)?;
    let pattern = ctx.get::<String>(1)?;

    if choice == pattern {
        return Ok(i64::MAX);
    }

    Ok(MATCHER.fuzzy_match(&choice, &pattern).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_search_and_filter_built_stay_disk_only() {
        let db = Connection::open_in_memory().unwrap();
        db.execute(Package::create_table(), []).unwrap();
        db.execute(
            "INSERT INTO packages (attribute, name, storePaths) VALUES ('figlet', 'figlet', ?1)",
            [r#"{"out":"00000000000000000000000000000000-figlet"}"#],
        )
        .unwrap();

        let results = search("figlet", &db, 10, false).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].present, Some(false));
        assert!(search("figlet", &db, 10, true).unwrap().is_empty());

        db.execute("UPDATE packages SET storePaths = ?1", [r#"{"out":"."}"#])
            .unwrap();
        assert_eq!(
            search("figlet", &db, 10, false).unwrap()[0].present,
            Some(true)
        );
        assert_eq!(search("figlet", &db, 10, true).unwrap().len(), 1);

        db.execute("UPDATE packages SET storePaths = NULL", [])
            .unwrap();
        assert!(search("figlet", &db, 10, false).unwrap().is_empty());
    }
}

use eyre::Context;
use rippkgs::Package;
use rusqlite::Connection;
use std::path::PathBuf;

use super::nix;

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

            let Some(out_path) = store_paths.get("out") else {
                // this is a package that doesn't have an out path, so it's not installable
                return Ok(None);
            };
            res.present = Some(
                PathBuf::from("/nix/store/").join(out_path).exists()
                    || nix::substitutable(out_path, nix::substituters()),
            );
            Ok(Some(res))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(err) => Err(err).context("executing query"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    fn index(out: Option<&str>) -> Connection {
        let db = Connection::open_in_memory().unwrap();
        db.execute(Package::create_table(), []).unwrap();
        let store_paths = out.map(|out| serde_json::json!({ "out": out }).to_string());
        db.execute(
            "INSERT INTO packages (attribute, name, storePaths) VALUES ('figlet', 'figlet', ?1)",
            [store_paths],
        )
        .unwrap();
        db
    }

    #[test]
    fn exact_lookup_checks_disk_then_configured_substituter_and_fails_closed() {
        let original_path = std::env::var_os("PATH").unwrap_or_default();
        let shell = std::env::split_paths(&original_path)
            .map(|dir| dir.join("sh"))
            .find(|path| path.exists())
            .unwrap();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("rippkgs-fake-nix-{unique}"));
        fs::create_dir(&dir).unwrap();
        let log = dir.join("calls");
        let pid_file = dir.join("pid");
        let script = format!(
            "#!{}\nprintf '%s\\n' \"$1 $2\" >> \"$RIPPKGS_TEST_LOG\"\n\
             case \"$1 $2\" in\n\
               'config show') printf 'https://cache.example/\\n'; exit 0;;\n\
               'path-info --store')\n\
                 case \"$RIPPKGS_TEST_STATUS\" in\n\
                   hit) exit 0;;\n\
                   timeout) printf '%s' \"$$\" > \"$RIPPKGS_TEST_PID\"; exec sleep 30;;\n\
                   *) exit 1;;\n\
                 esac;;\n\
             esac\nexit 1\n",
            shell.display()
        );
        let binary = dir.join("nix");
        fs::write(&binary, script).unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();

        struct Restore(OsString, std::path::PathBuf);
        impl Drop for Restore {
            fn drop(&mut self) {
                std::env::set_var("PATH", &self.0);
                std::env::remove_var("RIPPKGS_TEST_LOG");
                std::env::remove_var("RIPPKGS_TEST_PID");
                std::env::remove_var("RIPPKGS_TEST_STATUS");
                let _ = fs::remove_dir_all(&self.1);
            }
        }
        let _restore = Restore(original_path.clone(), dir.clone());
        let mut search_path = vec![dir.clone()];
        search_path.extend(std::env::split_paths(&original_path));
        std::env::set_var("PATH", std::env::join_paths(search_path).unwrap());
        std::env::set_var("RIPPKGS_TEST_LOG", &log);
        std::env::set_var("RIPPKGS_TEST_PID", &pid_file);

        let disk = search("figlet", &index(Some("."))).unwrap().unwrap();
        assert_eq!(disk.present, Some(true));
        assert!(!log.exists());

        let missing = "00000000000000000000000000000000-figlet";
        std::env::set_var("RIPPKGS_TEST_STATUS", "hit");
        let hit = search("figlet", &index(Some(missing))).unwrap().unwrap();
        assert_eq!(hit.present, Some(true));
        assert_eq!(
            serde_json::to_value(hit).unwrap(),
            serde_json::json!({
                "attribute": "figlet",
                "name": "figlet",
                "store_paths": {"out": missing},
                "present": true
            })
        );

        std::env::set_var("RIPPKGS_TEST_STATUS", "miss");
        assert_eq!(
            search("figlet", &index(Some(missing)))
                .unwrap()
                .unwrap()
                .present,
            Some(false)
        );

        std::env::set_var("RIPPKGS_TEST_STATUS", "timeout");
        let start = Instant::now();
        assert_eq!(
            search("figlet", &index(Some(missing)))
                .unwrap()
                .unwrap()
                .present,
            Some(false)
        );
        assert!(start.elapsed() < Duration::from_secs(8));
        #[cfg(target_os = "linux")]
        {
            let pid = fs::read_to_string(&pid_file).unwrap();
            assert!(!std::path::Path::new(&format!("/proc/{pid}")).exists());
        }

        std::env::set_var("PATH", &dir);
        fs::remove_file(&binary).unwrap();
        assert_eq!(
            search("figlet", &index(Some(missing)))
                .unwrap()
                .unwrap()
                .present,
            Some(false)
        );
        assert!(search("figlet", &index(None)).unwrap().is_none());
    }
}

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use super::data::Registry;

const WORKERS: usize = 16;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

fn store_hash(path: &str) -> Option<&str> {
    let basename = path.strip_prefix("/nix/store/").unwrap_or(path);
    let (hash, _) = basename.split_once('-')?;
    if hash.len() == 32
        && hash
            .bytes()
            .all(|c| b"0123456789abcdfghijklmnpqrsvwxyz".contains(&c))
    {
        Some(hash)
    } else {
        None
    }
}

fn check(agent: &ureq::Agent, substituter: &str, path: &str) -> Option<bool> {
    let hash = store_hash(path)?;
    let url = format!("{}/{}.narinfo", substituter.trim_end_matches('/'), hash);

    for attempt in 0..2 {
        match agent.head(&url).call() {
            Ok(_) => return Some(true),
            Err(ureq::Error::Status(404, _)) => return Some(false),
            Err(_) if attempt == 0 => std::thread::sleep(Duration::from_millis(100)),
            Err(_) => return None,
        }
    }
    None
}

pub fn availability(registry: &Registry, substituter: &str) -> HashMap<String, Option<bool>> {
    let paths: Vec<String> = registry
        .values()
        .filter_map(|info| info.store_paths.as_ref()?.get("out"))
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let next = AtomicUsize::new(0);
    let (sender, receiver) = mpsc::channel();
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(REQUEST_TIMEOUT)
        .timeout_read(REQUEST_TIMEOUT)
        .timeout_write(REQUEST_TIMEOUT)
        .build();

    std::thread::scope(|scope| {
        for _ in 0..WORKERS.min(paths.len()) {
            let sender = sender.clone();
            let agent = agent.clone();
            let paths = &paths;
            let next = &next;
            scope.spawn(move || loop {
                let Some(path) = paths.get(next.fetch_add(1, Ordering::Relaxed)) else {
                    break;
                };
                let _ = sender.send((path.clone(), check(&agent, substituter, path)));
            });
        }
    });
    drop(sender);
    receiver.into_iter().collect()
}

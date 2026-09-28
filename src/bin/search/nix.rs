use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const CONFIG_TIMEOUT: Duration = Duration::from_secs(2);
const LOOKUP_TIMEOUT: Duration = Duration::from_secs(5);

fn run_nix(args: &[&str], timeout: Duration) -> Option<String> {
    let mut child = Command::new("nix")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let Some(mut stdout) = child.stdout.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return None;
    };
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut output = String::new();
        let _ = sender.send(stdout.read_to_string(&mut output).map(|_| output));
    });
    let deadline = Instant::now() + timeout;

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return None;
                }
                let remaining = deadline.saturating_duration_since(Instant::now());
                let output = receiver.recv_timeout(remaining).ok()?.ok()?;
                return Some(output);
            }
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(10));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
}

fn parse_substituters(output: &str) -> Vec<String> {
    let configured = output
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once('=')?;
            (key.trim() == "substituters").then_some(value.trim())
        })
        .or_else(|| {
            let line = output.trim();
            (!line.contains('\n') && !line.contains('=')).then_some(line)
        })
        .unwrap_or_default();

    configured.split_whitespace().map(str::to_string).collect()
}

pub fn substituters() -> &'static [String] {
    static SUBSTITUTERS: OnceLock<Vec<String>> = OnceLock::new();
    SUBSTITUTERS.get_or_init(|| {
        run_nix(&["config", "show", "substituters"], CONFIG_TIMEOUT)
            .or_else(|| run_nix(&["show-config"], CONFIG_TIMEOUT))
            .map(|output| parse_substituters(&output))
            .unwrap_or_default()
    })
}

pub fn substitutable(out: &str, substituters: &[String]) -> bool {
    let deadline = Instant::now() + LOOKUP_TIMEOUT;
    let path = format!("/nix/store/{out}");

    for substituter in substituters {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        if run_nix(&["path-info", "--store", substituter, &path], remaining).is_some() {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::parse_substituters;

    #[test]
    fn parses_direct_and_legacy_nix_configuration() {
        assert_eq!(
            parse_substituters(" https://cache.one/  https://cache.two/ \n"),
            vec!["https://cache.one/", "https://cache.two/"]
        );
        assert_eq!(
            parse_substituters(
                "other = yes\nsubstituters = https://cache.one/  https://cache.two/\n"
            ),
            vec!["https://cache.one/", "https://cache.two/"]
        );
        assert!(parse_substituters("substituters = \n").is_empty());
        assert!(parse_substituters("").is_empty());
    }
}

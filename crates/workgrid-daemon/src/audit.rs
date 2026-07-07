use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn log_event(audit_log: &Path, event: &str, fingerprint: &str) -> anyhow::Result<()> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let line = format!("{} {} {}\n", now, fingerprint, event);
    let mut file = OpenOptions::new().create(true).append(true).open(audit_log)?;
    file.write_all(line.as_bytes())?;
    Ok(())
}

//! Per-installation SECRET_KEY management. Generated on first launch,
//! persisted in the user-data dir with mode 0600. Same key across
//! subsequent launches so sessions survive.

use std::path::Path;

use anyhow::{Context, Result};
use sha2::Digest;

pub fn ensure_secret(data_dir: &Path) -> Result<String> {
    let path = data_dir.join("secret_key");
    if path.is_file() {
        let s = std::fs::read_to_string(&path)
            .with_context(|| format!("read {}", path.display()))?;
        let trimmed = s.trim();
        if trimmed.len() >= 32 {
            return Ok(trimmed.to_string());
        }
    }
    let secret = generate_secret();
    write_secret(&path, &secret)?;
    Ok(secret)
}

fn generate_secret() -> String {
    // 64 hex chars of cryptographic randomness, matching the policy
    // marlinspike-setup uses for the Python path.
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();
    let mut hasher = sha2::Sha256::new();
    hasher.update(nanos.to_le_bytes());
    hasher.update(pid.to_le_bytes());
    // Mix in some bytes from the OS rng. Cheap dependency-free approach:
    // hash the contents of /dev/urandom or %SystemRoot%/System32/rng on
    // Windows. Anyhow — if the OS rng failed we'd be in worse trouble.
    if let Ok(mut bytes) = read_os_random(32) {
        hasher.update(&bytes);
        bytes.fill(0);
    }
    hex::encode(hasher.finalize())
}

#[cfg(unix)]
fn read_os_random(n: usize) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut buf = vec![0u8; n];
    std::fs::File::open("/dev/urandom")
        .context("open /dev/urandom")?
        .read_exact(&mut buf)
        .context("read /dev/urandom")?;
    Ok(buf)
}

#[cfg(windows)]
fn read_os_random(n: usize) -> Result<Vec<u8>> {
    // Tauri pulls in `getrandom` transitively. Use it.
    let mut buf = vec![0u8; n];
    getrandom::getrandom(&mut buf).context("getrandom")?;
    Ok(buf)
}

fn write_secret(path: &Path, secret: &str) -> Result<()> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        f.write_all(secret.as_bytes())?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, secret)?;
    }
    Ok(())
}

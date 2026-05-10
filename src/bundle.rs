//! Bundle extraction: takes the embedded `bundle.tar.zst` blob and
//! decompresses it into the user-data dir on first launch. Subsequent
//! launches detect the version stamp and skip re-extraction.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use log::{debug, info};

/// Compile-time embedded bundle. Produced by `scripts/build-bundle.sh`
/// before `cargo build`. When the `embed-bundle` feature is off, this
/// is a small placeholder and the binary expects an extracted bundle
/// in `runtime_root` at launch (dev mode).
#[cfg(feature = "embed-bundle")]
const BUNDLE_BLOB: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/bundle/",
    env!("GLASSMARLIN_TARGET_OS_STAMP"),
    "/bundle.tar.zst"
));

#[cfg(not(feature = "embed-bundle"))]
const BUNDLE_BLOB: &[u8] = b"";

const BUNDLE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn ensure_extracted(data_dir: &Path) -> Result<PathBuf> {
    let runtime_root = data_dir.join("runtime");
    let stamp_path = runtime_root.join(".bundle-version");

    if stamp_path.is_file() {
        match std::fs::read_to_string(&stamp_path) {
            Ok(s) if s.trim() == BUNDLE_VERSION => {
                debug!("bundle already extracted at {}", runtime_root.display());
                return Ok(runtime_root);
            }
            Ok(s) => {
                info!(
                    "bundle version mismatch (have {}, want {BUNDLE_VERSION}); re-extracting",
                    s.trim()
                );
            }
            Err(e) => {
                info!("stamp unreadable ({e}); re-extracting");
            }
        }
        let _ = std::fs::remove_dir_all(&runtime_root);
    }

    if BUNDLE_BLOB.is_empty() {
        return Err(anyhow!(
            "bundle is empty — was the binary built without the `embed-bundle` \
             feature? In dev mode, place an extracted bundle at {} manually.",
            runtime_root.display()
        ));
    }

    info!(
        "extracting {} MB bundle to {}",
        BUNDLE_BLOB.len() / 1024 / 1024,
        runtime_root.display()
    );

    std::fs::create_dir_all(&runtime_root)?;
    let dec = zstd::Decoder::new(BUNDLE_BLOB).context("zstd decoder")?;
    let mut archive = tar::Archive::new(dec);
    archive
        .unpack(&runtime_root)
        .with_context(|| format!("tar unpack to {}", runtime_root.display()))?;

    std::fs::write(&stamp_path, BUNDLE_VERSION).context("write stamp")?;

    info!("bundle extraction complete");
    Ok(runtime_root)
}

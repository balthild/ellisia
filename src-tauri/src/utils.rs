use std::borrow::Cow;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[cfg(windows)]
pub fn path_to_bytes(path: &Path) -> Cow<[u8]> {
    use std::os::windows::ffi::OsStrExt;
    let chars = path.as_os_str().encode_wide();
    let bytes = chars.flat_map(u16::to_le_bytes).collect();
    Cow::Owned(bytes)
}

#[cfg(unix)]
pub fn path_to_bytes(path: &Path) -> Cow<[u8]> {
    use std::os::unix::ffi::OsStrExt;
    Cow::Borrowed(path.as_os_str().as_bytes())
}

pub fn get_config_dir_path() -> Result<PathBuf> {
    let mut path = dirs::config_dir().context("Failed to get user config dir path")?;

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    path.push("Ellisia");
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    path.push("ellisia");

    Ok(path)
}

pub fn init_dir(path: &Path) -> Result<()> {
    if path.is_file() {
        std::fs::remove_file(path).context("Failed to init app config directory")?;
    }

    if !path.exists() {
        std::fs::create_dir_all(path).context("Failed to init app config directory")?;
    }

    Ok(())
}
